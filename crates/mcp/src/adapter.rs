use crate::protocol::{McpCapabilities, McpError, McpMessage, McpRequest, McpResponse};
use anyhow::Result;
use common::config::{parse_ttl_duration, SamgrahaConfig};
use registry::RegistryStore;
use schemas::audit::{AuditFinding, AuditStage, FindingStatus, SemanticReport};
use schemas::compilation::{CompilationRequest, CompilationScope};
use schemas::manifest::CachedRepoMetadata;
use schemas::search::{RetrievalLevel, SearchQuery, SectionQuery};
use services::compilation::CompilationService;
use services::planner::write_meta_file;
use services::project_planner::PlanOrchestrator;
use services::registry_client::{FileRegistryClient, RegistryClient};
use services::resolution::KnowledgeResolver;
use services::context_manager::ContextManager;
use services::KnowledgeRuntime;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Validate that the given path is within the repository root.
fn validate_target_path(repo_root: &Path, target: &Path) -> Result<()> {
    let canonical = target.canonicalize().map_err(|e| {
        anyhow::anyhow!("Target path '{}' cannot be resolved: {}", target.display(), e)
    })?;
    let repo_canonical = repo_root.canonicalize().map_err(|e| {
        anyhow::anyhow!("Repository root '{}' cannot be resolved: {}", repo_root.display(), e)
    })?;
    if !canonical.starts_with(&repo_canonical) {
        anyhow::bail!(
            "Path safety violation: '{}' is outside repository root '{}'",
            canonical.display(),
            repo_canonical.display()
        );
    }
    Ok(())
}

