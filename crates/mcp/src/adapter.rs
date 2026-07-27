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
use anyhow::{Context, Result};
use common::config::SamgrahaConfig;
use registry::standards_db::StandardsDb;
use schemas::manifest::RepositoryManifest;
use services::registry_client::RegistryClient;
use std::path::PathBuf;
use std::sync::Arc;

pub struct McpAdapter {
    repository_root: PathBuf,
    registry: Arc<dyn RegistryClient>,
    standards_db: Arc<StandardsDb>,
}

impl McpAdapter {
    pub fn new(repository_root: PathBuf, registry: Arc<dyn RegistryClient>, standards_db: Arc<StandardsDb>) -> Self {
        Self { repository_root, registry, standards_db }
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
            "register_standard_globally" => self.handle_register_standard_globally(&req),
            "list_standards" => self.handle_list_standards(&req),
            "get_standard_info" => self.handle_get_standard_info(&req),
            "get_standard_usecases" => self.handle_get_standard_usecases(&req),
            "get_standard_scripts" => self.handle_get_standard_scripts(&req),
            "get_standard_prompts" => self.handle_get_standard_prompts(&req),
            "get_standard_assets" => self.handle_get_standard_assets(&req),
            "seed_standard" => self.handle_seed_standard(&req),
            "check_usecase_complete" => self.handle_check_usecase_complete(&req),
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
        let target = self.target_root(req);
        let samgraha_dir = load_samgraha_dir(&target);
        samgraha_dir.join("knowledge.db")
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
        // §3.9 — per-repo activation: standard must already be globally registered.
        // Accept `standard_name` (new) or `path` (legacy — resolve name from manifest).
        let standard_name = if let Some(name) = req.params.get("standard_name").and_then(|v| v.as_str()) {
            name.to_string()
        } else if let Some(path_str) = req.params.get("path").and_then(|v| v.as_str()) {
            let path = PathBuf::from(path_str);
            if !path.exists() {
                return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
            }
            let manifest_path = services::register_standard::resolve_manifest_path(&path)?;
            let manifest_content = std::fs::read_to_string(&manifest_path)?;
            let raw: serde_yaml::Mapping = serde_yaml::from_str(&manifest_content)?;
            raw.get(&serde_yaml::Value::String("name".to_string()))
                .and_then(|v| v.as_str().map(String::from))
                .ok_or_else(|| anyhow::anyhow!("Manifest missing 'name' field"))?
        } else {
            return Err(anyhow::anyhow!("Missing 'standard_name' or 'path' parameter"));
        };

        // Look up global registry to get source_path
        let global = self.standards_db.get_standard(&standard_name)?
            .ok_or_else(|| anyhow::anyhow!("Standard '{}' is not globally registered — call register_standard_globally first", standard_name))?;

        let root = self.target_root(req);
        let samgraha_dir = load_samgraha_dir(&root);
        let db_path = self.knowledge_db_path(req);
        let timeout = req.params.get("timeout_secs").and_then(|v| v.as_u64());

        services::register_standard::activate_standard(
            &standard_name,
            &global.source_path,
            &db_path,
            &root,
            &samgraha_dir,
            timeout,
        )?;

        // Record which standard is now active for this repo (§3.9 step 6,
        // relocated) — one standard per repo at a time, tracked in
        // registry.db, not duplicated inside knowledge.db. Sourced from
        // `global` (already fetched above) — no local YAML re-parse.
        let registry_db = registry::registry_db::RegistryDb::open(&root)?;
        registry_db.set_active_standard(&registry::registry_db::ActiveStandard {
            name: standard_name.clone(),
            category: global.category.clone(),
            subcategory: global.subcategory.clone(),
            extends: global.extends.clone(),
            version: global.version.clone(),
            metadata_json: global.metadata_json.clone(),
            activated_at: String::new(),
        })?;

        Ok(serde_json::json!({
            "success": true,
            "standard": standard_name,
            "samgraha_dir": samgraha_dir.display().to_string(),
        }))
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

    fn handle_register_standard_globally(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let path_str = req.params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let path = PathBuf::from(path_str);
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }

        // §3.7 step 1 — copy into mcp_dir()/registry/<name>/ atomically
        let manifest_path = services::register_standard::resolve_manifest_path(&path)?;
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let raw: serde_yaml::Mapping = serde_yaml::from_str(&manifest_content)?;
        let get_str = |key: &str| -> Option<String> {
            raw.get(&serde_yaml::Value::String(key.to_string()))
                .and_then(|v| v.as_str().map(String::from))
        };
        let name = get_str("name").ok_or_else(|| anyhow::anyhow!("Manifest missing 'name' field"))?;
        let category = get_str("category").unwrap_or_default();
        let subcategory = get_str("subcategory");
        let extends = get_str("extends");
        let version = get_str("version").unwrap_or_else(|| "0.0.0".to_string());
        let description = get_str("description").unwrap_or_default();

        // §3.13 — detect re-registration for operation_log distinction
        let is_update = self.standards_db.get_standard(&name)?.is_some();

        let registry_dir = common::env::mcp_dir().join("registry").join(&name);
        // Atomic copy: temp dir → remove old → rename. On failure, old
        // target stays untouched (§3.7, §1.6).
        common::fs_sync::copy_dir_atomic(&path, &registry_dir, &common::fs_sync::DEFAULT_EXCLUDES)?;

        // §3.7 step 2 — run verify-gate against the copy, not the original
        let verify_status = if let Some(smoke_test) = get_str("smoke_test") {
            let smoke_path = services::register_standard::resolve_location(&registry_dir, &smoke_test)?;
            let status = std::process::Command::new(&smoke_path)
                .arg("--repo-root")
                .arg(&registry_dir)
                .status();
            match status {
                Ok(s) if s.success() => "passed",
                Ok(_) => "failed",
                Err(_) => "failed",
            }
        } else {
            "unverified"
        };

        // §3.7 cleanup-on-failure: if verify or upsert fails, remove the
        // directory we just wrote — leave exactly the state we found.
        if verify_status == "failed" {
            let _ = std::fs::remove_dir_all(&registry_dir);
            self.standards_db.log_operation(
                if is_update { "update_standard" } else { "register_globally" },
                &name,
                None,
                "global",
                "failed",
                &serde_json::json!({"path": path_str, "verify_status": "failed", "reason": "smoke_test failed"}).to_string(),
            )?;
            return Ok(serde_json::json!({
                "success": false,
                "standard": name,
                "verify_status": "failed",
                "error": "structural verify-gate (smoke_test) failed against the copied standard",
            }));
        }

        // §8.4 — catch-all metadata: diff raw keys against KNOWN_FIELDS
        const KNOWN_FIELDS: &[&str] = &[
            "name", "category", "subcategory", "extends", "smoke_test", "seeder_script",
            "scripts", "prompts", "usecases", "custom_tables", "domains", "assets", "templates",
        ];
        let extra: serde_json::Map<String, serde_json::Value> = raw.iter()
            .filter_map(|(k, v)| {
                let key = k.as_str()?.to_string();
                if KNOWN_FIELDS.contains(&key.as_str()) { return None; }
                let json_v = serde_json::to_value(v).ok()?;
                Some((key, json_v))
            })
            .collect();
        let metadata_json = serde_json::Value::Object(extra).to_string();

        // §3.7 step 3 — upsert with source_path pointing at the local copy
        let source_path = registry_dir.display().to_string();
        let upsert_result = self.standards_db.upsert_standard(
            &name,
            &category,
            subcategory.as_deref(),
            &source_path,
            false,
            extends.as_deref(),
            &version,
            &description,
            &metadata_json,
            verify_status,
        );

        if let Err(e) = upsert_result {
            let _ = std::fs::remove_dir_all(&registry_dir);
            return Err(e);
        }

        self.standards_db.log_operation(
            if is_update { "update_standard" } else { "register_globally" },
            &name,
            None,
            "global",
            "ok",
            &serde_json::json!({"path": path_str, "source_path": source_path, "verify_status": verify_status}).to_string(),
        )?;

        Ok(serde_json::json!({
            "success": true,
            "standard": name,
            "verify_status": verify_status,
            "source_path": source_path,
        }))
    }

