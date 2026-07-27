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
            "validate_standard_metadata" => self.handle_validate_standard_metadata(&req),
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

        // Note the previously active standard (if any, and if different)
        // *before* touching anything — its cleanup only happens after the
        // new standard's activation has fully succeeded, below. Cleaning
        // it up first would mean a failed activate_standard (seeder
        // error, Layer A audit failure, etc.) leaves the repo with
        // neither the old standard's data nor the new one's — worse than
        // the state before this call started, and a direct violation of
        // the "leave exactly the state you found on failure" discipline
        // every other step in this flow already follows.
        let registry_db = registry::registry_db::RegistryDb::open(&root)?;
        let previous = registry_db.get_active_standard().ok().flatten()
            .filter(|old| old.name != standard_name);

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
        // registry.db's active_standard is the single source of truth for
        // "which standard is active" — deliberately not mirrored into
        // samgraha.toml (that file is committed policy an operator edits;
        // this is runtime state an MCP call changes, and duplicating it
        // would just reintroduce the drift-prone second-source problem
        // this table exists to avoid).
        registry_db.set_active_standard(&registry::registry_db::ActiveStandard {
            name: standard_name.clone(),
            category: global.category.clone(),
            subcategory: global.subcategory.clone(),
            extends: global.extends.clone(),
            version: global.version.clone(),
            metadata_json: global.metadata_json.clone(),
            activated_at: String::new(),
        })?;

        // Now that the new standard is fully active, clean up whatever
        // standard was active before it — its local copy directory and
        // its DB rows (artifact/artifact_type excepted — historical
        // output, never deleted, same as every other reseed).
        if let Some(old) = previous {
            let old_dir = samgraha_dir.join(&old.name);
            if old_dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(&old_dir) {
                    tracing::warn!("cleanup: failed to remove previous standard's local copy at {}: {e}", old_dir.display());
                }
            }
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                if let Err(e) = services::register_standard::delete_existing(&conn, &old.name) {
                    tracing::warn!("cleanup: failed to delete previous standard '{}' rows: {e}", old.name);
                }
            }
        }

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

        // §3.7 step 1 — copy into mcp_dir()/registry/<category>/<name>/
        // atomically. Category nests the registry directory the same way
        // the real standards corpus (Kriti/samgraha/system/<category>/<name>/)
        // already does on disk — verified directly: no standard.yaml in
        // that corpus declares a `category:` field at all, every one of
        // them communicates category purely through where its directory
        // sits. So `category:` in the manifest is an override if present,
        // never the only source — the fallback infers it from `path`'s
        // own parent directory name, matching what's actually on disk
        // today. `subcategory` stays DB-only metadata (no folder for it
        // anywhere in the real corpus either — e.g. dev/react_dev vs
        // dev/fastapi_dev are distinguished by name, not a frontend/
        // backend subfolder).
        let manifest_path = services::register_standard::resolve_manifest_path(&path)?;
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let raw: serde_yaml::Mapping = serde_yaml::from_str(&manifest_content)?;
        let get_str = |key: &str| -> Option<String> {
            raw.get(&serde_yaml::Value::String(key.to_string()))
                .and_then(|v| v.as_str().map(String::from))
        };
        let name = get_str("name").ok_or_else(|| anyhow::anyhow!("Manifest missing 'name' field"))?;
        let category = get_str("category").unwrap_or_else(|| {
            path.parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        });
        let subcategory = get_str("subcategory");
        let extends = get_str("extends");
        let version = get_str("version").unwrap_or_else(|| "0.0.0".to_string());
        let description = get_str("description").unwrap_or_default();

        // §3.13 — detect re-registration; also capture the *previous*
        // source_path so a category change (this name registering under
        // a different category than last time) doesn't leave the old
        // nested directory orphaned — cleaned up after the new
        // registration succeeds, §ceanup-below.
        let previous = self.standards_db.get_standard(&name)?;
        let is_update = previous.is_some();
        let previous_registry_dir = previous.map(|row| PathBuf::from(row.source_path));

        let registry_dir = common::env::mcp_dir().join("registry").join(&category).join(&name);
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
            if let Err(e) = std::fs::remove_dir_all(&registry_dir) {
                tracing::warn!("cleanup: failed to remove registry dir at {}: {e}", registry_dir.display());
            }
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

        // Standard metadata contract — validate standard.metadata.json
        // against metadata/standard.metadata.schema.json if the standard
        // ships one (optional: a standard with no custom tables,
        // templates, or proposal generation doesn't need it). Same
        // cleanup-on-failure discipline as the smoke_test gate above.
        let metadata_path = registry_dir.join("standard.metadata.json");
        if metadata_path.exists() {
            let metadata = match services::metadata_validate::load_and_validate_metadata(&metadata_path) {
                Ok(m) => m,
                Err(e) => {
                    if let Err(e2) = std::fs::remove_dir_all(&registry_dir) {
                        tracing::warn!("cleanup: failed to remove registry dir at {}: {e2}", registry_dir.display());
                    }
                    self.standards_db.log_operation(
                        if is_update { "update_standard" } else { "register_globally" },
                        &name,
                        None,
                        "global",
                        "failed",
                        &serde_json::json!({"path": path_str, "reason": format!("standard.metadata.json invalid: {e}")}).to_string(),
                    )?;
                    return Ok(serde_json::json!({
                        "success": false,
                        "standard": name,
                        "error": format!("standard.metadata.json failed validation: {e}"),
                    }));
                }
            };
            // Also validate proposal_template consistency: if set, it must
            // name a template with role='proposal'.
            if let Err(e) = services::metadata_validate::validate_proposal_template_consistency(&metadata) {
                if let Err(e2) = std::fs::remove_dir_all(&registry_dir) {
                    tracing::warn!("cleanup: failed to remove registry dir at {}: {e2}", registry_dir.display());
                }
                self.standards_db.log_operation(
                    if is_update { "update_standard" } else { "register_globally" },
                    &name,
                    None,
                    "global",
                    "failed",
                    &serde_json::json!({"path": path_str, "reason": format!("proposal_template inconsistency: {e}")}).to_string(),
                )?;
                return Ok(serde_json::json!({
                    "success": false,
                    "standard": name,
                    "error": format!("proposal_template inconsistency: {e}"),
                }));
            }
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
            if let Err(e2) = std::fs::remove_dir_all(&registry_dir) {
                tracing::warn!("cleanup: failed to remove registry dir at {}: {e2}", registry_dir.display());
            }
            return Err(e);
        }

        // Category changed since the previous registration of this name
        // — the old nested directory is now orphaned (a different path
        // under registry/), remove it. Only after the new registration
        // fully succeeded — a failure above leaves the previous
        // registration, at its previous path, completely untouched.
        if let Some(old_dir) = previous_registry_dir {
            if old_dir != registry_dir {
                if let Err(e) = std::fs::remove_dir_all(&old_dir) {
                    tracing::warn!("cleanup: failed to remove old category dir at {}: {e}", old_dir.display());
                }
            }
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
            let location: String = row.get(2)?;
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "location": location,
                "purpose": row.get::<_, String>(3)?,
                "file_exists": std::path::Path::new(&location).exists(),
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
        // `standard_asset.kind` was replaced by `kind_id` (FK into
        // asset_kind) when the free-text kind/type columns became
        // relations — this handler still needs the kind *name* for
        // callers, so it joins back to asset_kind rather than exposing
        // the raw id.
        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match kind_filter {
            Some(k) => (
                "SELECT sa.id, ak.name, sa.name, sa.location, sa.purpose \
                 FROM standard_asset sa JOIN asset_kind ak ON ak.id = sa.kind_id \
                 WHERE sa.standard = ?1 AND ak.name = ?2 ORDER BY ak.name, sa.name",
                vec![Box::new(name.to_string()), Box::new(k.to_string())],
            ),
            None => (
                "SELECT sa.id, ak.name, sa.name, sa.location, sa.purpose \
                 FROM standard_asset sa JOIN asset_kind ak ON ak.id = sa.kind_id \
                 WHERE sa.standard = ?1 ORDER BY ak.name, sa.name",
                vec![Box::new(name.to_string())],
            ),
        };
        let mut stmt = conn.prepare(sql)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            let location: String = row.get(3)?;
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "kind": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "location": location,
                "purpose": row.get::<_, String>(4)?,
                "file_exists": std::path::Path::new(&location).exists(),
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

    fn handle_validate_standard_metadata(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let standard_name = req.params.get("standard")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'standard' parameter"))?;
        let root = self.target_root(req);
        let samgraha_dir = load_samgraha_dir(&root);
        let local_copy = samgraha_dir.join(standard_name);
        let metadata_path = local_copy.join("standard.metadata.json");

        let mut result = serde_json::json!({
            "standard": standard_name,
            "layer_a": null,
            "layer_b": null,
        });

        // Layer B — JSON Schema validation (if metadata file exists)
        if metadata_path.exists() {
            match services::metadata_validate::load_and_validate_metadata(&metadata_path) {
                Ok(metadata) => {
                    // Also check proposal_template consistency
                    let consistency = services::metadata_validate::validate_proposal_template_consistency(&metadata);
                    result["layer_b"] = serde_json::json!({
                        "valid": consistency.is_ok(),
                        "schema_validation": "passed",
                        "proposal_template_consistency": if consistency.is_ok() { "passed" } else { "failed" },
                        "error": consistency.err().map(|e| e.to_string()),
                    });
                }
                Err(e) => {
                    result["layer_b"] = serde_json::json!({
                        "valid": false,
                        "schema_validation": "failed",
                        "error": e.to_string(),
                    });
                }
            }
        } else {
            result["layer_b"] = serde_json::json!({
                "valid": true,
                "schema_validation": "skipped — no standard.metadata.json",
            });
        }

        // Layer A — structural completeness audit (if knowledge.db has rows for this standard)
        let db_path = self.knowledge_db_path(req);
        if db_path.exists() {
            let conn = rusqlite::Connection::open(&db_path)?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            let has_data: bool = conn.query_row(
                "SELECT COUNT(*) FROM usecase WHERE standard = ?1",
                rusqlite::params![standard_name],
                |r| r.get::<_, i64>(0),
            ).unwrap_or(0) > 0;

            if has_data {
                match services::layer_a_audit::run_layer_a_audit(&conn, standard_name) {
                    Ok(()) => {
                        result["layer_a"] = serde_json::json!({
                            "valid": true,
                        });
                    }
                    Err(e) => {
                        result["layer_a"] = serde_json::json!({
                            "valid": false,
                            "error": e.to_string(),
                        });
                    }
                }
            } else {
                result["layer_a"] = serde_json::json!({
                    "valid": true,
                    "note": "no usecase rows for this standard — audit skipped",
                });
            }
        } else {
            result["layer_a"] = serde_json::json!({
                "valid": true,
                "note": "knowledge.db does not exist — standard not activated",
            });
        }

        let both_valid = result["layer_a"]["valid"].as_bool().unwrap_or(true)
            && result["layer_b"]["valid"].as_bool().unwrap_or(true);
        result["valid"] = serde_json::json!(both_valid);

        Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;
    use services::registry_client::FileRegistryClient;

    /// `SAMGRAHA_MCP_DIR` (read by `common::env::mcp_dir()`) is a
    /// process-wide env var — this test owns it exclusively (no other
    /// test in this codebase sets it) and runs every scenario that needs
    /// it (metadata success, metadata rejection, category-change
    /// cleanup) sequentially in this one function, rather than separate
    /// `#[test]`s, to avoid a race if cargo runs tests in this binary
    /// concurrently against different tempdirs — confirmed this isn't
    /// theoretical: a second, separate test setting the same var failed
    /// intermittently under the full suite.
    #[test]
    fn handle_register_standard_globally_validates_metadata() {
        let tmp = tempfile::tempdir().unwrap();
        let mcp_home = tmp.path().join("mcp-home");
        std::fs::create_dir_all(&mcp_home).unwrap();
        std::env::set_var("SAMGRAHA_MCP_DIR", &mcp_home);

        let registry: Arc<dyn RegistryClient> = Arc::new(FileRegistryClient::new(tmp.path()));
        let standards_db = Arc::new(StandardsDb::open_in_memory().unwrap());
        let adapter = McpAdapter::new(tmp.path().to_path_buf(), registry, standards_db);

        // Source laid out the same way the real standards corpus is —
        // system/<category>/<name>/ — with no `category:` field in
        // standard.yaml, so category must be inferred from the source
        // path's own parent directory name.
        let good_dir = tmp.path().join("system").join("dev").join("good-std");
        std::fs::create_dir_all(&good_dir).unwrap();
        std::fs::write(good_dir.join("standard.yaml"), "name: good-std\n").unwrap();
        std::fs::write(
            good_dir.join("standard.metadata.json"),
            r#"{"custom_tables":[{"name":"t1","purpose":"p"}]}"#,
        ).unwrap();
        let req = McpRequest {
            id: "1".into(),
            method: "register_standard_globally".into(),
            params: [("path".to_string(), serde_json::json!(good_dir.display().to_string()))].into_iter().collect(),
            repo: None,
        };
        let result = adapter.handle_register_standard_globally(&req).unwrap();
        assert_eq!(result["success"], serde_json::json!(true), "got: {result}");
        let row = adapter.standards_db.get_standard("good-std").unwrap().unwrap();
        assert_eq!(row.category, "dev", "category should be inferred from the source path's parent dir");
        assert!(
            mcp_home.join("registry").join("dev").join("good-std").join("standard.yaml").exists(),
            "registry copy should be nested under registry/<category>/<name>/, matching the real standards corpus layout"
        );

        // Invalid standard.metadata.json (schema violation: unknown field,
        // additionalProperties:false) → rejected, nothing upserted, the
        // directory register_standard_globally copied is cleaned up.
        // This is exactly the embedded-schema fix's real integration
        // point — no fixture faking a file at the wrong path this time.
        let bad_dir = tmp.path().join("system").join("hackathon").join("bad-std");
        std::fs::create_dir_all(&bad_dir).unwrap();
        std::fs::write(bad_dir.join("standard.yaml"), "name: bad-std\n").unwrap();
        std::fs::write(bad_dir.join("standard.metadata.json"), r#"{"not_a_real_field": true}"#).unwrap();
        let req = McpRequest {
            id: "2".into(),
            method: "register_standard_globally".into(),
            params: [("path".to_string(), serde_json::json!(bad_dir.display().to_string()))].into_iter().collect(),
            repo: None,
        };
        let result = adapter.handle_register_standard_globally(&req).unwrap();
        assert_eq!(result["success"], serde_json::json!(false), "got: {result}");
        assert!(adapter.standards_db.get_standard("bad-std").unwrap().is_none());
        assert!(
            !mcp_home.join("registry").join("hackathon").join("bad-std").exists(),
            "cleanup-on-failure should remove the copied registry directory"
        );

        // Third phase, same adapter/tempdir — re-registering a name under
        // a *different* category than it was previously registered
        // under must move the registry directory, not leave the old one
        // orphaned. Kept in this same test function (not a separate
        // #[test]) because SAMGRAHA_MCP_DIR is a process-wide env var —
        // a second test setting/unsetting it independently races this
        // one if cargo runs them concurrently (confirmed: an earlier,
        // separate version of this scenario failed intermittently under
        // the full suite while passing every time in isolation).
        let dev_dir = tmp.path().join("system").join("dev").join("movable-std");
        std::fs::create_dir_all(&dev_dir).unwrap();
        std::fs::write(dev_dir.join("standard.yaml"), "name: movable-std\n").unwrap();
        let req = McpRequest {
            id: "3".into(),
            method: "register_standard_globally".into(),
            params: [("path".to_string(), serde_json::json!(dev_dir.display().to_string()))].into_iter().collect(),
            repo: None,
        };
        adapter.handle_register_standard_globally(&req).unwrap();
        let old_dir = mcp_home.join("registry").join("dev").join("movable-std");
        assert!(old_dir.exists());

        // Re-register the same name from a source now living under a
        // different category.
        let hackathon_dir = tmp.path().join("system").join("hackathon").join("movable-std");
        std::fs::create_dir_all(&hackathon_dir).unwrap();
        std::fs::write(hackathon_dir.join("standard.yaml"), "name: movable-std\n").unwrap();
        let req = McpRequest {
            id: "4".into(),
            method: "register_standard_globally".into(),
            params: [("path".to_string(), serde_json::json!(hackathon_dir.display().to_string()))].into_iter().collect(),
            repo: None,
        };
        let result = adapter.handle_register_standard_globally(&req).unwrap();
        assert_eq!(result["success"], serde_json::json!(true), "got: {result}");

        let new_dir = mcp_home.join("registry").join("hackathon").join("movable-std");
        assert!(new_dir.join("standard.yaml").exists(), "new-category directory should exist");
        assert!(
            !old_dir.exists(),
            "old-category directory should be removed, not left orphaned, after the category changed"
        );
        let row = adapter.standards_db.get_standard("movable-std").unwrap().unwrap();
        assert_eq!(row.category, "hackathon");
        assert_eq!(row.source_path, new_dir.display().to_string());

        std::env::remove_var("SAMGRAHA_MCP_DIR");
    }

    #[test]
    fn handle_get_standard_assets_joins_kind_name_and_reports_file_exists() {
        // Regression test: standard_asset.kind was replaced by kind_id
        // (FK into asset_kind) when free-text kind/type columns became
        // relations — the handler's SQL still selected a `kind` column
        // that no longer existed, which would fail at `conn.prepare()`
        // time on every real call. No test exercised this handler at
        // all before, so it went unnoticed.
        let tmp = tempfile::tempdir().unwrap();
        let repo_root = tmp.path().join("repo");
        let samgraha_dir = repo_root.join(".samgraha");
        std::fs::create_dir_all(&samgraha_dir).unwrap();
        let db_path = samgraha_dir.join("knowledge.db");

        let existing_file = samgraha_dir.join("guide.md");
        std::fs::write(&existing_file, "# Guide").unwrap();

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        registry::core_schema::ensure_current_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO asset_kind (standard, name) VALUES ('t', 'guide')",
            [],
        ).unwrap();
        let kind_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO standard_asset (standard, kind_id, name, location, purpose) \
             VALUES ('t', ?1, 'setup-guide', ?2, 'onboarding')",
            rusqlite::params![kind_id, existing_file.display().to_string()],
        ).unwrap();
        conn.execute(
            "INSERT INTO standard_asset (standard, kind_id, name, location, purpose) \
             VALUES ('t', ?1, 'missing-guide', ?2, 'never written')",
            rusqlite::params![kind_id, samgraha_dir.join("nope.md").display().to_string()],
        ).unwrap();
        drop(conn);

        let registry: Arc<dyn RegistryClient> = Arc::new(FileRegistryClient::new(&repo_root));
        let standards_db = Arc::new(StandardsDb::open_in_memory().unwrap());
        let adapter = McpAdapter::new(repo_root.clone(), registry, standards_db);

        let req = McpRequest {
            id: "1".into(),
            method: "get_standard_assets".into(),
            params: [
                ("name".to_string(), serde_json::json!("t")),
                ("repo_path".to_string(), serde_json::json!(repo_root.display().to_string())),
            ].into_iter().collect(),
            repo: None,
        };
        let result = adapter.handle_get_standard_assets(&req).unwrap();
        assert_eq!(result["count"], serde_json::json!(2), "got: {result}");
        let assets = result["assets"].as_array().unwrap();
        let setup = assets.iter().find(|a| a["name"] == "setup-guide").unwrap();
        assert_eq!(setup["kind"], serde_json::json!("guide"));
        assert_eq!(setup["file_exists"], serde_json::json!(true));
        let missing = assets.iter().find(|a| a["name"] == "missing-guide").unwrap();
        assert_eq!(missing["file_exists"], serde_json::json!(false));

        // kind filter also goes through the asset_kind join, not a raw column.
        let req_filtered = McpRequest {
            id: "2".into(),
            method: "get_standard_assets".into(),
            params: [
                ("name".to_string(), serde_json::json!("t")),
                ("kind".to_string(), serde_json::json!("guide")),
                ("repo_path".to_string(), serde_json::json!(repo_root.display().to_string())),
            ].into_iter().collect(),
            repo: None,
        };
        let filtered = adapter.handle_get_standard_assets(&req_filtered).unwrap();
        assert_eq!(filtered["count"], serde_json::json!(2), "got: {filtered}");
    }

    #[test]
    fn handle_register_standard_leaves_old_standard_untouched_when_new_one_fails() {
        // Regression test for the ordering bug fixed in this pass: the
        // previously-active standard's cleanup must run only *after* the
        // new standard's activation succeeds. If it ran first (as an
        // earlier version of this code did) a failed activation of the
        // new standard would leave the repo with neither standard's
        // data — worse than the state before the call.
        let tmp = tempfile::tempdir().unwrap();
        let repo_root = tmp.path().to_path_buf();

        let std_a_dir = tmp.path().join("std-a-source");
        std::fs::create_dir_all(&std_a_dir).unwrap();
        std::fs::write(std_a_dir.join("standard.yaml"), "name: std-a\n").unwrap();

        let std_b_dir = tmp.path().join("std-b-source");
        std::fs::create_dir_all(&std_b_dir).unwrap();
        // Declares a seeder_script that doesn't exist — activate_standard
        // fails cleanly at the "resolve seeder script" step.
        std::fs::write(std_b_dir.join("standard.yaml"), "name: std-b\nseeder_script: missing.py\n").unwrap();

        let registry: Arc<dyn RegistryClient> = Arc::new(FileRegistryClient::new(&repo_root));
        let standards_db = Arc::new(StandardsDb::open_in_memory().unwrap());
        standards_db.upsert_standard(
            "std-a", "dev", None, &std_a_dir.display().to_string(),
            false, None, "1.0.0", "", "{}", "unverified",
        ).unwrap();
        standards_db.upsert_standard(
            "std-b", "dev", None, &std_b_dir.display().to_string(),
            false, None, "1.0.0", "", "{}", "unverified",
        ).unwrap();
        let adapter = McpAdapter::new(repo_root.clone(), registry, standards_db);

        let activate = |name: &str| McpRequest {
            id: "1".into(),
            method: "register_standard".into(),
            params: [("standard_name".to_string(), serde_json::json!(name))].into_iter().collect(),
            repo: None,
        };

        // Activate std-a — succeeds, becomes the active standard.
        let result = adapter.handle_register_standard(&activate("std-a")).unwrap();
        assert_eq!(result["success"], serde_json::json!(true), "got: {result}");
        let samgraha_dir = load_samgraha_dir(&repo_root);
        assert!(samgraha_dir.join("std-a").join("standard.yaml").exists());

        // Attempt to switch to std-b — its activation fails.
        let err = adapter.handle_register_standard(&activate("std-b")).unwrap_err();
        assert!(err.to_string().contains("does not exist"), "got: {err}");

        // std-a must still be fully intact — directory and active_standard row.
        assert!(
            samgraha_dir.join("std-a").join("standard.yaml").exists(),
            "std-a's local copy must survive a failed std-b activation"
        );
        let registry_db = registry::registry_db::RegistryDb::open(&repo_root).unwrap();
        let active = registry_db.get_active_standard().unwrap().unwrap();
        assert_eq!(active.name, "std-a", "active_standard must still point at std-a, not be cleared or switched");
        assert!(
            !samgraha_dir.join("std-b").exists(),
            "std-b's failed activation shouldn't leave a partial directory either"
        );
    }
}
