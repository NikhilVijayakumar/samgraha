use anyhow::Result;
use common::config::SamgrahaConfig;
use mcp::adapter::McpAdapter;
use mcp::protocol::{McpMessage, McpRequest};
use services::registry_client::FileRegistryClient;
use services::KnowledgeRuntime;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct JsonRpcRequest {
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(serde::Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

fn main() -> Result<()> {
    common::load_dotenv();
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("samgraha=info".parse().unwrap()),
        )
        .with_target(false)
        .try_init();

    check_expiry();

    let root = discover_root()?;
    let config = load_config(&root)?;
    let runtime = Arc::new(KnowledgeRuntime::new(&root, config)?);
    let registry = Arc::new(FileRegistryClient::new(&root));
    let adapter = McpAdapter::new(runtime, registry);

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        // PowerShell always prepends a UTF-8 BOM when piping a string literal to a
        // native process's stdin — strip it so the first request from a plain
        // `'...' | mcp.exe` pipe on Windows doesn't fail to parse.
        let line = line.strip_prefix('\u{feff}').unwrap_or(&line).to_string();
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(serde_json::Value::Null),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err).unwrap());
                let _ = stdout.flush();
                continue;
            }
        };

        // JSON-RPC 2.0 notifications have no id — server must not respond.
        if req.id.is_none() {
            continue;
        }

        let response = handle(&adapter, &req);
        let _ = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap());
        let _ = stdout.flush();
    }

    adapter.notify_disconnect();
    Ok(())
}

fn handle(adapter: &McpAdapter, req: &JsonRpcRequest) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => {
            adapter.notify_connect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.clone(),
                result: Some(serde_json::json!({
                    "protocolVersion": "2025-03-26",
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": "samgraha-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })),
                error: None,
            }
        }
        "notifications/initialized" => {
            // Handled upstream — loop skips when id.is_none().
            unreachable!()
        }
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(serde_json::json!({ "tools": tool_definitions() })),
            error: None,
        },
        "tools/call" => {
            let name = match req.params.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id.clone(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing 'name' in tool call".to_string(),
                        }),
                    };
                }
            };

            let arguments = req
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            let params = arguments
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<std::collections::HashMap<_, _>>()
                })
                .unwrap_or_default();

            let mcp_req = McpRequest {
                id: req
                    .id
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                method: name.to_string(),
                params,
                repo: None,
            };

            match adapter.handle_message(McpMessage::Request(mcp_req)) {
                McpMessage::Response(resp) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: Some(tool_result(&resp.result, false)),
                    error: None,
                },
                McpMessage::Error(err) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: Some(tool_result(&serde_json::json!({ "error": err.message }), true)),
                    error: None,
                },
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: Some(tool_result(&serde_json::json!({ "error": "Internal error" }), true)),
                    error: None,
                },
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
        },
    }
}

/// Wrap a handler's raw JSON payload into the MCP `tools/call` result shape
/// (`content` text blocks + `isError`), which is what MCP clients (Claude Code,
/// opencode, etc.) actually read. Returning bare JSON-RPC `result` without this
/// wrapper renders as empty output in every client, regardless of what the
/// handler produced.
fn tool_result(payload: &serde_json::Value, is_error: bool) -> serde_json::Value {
    let text = serde_json::to_string_pretty(payload).unwrap_or_else(|_| payload.to_string());
    serde_json::json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": payload,
        "isError": is_error,
    })
}

fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "init",
            "description": "Initialize samgraha.toml and .samgraha/ for a repository, or backfill any keys missing from an existing samgraha.toml (never overwrites a key that's already there). Run this first in a repo with no samgraha.toml before compile/register_repository. Pass 'repo_path' to target a different repository than the one this MCP session is anchored to — required for a global/user-scope server bootstrapping a repo it wasn't started in.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "force": { "type": "boolean", "description": "Overwrite existing samgraha.toml with a fresh template instead of backfilling missing keys" },
                    "repo_path": { "type": "string", "description": "Absolute path to the repository to initialize, if not the one this MCP session is anchored to" }
                }
            }
        }),
        serde_json::json!({
            "name": "compile",
            "description": "Compile documentation into knowledge database. Omit 'path' to compile Samgraha itself; provide 'path' to compile an external repository into its own knowledge.db.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "force": { "type": "boolean", "description": "Force recompile all" },
                    "domains": { "type": "array", "items": { "type": "string" }, "description": "Domains to compile" },
                    "path": { "type": "string", "description": "Absolute path to an external repository to compile into its own .samgraha/knowledge.db" }
                }
            }
        }),
        serde_json::json!({
            "name": "sync",
            "description": "Read a compiled repository's manifest.json, register it in the local registry, and write a .meta file so the Planner can resolve it offline. Run after compile when integrating an external repo.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the repository root (must contain .samgraha/manifest.json)" }
                },
                "required": ["path"]
            }
        }),
        serde_json::json!({
            "name": "get_plan",
            "description": "Return the current Knowledge Plan — shows which repositories are loaded, their priority, status (loaded/stale/missing/unresolved/required_missing), and revision.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "switch_context",
            "description": "Switch the active knowledge context to an already-loaded named context (see list_contexts)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Context name to activate" }
                },
                "required": ["name"]
            }
        }),
        serde_json::json!({
            "name": "list_contexts",
            "description": "List loaded knowledge contexts and which one is active",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "search",
            "description": "Search compiled knowledge. Pass 'repo_path' to search a different local repository instead of the one this MCP session is anchored to.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "level": { "type": "string", "enum": ["metadata", "summary", "section", "full"], "description": "Retrieval level" },
                    "domain": { "type": "string", "description": "Filter by domain" },
                    "limit": { "type": "integer", "description": "Max results" },
                    "offset": { "type": "integer", "description": "Result offset" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to search" }
                },
                "required": ["query"]
            }
        }),
        serde_json::json!({
            "name": "get_sections",
            "description": "Get document sections by semantic type. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "semantic_type": { "type": "string", "description": "Semantic type filter" },
                    "domain": { "type": "string", "description": "Filter by domain" },
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["semantic_type"]
            }
        }),
        serde_json::json!({
            "name": "audit",
            "description": "Run audit checks on documentation. For a domain audit (no 'pipeline', or pipeline: 'doc'), the response's semantic_review.tasks bundles per-section LLM review work (section content + rubric) that the calling agent should judge and report via store_section_report — see semantic_review.instruction for the exact next step. Pass 'pipeline' to run a structural pipeline instead (e.g. 'architecture', 'documentation-structure', 'build', 'security', 'consistency', 'coverage', 'dependency', or any domain name) — those return a PipelineReport directly, no semantic_review. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain to audit (Documentation Audit only, ignored if 'pipeline' is set)" },
                    "providers": { "type": "array", "items": { "type": "string" }, "description": "Audit providers, e.g. [\"deterministic\"] or [\"deterministic\", \"semantic\"]" },
                    "pipeline": { "type": "string", "description": "Run a custom pipeline instead of the Documentation Audit: doc, architecture, build, security, consistency, coverage, dependency, documentation-structure, vision, design, readme, prototype, external-context, engineering, feature, feature-technical, feature-design, deterministic-runtime, external-context-ownership, implementation, help" },
                    "inspect_artifact": { "type": "boolean", "description": "build pipeline only: verify the declared binary artifact exists" },
                    "runtime": { "type": "boolean", "description": "security pipeline only: connect to the running app and verify auth/TLS/rate-limiting" },
                    "execute": { "type": "boolean", "description": "build pipeline only: actually run the declared build command" },
                    "dry_run": { "type": "boolean", "description": "build pipeline only: dry-run the declared build command" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        serde_json::json!({
            "name": "info",
            "description": "Get runtime information. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        serde_json::json!({
            "name": "get_document",
            "description": "Get document metadata and section table of contents. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "Document ID" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "get_document_section",
            "description": "Get paginated content of a specific section. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "Document ID" },
                    "section": { "description": "Section index (integer) or heading (string)" },
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["id", "section"]
            }
        }),
        serde_json::json!({
            "name": "list_domains",
            "description": "List available documentation domains. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        serde_json::json!({
            "name": "list_repositories",
            "description": "List registered repositories. Pass 'repo_path' to read a different local repository's registry.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository whose registry to read" }
                }
            }
        }),
        serde_json::json!({
            "name": "register_repository",
            "description": "Register a repository from its manifest JSON. Pass 'repo_path' to register into a different local repository's registry instead of the one this MCP session is anchored to.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "manifest": { "type": "string", "description": "RepositoryManifest as JSON string" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository whose registry to register into" }
                },
                "required": ["manifest"]
            }
        }),
        serde_json::json!({
            "name": "unregister_repository",
            "description": "Unregister a repository by UUID. Pass 'repo_path' to target a different local repository's registry.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "uuid": { "type": "string", "description": "Repository UUID" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository whose registry to target" }
                },
                "required": ["uuid"]
            }
        }),
        serde_json::json!({
            "name": "synchronize_repository",
            "description": "Synchronize dependency metadata from their manifests. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        serde_json::json!({
            "name": "resolve_dependencies",
            "description": "Resolve dependency graph for all registered repositories. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        serde_json::json!({
            "name": "repository_status",
            "description": "Get computed status of all registered repositories. Pass 'repo_path' to read a different local repository's registry.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository whose registry to read" }
                }
            }
        }),
        serde_json::json!({
            "name": "workspace_status",
            "description": "Get workspace-level status across all registered repositories. Pass 'repo_path' to read a different local repository's registry.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository whose registry to read" }
                }
            }
        }),
        serde_json::json!({
            "name": "get_product_knowledge_context",
            "description": "Get a repository's compiled Product Knowledge context (repository_metadata: source/test/scripts dirs, dependencies, pipeline commands, repo identity) — empty until compile has run at least once. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        // ── Semantic Audit Tools ─────────────────────────────────────────────
        serde_json::json!({
            "name": "get_documents_by_domain",
            "description": "List compiled documents in a domain. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain/standard name" },
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["domain"]
            }
        }),
        serde_json::json!({
            "name": "get_section",
            "description": "Get a single section by database primary key. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "section_id": { "type": "integer", "description": "Section primary key" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["section_id"]
            }
        }),
        serde_json::json!({
            "name": "get_audit_knowledge",
            "description": "Serve audit knowledge file content for a domain section type. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain name" },
                    "section_type": { "type": "string", "description": "Section semantic type" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["domain", "section_type"]
            }
        }),
        serde_json::json!({
            "name": "get_audit_report",
            "description": "Get the latest audit report for a scope. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain name" },
                    "document_id": { "type": "integer", "description": "Optional document ID" },
                    "section_id": { "type": "integer", "description": "Optional section ID" },
                    "stage": { "type": "string", "enum": ["deterministic", "section", "document", "cross_domain"], "description": "Audit stage" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["domain", "stage"]
            }
        }),
        serde_json::json!({
            "name": "get_section_changed",
            "description": "Check if a section changed since last audit (incremental skip). Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "section_id": { "type": "integer", "description": "Section primary key" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["section_id"]
            }
        }),
        serde_json::json!({
            "name": "check_gate",
            "description": "Check if a stage gate is clear before proceeding. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "stage": { "type": "string", "enum": ["deterministic", "section", "document", "cross_domain"], "description": "Audit stage to check" },
                    "document_id": { "type": "integer", "description": "Optional document ID for scoped check" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["stage"]
            }
        }),
        serde_json::json!({
            "name": "store_section_report",
            "description": "Agent writes section audit findings; validates schema before persist. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_json": { "type": "object", "description": "SemanticReport as JSON object" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["report_json"]
            }
        }),
        serde_json::json!({
            "name": "store_document_report",
            "description": "Agent writes document-level audit findings; validates schema. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_json": { "type": "object", "description": "SemanticReport as JSON object" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["report_json"]
            }
        }),
        serde_json::json!({
            "name": "store_cross_domain_report",
            "description": "Agent writes cross-domain audit findings; validates schema. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_json": { "type": "object", "description": "SemanticReport as JSON object" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["report_json"]
            }
        }),
        serde_json::json!({
            "name": "update_finding_status",
            "description": "Mark a semantic-stage finding (from store_section_report/store_document_report/store_cross_domain_report) as Fixed / Accepted / Ignored / False Positive. 'report_id' must be the numeric id of a stored SemanticReport — this does NOT apply to plain domain-audit findings or pipeline (architecture/documentation-structure/build/security/consistency/coverage/help) findings; use audit_fix_accept/audit_fix_reject for those instead. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_id": { "type": "integer", "description": "Numeric id of a stored SemanticReport (Section/Document/CrossDomain stage)" },
                    "criterion_id": { "type": "string", "description": "Finding criterion ID to update" },
                    "status": { "type": "string", "enum": ["open", "fixed", "accepted", "ignored", "false_positive"], "description": "New finding status" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["report_id", "criterion_id", "status"]
            }
        }),
        // ── Fix Plan Tools ──────────────────────────────────────────────────
        // Work for a finding from ANY audit source (domain audit, pipeline,
        // or semantic stage) — 'report_id'/'report_type' here are bookkeeping
        // tags on the fix session, not a foreign key, so any string/number
        // identifying where the finding came from is valid (e.g. the domain
        // name and 0 for a plain `audit` call, or a stored pipeline report_id).
        serde_json::json!({
            "name": "audit_fix_plan",
            "description": "Generate a fix plan for one finding, for human review — does not modify files. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "finding": { "type": "object", "description": "AuditFinding JSON object, e.g. one entry from an audit response's findings[] array" },
                    "domain": { "type": "string", "description": "Domain the finding belongs to" },
                    "report_id": { "type": "integer", "description": "Bookkeeping id for the source report (not validated against a table)" },
                    "report_type": { "type": "string", "description": "Bookkeeping label for the source report, e.g. 'doc', 'architecture', 'documentation-structure', 'section'" },
                    "target_path": { "type": "string", "description": "File the fix should target" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["finding", "domain", "report_id", "report_type", "target_path"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_apply",
            "description": "Run the full fix pipeline for one finding: plan, execute, verify, retry up to the configured max attempts. Modifies files. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "finding": { "type": "object", "description": "AuditFinding JSON object, e.g. one entry from an audit response's findings[] array" },
                    "domain": { "type": "string", "description": "Domain the finding belongs to" },
                    "report_id": { "type": "integer", "description": "Bookkeeping id for the source report (not validated against a table)" },
                    "report_type": { "type": "string", "description": "Bookkeeping label for the source report, e.g. 'doc', 'architecture', 'documentation-structure', 'section'" },
                    "target_path": { "type": "string", "description": "File the fix should target" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["finding", "domain", "report_id", "report_type", "target_path"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_accept",
            "description": "Shorthand for update_finding_status(status: 'fixed') — same report_id constraint applies (must be a stored SemanticReport id, not a pipeline/domain-audit report_id). Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_id": { "type": "integer", "description": "Report database ID (must be a stored SemanticReport id)" },
                    "criterion_id": { "type": "string", "description": "Finding criterion ID to mark fixed" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["report_id", "criterion_id"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_reject",
            "description": "Shorthand for update_finding_status(status: 'accepted') — same report_id constraint as audit_fix_accept. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_id": { "type": "integer", "description": "Report database ID (must be a stored SemanticReport id)" },
                    "criterion_id": { "type": "string", "description": "Finding criterion ID to mark accepted" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["report_id", "criterion_id"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_status",
            "description": "Get a fix session's status and its attempt history. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "integer", "description": "Fix session id, returned by audit_fix_apply" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["session_id"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_list",
            "description": "List fix sessions, paginated. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "Page size, default 20" },
                    "offset": { "type": "integer", "description": "Page offset" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        serde_json::json!({
            "name": "audit_fix_plan_list",
            "description": "List fix plans generated within a fix session. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "integer", "description": "Fix session id" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["session_id"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_plan_get",
            "description": "Get a single fix plan and its steps. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "plan_id": { "type": "integer", "description": "Fix plan id" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["plan_id"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_plan_render",
            "description": "Render a fix plan as markdown using a fix-plan-templates template. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "plan_id": { "type": "integer", "description": "Fix plan id" },
                    "template": { "type": "string", "description": "Template name under docs/raw/fix-plan-templates, default 'documentation'" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["plan_id"]
            }
        }),
        serde_json::json!({
            "name": "audit_fix_templates",
            "description": "List available fix-plan-templates. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                }
            }
        }),
        serde_json::json!({
            "name": "update_report_finding_status",
            "description": "Mark a stored pipeline-report finding (architecture/documentation-structure/build/security/consistency/coverage/help — the report_findings table) as fixed/accepted/ignored/false_positive by its row id. Note: no MCP tool currently returns that row id from a live audit call; it must be read from the registry directly (report_findings.id) until a query tool is added. Pass 'repo_path' to target a different local repository.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "finding_id": { "type": "integer", "description": "report_findings.id — the stored finding's row id" },
                    "status": { "type": "string", "enum": ["open", "fixed", "accepted", "ignored", "false_positive"], "description": "New finding status" },
                    "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
                },
                "required": ["finding_id", "status"]
            }
        }),
        // ── Project Planner Tools ─────────────────────────────────────────
        serde_json::json!({
            "name": "project_plan",
            "description": "Create a new project plan for a given use case. Generates a phasewise plan with dependency-aware phase ordering.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "case": { "type": "string", "enum": ["new_project", "docs_audit", "impl_test_audit", "build_audit"], "description": "Project goal" },
                    "title": { "type": "string", "description": "Optional plan title (defaults to case name)" }
                },
                "required": ["case"]
            }
        }),
        serde_json::json!({
            "name": "project_plan_get",
            "description": "Get a project plan with all its phases and statuses.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "Plan ID" }
                },
                "required": ["plan_id"]
            }
        }),
        serde_json::json!({
            "name": "project_plan_list",
            "description": "List all project plans with their status.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "project_plan_execute",
            "description": "Execute a phase within a project plan. Omitting phase_number executes the next pending phase.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "Plan ID" },
                    "phase_number": { "type": "integer", "description": "Optional phase number to execute (default: next pending)" }
                },
                "required": ["plan_id"]
            }
        }),
        serde_json::json!({
            "name": "project_plan_status",
            "description": "Get progress summary for a project plan.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "Plan ID" }
                },
                "required": ["plan_id"]
            }
        }),
        serde_json::json!({
            "name": "project_plan_abort",
            "description": "Abort a project plan (mark as failed).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "Plan ID" },
                    "reason": { "type": "string", "description": "Reason for abort" }
                },
                "required": ["plan_id"]
            }
        }),
    ]
}