    fn handle_list_standards(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let category = req.params.get("category").and_then(|v| v.as_str());
        let subcategory = req.params.get("subcategory").and_then(|v| v.as_str());
        let standards = self.standards_db.list_standards(category, subcategory)?;
        Ok(serde_json::json!({
            "standards": standards.iter().map(|s| serde_json::json!({
                "name": s.name,
                "category": s.category,
                "subcategory": s.subcategory,
                "version": s.version,
                "verify_status": s.verify_status,
            })).collect::<Vec<_>>(),
            "count": standards.len(),
        }))
    }

    fn handle_get_standard_info(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let name = req.params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        let row = self.standards_db.get_standard(name)?
            .ok_or_else(|| anyhow::anyhow!("Standard '{}' not found in global registry", name))?;
        Ok(serde_json::json!({
            "name": row.name,
            "category": row.category,
            "subcategory": row.subcategory,
            "source_path": row.source_path,
            "is_abstract": row.is_abstract,
            "extends": row.extends,
            "version": row.version,
            "description": row.description,
            "metadata_json": row.metadata_json,
            "verify_status": row.verify_status,
            "verified_at": row.verified_at,
            "registered_at": row.registered_at,
            "updated_at": row.updated_at,
        }))
    }

    fn handle_get_standard_usecases(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let name = req.params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        let db_path = self.knowledge_db_path(req);
        let conn = rusqlite::Connection::open(&db_path)?;
        let mut stmt = conn.prepare(
            "SELECT u.id, u.name, u.description, u.data, u.domain_id,
                    d.key as domain_key
             FROM usecase u
             LEFT JOIN domain d ON d.id = u.domain_id
             WHERE u.standard = ?1
             ORDER BY u.id",
        )?;
        let rows = stmt.query_map(rusqlite::params![name], |row| {
            let data_str: String = row.get(3)?;
            let data: serde_json::Value = serde_json::from_str(&data_str).unwrap_or(serde_json::json!({}));
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "data": data,
                "domain": row.get::<_, Option<String>>(5)?,
            }))
        })?;
        let mut usecases = Vec::new();
        for row in rows {
            usecases.push(row?);
        }
        Ok(serde_json::json!({
            "standard": name,
            "usecases": usecases,
            "count": usecases.len(),
        }))
    }

    fn handle_get_standard_scripts(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let name = req.params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        let db_path = self.knowledge_db_path(req);
        let conn = rusqlite::Connection::open(&db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, location, purpose FROM script WHERE standard = ?1 ORDER BY name",
        )?;
        let rows = stmt.query_map(rusqlite::params![name], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "location": row.get::<_, String>(2)?,
                "purpose": row.get::<_, String>(3)?,
            }))
        })?;
        let mut scripts = Vec::new();
        for row in rows {
            scripts.push(row?);
        }
        Ok(serde_json::json!({
            "standard": name,
            "scripts": scripts,
            "count": scripts.len(),
        }))
    }

    fn handle_get_standard_prompts(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let name = req.params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        let db_path = self.knowledge_db_path(req);
        let conn = rusqlite::Connection::open(&db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, purpose FROM prompt WHERE standard = ?1 ORDER BY name",
        )?;
        let rows = stmt.query_map(rusqlite::params![name], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "purpose": row.get::<_, String>(2)?,
            }))
        })?;
        let mut prompts = Vec::new();
        for row in rows {
            prompts.push(row?);
        }
        Ok(serde_json::json!({
            "standard": name,
            "prompts": prompts,
            "count": prompts.len(),
        }))
    }

    fn handle_get_standard_assets(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let name = req.params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        let kind_filter = req.params.get("kind").and_then(|v| v.as_str());
        let db_path = self.knowledge_db_path(req);
        let conn = rusqlite::Connection::open(&db_path)?;
        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match kind_filter {
            Some(k) => (
                "SELECT id, kind, name, location, purpose FROM standard_asset WHERE standard = ?1 AND kind = ?2 ORDER BY kind, name",
                vec![Box::new(name.to_string()), Box::new(k.to_string())],
            ),
            None => (
                "SELECT id, kind, name, location, purpose FROM standard_asset WHERE standard = ?1 ORDER BY kind, name",
                vec![Box::new(name.to_string())],
            ),
        };
        let mut stmt = conn.prepare(sql)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "kind": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "location": row.get::<_, String>(3)?,
                "purpose": row.get::<_, String>(4)?,
            }))
        })?;
        let mut assets = Vec::new();
        for row in rows {
            assets.push(row?);
        }
        Ok(serde_json::json!({
            "standard": name,
            "assets": assets,
            "count": assets.len(),
        }))
    }

    fn handle_seed_standard(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let name = req.params.get("standard")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'standard' parameter"))?;
        let root = self.target_root(req);
        let db_path = self.knowledge_db_path(req);
        let usecase_filter = req.params.get("usecase").and_then(|v| v.as_str());

        let result = services::seed_standard::seed_standard(&db_path, name, &root, usecase_filter)?;

        self.standards_db.log_operation(
            "seed",
            name,
            Some(&root.display().to_string()),
            "repo",
            "ok",
            &serde_json::json!({"executed": result.executed.len()}).to_string(),
        )?;

        Ok(serde_json::to_value(result)?)
    }

    fn handle_check_usecase_complete(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let usecase_name = req.params.get("usecase")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'usecase' parameter"))?;
        let standard_name = req.params.get("standard")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'standard' parameter — required to disambiguate usecase name"))?;
        let root = self.target_root(req);
        let db_path = self.knowledge_db_path(req);
        let extra_args: Vec<String> = req.params.get("extra_args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Look up the standard's source_path from the global registry
        let registry_row = self.standards_db.get_standard(standard_name)?
            .ok_or_else(|| anyhow::anyhow!("Standard '{}' not found in global registry", standard_name))?;
        let standard_source = PathBuf::from(&registry_row.source_path);

        let conn = rusqlite::Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        // §8.5 — find the usecase with standard filter (usecase names are only unique per standard)
        let data_str: String = conn.query_row(
            "SELECT data FROM usecase WHERE standard = ?1 AND name = ?2",
            rusqlite::params![standard_name, usecase_name],
            |row| row.get(0),
        ).context(format!("No usecase '{}' in standard '{}'", usecase_name, standard_name))?;
        let data: serde_json::Value = serde_json::from_str(&data_str).unwrap_or(serde_json::json!({}));
        let verify_script = data.get("verify_script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Usecase '{}' has no verify_script declared", usecase_name))?;

        // §2.4.2 — resolve verify_script against the standard's own source directory,
        // not the target consuming repo
        let script_path = services::register_standard::resolve_location(&standard_source, verify_script)?;
        let script_path = std::path::PathBuf::from(&script_path);
        if !script_path.exists() {
            return Ok(serde_json::json!({"complete": false, "error": format!("Verify script not found: {}", script_path.display())}));
        }

        let mut cmd = std::process::Command::new(&script_path);
        cmd.arg("--repo-root").arg(&root);
        for arg in &extra_args {
            cmd.arg(arg);
        }
        let status = cmd.status()?;
        Ok(serde_json::json!({
            "complete": status.success(),
            "exit_code": status.code(),
        }))
    }
}

/// Load `samgraha_dir` from `root/samgraha.toml`. Falls back to the default
/// (`<root>/.samgraha`) if the config is absent or unparseable — the adapter
/// must never fail to locate `knowledge.db` because of a missing config key.
fn load_samgraha_dir(root: &std::path::Path) -> PathBuf {
    let config_path = root.join("samgraha.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = toml::from_str::<SamgrahaConfig>(&content) {
            return config.repository.resolve_samgraha_dir(root);
        }
    }
    SamgrahaConfig::default().repository.resolve_samgraha_dir(root)
}
