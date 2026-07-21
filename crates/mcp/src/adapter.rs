//! MCP adapter — the execution substrate's dispatch layer. Every handler
//! here either (a) records/reads repository registration
//! (`register_repository`/`unregister_repository`/`list_repositories`/
//! `repository_status`, backed by `registry.db`, unchanged from before
//! this pivot), or (b) registers/executes a knowledge standard's declared
//! usecases (`register_standard`/`run_script_step`/`prepare_semantic_step`/
//! `complete_semantic_step`, backed by `knowledge.db`'s core schema).
//!
//! Samgraha never interprets what a script computes or what a prompt
//! means — it only ever moves envelopes between the standard's own
//! scripts and whichever model is driving the calling MCP client.

use crate::protocol::{McpMessage, McpRequest, McpResponse, McpError};
use anyhow::Result;
use schemas::manifest::RepositoryManifest;
use services::registry_client::RegistryClient;
use std::path::PathBuf;
use std::sync::Arc;

pub struct McpAdapter {
    repository_root: PathBuf,
    registry: Arc<dyn RegistryClient>,
}

impl McpAdapter {
    pub fn new(repository_root: PathBuf, registry: Arc<dyn RegistryClient>) -> Self {
        Self { repository_root, registry }
    }

    pub fn notify_connect(&self) {
        tracing::info!("MCP client connected");
    }

    pub fn notify_disconnect(&self) {
        tracing::info!("MCP client disconnected");
    }

    pub fn handle_message(&self, message: McpMessage) -> McpMessage {
        let McpMessage::Request(req) = message else {
            return McpMessage::Error(McpError {
                id: None,
                code: -32600,
                message: "Expected a request".to_string(),
            });
        };

        let result = match req.method.as_str() {
            "init" => self.handle_init(&req),
            "register_repository" => self.handle_register_repository(&req),
            "unregister_repository" => self.handle_unregister_repository(&req),
            "list_repositories" => self.handle_list_repositories(),
            "repository_status" => self.handle_repository_status(),
            "register_standard" => self.handle_register_standard(&req),
            "run_script_step" => self.handle_run_script_step(&req),
            "prepare_semantic_step" => self.handle_prepare_semantic_step(&req),
            "complete_semantic_step" => self.handle_complete_semantic_step(&req),
            other => Err(anyhow::anyhow!("Unknown method: {other}")),
        };

        match result {
            Ok(value) => McpMessage::Response(McpResponse { id: req.id.clone(), result: value }),
            Err(e) => McpMessage::Error(McpError { id: Some(req.id.clone()), code: -32000, message: e.to_string() }),
        }
    }

    fn target_root(&self, req: &McpRequest) -> PathBuf {
        req.params.get("repo_path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.repository_root.clone())
    }

    fn knowledge_db_path(&self, req: &McpRequest) -> PathBuf {
        self.target_root(req).join(".samgraha").join("knowledge.db")
    }

    fn handle_init(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let root = self.target_root(req);
        let options = common::config::InitOptions {
            force: req.params.get("force").and_then(|v| v.as_bool()).unwrap_or(false),
            standard_system: req.params.get("standard_system").and_then(|v| v.as_str()).map(String::from),
            auto_detect_dirs: req.params.get("auto_detect").and_then(|v| v.as_bool()).unwrap_or(false),
            ..Default::default()
        };
        let result = services::init_repository(&root, &options)?;
        Ok(serde_json::json!({
            "status": result.status,
            "root": result.root.display().to_string(),
            "env_path": result.env_path.display().to_string(),
        }))
    }

    fn handle_register_repository(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let manifest_str = req.params.get("manifest")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'manifest' parameter (JSON string)"))?;
        let manifest: RepositoryManifest = serde_json::from_str(manifest_str)?;
        self.registry.register(&manifest)?;
        Ok(serde_json::json!({
            "success": true,
            "action": "register",
            "repository": manifest.repository.id,
            "uuid": manifest.repository.uuid.to_string(),
        }))
    }

    fn handle_unregister_repository(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let uuid_str = req.params.get("uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'uuid' parameter"))?;
        let uuid = uuid::Uuid::parse_str(uuid_str)?;
        self.registry.unregister(&uuid)?;
        Ok(serde_json::json!({ "success": true, "action": "unregister", "uuid": uuid_str }))
    }

    fn handle_list_repositories(&self) -> Result<serde_json::Value> {
        let entries = self.registry.list()?;
        Ok(serde_json::json!({
            "repositories": entries.iter().map(|e| serde_json::json!({
                "id": e.repository.id,
                "uuid": e.repository.uuid.to_string(),
                "repository_root": e.repository_root,
            })).collect::<Vec<_>>(),
            "count": entries.len(),
        }))
    }

    fn handle_repository_status(&self) -> Result<serde_json::Value> {
        let entries = self.registry.list()?;
        Ok(serde_json::json!({ "registered_count": entries.len() }))
    }

    fn handle_register_standard(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let path_str = req.params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let path = PathBuf::from(path_str);
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }
        let db_path = self.knowledge_db_path(req);
        let result = services::register_standard::register_standard(&path, &db_path)?;
        Ok(serde_json::to_value(result)?)
    }

    fn handle_run_script_step(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let step_id = req.params.get("step_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'step_id' parameter"))?;
        let root = self.target_root(req);
        let db_path = self.knowledge_db_path(req);
        let input = req.params.get("input").cloned().unwrap_or_else(|| serde_json::json!({}));
        let timeout = req.params.get("timeout_secs").and_then(|v| v.as_u64());
        services::step_execution::run_script_step(&db_path, step_id, &root, &input, timeout)
    }

    fn handle_prepare_semantic_step(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let step_id = req.params.get("step_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'step_id' parameter"))?;
        let db_path = self.knowledge_db_path(req);
        let prep = services::step_execution::prepare_semantic_step(&db_path, step_id)?;
        Ok(serde_json::to_value(prep)?)
    }

    fn handle_complete_semantic_step(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let step_id = req.params.get("step_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'step_id' parameter"))?;
        let status = req.params.get("status").and_then(|v| v.as_str()).unwrap_or("ok");
        let root = self.target_root(req);
        let db_path = self.knowledge_db_path(req);
        services::step_execution::complete_semantic_step(&db_path, step_id, &root, status)?;
        Ok(serde_json::json!({ "recorded": true, "step_id": step_id, "status": status }))
    }
}