fn check_expiry() {
    let expiry_str = option_env!("SAMGRAHA_EXPIRY");
    let Some(expiry) = expiry_str else { return };
    let now = chrono::Utc::now();
    // Try full datetime first, then date-only fallback
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expiry) {
        if now > dt {
            eprintln!("ERROR: This binary expired at {expiry} UTC. Build a new one.");
            std::process::exit(1);
        }
        return;
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(expiry, "%Y-%m-%d") {
        let expiry_date = d.and_hms_opt(23, 59, 59).unwrap();
        if now.naive_utc() > expiry_date {
            eprintln!("ERROR: This binary expired on {expiry}. Build a new one.");
            std::process::exit(1);
        }
        return;
    }
    eprintln!("Warning: SAMGRAHA_EXPIRY='{expiry}' not YYYY-MM-DD or RFC3339, ignored");
}

/// Walk up from `start` looking for an existing `.samgraha/` or `samgraha.toml`.
/// If none is found in any ancestor, `start` itself is returned uninitialized —
/// the server still boots so the `init` tool is reachable to bootstrap it;
/// every other tool that needs a compiled repo will fail with its own error.
fn discover_root() -> Result<std::path::PathBuf> {
    let start = match std::env::var_os("SAMGRAHA_REPO") {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir()?,
    };
    let mut current = Some(start.as_path());
    while let Some(dir) = current {
        if dir.join(".samgraha").is_dir() || dir.join("samgraha.toml").exists() {
            return Ok(dir.to_path_buf());
        }
        current = dir.parent();
    }
    Ok(start)
}

