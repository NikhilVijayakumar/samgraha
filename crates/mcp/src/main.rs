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
                        "version": "0.1.0"
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
                    result: Some(resp.result),
                    error: None,
                },
                McpMessage::Error(err) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: err.code,
                        message: err.message,
                    }),
                },
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Internal error".to_string(),
                    }),
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

fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
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
            "name": "search",
            "description": "Search compiled knowledge",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "level": { "type": "string", "enum": ["metadata", "summary", "section", "full"], "description": "Retrieval level" },
                    "domain": { "type": "string", "description": "Filter by domain" },
                    "limit": { "type": "integer", "description": "Max results" },
                    "offset": { "type": "integer", "description": "Result offset" }
                },
                "required": ["query"]
            }
        }),
        serde_json::json!({
            "name": "get_sections",
            "description": "Get document sections by semantic type",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "semantic_type": { "type": "string", "description": "Semantic type filter" },
                    "domain": { "type": "string", "description": "Filter by domain" },
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" }
                },
                "required": ["semantic_type"]
            }
        }),
        serde_json::json!({
            "name": "audit",
            "description": "Run audit checks on documentation",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain to audit" },
                    "providers": { "type": "array", "items": { "type": "string" }, "description": "Audit providers" }
                }
            }
        }),
        serde_json::json!({
            "name": "info",
            "description": "Get runtime information",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "get_document",
            "description": "Get document metadata and section table of contents",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "Document ID" }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "get_document_section",
            "description": "Get paginated content of a specific section",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "Document ID" },
                    "section": { "description": "Section index (integer) or heading (string)" },
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" }
                },
                "required": ["id", "section"]
            }
        }),
        serde_json::json!({
            "name": "list_domains",
            "description": "List available documentation domains",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "list_repositories",
            "description": "List registered repositories",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" }
                }
            }
        }),
        serde_json::json!({
            "name": "register_repository",
            "description": "Register a repository from its manifest JSON",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "manifest": { "type": "string", "description": "RepositoryManifest as JSON string" }
                },
                "required": ["manifest"]
            }
        }),
        serde_json::json!({
            "name": "unregister_repository",
            "description": "Unregister a repository by UUID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "uuid": { "type": "string", "description": "Repository UUID" }
                },
                "required": ["uuid"]
            }
        }),
        serde_json::json!({
            "name": "synchronize_repository",
            "description": "Synchronize dependency metadata from their manifests",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "resolve_dependencies",
            "description": "Resolve dependency graph for all registered repositories",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "repository_status",
            "description": "Get computed status of all registered repositories",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" }
                }
            }
        }),
        serde_json::json!({
            "name": "workspace_status",
            "description": "Get workspace-level status across all registered repositories",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" }
                }
            }
        }),
        // ── Semantic Audit Tools ─────────────────────────────────────────────
        serde_json::json!({
            "name": "get_documents_by_domain",
            "description": "List compiled documents in a domain",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain/standard name" },
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" }
                },
                "required": ["domain"]
            }
        }),
        serde_json::json!({
            "name": "get_section",
            "description": "Get a single section by database primary key",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "section_id": { "type": "integer", "description": "Section primary key" }
                },
                "required": ["section_id"]
            }
        }),
        serde_json::json!({
            "name": "get_audit_knowledge",
            "description": "Serve audit knowledge file content for a domain section type",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain name" },
                    "section_type": { "type": "string", "description": "Section semantic type" }
                },
                "required": ["domain", "section_type"]
            }
        }),
        serde_json::json!({
            "name": "get_audit_report",
            "description": "Get the latest audit report for a scope",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain name" },
                    "document_id": { "type": "integer", "description": "Optional document ID" },
                    "section_id": { "type": "integer", "description": "Optional section ID" },
                    "stage": { "type": "string", "enum": ["deterministic", "section", "document", "cross_domain"], "description": "Audit stage" }
                },
                "required": ["domain", "stage"]
            }
        }),
        serde_json::json!({
            "name": "get_section_changed",
            "description": "Check if a section changed since last audit (incremental skip)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "section_id": { "type": "integer", "description": "Section primary key" }
                },
                "required": ["section_id"]
            }
        }),
        serde_json::json!({
            "name": "check_gate",
            "description": "Check if a stage gate is clear before proceeding",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "stage": { "type": "string", "enum": ["deterministic", "section", "document", "cross_domain"], "description": "Audit stage to check" },
                    "document_id": { "type": "integer", "description": "Optional document ID for scoped check" }
                },
                "required": ["stage"]
            }
        }),
        serde_json::json!({
            "name": "store_section_report",
            "description": "Agent writes section audit findings; validates schema before persist",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_json": { "type": "object", "description": "SemanticReport as JSON object" }
                },
                "required": ["report_json"]
            }
        }),
        serde_json::json!({
            "name": "store_document_report",
            "description": "Agent writes document-level audit findings; validates schema",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_json": { "type": "object", "description": "SemanticReport as JSON object" }
                },
                "required": ["report_json"]
            }
        }),
        serde_json::json!({
            "name": "store_cross_domain_report",
            "description": "Agent writes cross-domain audit findings; validates schema",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_json": { "type": "object", "description": "SemanticReport as JSON object" }
                },
                "required": ["report_json"]
            }
        }),
        serde_json::json!({
            "name": "update_finding_status",
            "description": "Engineer marks a finding as Fixed / Accepted / False Positive",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "report_id": { "type": "integer", "description": "Report database ID" },
                    "criterion_id": { "type": "string", "description": "Finding criterion ID to update" },
                    "status": { "type": "string", "enum": ["open", "fixed", "accepted", "ignored", "false_positive"], "description": "New finding status" }
                },
                "required": ["report_id", "criterion_id", "status"]
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

fn discover_root() -> Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    let mut current = Some(cwd.as_path());
    while let Some(dir) = current {
        if dir.join("samgraha.toml").exists() || dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }
        current = dir.parent();
    }
    Ok(cwd)
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