/// Load `<root>/samgraha.toml` if present, else defaults. Same fallback
/// `compile_external` already used inline — extracted so `runtime_for` can share it.
fn load_repo_config(root: &Path) -> SamgrahaConfig {
    root.join("samgraha.toml")
        .to_str()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn parse_finding(req: &McpRequest) -> Result<AuditFinding> {
    let finding_val = req.params.get("finding")
        .ok_or_else(|| anyhow::anyhow!("Missing 'finding' parameter"))?;
    serde_json::from_value(finding_val.clone())
        .map_err(|e| anyhow::anyhow!("Invalid 'finding' JSON: {}", e))
}

fn parse_string(req: &McpRequest, name: &str) -> Result<String> {
    req.params.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing '{}' parameter", name))
}

fn parse_i64(req: &McpRequest, name: &str) -> Result<i64> {
    req.params.get(name)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid '{}' parameter", name))
}

pub struct McpAdapter {
    runtime: Arc<KnowledgeRuntime>,
    registry: Arc<dyn RegistryClient>,
    capabilities: McpCapabilities,
    context_manager: ContextManager,
    orchestrator: PlanOrchestrator,
}

impl McpAdapter {
    pub fn new(runtime: Arc<KnowledgeRuntime>, registry: Arc<dyn RegistryClient>) -> Self {
        let context_manager = ContextManager::new(Duration::from_secs(300));
        context_manager.ensure("primary", &runtime.context.repository_root, &runtime.context.config);
        let mut caps = McpCapabilities::default_capabilities();
        caps.methods.push("init".to_string());
        caps.methods.push("list_repositories".to_string());
        caps.methods.push("register_repository".to_string());
        caps.methods.push("unregister_repository".to_string());
        caps.methods.push("synchronize_repository".to_string());
        caps.methods.push("resolve_dependencies".to_string());
        caps.methods.push("repository_status".to_string());
        caps.methods.push("workspace_status".to_string());
        caps.methods.push("get_product_knowledge_context".to_string());
        caps.methods.push("get_documents_by_domain".to_string());
        caps.methods.push("get_section".to_string());
        caps.methods.push("get_audit_knowledge".to_string());
        caps.methods.push("get_audit_report".to_string());
        caps.methods.push("get_section_changed".to_string());
        caps.methods.push("check_gate".to_string());
        caps.methods.push("store_section_report".to_string());
        caps.methods.push("store_document_report".to_string());
        caps.methods.push("store_cross_domain_report".to_string());
        caps.methods.push("update_finding_status".to_string());
        caps.methods.push("sync".to_string());
        caps.methods.push("get_plan".to_string());
        caps.methods.push("switch_context".to_string());
        caps.methods.push("list_contexts".to_string());
        caps.methods.push("report_templates".to_string());
        caps.methods.push("report_generate".to_string());
        caps.methods.push("report_sessions".to_string());
        caps.methods.push("audit_fix_plan".to_string());
        caps.methods.push("audit_fix_apply".to_string());
        caps.methods.push("audit_fix_accept".to_string());
        caps.methods.push("audit_fix_reject".to_string());
        caps.methods.push("audit_fix_status".to_string());
        caps.methods.push("audit_fix_list".to_string());
        caps.methods.push("audit_fix_plan_list".to_string());
        caps.methods.push("audit_fix_plan_get".to_string());
        caps.methods.push("audit_fix_plan_render".to_string());
        caps.methods.push("audit_fix_templates".to_string());
        caps.methods.push("project_plan".to_string());
        caps.methods.push("project_plan_get".to_string());
        caps.methods.push("project_plan_list".to_string());
        caps.methods.push("project_plan_execute".to_string());
        caps.methods.push("project_plan_status".to_string());
        caps.methods.push("project_plan_abort".to_string());
        let orchestrator = PlanOrchestrator::new(
            Arc::clone(&runtime),
            Arc::clone(&runtime.registry),
        );
        Self {
            runtime,
            registry,
            capabilities: caps,
            context_manager,
            orchestrator,
        }
    }

    pub fn capabilities(&self) -> &McpCapabilities {
        &self.capabilities
    }

    fn ensure_context(&self) {
        self.context_manager.ensure(
            "primary",
            &self.runtime.context.repository_root,
            &self.runtime.context.config,
        );
    }

    /// Optional `repo_path` param present on most tools: when given, build a fresh
    /// runtime rooted there instead of using the session's anchored repo. Same
    /// per-call construction `compile_external` already does for `compile`/`sync` —
    /// this just makes the same escape hatch available to every other tool, so one
    /// globally-configured MCP server can operate on any local repo, not just its own.
    fn runtime_for(&self, req: &McpRequest) -> Result<Arc<KnowledgeRuntime>> {
        match req.params.get("repo_path").and_then(|v| v.as_str()) {
            Some(p) => {
                let root = std::path::PathBuf::from(p);
                let config = load_repo_config(&root);
                Ok(Arc::new(KnowledgeRuntime::new(&root, config)?))
            }
            None => Ok(Arc::clone(&self.runtime)),
        }
    }

    /// Same escape hatch as `runtime_for`, for registry-backed tools.
    fn registry_for(&self, req: &McpRequest) -> Arc<dyn RegistryClient> {
        match req.params.get("repo_path").and_then(|v| v.as_str()) {
            Some(p) => Arc::new(FileRegistryClient::new(Path::new(p))),
            None => Arc::clone(&self.registry),
        }
    }

    pub fn notify_connect(&self) {
        self.context_manager.connect();
        self.ensure_context();
    }

    pub fn notify_disconnect(&self) {
        self.context_manager.disconnect();
        self.context_manager.maybe_dispose();
    }

    pub fn handle_message(&self, message: McpMessage) -> McpMessage {
        match message {
            McpMessage::Request(req) => self.handle_request(req),
            McpMessage::Notification(_) => McpMessage::Response(McpResponse {
                id: "ack".to_string(),
                result: serde_json::json!({}),
            }),
            _ => McpMessage::Error(McpError {
                id: None,
                code: -32600,
                message: "Invalid message type".to_string(),
            }),
        }
    }

    fn handle_request(&self, req: McpRequest) -> McpMessage {
        let result: Result<serde_json::Value> = match req.method.as_str() {
            "ping"                    => Ok(serde_json::json!({"pong": "pong"})),
            "capabilities"            => Ok(serde_json::to_value(&self.capabilities).unwrap_or_default()),
            "init"                    => self.handle_init(&req),
            "compile"                 => self.handle_compile(&req),
            "search"                  => self.handle_search(&req),
            "get_sections"            => self.handle_get_sections(&req),
            "audit"                   => self.handle_audit(&req),
            "info"                    => self.handle_info(&req),
            "get_document"            => self.handle_get_document(&req),
            "get_document_section"    => self.handle_get_document_section(&req),
            "list_domains"            => self.handle_list_domains(&req),
            "list_repositories"       => self.handle_list_repositories(&req),
            "register_repository"     => self.handle_register_repository(&req),
            "unregister_repository"   => self.handle_unregister_repository(&req),
            "synchronize_repository"  => self.handle_synchronize_repository(&req),
            "resolve_dependencies"    => self.handle_resolve_dependencies(&req),
            "repository_status"       => self.handle_repository_status(&req),
            "workspace_status"        => self.handle_workspace_status(&req),
            "get_product_knowledge_context" => self.handle_get_product_knowledge_context(&req),
            // Semantic audit handlers
            "get_documents_by_domain" => self.handle_get_documents_by_domain(&req),
            "get_section"             => self.handle_get_section(&req),
            "get_audit_knowledge"     => self.handle_get_audit_knowledge(&req),
            "get_audit_report"        => self.handle_get_audit_report(&req),
            "get_section_changed"     => self.handle_get_section_changed(&req),
            "check_gate"              => self.handle_check_gate(&req),
            "store_section_report"    => self.handle_store_section_report(&req),
            "store_document_report"   => self.handle_store_document_report(&req),
            "store_cross_domain_report" => self.handle_store_cross_domain_report(&req),
            "update_finding_status"   => self.handle_update_finding_status(&req),
            "sync"                    => self.handle_sync(&req),
            "get_plan"                => self.handle_get_plan(),
            "switch_context"          => self.handle_switch_context(&req),
            "list_contexts"           => self.handle_list_contexts(),
            "report_templates"        => self.handle_report_templates(&req),
            "report_generate"         => self.handle_report_generate(&req),
            "report_sessions"         => self.handle_report_sessions(&req),
            // Audit-fix pipeline handlers
            "audit_fix_plan"          => self.handle_audit_fix_plan(&req),
            "audit_fix_apply"         => self.handle_audit_fix_apply(&req),
            "audit_fix_accept"        => self.handle_audit_fix_accept(&req),
            "audit_fix_reject"        => self.handle_audit_fix_reject(&req),
            "audit_fix_status"        => self.handle_audit_fix_status(&req),
            "audit_fix_list"          => self.handle_audit_fix_list(&req),
            "audit_fix_plan_list"     => self.handle_audit_fix_plan_list(&req),
            "audit_fix_plan_get"      => self.handle_audit_fix_plan_get(&req),
            "audit_fix_plan_render"   => self.handle_audit_fix_plan_render(&req),
            "audit_fix_templates"     => self.handle_audit_fix_templates(&req),
            // Project planner handlers
            "project_plan"            => self.handle_project_plan(&req),
            "project_plan_get"        => self.handle_project_plan_get(&req),
            "project_plan_list"       => self.handle_project_plan_list(),
            "project_plan_execute"    => self.handle_project_plan_execute(&req),
            "project_plan_status"     => self.handle_project_plan_status(&req),
            "project_plan_abort"      => self.handle_project_plan_abort(&req),
            _                         => Err(anyhow::anyhow!("Unknown method: {}", req.method)),
        };

        match result {
            Ok(res) => McpMessage::Response(McpResponse { id: req.id, result: res }),
            Err(e) => McpMessage::Error(McpError {
                id: Some(req.id),
                code: -32000,
                message: e.to_string(),
            }),
        }
    }

    // ── Pagination helpers ────────────────────────────────────────────────────

    /// Extract limit and offset from request params.
    /// `max` is accepted as a backward-compatible alias for `limit`.
    fn page_params(req: &McpRequest, default_limit: usize) -> (usize, usize) {
        let limit = req.params.get("limit")
            .or_else(|| req.params.get("max"))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(default_limit);
        let offset = req.params.get("offset")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0);
        (limit, offset)
    }

    /// Slice a vec and build the pagination envelope with a dynamic key name.
    fn paginate<T: serde::Serialize>(
        items: Vec<T>,
        offset: usize,
        limit: usize,
        key: &str,
    ) -> serde_json::Value {
        let total = items.len();
        let start = offset.min(total);
        let end = (offset + limit).min(total);
        let page = serde_json::to_value(&items[start..end]).unwrap_or(serde_json::json!([]));
        let mut obj = serde_json::json!({
            "total": total,
            "offset": offset,
            "limit": limit,
            "has_more": end < total,
        });
        obj[key] = page;
        obj
    }

    /// Paginate a string body as lines.
    fn paginate_lines(content: &str, offset: usize, limit: usize) -> serde_json::Value {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        let start = offset.min(total);
        let end = (offset + limit).min(total);
        let page = lines[start..end].join("\n");
        serde_json::json!({
            "content": page,
            "total_lines": total,
            "offset": offset,
            "limit": limit,
            "has_more": end < total,
        })
    }

    // ── Knowledge methods ─────────────────────────────────────────────────────

    /// Initialize (or backfill) `samgraha.toml` + `.samgraha/` for the repo this
    /// MCP session is bound to. Mirrors the CLI `init` command so pure-MCP
    /// clients can bootstrap a repo without dropping to a shell.
    fn handle_init(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let force = req.params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let owned_root;
        let root: &Path = match req.params.get("repo_path").and_then(|v| v.as_str()) {
            Some(p) => {
                owned_root = std::path::PathBuf::from(p);
                &owned_root
            }
            None => &self.runtime.context.repository_root,
        };
        let result = services::init::init_repository(root, force)?;
        Ok(serde_json::json!({
            "status": result.status,
            "root": result.root.display().to_string(),
            "env_path": result.env_path.display().to_string(),
            "config": result.config,
        }))
    }

    fn handle_compile(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let force = req.params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let domains = req.params.get("domains")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_default();
        let scope = if domains.is_empty() {
            CompilationScope::Repository
        } else {
            CompilationScope::Domains(domains)
        };
        let request = CompilationRequest { scope, force, watch: false };

        if let Some(path_str) = req.params.get("path").and_then(|v| v.as_str()) {
            let root = std::path::PathBuf::from(path_str);
            let result = Self::compile_external(&self, &root, &request)?;
            return Ok(serde_json::to_value(&result)?);
        }

        let result = self.runtime.compile(&request)?;
        if result.success && self.runtime.context.config.resolver.auto_refresh {
            // Hybrid auto-registration: keep declared dependencies/interests
            // registered without requiring an explicit `sync` call. Manual
            // register/sync remain available at any time; failures here are
            // logged, not fatal — a registry hiccup shouldn't fail a compile.
            if let Err(e) = self.registry.sync(&self.runtime.context.config) {
                tracing::warn!("Registry sync after compile failed: {e}");
            }
        }
        Ok(serde_json::to_value(&result)?)
    }

    /// Compile a repository at `root` into its own `.samgraha/knowledge.db` — not Samgraha's.
    fn compile_external(
        &self,
        root: &std::path::Path,
        request: &CompilationRequest,
    ) -> Result<schemas::compilation::CompilationResult> {
        // Load target repo's own config if present, else use defaults.
        let target_config = load_repo_config(root);
        let db_path = root.join(".samgraha").join("knowledge.db");
        std::fs::create_dir_all(root.join(".samgraha"))?;
        let ext_registry = Arc::new(RegistryStore::open(&db_path)?);
        CompilationService::execute(
            root,
            &target_config,
            request,
            &self.runtime.standard_registry,
            ext_registry,
        )
    }

    /// Sync: read a compiled repo's manifest.json, register it in the local registry,
    /// and write a .meta file so the Planner can resolve this dep offline.
    fn handle_sync(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let path_str = req.params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let root = std::path::Path::new(path_str);
        let manifest_path = root.join(".samgraha").join("manifest.json");
        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", manifest_path.display(), e))?;
        let manifest: schemas::manifest::RepositoryManifest = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid manifest.json: {}", e))?;

        // Registry upsert.
        self.registry.register(&manifest)?;

        // Write .meta file → Planner reads this offline for path + cached metadata.
        let ttl_secs = parse_ttl_duration(&self.runtime.context.config.resolver.metadata_ttl)
            .unwrap_or(86400);
        let now = chrono::Utc::now();
        let meta = CachedRepoMetadata {
            repository: manifest.repository.clone(),
            revision: manifest.revision,
            repository_root: manifest.repository_root.clone(),
            knowledge: manifest.knowledge.clone(),
            exports: manifest.exports.clone(),
            audit: manifest.audit.status.clone(),
            last_sync: now.to_rfc3339(),
            expires: (now + chrono::Duration::seconds(ttl_secs)).to_rfc3339(),
            dependencies: manifest.dependencies.clone(),
        };
        let local_root = &self.runtime.context.repository_root;
        if let Err(e) = write_meta_file(local_root, &meta) {
            tracing::warn!("Cannot write .meta for '{}': {}", manifest.repository.id, e);
        }

        Ok(serde_json::json!({
            "success": true,
            "action": "sync",
            "repository": manifest.repository.id,
            "uuid": manifest.repository.uuid.to_string(),
            "revision": manifest.revision,
        }))
    }

    fn handle_search(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let query = req.params.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

        let level = req.params.get("level").and_then(|v| v.as_str()).unwrap_or("metadata");
        let domain = req.params.get("domain").and_then(|v| v.as_str());
        let (limit, offset) = Self::page_params(req, 20);

        let search_level = match level {
            "summary"  => RetrievalLevel::Summary,
            "section"  => RetrievalLevel::Section,
            "full"     => RetrievalLevel::Full,
            _          => RetrievalLevel::Metadata,
        };

        // Fetch with limit + offset headroom so we can compute total.
        // Fetch all and paginate in memory — search results are bounded by max_results.
        let search_query = SearchQuery {
            query: query.to_string(),
            domain: domain.map(|d| d.to_string()),
            level: search_level,
            max_results: usize::MAX,
            ..Default::default()
        };

        let has_repo_path = req.params.contains_key("repo_path");
        let runtime = self.runtime_for(req)?;
        let results = if has_repo_path {
            runtime.search(&search_query)?
        } else {
            self.ensure_context();
            match self.context_manager.with_context(|c| c.search(&search_query)) {
                Some(r) => r?,
                None => runtime.search(&search_query)?,
            }
        };
        let mut out = Self::paginate(results.results, offset, limit, "results");
        out["query"] = serde_json::Value::String(query.to_string());
        Ok(out)
    }

    fn handle_get_sections(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let semantic_type = req.params.get("semantic_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'semantic_type' parameter"))?;

        let domain = req.params.get("domain").and_then(|v| v.as_str());
        let (limit, offset) = Self::page_params(req, 50);

        let query = SectionQuery {
            semantic_type: semantic_type.to_string(),
            domain: domain.map(|d| d.to_string()),
            max_results: usize::MAX,
            document_id: None,
        };

        let has_repo_path = req.params.contains_key("repo_path");
        let runtime = self.runtime_for(req)?;
        let response = if has_repo_path {
            runtime.get_sections(&query)?
        } else {
            self.ensure_context();
            match self.context_manager.with_context(|c| c.get_sections(&query)) {
                Some(r) => r?,
                None => runtime.get_sections(&query)?,
            }
        };
        let mut out = Self::paginate(response.sections, offset, limit, "sections");
        out["semantic_type"] = serde_json::Value::String(semantic_type.to_string());
        Ok(out)
    }

    fn handle_audit(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let pipeline_name = req.params.get("pipeline").and_then(|v| v.as_str());
        let pipeline_kind = match pipeline_name {
            Some(name) => schemas::audit::PipelineKind::from_str(name)
                .ok_or_else(|| anyhow::anyhow!(
                    "Unknown pipeline '{}'. Valid values: doc, build, security, consistency, coverage, dependency",
                    name
                ))?,
            None => schemas::audit::PipelineKind::Doc,
        };

        if pipeline_kind != schemas::audit::PipelineKind::Doc {
            let inspect_artifact = req.params.get("inspect_artifact").and_then(|v| v.as_bool()).unwrap_or(false);
            let runtime_mode = req.params.get("runtime").and_then(|v| v.as_bool()).unwrap_or(false);
            let execute = req.params.get("execute").and_then(|v| v.as_bool()).unwrap_or(false);
            let dry_run = req.params.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);
            if (execute || dry_run) && pipeline_kind != schemas::audit::PipelineKind::Build {
                anyhow::bail!("'execute'/'dry_run' only apply to the build pipeline");
            }
            let runtime = self.runtime_for(req)?;
            let report = runtime.run_pipeline(&pipeline_kind, inspect_artifact, runtime_mode, execute, dry_run)?;
            return Ok(serde_json::to_value(&report)?);
        }

        let domain = req.params.get("domain").and_then(|v| v.as_str());
        let providers = req.params.get("providers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["deterministic".to_string()]);

        let has_repo_path = req.params.contains_key("repo_path");
        let runtime = self.runtime_for(req)?;
        let cross_repo_docs = if has_repo_path {
            None
        } else {
            self.ensure_context();
            self.context_manager.with_context(|c| c.package.all_documents().ok()).flatten()
        };
        let report = runtime.audit(domain, &providers, cross_repo_docs.as_deref())?;
        Ok(serde_json::to_value(&report)?)
    }

    fn handle_info(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let has_repo_path = req.params.contains_key("repo_path");
        let runtime = self.runtime_for(req)?;
        let mut info = serde_json::to_value(&runtime.info())?;
        if !has_repo_path && (self.context_manager.store_count() > 0 || self.context_manager.is_context_valid()) {
            info["context_stores"] = serde_json::json!(self.context_manager.store_count());
            info["context_valid"] = serde_json::json!(self.context_manager.is_context_valid());
        }
        Ok(info)
    }

    /// Return the current Knowledge Plan so the client can inspect repo status.
    fn handle_get_plan(&self) -> Result<serde_json::Value> {
        match self.context_manager.with_context(|c| {
            serde_json::to_value(&c.plan.entries).map(|entries| serde_json::json!({
                "context_valid": c.is_valid(),
                "store_count": c.store_count(),
                "entries": entries,
            }))
        }) {
            Some(r) => r.map_err(Into::into),
            None => Ok(serde_json::json!({
                "context_valid": false,
                "store_count": 0,
                "entries": [],
            })),
        }
    }

    fn handle_switch_context(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let name = req.params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        if self.context_manager.activate(name) {
            Ok(serde_json::json!({ "active": name }))
        } else {
            Err(anyhow::anyhow!("Context '{}' not loaded — call ensure or sync first", name))
        }
    }

    fn handle_list_contexts(&self) -> Result<serde_json::Value> {
        let names = self.context_manager.context_names();
        let active = self.context_manager.active_name();
        Ok(serde_json::json!({ "contexts": names, "active": active }))
    }

    /// Returns document metadata and section TOC only — no body content.
    /// Use `get_document_section` to fetch section content.
    fn handle_get_document(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let doc_id = req.params.get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;

        let runtime = self.runtime_for(req)?;
        let doc = runtime.get_document(doc_id)?
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

        let raw = doc.body.raw();
        let total_lines = raw.lines().count();

        let sections: Vec<serde_json::Value> = doc.body.sections()
            .iter()
            .enumerate()
            .map(|(i, s)| serde_json::json!({
                "index": i,
                "heading": s.heading,
                "semantic_type": s.semantic_type,
                "level": s.level,
                "required": s.required,
                "line_start": s.source_span.as_ref().map(|sp| sp.line_start),
                "line_end": s.source_span.as_ref().map(|sp| sp.line_end),
                "subsection_count": s.subsections.len(),
            }))
            .collect();

        Ok(serde_json::json!({
            "id": doc.id,
            "title": doc.title,
            "standard": doc.standard,
            "path": doc.path.as_str(),
            "hash": doc.hash,
            "created_at": doc.created_at,
            "updated_at": doc.updated_at,
            "total_lines": total_lines,
            "section_count": sections.len(),
            "sections": sections,
        }))
    }

    /// Fetch paginated content of a specific section.
    ///
    /// `section` param: integer index (0-based) OR string (case-insensitive heading match).
    /// `offset` / `limit` paginate the section's line content (default limit: 100).
    fn handle_get_document_section(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let doc_id = req.params.get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;

        let runtime = self.runtime_for(req)?;
        let doc = runtime.get_document(doc_id)?
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

        let section_param = req.params.get("section")
            .ok_or_else(|| anyhow::anyhow!("Missing 'section' parameter (index or heading name)"))?;

        let sections = doc.body.sections();

        let section = if let Some(idx) = section_param.as_u64() {
            sections.into_iter().nth(idx as usize)
        } else if let Some(name) = section_param.as_str() {
            let name_lc = name.to_lowercase();
            sections.into_iter().find(|s| s.heading.to_lowercase().contains(&name_lc))
        } else {
            return Err(anyhow::anyhow!("'section' must be an integer index or string heading name"));
        };

        let section = section.ok_or_else(|| anyhow::anyhow!("Section not found: {}", section_param))?;

        let (limit, offset) = Self::page_params(req, 100);
        let mut content_page = Self::paginate_lines(&section.body, offset, limit);

        let subsections: Vec<serde_json::Value> = section.subsections.iter()
            .enumerate()
            .map(|(i, s)| serde_json::json!({
                "index": i,
                "heading": s.heading,
                "semantic_type": s.semantic_type,
            }))
            .collect();

        content_page["heading"] = serde_json::Value::String(section.heading.clone());
        content_page["semantic_type"] = serde_json::Value::String(section.semantic_type.clone());
        content_page["required"] = serde_json::Value::Bool(section.required);
        content_page["subsections"] = serde_json::json!(subsections);

        Ok(content_page)
    }

    fn handle_list_domains(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let runtime = self.runtime_for(req)?;
        let domains = runtime.get_domains()?;

        if domains.is_empty() {
            return Ok(serde_json::json!({
                "domains": domains,
                "count": 0,
                "message": "No documents compiled yet. Run 'compile' on this repository first, then call list_domains again.",
            }));
        }

        Ok(serde_json::json!({
            "domains": domains,
            "count": domains.len(),
        }))
    }

    // ── Registry methods ──────────────────────────────────────────────────────

    fn handle_list_repositories(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let (limit, offset) = Self::page_params(req, 50);
        let entries = self.registry_for(req).list()?;
        if entries.is_empty() {
            let mut out = Self::paginate(entries, offset, limit, "repositories");
            out["message"] = serde_json::json!(
                "No repositories registered. Call register_repository with a manifest (see .samgraha/manifest.json after running 'compile') to register this repo."
            );
            return Ok(out);
        }
        Ok(Self::paginate(entries, offset, limit, "repositories"))
    }

    fn handle_register_repository(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let manifest_str = req.params.get("manifest")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'manifest' parameter (JSON string)"))?;
        let manifest: schemas::manifest::RepositoryManifest = serde_json::from_str(manifest_str)?;
        self.registry_for(req).register(&manifest)?;

        // Auto-compile if the repo's knowledge.db is missing — the manifest can be
        // handed to register_repository before the repo has ever been compiled.
        let root = std::path::Path::new(&manifest.repository_root);
        let db_path = root.join(&manifest.knowledge.location);
        let auto_compiled = if !db_path.exists() {
            let request = CompilationRequest { scope: CompilationScope::Repository, force: false, watch: false };
            self.compile_external(root, &request)?;
            true
        } else {
            false
        };

        Ok(serde_json::json!({
            "success": true,
            "action": "register",
            "repository": manifest.repository.id,
            "uuid": manifest.repository.uuid.to_string(),
            "auto_compiled": auto_compiled,
        }))
    }

    fn handle_unregister_repository(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let uuid_str = req.params.get("uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'uuid' parameter"))?;
        let uuid = uuid::Uuid::parse_str(uuid_str)?;
        self.registry_for(req).unregister(&uuid)?;
        Ok(serde_json::json!({
            "success": true,
            "action": "unregister",
            "uuid": uuid_str,
        }))
    }

    fn handle_synchronize_repository(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let runtime = self.runtime_for(req)?;
        let registry = self.registry_for(req);
        registry.sync(&runtime.context.config)?;
        let entries = registry.list()?;
        Ok(serde_json::json!({
            "success": true,
            "action": "sync",
            "dependencies_synced": entries.len(),
            "entries": entries.iter().map(|e| serde_json::json!({
                "id": e.repository.id,
                "uuid": e.repository.uuid.to_string(),
                "revision": e.revision,
                "last_sync": e.last_sync,
            })).collect::<Vec<_>>(),
        }))
    }

    fn handle_resolve_dependencies(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let runtime = self.runtime_for(req)?;
        let db = registry::registry_db::RegistryDb::open(
            &runtime.context.repository_root
        ).ok();
        use common::config::parse_ttl_duration;
        let ttl_seconds = parse_ttl_duration(&runtime.context.config.resolver.metadata_ttl)
            .unwrap_or(86400);
        let (resolved, unresolved) = KnowledgeResolver::resolve_dependency_graph(
            &runtime.context.config.repository.dependencies,
            &runtime.context.repository_root,
            db.as_ref(),
            ttl_seconds,
        );
        Ok(serde_json::json!({
            "dependencies": resolved.iter().map(|d| serde_json::json!({
                "name": d.name,
                "path": d.path.as_ref().map(|p| p.to_string_lossy().to_string()),
                "available": d.available,
                "required": d.required,
                "revision": d.revision,
            })).collect::<Vec<_>>(),
            "unresolved": unresolved,
            "count": resolved.len(),
        }))
    }

    fn handle_repository_status(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let (limit, offset) = Self::page_params(req, 50);
        let entries = self.registry_for(req).list()?;
        let now = std::time::SystemTime::now();
        let statuses: Vec<serde_json::Value> = entries.iter()
            .map(|e| serde_json::json!({
                "id": e.repository.id,
                "uuid": e.repository.uuid.to_string(),
                "status": format!("{:?}", e.status(now)),
                "revision": e.revision,
                "audit": e.audit,
                "exports": e.exports,
            }))
            .collect();
        if statuses.is_empty() {
            let mut out = Self::paginate(statuses, offset, limit, "repositories");
            out["message"] = serde_json::json!(
                "No repositories registered. Call register_repository with a manifest to register this repo before checking status."
            );
            return Ok(out);
        }
        Ok(Self::paginate(statuses, offset, limit, "repositories"))
    }

    /// Returns this repository's compiled Product Knowledge context — the
    /// `repository_metadata` key-value snapshot written during `compile`
    /// (source/test/scripts dirs, declared dependencies, pipeline commands,
    /// repo identity). Empty object if `compile` hasn't run yet. Distinct
    /// from `repository_status`: that's a workspace-wide view across every
    /// registered repository; this is single-repo depth for the repository
    /// this MCP session is bound to.
    fn handle_get_product_knowledge_context(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let runtime = self.runtime_for(req)?;
        let metadata = runtime.registry.get_repository_metadata()?;
        Ok(serde_json::json!({ "context": metadata }))
    }

    fn handle_workspace_status(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let (limit, offset) = Self::page_params(req, 50);
        let entries = self.registry_for(req).list()?;
        let now = std::time::SystemTime::now();
        let repos: Vec<serde_json::Value> = entries.iter()
            .map(|e| serde_json::json!({
                "id": e.repository.id,
                "uuid": e.repository.uuid.to_string(),
                "status": format!("{:?}", e.status(now)),
            }))
            .collect();
        let mut out = Self::paginate(repos, offset, limit, "repositories");
        out["registered"] = serde_json::json!(entries.len());
        Ok(out)
    }

    // ── Semantic Audit Handlers ─────────────────────────────────────────────

    fn handle_get_documents_by_domain(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain = req.params.get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'domain' parameter"))?;
        let (limit, offset) = Self::page_params(req, 50);
        let docs = self.runtime_for(req)?.get_documents_by_domain(domain)?;
        if docs.is_empty() {
            let mut out = Self::paginate(docs, offset, limit, "documents");
            out["message"] = serde_json::json!(format!(
                "No documents found for domain '{}'. Call list_domains to see available domains, or run 'compile' if none exist yet.",
                domain
            ));
            return Ok(out);
        }
        Ok(Self::paginate(docs, offset, limit, "documents"))
    }

    fn handle_get_section(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let section_id = req.params.get("section_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'section_id' parameter"))?;
        let section = self.runtime_for(req)?.get_section_by_id(section_id)?
            .ok_or_else(|| anyhow::anyhow!("Section not found: {}", section_id))?;
        Ok(serde_json::to_value(&section)?)
    }

    fn handle_get_audit_knowledge(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain = req.params.get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'domain' parameter"))?;
        let section_type = req.params.get("section_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'section_type' parameter"))?;
        let content = self.runtime_for(req)?.get_audit_knowledge(domain, section_type)?;
        Ok(serde_json::json!({ "content": content, "domain": domain, "section_type": section_type }))
    }

    fn handle_get_audit_report(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain = req.params.get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'domain' parameter"))?;
        let stage_str = req.params.get("stage")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'stage' parameter"))?;
        let document_id = req.params.get("document_id").and_then(|v| v.as_i64());
        let section_id = req.params.get("section_id").and_then(|v| v.as_i64());

        let stage = match stage_str {
            "deterministic" => AuditStage::Deterministic,
            "section" => AuditStage::Section,
            "document" => AuditStage::Document,
            "cross_domain" => AuditStage::CrossDomain,
            _ => return Err(anyhow::anyhow!("Invalid stage: {}", stage_str)),
        };

        match self.runtime_for(req)?.get_audit_report(domain, document_id, section_id, stage)? {
            Some(r) => Ok(serde_json::to_value(&r)?),
            None => Ok(serde_json::json!({"report": null, "domain": domain, "stage": stage_str})),
        }
    }

    fn handle_get_section_changed(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let section_id = req.params.get("section_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'section_id' parameter"))?;
        let result = self.runtime_for(req)?.get_section_changed(section_id)?;
        Ok(serde_json::to_value(&result)?)
    }

    fn handle_check_gate(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let stage_str = req.params.get("stage")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'stage' parameter"))?;
        let document_id = req.params.get("document_id").and_then(|v| v.as_i64());

        let stage = match stage_str {
            "deterministic" => AuditStage::Deterministic,
            "section" => AuditStage::Section,
            "document" => AuditStage::Document,
            "cross_domain" => AuditStage::CrossDomain,
            _ => return Err(anyhow::anyhow!("Invalid stage: {}", stage_str)),
        };

        let result = self.runtime_for(req)?.check_gate(stage, document_id)?;
        Ok(serde_json::to_value(&result)?)
    }

    fn handle_store_section_report(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let report_json = req.params.get("report_json")
            .ok_or_else(|| anyhow::anyhow!("Missing 'report_json' parameter"))?;
        let report: SemanticReport = serde_json::from_value(report_json.clone())
            .map_err(|e| anyhow::anyhow!("Invalid report schema: {}", e))?;
        let id = self.runtime_for(req)?.store_section_report(&report)?;
        Ok(serde_json::json!({"report_id": id, "status": "stored"}))
    }

    fn handle_store_document_report(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let report_json = req.params.get("report_json")
            .ok_or_else(|| anyhow::anyhow!("Missing 'report_json' parameter"))?;
        let report: SemanticReport = serde_json::from_value(report_json.clone())
            .map_err(|e| anyhow::anyhow!("Invalid report schema: {}", e))?;
        let id = self.runtime_for(req)?.store_document_report(&report)?;
        Ok(serde_json::json!({"report_id": id, "status": "stored"}))
    }

    fn handle_store_cross_domain_report(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let report_json = req.params.get("report_json")
            .ok_or_else(|| anyhow::anyhow!("Missing 'report_json' parameter"))?;
        let report: SemanticReport = serde_json::from_value(report_json.clone())
            .map_err(|e| anyhow::anyhow!("Invalid report schema: {}", e))?;
        let id = self.runtime_for(req)?.store_cross_domain_report(&report)?;
        Ok(serde_json::json!({"report_id": id, "status": "stored"}))
    }

    fn handle_update_finding_status(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let report_id = req.params.get("report_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'report_id' parameter"))?;
        let criterion_id = req.params.get("criterion_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'criterion_id' parameter"))?;
        let status_str = req.params.get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'status' parameter"))?;

        let status = match status_str {
            "open" => FindingStatus::Open,
            "fixed" => FindingStatus::Fixed,
            "accepted" => FindingStatus::Accepted,
            "ignored" => FindingStatus::Ignored,
            "false_positive" => FindingStatus::FalsePositive,
            _ => return Err(anyhow::anyhow!("Invalid status: {}", status_str)),
        };

        self.runtime_for(req)?.update_finding_status(report_id, criterion_id, status)?;
        Ok(serde_json::json!({"success": true}))
    }

    // ── Pipeline Report MCP Methods ──────────────────────────────────────

    fn handle_report_templates(&self, _req: &McpRequest) -> Result<serde_json::Value> {
        let templates_dir = self.runtime.context.repository_root.join("docs/raw/report-templates");
        let names = services::reporting::list_templates(&templates_dir)?;
        // Always include the built-in default
        let mut templates = vec!["pipeline-default".to_string()];
        templates.extend(names.into_iter().filter(|n| n != "pipeline-default"));
        Ok(serde_json::json!({
            "templates": templates.iter().map(|n| serde_json::json!({
                "name": n,
                "path": format!("{}.md", n),
                "builtin": n == "pipeline-default",
            })).collect::<Vec<_>>(),
        }))
    }

    fn handle_report_generate(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let audit_type = req.params.get("type")
            .or_else(|| req.params.get("pipeline"))
            .and_then(|v| v.as_str())
            .unwrap_or("build");
        let templates_dir = self.runtime.context.repository_root.join("docs/raw/report-templates");
        let rendered = services::reporting::render_report(audit_type, &templates_dir, self.runtime.registry.as_ref())?;
        Ok(serde_json::json!({
            "type": audit_type,
            "markdown": rendered,
        }))
    }

    fn handle_report_sessions(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let audit_type = req.params.get("type")
            .or_else(|| req.params.get("pipeline"))
            .and_then(|v| v.as_str())
            .unwrap_or("build");
        let (limit, _offset) = crate::adapter::McpAdapter::page_params(req, 50);
        let sessions = self.runtime.query_sessions_by_type(audit_type, limit)?;
        Ok(serde_json::json!({
            "sessions": sessions,
            "count": sessions.len(),
        }))
    }

    // ── Audit-Fix MCP Methods ─────────────────────────────────────────────

    /// Generate a fix plan without auto-execution (human review mode).
    fn handle_audit_fix_plan(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let finding = parse_finding(req)?;
        let domain = parse_string(req, "domain")?;
        let report_id = parse_i64(req, "report_id")?;
        let report_type = parse_string(req, "report_type")?;
        let target_path_str = parse_string(req, "target_path")?;
        let target_path = std::path::PathBuf::from(&target_path_str);
        validate_target_path(&self.runtime.context.repository_root, &target_path)?;
        let plan = self.runtime.generate_fix_plan(&finding, &domain, report_id, &report_type, &target_path)?;
        Ok(serde_json::json!(plan))
    }

    /// Run the full audit-fix pipeline: plan, execute, verify, retry.
    fn handle_audit_fix_apply(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let finding = parse_finding(req)?;
        let domain = parse_string(req, "domain")?;
        let report_id = parse_i64(req, "report_id")?;
        let report_type = parse_string(req, "report_type")?;
        let target_path_str = parse_string(req, "target_path")?;
        let target_path = std::path::PathBuf::from(&target_path_str);
        validate_target_path(&self.runtime.context.repository_root, &target_path)?;
        let session = self.runtime.apply_finding_fix(&finding, &domain, report_id, &report_type, &target_path)?;
        Ok(serde_json::json!(session))
    }

    /// Accept a fix — delegates to `update_finding_status` with status "fixed".
    fn handle_audit_fix_accept(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let report_id = parse_i64(req, "report_id")?;
        let criterion_id = parse_string(req, "criterion_id")?;
        self.runtime.update_finding_status(report_id, &criterion_id, FindingStatus::Fixed)?;
        Ok(serde_json::json!({"success": true}))
    }

    /// Reject a fix — delegates to `update_finding_status` with status "accepted".
    fn handle_audit_fix_reject(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let report_id = parse_i64(req, "report_id")?;
        let criterion_id = parse_string(req, "criterion_id")?;
        self.runtime.update_finding_status(report_id, &criterion_id, FindingStatus::Accepted)?;
        Ok(serde_json::json!({"success": true}))
    }

    /// Get the status of a fix session by session_id.
    fn handle_audit_fix_status(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let session_id = parse_i64(req, "session_id")?;
        let session = self.runtime.registry.get_fix_session(session_id)?;
        let attempts = self.runtime.registry.get_fix_attempts(session_id)?;
        Ok(serde_json::json!({
            "session": session,
            "attempts": attempts,
        }))
    }

    /// List fix sessions (paginated).
    fn handle_audit_fix_list(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let (limit, offset) = McpAdapter::page_params(req, 20);
        let sessions = self.runtime.registry.query_fix_sessions(limit, offset)?;
        Ok(McpAdapter::paginate(sessions, offset, limit, "sessions"))
    }

    /// List fix plans for a session.
    fn handle_audit_fix_plan_list(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let session_id = parse_i64(req, "session_id")?;
        let plans = self.runtime.registry.query_fix_plans_by_session(session_id)?;
        Ok(serde_json::json!({ "plans": plans, "count": plans.len() }))
    }

    /// Get a single fix plan by plan_id.
    fn handle_audit_fix_plan_get(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let plan_id = parse_i64(req, "plan_id")?;
        let plan = self.runtime.registry.get_fix_plan(plan_id)?;
        let steps = plan.as_ref().and_then(|p| {
            self.runtime.registry.get_fix_plan_steps(p.id.unwrap_or(0)).ok()
        }).unwrap_or_default();
        Ok(serde_json::json!({
            "plan": plan,
            "steps": steps,
        }))
    }

    /// Render a fix plan as markdown using the template engine.
    fn handle_audit_fix_plan_render(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let plan_id = parse_i64(req, "plan_id")?;
        let template_name = req.params.get("template")
            .and_then(|v| v.as_str())
            .unwrap_or("documentation")
            .to_string();
        let plan = self.runtime.registry.get_fix_plan(plan_id)?
            .ok_or_else(|| anyhow::anyhow!("Fix plan not found: {}", plan_id))?;
        let templates_dir = self.runtime.context.repository_root.join("docs/raw/fix-plan-templates");
        let template = services::reporting::read_template(&templates_dir, &template_name)?;
        let markdown = services::reporting::render_fix_plan(&plan, &template);
        Ok(serde_json::json!({
            "plan_id": plan_id,
            "template": template_name,
            "markdown": markdown,
        }))
    }

    /// List available fix plan templates.
    fn handle_audit_fix_templates(&self, _req: &McpRequest) -> Result<serde_json::Value> {
        let templates_dir = self.runtime.context.repository_root.join("docs/raw/fix-plan-templates");
        let names = services::reporting::list_fix_plan_templates(&templates_dir)?;
        Ok(serde_json::json!({
            "templates": names.iter().map(|n| serde_json::json!({
                "name": n,
                "path": format!("{}-plan.md", n),
            })).collect::<Vec<_>>(),
        }))
    }

    // ── Project Planner MCP Methods ──────────────────────────────────────

    fn handle_project_plan(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let case_str = parse_string(req, "case")?;
        let case = schemas::ProjectCase::from_str(&case_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid case: '{}' (expected: new_project, docs_audit, impl_test_audit, build_audit)", case_str))?;
        let title = req.params.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(&case_str)
            .to_string();
        let plan_with_phases = self.orchestrator.create_plan(&case, &title)?;
        Ok(serde_json::json!({
            "plan_id": plan_with_phases.plan.id,
            "title": plan_with_phases.plan.title,
            "case": plan_with_phases.plan.case_type,
            "status": plan_with_phases.plan.status,
            "phases": plan_with_phases.phases.iter().map(|p| serde_json::json!({
                "phase_number": p.phase_number,
                "name": p.name,
                "phase_type": p.phase_type,
                "status": p.status,
                "dependencies": p.dependencies,
            })).collect::<Vec<_>>(),
        }))
    }

    fn handle_project_plan_get(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let plan_id = parse_string(req, "plan_id")?;
        let plan = self.orchestrator.get_plan(&plan_id)?;
        Ok(serde_json::to_value(plan)?)
    }

    fn handle_project_plan_list(&self) -> Result<serde_json::Value> {
        let plans = self.orchestrator.list_plans()?;
        Ok(serde_json::json!({ "plans": plans, "count": plans.len() }))
    }

    fn handle_project_plan_execute(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let plan_id = parse_string(req, "plan_id")?;
        let phase_number = req.params.get("phase_number").and_then(|v| v.as_u64()).map(|n| n as u32);
        let result = self.orchestrator.execute_phase(&plan_id, phase_number)?;
        Ok(result)
    }

    fn handle_project_plan_status(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let plan_id = parse_string(req, "plan_id")?;
        let progress = self.orchestrator.get_progress(&plan_id)?;
        Ok(serde_json::to_value(progress)?)
    }

    fn handle_project_plan_abort(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let plan_id = parse_string(req, "plan_id")?;
        let reason = req.params.get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("aborted")
            .to_string();
        self.orchestrator.abort_plan(&plan_id, &reason)?;
        Ok(serde_json::json!({ "success": true, "plan_id": plan_id }))
    }
}