fn load_config(root: &Path) -> Result<SamgrahaConfig> {
    let config_path = root.join("samgraha.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: SamgrahaConfig = toml::from_str(&content)?;
        return Ok(config);
    }
    Ok(SamgrahaConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_result_wraps_payload_as_text_content() {
        let payload = serde_json::json!({ "domains": ["architecture"], "count": 1 });
        let wrapped = tool_result(&payload, false);
        assert_eq!(wrapped["isError"], false);
        let text = wrapped["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("architecture"));
        assert_eq!(wrapped["structuredContent"], payload);
    }

    #[test]
    fn tool_result_marks_errors() {
        let wrapped = tool_result(&serde_json::json!({ "error": "boom" }), true);
        assert_eq!(wrapped["isError"], true);
        assert!(wrapped["content"][0]["text"].as_str().unwrap().contains("boom"));
    }

    #[test]
    fn discover_root_honors_samgraha_repo_env_override() {
        let dir = std::env::temp_dir().join(format!(
            "samgraha-mcp-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join(".samgraha")).unwrap();

        // SAFETY: test-only, single-threaded access to this env var.
        unsafe { std::env::set_var("SAMGRAHA_REPO", &dir) };
        let result = discover_root();
        unsafe { std::env::remove_var("SAMGRAHA_REPO") };

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result.unwrap(), dir);
    }
}
