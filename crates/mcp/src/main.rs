use anyhow::Result;
use mcp::adapter::McpAdapter;
use mcp::protocol::{McpMessage, McpRequest};
use services::registry_client::FileRegistryClient;
use std::io::{self, BufRead, Write};
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

    let root = discover_root()?;
    let registry = Arc::new(FileRegistryClient::new(&root));
    let adapter = McpAdapter::new(root, registry);

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
            "description": "Initialize samgraha.toml and .samgraha/ for a repository. Backfills missing keys if samgraha.toml already exists.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "force": { "type": "boolean", "description": "Overwrite existing samgraha.toml with a fresh template instead of backfilling missing keys" },
                    "repo_path": { "type": "string", "description": "Absolute path to the repository to initialize (default: current repo)" },
                    "standard_system": { "type": "string", "description": "Standard system name to declare in samgraha.toml" },
                    "auto_detect": { "type": "boolean", "description": "Probe repo for docs/, src|crates/, tests/, scripts/ and set literal paths if found" }
                }
            }
        }),
        serde_json::json!({
            "name": "register_repository",
            "description": "Register a repository's manifest into the local cross-repo registry (registry.db), for other repos to resolve it as a dependency.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "manifest": { "type": "string", "description": "JSON-encoded RepositoryManifest" }
                },
                "required": ["manifest"]
            }
        }),
        serde_json::json!({
            "name": "unregister_repository",
            "description": "Remove a repository from the local registry by UUID.",
            "inputSchema": {
                "type": "object",
                "properties": { "uuid": { "type": "string" } },
                "required": ["uuid"]
            }
        }),
        serde_json::json!({
            "name": "list_repositories",
            "description": "List every repository registered in the local registry.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "repository_status",
            "description": "Summary count of registered repositories.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "register_standard",
            "description": "Register a knowledge standard's standard.yaml (usecases, steps, scripts, prompts, custom tables) into a repository's knowledge.db. Re-registering replaces the standard's prior rows entirely. Samgraha never interprets what a script computes or what a prompt means — it only catalogs names, paths, and content so they can be dispatched by name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the standard's source root (containing standard.yaml)" },
                    "repo_path": { "type": "string", "description": "Target repository whose .samgraha/knowledge.db should receive this standard (default: current repo)" }
                },
                "required": ["path"]
            }
        }),
        serde_json::json!({
            "name": "run_script_step",
            "description": "Run a usecase's kind='deterministic' step's script directly (no model involved). Runs via the fixed --repo-root/--in/--out contract and records an execution row.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "step_id": { "type": "integer", "description": "The step's row id in knowledge.db's step table" },
                    "repo_path": { "type": "string", "description": "Repository the step runs against (default: current repo) — pass a different path to fan out one step definition across multiple target repos" },
                    "input": { "type": "object", "description": "Arbitrary JSON passed to the script via --in; samgraha never interprets it" },
                    "timeout_secs": { "type": "integer" }
                },
                "required": ["step_id"]
            }
        }),
        serde_json::json!({
            "name": "prepare_semantic_step",
            "description": "Stage a usecase's kind='semantic' step for the calling agent: returns the step's mapped prompt content verbatim. The calling agent reasons over it off-MCP, then calls complete_semantic_step once done. Whatever the agent's result means (a score, generated content, anything) is decided entirely by the prompt's own content, never by samgraha.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "step_id": { "type": "integer" }
                },
                "required": ["step_id"]
            }
        }),
        serde_json::json!({
            "name": "complete_semantic_step",
            "description": "Record that a kind='semantic' step's agent-side reasoning finished. Does not persist the agent's result itself — persisting it (to a custom table, a file, anything) is the job of the next deterministic step in the usecase's sequence, run via run_script_step with the agent's result as its input.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "step_id": { "type": "integer" },
                    "repo_path": { "type": "string" },
                    "status": { "type": "string", "description": "e.g. 'ok' or 'failed' — default 'ok'" }
                },
                "required": ["step_id"]
            }
        }),
    ]
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_result_wraps_payload_as_text_content() {
        let payload = serde_json::json!({ "standard": "python_hackathon", "count": 1 });
        let wrapped = tool_result(&payload, false);
        assert_eq!(wrapped["isError"], false);
        let text = wrapped["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("python_hackathon"));
    }

    #[test]
    fn tool_definitions_covers_every_dispatched_method() {
        let names: Vec<String> = tool_definitions()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        for expected in [
            "init", "register_repository", "unregister_repository",
            "list_repositories", "repository_status", "register_standard",
            "run_script_step", "prepare_semantic_step", "complete_semantic_step",
        ] {
            assert!(names.contains(&expected.to_string()), "missing tool def for {expected}");
        }
    }
}
