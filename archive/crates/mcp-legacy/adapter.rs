use crate::protocol::{McpCapabilities, McpError, McpMessage, McpRequest, McpResponse};
use anyhow::{Context, Result};
use common::config::{parse_ttl_duration, RepositoryKind, SamgrahaConfig};
use registry::migration;
use registry::RegistryStore;
use schemas::audit::{AuditFinding, AuditStage, FindingStatus, SemanticReport};
use schemas::compilation::{CompilationRequest, CompilationScope};
use schemas::manifest::CachedRepoMetadata;
use schemas::search::{RetrievalLevel, SearchQuery, SectionQuery};
use services::compilation::PipelineFactory;
use services::knowledge_publish;
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

/// Resolve a `standards.id` from a system name, same query
/// `standards::db_reader::from_standards_db` already uses to pick which
/// standard a repo loads. `None` falls back to whichever system has
/// `is_default = 1` — same fallback `from_standards_db` uses for
/// `samgraha.toml`'s `standard_system` being unset.
fn resolve_standard_id(conn: &rusqlite::Connection, system_name: Option<&str>) -> Result<i64> {
    match system_name {
        Some(name) => conn.query_row(
            "SELECT s.id FROM standards s
             JOIN systems sys ON s.system_id = sys.id
             WHERE sys.name = ?1 AND s.name = 'documentation-standards'
             LIMIT 1",
            [name],
            |row| row.get(0),
        )
        .with_context(|| format!("No documentation-standards found for system '{}'", name)),
        None => conn.query_row(
            "SELECT s.id FROM standards s
             JOIN systems sys ON s.system_id = sys.id
             WHERE sys.is_default = 1 AND s.name = 'documentation-standards'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .context("No default documentation-standards found in DB"),
    }
}

/// Deserialized `assets:` block from a standard's `system.yaml`.
/// Controls which directories under the standard root are shipped as
/// syncable assets and what to exclude.
#[derive(Debug, Clone, serde::Deserialize, Default)]
struct StandardAssets {
    /// Directory name for scripts (default: "script").
    #[serde(default = "default_assets_scripts")]
    pub scripts: String,
    /// Directory name for templates (default: "templates").
    #[serde(default = "default_assets_templates")]
    pub templates: String,
    /// Glob patterns to exclude from both scripts and templates.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Custom data tables this standard creates inside knowledge.db.
    /// Absent = no custom tables.
    #[serde(default)]
    pub data_tables: Option<DataTablesConfig>,
    /// Directory names for seed-once assets (audit/analysis rubrics
    /// and prompt templates). Each entry is copied on first init,
    /// never overwritten on re-sync.
    #[serde(default)]
    pub audit_analysis: Vec<String>,
    /// Catalog of named scripts/prompt assets this standard ships.
    /// Each entry is written to standard_assets in knowledge.db
    /// for name→path resolution by the execution engine (§3.2).
    #[serde(default)]
    pub catalog: Vec<CatalogEntry>,
}

/// A single named asset in a standard's catalog (§3.2 of the MCP
/// execution substrate proposal).
#[derive(Debug, Clone, serde::Deserialize)]
struct CatalogEntry {
    /// Unique name within the standard (e.g. "scoring_audit").
    pub name: String,
    /// "script" or "prompt".
    pub kind: String,
    /// Path relative to the standard root (e.g. "script/scoring_audit.py").
    pub path: String,
    /// One-line human-readable purpose.
    #[serde(default)]
    pub purpose: String,
}

/// Declares custom data tables a standard owns. The `prefix` is
/// mandatory and must not collide with any name in
/// `RESERVED_TABLE_NAMES` — enforced at register time.
#[derive(Debug, Clone, serde::Deserialize)]
struct DataTablesConfig {
    /// Script that creates/migrates these tables (relative to standard root).
    pub owner_script: String,
    /// Mandatory prefix every table name must start with (e.g. "hackathon_").
    pub prefix: String,
    /// Human-readable purpose of this table set.
    #[serde(default)]
    pub purpose: String,
}

fn default_assets_scripts() -> String { "script".to_string() }
fn default_assets_templates() -> String { "templates".to_string() }

/// Resolve the effective exclusion list: caller-supplied patterns merged
/// with the unconditional defaults (`__pycache__`, `.pyc`).
fn effective_excludes(extra: &[String]) -> Vec<String> {
    let mut v: Vec<String> = common::fs_sync::DEFAULT_EXCLUDES
        .iter().map(|s| s.to_string()).collect();
    v.extend(extra.iter().cloned());
    v
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
        caps.methods.push("store_pipeline_check_report".to_string());
        caps.methods.push("get_pipeline_check_report".to_string());
        caps.methods.push("check_pipeline_gate".to_string());
        caps.methods.push("get_summary_report".to_string());
        caps.methods.push("update_finding_status".to_string());
        caps.methods.push("update_report_finding_status".to_string());
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
        caps.methods.push("list_standards".to_string());
        caps.methods.push("get_standard".to_string());
        caps.methods.push("get_standard_doc".to_string());
        caps.methods.push("register_standard".to_string());
        caps.methods.push("set_default_standard".to_string());
        caps.methods.push("sync_standards".to_string());
        caps.methods.push("get_plan_settings".to_string());
        caps.methods.push("get_plan_scenarios".to_string());
        caps.methods.push("list_script_checks".to_string());
        caps.methods.push("run_check".to_string());
        caps.methods.push("run_system_script".to_string());
        caps.methods.push("run_system_validate".to_string());
        caps.methods.push("run_system_calculate".to_string());
        caps.methods.push("run_system_report".to_string());
        caps.methods.push("run_system_scaffold".to_string());
        caps.methods.push("run_system_plan_generation".to_string());
        caps.methods.push("run_system_assemble".to_string());
        caps.methods.push("generate".to_string());
        caps.methods.push("store_generated_content".to_string());
        caps.methods.push("store_system_plan".to_string());
        caps.methods.push("get_system_plan".to_string());
        caps.methods.push("store_plan_generation_input".to_string());
        caps.methods.push("get_plan_generation_input".to_string());
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

    /// Repository Matrix (docs/crates-refactor-proposal.md §5): these methods
    /// read `knowledge.db`'s document/section tables, which `compile_knowledge`
    /// never populates for Knowledge Repositories — a Knowledge Repository
    /// produces Knowledge Systems for others to consume, it isn't queried at
    /// runtime itself. Centralizes what used to be one ad hoc check living
    /// only inside `handle_search` (Gap 13) into a single dispatch-point gate
    /// covering every runtime document/section query tool.
    const KNOWLEDGE_REPO_BLOCKED_METHODS: &[&str] = &[
        "search",
        "get_sections",
        "get_document",
        "get_document_section",
        "get_documents_by_domain",
        "get_section",
        "get_section_changed",
        "get_audit_knowledge",
        "get_audit_report",
        "get_summary_report",
        "get_product_knowledge_context",
    ];

    fn handle_request(&self, req: McpRequest) -> McpMessage {
        if Self::KNOWLEDGE_REPO_BLOCKED_METHODS.contains(&req.method.as_str()) {
            if let Ok(rt) = self.runtime_for(&req) {
                if rt.context.config.repository.kind == RepositoryKind::Knowledge {
                    return McpMessage::Error(McpError {
                        id: Some(req.id),
                        code: -32000,
                        message: format!(
                            "'{}' is not available for Knowledge Repositories — they produce \
                             Knowledge Systems for others to consume, not query at runtime. \
                             Use 'knowledge publish'/'knowledge pull' instead.",
                            req.method
                        ),
                    });
                }
            }
        }

        let result: Result<serde_json::Value> = match req.method.as_str() {
            "ping"                    => Ok(serde_json::json!({"pong": "pong"})),
            "capabilities"            => Ok(serde_json::to_value(&self.capabilities).unwrap_or_default()),
            "init"                    => self.handle_init(&req),
            "compile"                 => self.handle_compile(&req),
            "search"                  => self.handle_search(&req),
            "get_sections"            => self.handle_get_sections(&req),
            "audit"                   => self.handle_audit(&req),
            "audit_runs"              => self.handle_audit_runs(&req),
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
            "store_pipeline_check_report" => self.handle_store_pipeline_check_report(&req),
            "get_pipeline_check_report" => self.handle_get_pipeline_check_report(&req),
            "check_pipeline_gate"     => self.handle_check_pipeline_gate(&req),
            "get_summary_report"      => self.handle_get_summary_report(&req),
            "update_finding_status"   => self.handle_update_finding_status(&req),
            "update_report_finding_status" => self.handle_update_report_finding_status(&req),
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
            // Standards management
            "list_standards"          => self.handle_list_standards(),
            "get_standard"            => self.handle_get_standard(&req),
            "get_standard_doc"        => self.handle_get_standard_doc(&req),
            "register_standard"       => self.handle_register_standard(&req),
            "push_standards"          => self.handle_push_standards(),
            "set_default_standard"    => self.handle_set_default_standard(&req),
            "sync_standards"          => self.handle_sync_standards(&req),
            "check_knowledge_staleness" => self.handle_check_staleness(&req),
            // Phase 5: plan orchestration metadata
            "get_plan_settings"       => self.handle_get_plan_settings(),
            "get_plan_scenarios"      => self.handle_get_plan_scenarios(&req),
            "list_script_checks"      => self.handle_list_script_checks(&req),
            "run_check"               => self.handle_run_check(&req),
            "run_system_script"       => self.handle_run_system_script(&req),
            "run_system_validate"     => self.handle_run_system_script_with_capability(&req, audit::capability::Capability::Validate),
            "run_system_calculate"    => self.handle_run_system_script_with_capability(&req, audit::capability::Capability::Calculate),
            "run_system_report"       => self.handle_run_system_script_with_capability(&req, audit::capability::Capability::Report),
            "run_system_scaffold"     => self.handle_run_system_script_with_capability(&req, audit::capability::Capability::Scaffold),
            "run_system_plan_generation" => self.handle_run_system_script_with_capability(&req, audit::capability::Capability::PlanGeneration),
            "run_system_assemble"     => self.handle_run_system_script_with_capability(&req, audit::capability::Capability::Assemble),
            // Content generation handlers
            "generate"                => self.handle_generate(&req),
            "store_generated_content" => self.handle_store_generated_content(&req),
            // System plan storage (§8.4)
            "store_system_plan"          => self.handle_store_system_plan(&req),
            "get_system_plan"            => self.handle_get_system_plan(&req),
            "store_plan_generation_input" => self.handle_store_plan_generation_input(&req),
            "get_plan_generation_input"  => self.handle_get_plan_generation_input(&req),
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
        let standard_system = req.params.get("standard_system")
            .and_then(|v| v.as_str()).map(|s| s.to_string());
        let script_overrides: std::collections::HashMap<String, String> =
            req.params.get("script_overrides")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
        let check_overrides: std::collections::HashMap<String, String> =
            req.params.get("check_overrides")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
        let auto_detect = req.params.get("auto_detect")
            .and_then(|v| v.as_bool()).unwrap_or(false);
        let sync = req.params.get("sync")
            .and_then(|v| v.as_bool()).unwrap_or(false);

        let owned_root;
        let root: &Path = match req.params.get("repo_path").and_then(|v| v.as_str()) {
            Some(p) => {
                owned_root = std::path::PathBuf::from(p);
                &owned_root
            }
            None => &self.runtime.context.repository_root,
        };
        let options = common::config::InitOptions {
            force,
            standard_system,
            script_overrides,
            check_overrides,
            auto_detect_dirs: auto_detect,
            sync_knowledge_system: sync,
        };
        let result = services::init::init_repository(root, &options)?;

        Ok(serde_json::json!({
            "status": result.status,
            "root": result.root.display().to_string(),
            "env_path": result.env_path.display().to_string(),
            "config": result.config,
            "sync_result": result.sync_result.as_ref().map(|s| serde_json::json!({
                "standards_synced": s.standards_synced,
                "help_documents_synced": s.help_documents_synced,
                "scripts_synced": s.scripts_synced,
            })),
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
        PipelineFactory::compile(
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
        // Repository Matrix kind gate now lives centrally in `handle_request`
        // (Gap 13) — every method in `KNOWLEDGE_REPO_BLOCKED_METHODS`,
        // including this one, is already rejected before reaching here.
        let runtime = self.runtime_for(req)?;

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
        let standard_name = req.params.get("standard").and_then(|v| v.as_str());

        // Standard-driven audit: Model A (audit/pipelines/*.yaml) if the
        // standard shipped one, else the DB-backed path (Phase 1/2 —
        // StandardRegistry.audit_rules/.scoring, populated by
        // knowledge-hub-loader.py at register_standard time). Neither
        // real standard (python_hackathon, base_dev) uses Model A, so this
        // fallback is what actually makes `standard: "python_hackathon"`
        // work, not just `standard: "<a Model A standard>"`.
        if let Some(std_name) = standard_name {
            let runtime = self.runtime_for(req)?;
            let has_model_a = !audit::pipeline_factory::PipelineFactory::load_yaml_pipelines_for_standard(
                &runtime.context.repository_root,
                std_name,
            )
            .is_empty();
            return if has_model_a {
                self.handle_yaml_pipeline_audit(req, std_name)
            } else {
                self.handle_db_backed_standard_audit(req, std_name)
            };
        }

        let pipeline_kind = match pipeline_name {
            Some(name) => schemas::audit::PipelineKind::from_str(name)
                .ok_or_else(|| anyhow::anyhow!(
                    "Unknown pipeline '{}'. Valid values: doc, build, security, consistency, coverage, \
                     architecture, dependency, documentation-structure, vision, design, readme, prototype, \
                     external-context, engineering, feature, feature-technical, feature-design, \
                     deterministic-runtime, external-context-ownership, implementation, help",
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
            let providers = req.params.get("providers")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
                .unwrap_or_else(|| vec!["deterministic".to_string()]);

            let runtime = self.runtime_for(req)?;
            let mut report = runtime.run_pipeline(&pipeline_kind, inspect_artifact, runtime_mode, execute, dry_run)?;
            if providers.iter().any(|p| p == "semantic") {
                report.semantic_review = runtime.build_pipeline_semantic_review(&pipeline_kind)?;
            }
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

    /// Handle YAML pipeline audit execution via the standard parameter.
    /// `domain` picks which of the standard's pipelines to run (a standard
    /// holds one pipeline per domain, e.g. `infrastructure`, `engineering`);
    /// it's required whenever the standard defines more than one.
    /// `model` is optional, self-reported by the caller, and archived with
    /// the run so history can be grouped by it later (leaderboards, etc.) —
    /// samgraha has no protocol-level way to learn it on its own.
    fn handle_yaml_pipeline_audit(&self, req: &McpRequest, standard_name: &str) -> Result<serde_json::Value> {
        use audit::pipeline::Pipeline;

        let runtime = self.runtime_for(req)?;
        let project_root = &runtime.context.repository_root;
        let domain = req.params.get("domain").and_then(|v| v.as_str());
        let model = req.params.get("model").and_then(|v| v.as_str());

        let pipelines =
            audit::pipeline_factory::PipelineFactory::load_yaml_pipelines_for_standard(project_root, standard_name);
        if pipelines.is_empty() {
            anyhow::bail!(
                "No YAML pipelines found for standard '{}'. Expected .samgraha/standards/{}/audit/pipelines/*.yaml",
                standard_name,
                standard_name
            );
        }

        let pipeline = match domain {
            Some(d) => pipelines
                .into_iter()
                .find(|p| p.def.pipeline.name == d)
                .ok_or_else(|| {
                    anyhow::anyhow!("Standard '{}' has no pipeline for domain '{}'", standard_name, d)
                })?,
            None if pipelines.len() == 1 => pipelines.into_iter().next().unwrap(),
            None => {
                let names: Vec<_> = pipelines.iter().map(|p| p.def.pipeline.name.clone()).collect();
                anyhow::bail!(
                    "Standard '{}' defines multiple pipelines ({}); pass 'domain' to select one",
                    standard_name,
                    names.join(", ")
                );
            }
        };

        let ctx = audit::pipeline::PipelineContext::new(
            project_root.to_path_buf(),
            runtime.context.config.clone(),
        );

        let report = pipeline.run(&ctx);
        if let Err(e) = runtime.store_standard_audit_run(standard_name, &pipeline.def.pipeline.name, model, &report) {
            tracing::warn!("Failed to archive standard audit run for '{}': {}", standard_name, e);
        }
        Ok(serde_json::to_value(&report)?)
    }

    /// Handle a standard-driven audit via `StandardRegistry.audit_rules`
    /// (Phase 1/2 of docs/crates-refactor-proposal.md) — the path for a
    /// registered standard with no `audit/pipelines/*.yaml` (Model A), i.e.
    /// both real standards checked against this session. Builds its own
    /// `StandardRegistry`/`AuditFramework` scoped to `standard_name` instead
    /// of using `self.runtime`'s, which is scoped to whatever `system_name`
    /// `samgraha.toml` configured — the two are independent, and a caller
    /// should be able to name any registered standard per-call.
    fn handle_db_backed_standard_audit(&self, req: &McpRequest, standard_name: &str) -> Result<serde_json::Value> {
        let domain = req.params.get("domain").and_then(|v| v.as_str());
        let model = req.params.get("model").and_then(|v| v.as_str());
        let providers = req.params.get("providers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["deterministic".to_string()]);

        let runtime = self.runtime_for(req)?;
        let project_root = runtime.context.repository_root.clone();
        let config = runtime.context.config.clone();

        let standard_registry = std::sync::Arc::new(
            standards::StandardRegistry::from_standards_db_and_overrides_with_system(&project_root, Some(standard_name))
                .with_context(|| format!("Failed to load standard '{}'", standard_name))?,
        );
        if standard_registry.all().is_empty() {
            anyhow::bail!(
                "Standard '{}' has no registered domains. Check it was registered via register_standard.",
                standard_name
            );
        }

        let mut framework = audit::framework::AuditFramework::new(std::sync::Arc::clone(&standard_registry));
        let script_checks = standard_registry.script_checks().to_vec();
        let det_config = config.clone();
        let det_root = project_root.clone();
        framework.register_provider("deterministic", std::sync::Arc::new(move |docs, rules, standard| {
            audit::providers::DeterministicAuditProvider::execute(docs, rules, standard, Some(&det_config), Some(&det_root), &script_checks)
        }));
        framework.register_provider("semantic", std::sync::Arc::new(|docs, rules, standard| {
            services::SemanticAuditProvider::execute(docs, rules, standard)
        }));

        // Standard-driven rules (file_presence/glob_match/script_output) act
        // on the repository filesystem directly via `root`, not on compiled
        // Document objects the way samgraha's own built-in domains do — no
        // document set to pass here.
        let report = framework.execute(domain, &[], &providers)?;

        if let Some(d) = domain {
            let report_json = serde_json::to_string(&report)?;
            if let Err(e) = runtime.registry.store_standard_audit_run(
                standard_name, d, model, report.score.overall, &report_json, None,
            ) {
                tracing::warn!("Failed to archive standard audit run for '{}': {}", standard_name, e);
            }
        }

        Ok(serde_json::to_value(&report)?)
    }

    fn handle_audit_runs(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let standard = req.params.get("standard").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'standard' parameter"))?;
        let domain = req.params.get("domain").and_then(|v| v.as_str());
        let limit = req.params.get("limit").and_then(|v| v.as_i64()).unwrap_or(20);
        let runtime = self.runtime_for(req)?;
        let runs = runtime.list_standard_audit_runs(standard, domain, limit)?;
        Ok(serde_json::to_value(&runs)?)
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

    // ── Content Generation MCP Methods ──────────────────────────────────

    /// Build a content-generation task list mirroring `build_semantic_review`'s
    /// pattern. Mode is resolved from the standard's `generation_granularity`,
    /// never caller-supplied (same principle as domain cardinality).
    fn handle_generate(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain_key = req.params.get("standard")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'standard' parameter (domain key)"))?;
        let runtime = self.runtime_for(req)?;

        // Look up the StandardDefinition for this domain.
        let std_def = runtime.standard_registry.get_by_domain(domain_key)
            .ok_or_else(|| anyhow::anyhow!("Unknown domain '{}'", domain_key))?;

        let mode = std_def.generation_granularity.clone();
        let content_kind = std_def.content_kind.clone();

        if content_kind == "code" {
            // Code domains skip the generate→store→assemble pipeline entirely.
            // scaffold creates folder structure; caller writes files directly.
            return Ok(serde_json::json!({
                "mode": "code",
                "instruction": "This domain generates code, not documentation. \
                    Use run_system_scaffold to create the folder structure, \
                    then write files directly. store_generated_content and \
                    assemble are not used for code domains.",
                "tasks": [],
            }));
        }

        // Collect already-generated section semantic_types from knowledge.db
        // to skip sections that have content stored already.
        let generated_sections: std::collections::HashSet<String> = {
            let mut set = std::collections::HashSet::new();
            if let Ok(Some(doc)) = runtime.registry.get_document_by_path(
                &format!(".samgraha/generated/{}", domain_key)
            ) {
                if let Ok(sections) = runtime.registry.get_all_sections_for_document(doc.id) {
                    for sec in sections {
                        set.insert(sec.semantic_type);
                    }
                }
            }
            set
        };

        // Build section tasks: one per required_sections entry not yet generated.
        let mut tasks: Vec<serde_json::Value> = Vec::new();

        // Build mapping: section_catalog_id → semantic_type for resolving
        // section_dependencies edges (which use DB IDs) to semantic_type strings.
        let catalog_id_to_type: std::collections::HashMap<i64, String> = std_def.required_sections.iter()
            .filter_map(|s| s.section_catalog_id.map(|id| (id, s.semantic_type.clone())))
            .collect();
        // Inverse: semantic_type → section_catalog_id (used by store handler)
        let _type_to_catalog_id: std::collections::HashMap<String, i64> = std_def.required_sections.iter()
            .filter_map(|s| s.section_catalog_id.map(|id| (s.semantic_type.clone(), id)))
            .collect();
        // Build dependency edges: section_semantic_type → Vec<dependency_semantic_type>
        let deps_by_type: std::collections::HashMap<String, Vec<String>> = {
            let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            for &(sec_id, dep_id) in &std_def.section_dependencies {
                if let (Some(sec_type), Some(dep_type)) = (catalog_id_to_type.get(&sec_id), catalog_id_to_type.get(&dep_id)) {
                    map.entry(sec_type.clone()).or_default().push(dep_type.clone());
                }
            }
            map
        };

        for (idx, sec_def) in std_def.required_sections.iter().enumerate() {
            if generated_sections.contains(&sec_def.semantic_type) {
                continue;
            }

            // For section mode: defer sections whose upstream dependencies
            // haven't been generated yet — using explicit section_dependencies
            // edges, not sort_order position.
            if mode == "section" || mode == "hybrid" {
                if let Some(deps) = deps_by_type.get(&sec_def.semantic_type) {
                    let deps_met = deps.iter()
                        .all(|dep| generated_sections.contains(dep));
                    if !deps_met {
                        continue; // defer to later call
                    }
                }
            }

            let instruction = format!(
                "Generate content for the '{}' section of domain '{}'. \
                 Use the template content and upstream context provided. \
                 When done, call store_generated_content with the result.",
                sec_def.canonical_name, domain_key
            );

            tasks.push(serde_json::json!({
                "target": {
                    "domain": domain_key,
                    "section": sec_def.semantic_type,
                },
                "template_content": null, // templates loaded from standards.db by caller
                "upstream_context": null, // populated by caller from already-generated domains
                "instruction": instruction,
                "section_index": idx,
                "total_sections": std_def.required_sections.len(),
            }));
        }

        let overall_instruction = format!(
            "Generate documentation content for domain '{}'. \
             Mode: {}. Complete each task by generating content and calling \
             store_generated_content. Sections already generated are skipped.",
            domain_key, mode
        );

        Ok(serde_json::json!({
            "mode": mode,
            "domain": domain_key,
            "content_kind": content_kind,
            "instruction": overall_instruction,
            "tasks": tasks,
            "tasks_count": tasks.len(),
        }))
    }

    /// Accept generated content back and persist it to knowledge.db's
    /// `documents`/`document_sections` — mirrors `store_section_report`'s
    /// pattern for audit findings.
    fn handle_store_generated_content(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain_key = req.params.get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'domain' parameter"))?;
        let content = req.params.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let section = req.params.get("section")
            .and_then(|v| v.as_str());
        let git_revision = req.params.get("git_revision")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let document_id_opt = req.params.get("document_id")
            .and_then(|v| v.as_i64());

        let runtime = self.runtime_for(req)?;

        // Validate domain exists in the standard.
        let std_def = runtime.standard_registry.get_by_domain(domain_key)
            .ok_or_else(|| anyhow::anyhow!("Unknown domain '{}'", domain_key))?;

        if std_def.content_kind == "code" {
            anyhow::bail!(
                "Domain '{}' is a code domain — store_generated_content is not used. \
                 Write files directly after scaffold.",
                domain_key
            );
        }

        // If section is provided, validate it exists in section_catalog.
        if let Some(sec_type) = section {
            let sec_exists = std_def.required_sections.iter()
                .any(|s| s.semantic_type == sec_type);
            if !sec_exists {
                anyhow::bail!(
                    "Section '{}' not found in domain '{}' section_catalog",
                    sec_type, domain_key
                );
            }
        }

        // Check section dependencies (§5.1 of proposal): reject if any
        // dependency hasn't been generated yet — using explicit
        // section_dependencies edges, not sort_order position.
        if let Some(sec_type) = section {
            // Build mapping from section_catalog_id → semantic_type
            let catalog_id_to_type: std::collections::HashMap<i64, String> = std_def.required_sections.iter()
                .filter_map(|s| s.section_catalog_id.map(|id| (id, s.semantic_type.clone())))
                .collect();
            let type_to_catalog_id: std::collections::HashMap<String, i64> = std_def.required_sections.iter()
                .filter_map(|s| s.section_catalog_id.map(|id| (s.semantic_type.clone(), id)))
                .collect();

            // Get the set of already-generated sections for this domain.
            let generated: std::collections::HashSet<String> = {
                let mut set = std::collections::HashSet::new();
                if let Ok(Some(doc)) = runtime.registry.get_document_by_path(
                    &format!(".samgraha/generated/{}", domain_key)
                ) {
                    if let Ok(sections) = runtime.registry.get_all_sections_for_document(doc.id) {
                        for sec in sections {
                            set.insert(sec.semantic_type);
                        }
                    }
                }
                set
            };

            // Find this section's dependencies via section_dependencies table.
            let missing: Vec<String> = if let Some(&sec_cat_id) = type_to_catalog_id.get(sec_type) {
                std_def.section_dependencies.iter()
                    .filter(|&&(sec_id, _)| sec_id == sec_cat_id)
                    .filter_map(|&(_, dep_id)| catalog_id_to_type.get(&dep_id))
                    .filter(|dep_type| !generated.contains(*dep_type))
                    .cloned()
                    .collect()
            } else {
                Vec::new() // no catalog ID — builtin standard, skip check
            };

            if !missing.is_empty() {
                return Ok(serde_json::json!({
                    "error": "dependency_unmet",
                    "missing_sections": missing,
                    "message": format!(
                        "Section '{}' depends on sections that haven't been generated yet: {:?}",
                        sec_type, missing
                    ),
                }));
            }
        }

        // Determine or create the document for this domain.
        let doc_path = format!(".samgraha/generated/{}", domain_key);
        let doc_id = if let Some(id) = document_id_opt {
            id
        } else if let Ok(Some(doc)) = runtime.registry.get_document_by_path(&doc_path) {
            doc.id
        } else {
            // Create a new document.
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let doc = schemas::document::Document {
                id: 0, // auto-increment
                path: schemas::document::DocumentPath(std::path::PathBuf::from(&doc_path)),
                hash: String::new(),
                standard: domain_key.to_string(),
                title: format!("Generated: {}", domain_key),
                body: schemas::document::DocumentBody::Generic {
                    raw: String::new(),
                    sections: Vec::new(),
                },
                metadata: schemas::document::DocumentMetadata {
                    title: format!("Generated: {}", domain_key),
                    purpose: "Auto-generated by content generation pipeline".to_string(),
                    document_type: Some("generated".to_string()),
                    status: Some("generating".to_string()),
                    ownership: None,
                    tags: vec!["generated".to_string()],
                    extra: std::collections::HashMap::new(),
                },
                provenance: None,
                quality: Default::default(),
                created_at: now.clone(),
                updated_at: now,
            };
            let body_json = serde_json::to_string(&doc.body)?;
            let meta_json = serde_json::to_string(&doc.metadata)?;
            let qual_json = serde_json::to_string(&doc.quality)?;
            runtime.registry.conn.execute(
                "INSERT INTO documents (path, hash, standard, title, body, metadata, quality, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    doc.path.as_str(),
                    doc.hash,
                    doc.standard,
                    doc.title,
                    body_json,
                    meta_json,
                    qual_json,
                    doc.created_at,
                    doc.updated_at,
                ],
            )?;
            runtime.registry.conn.last_insert_rowid()
        };

        // Write the section content.
        let semantic_type = section.unwrap_or("full_document");
        let canonical_name = if let Some(sec_type) = section {
            std_def.required_sections.iter()
                .find(|s| s.semantic_type == sec_type)
                .map(|s| s.canonical_name.clone())
                .unwrap_or_else(|| sec_type.to_string())
        } else {
            domain_key.to_string()
        };

        let section_obj = schemas::document::DocumentSection {
            heading: canonical_name.clone(),
            semantic_type: semantic_type.to_string(),
            level: 1,
            body: content.to_string(),
            required: true,
            source_span: None,
            subsections: Vec::new(),
            hash: String::new(),
        };

        // Concurrent-write protection (§5.1 of proposal): if a section with
        // the same semantic_type already exists and was generated by a
        // different git_revision, reject — the caller must re-generate
        // against the now-updated upstream context.
        //
        // Atomic: savepoint wraps check + delete + insert so concurrent
        // callers cannot both pass the check before either writes.
        // Uses raw SQL savepoint commands (&self, no &mut needed).
        let sp_name = format!("gen_store_{}_{}", doc_id, semantic_type.replace('/', "_"));
        {
            runtime.registry.conn.execute(
                &format!("SAVEPOINT {}", sp_name), [],
            )?;

            // Conflict check inside the savepoint.
            let mut conflict = false;
            if let Ok(Some(existing)) = runtime.registry.get_document_by_path(&doc_path) {
                if let Ok(existing_sections) = runtime.registry.get_all_sections_for_document(existing.id) {
                    for existing_sec in &existing_sections {
                        if existing_sec.semantic_type == semantic_type {
                            if !git_revision.is_empty() && existing_sec.content != content {
                                conflict = true;
                            }
                            break;
                        }
                    }
                }
            }

            if conflict {
                runtime.registry.conn.execute(
                    &format!("ROLLBACK TO SAVEPOINT {}", sp_name), [],
                )?;
                return Ok(serde_json::json!({
                    "error": "conflict",
                    "message": format!(
                        "Section '{}' already exists with different content. \
                         A concurrent generation session has written to this section. \
                         Re-generate against the updated upstream context and retry.",
                        semantic_type
                    ),
                    "document_id": doc_id,
                    "section": semantic_type,
                }));
            }

            // Delete + insert inside the same savepoint.
            runtime.registry.conn.execute(
                "DELETE FROM document_sections WHERE document_id = ?1 AND semantic_type = ?2",
                rusqlite::params![doc_id, semantic_type],
            )?;
            runtime.registry.insert_document_sections(doc_id, &[section_obj])?;

            runtime.registry.conn.execute(
                &format!("RELEASE SAVEPOINT {}", sp_name), [],
            )?;
        }

        Ok(serde_json::json!({
            "status": "stored",
            "document_id": doc_id,
            "domain": domain_key,
            "section": semantic_type,
            "git_revision": git_revision,
        }))
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

    fn handle_store_pipeline_check_report(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let report_json = req.params.get("report_json")
            .ok_or_else(|| anyhow::anyhow!("Missing 'report_json' parameter"))?;
        let report: schemas::audit::PipelineCheckReport = serde_json::from_value(report_json.clone())
            .map_err(|e| anyhow::anyhow!("Invalid report schema: {}", e))?;
        let id = self.runtime_for(req)?.store_pipeline_check_report(&report)?;
        Ok(serde_json::json!({"report_id": id, "status": "stored"}))
    }

    fn handle_get_pipeline_check_report(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let pipeline = req.params.get("pipeline")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pipeline' parameter"))?;
        let check_id = req.params.get("check_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'check_id' parameter"))?;
        match self.runtime_for(req)?.get_pipeline_check_report(pipeline, check_id)? {
            Some(r) => Ok(serde_json::to_value(&r)?),
            None => Ok(serde_json::json!({"report": null, "pipeline": pipeline, "check_id": check_id})),
        }
    }

    fn handle_check_pipeline_gate(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let pipeline = req.params.get("pipeline")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pipeline' parameter"))?;
        let result = self.runtime_for(req)?.check_pipeline_gate(pipeline)?;
        Ok(serde_json::to_value(&result)?)
    }

    fn handle_get_summary_report(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let target_type = req.params.get("target_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'target_type' parameter"))?;
        let target_name = req.params.get("target_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'target_name' parameter"))?;
        let report = self.runtime_for(req)?.get_summary_report(target_type, target_name)?;
        Ok(serde_json::to_value(&report)?)
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

    fn handle_update_report_finding_status(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let finding_id = req.params.get("finding_id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'finding_id' parameter"))?;
        let status = req.params.get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'status' parameter"))?;
        if !matches!(status, "open" | "fixed" | "accepted" | "ignored" | "false_positive") {
            return Err(anyhow::anyhow!("Invalid status: {}", status));
        }

        self.runtime_for(req)?.update_report_finding_status(finding_id, status)?;
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
            "markdown": rendered.markdown,
            "json": rendered.json,
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

    // ── Standards management ──────────────────────────────────────────────

    fn handle_list_standards(&self) -> Result<serde_json::Value> {
        let standards = self.runtime.standard_registry.all();
        let items: Vec<serde_json::Value> = standards
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "version": s.version,
                    "domain": s.domain,
                    "description": s.description,
                    "rules_count": s.audit_rules.len(),
                    "sections_count": s.required_sections.len(),
                })
            })
            .collect();
        Ok(serde_json::json!({
            "standards": items,
            "total": items.len(),
        }))
    }

    fn handle_get_standard(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain = parse_string(req, "domain")?;
        let version = req.params.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0");
        let std = self.runtime.standard_registry
            .get(&domain, version)
            .ok_or_else(|| anyhow::anyhow!("Standard '{}/{}' not found", domain, version))?;
        Ok(serde_json::to_value(std)?)
    }

    /// Registers into the shared `standards.db` shipped next to this binary
    /// (`common::env::mcp_dir()`) by default — every repo's `sync_standards`
    /// pulls from there. Pass `local: true` to write straight into this
    /// repo's own `.samgraha/standards.db` instead (self-hosting/bootstrap
    /// only — bypasses sync).
    fn handle_get_standard_doc(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain = parse_string(req, "domain")?;
        let doc = self.runtime.standard_registry
            .get_standard_doc(&domain)
            .ok_or_else(|| anyhow::anyhow!("No standard doc for domain '{}'", domain))?;
        Ok(serde_json::to_value(doc)?)
    }

    fn handle_register_standard(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let path = parse_string(req, "path")?;
        let path = std::path::PathBuf::from(&path);
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }
        let no_push = req.params.get("no_push").and_then(|v| v.as_bool()).unwrap_or(false);
        let local_db = self.runtime.context.repository_root.join(".samgraha").join("standards.db");
        if let Some(parent) = local_db.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Shared with CLI `knowledge publish` (Optimization 8) — same
        // loader-invocation logic, one place.
        let loader = services::knowledge_publish::resolve_knowledge_hub_loader()?;
        let system = req.params.get("system").and_then(|v| v.as_str());
        let layout = req.params.get("layout").and_then(|v| v.as_str()).map(std::path::Path::new);
        let dry_run = req.params.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);

        // Soft-warning: abstract systems should not be registered standalone.
        if let Some(sys_name) = system {
            let system_yaml = path.join("system.yaml");
            if system_yaml.is_file() {
                if let Ok(contents) = std::fs::read_to_string(&system_yaml) {
                    if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&contents) {
                        if doc.get("abstract").and_then(|v| v.as_bool()).unwrap_or(false) {
                            tracing::warn!(
                                "System '{}' is marked abstract — it provides the base domain set \
                                 and should not be registered standalone. Register a concrete \
                                 system (e.g. electron_dev, fastapi_dev) that extends '{}'.",
                                sys_name, sys_name
                            );
                        }
                    }
                }
            }
        }

        let loader_output = services::knowledge_publish::run_knowledge_hub_loader(
            &loader, &local_db, &path, system, layout, dry_run,
        )?;

        // --- data_tables prefix collision check (§2.2) ---
        // Must happen before asset copy so a bad prefix fails fast.
        let system_yaml = path.join("system.yaml");
        let assets = if system_yaml.is_file() {
            std::fs::read_to_string(&system_yaml)
                .ok()
                .and_then(|c| serde_yaml::from_str::<StandardAssets>(&c).ok())
                .unwrap_or_default()
        } else {
            StandardAssets::default()
        };
        if let Some(ref dt) = assets.data_tables {
            let prefix = dt.prefix.trim();
            if prefix.is_empty() {
                return Err(anyhow::anyhow!(
                    "data_tables.prefix must be non-empty"
                ));
            }
            // Check against every reserved table name — the prefix
            // must not match any reserved name exactly, and no reserved
            // name may start with this prefix (would mean a future
            // core table lands in the standard's namespace).
            for &reserved in migration::RESERVED_TABLE_NAMES {
                if reserved == prefix || reserved.starts_with(prefix) {
                    return Err(anyhow::anyhow!(
                        "data_tables.prefix '{}' collides with reserved table name '{}'. \
                         Choose a different prefix.",
                        prefix, reserved
                    ));
                }
            }
        }

        // A standard's own script/ and templates/ directories — recursively
        // copied into local .samgraha/ for immediate use and into the
        // namespaced global store (mcp_dir()/systems/<name>/) so a future
        // sync in another repo picks them up.
        let mut scripts_copied = 0usize;
        let mut templates_copied = 0usize;

        // Pre-compute excludes and system name outside the dry_run gate
        // so audit/analysis and catalog sections can use them too.
        let excludes = effective_excludes(&assets.exclude);
        let exclude_refs: Vec<&str> = excludes.iter().map(|s| s.as_str()).collect();

        let sys_name = system
            .map(|s| s.to_string())
            .or_else(|| {
                if system_yaml.is_file() {
                    std::fs::read_to_string(&system_yaml)
                        .ok()
                        .and_then(|c| serde_yaml::from_str::<serde_yaml::Value>(&c).ok())
                        .and_then(|v| v.get("name")?.as_str()?.to_string().into())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        if !dry_run {
            // --- Scripts ---
            let source_scripts = path.join(&assets.scripts);
            if source_scripts.is_dir() {
                // Local: .samgraha/scripts/
                let local_scripts = self.runtime.context.repository_root
                    .join(".samgraha").join("scripts");
                scripts_copied += common::fs_sync::copy_dir_recursive(
                    &source_scripts, &local_scripts, &exclude_refs,
                )?;
                // Global: mcp_dir()/systems/<name>/scripts/
                if !no_push {
                    let global_scripts = common::env::mcp_dir()
                        .join("systems").join(&sys_name).join("scripts");
                    common::fs_sync::copy_dir_recursive(
                        &source_scripts, &global_scripts, &exclude_refs,
                    )?;
                }
            }

            // --- Templates ---
            let source_templates = path.join(&assets.templates);
            if source_templates.is_dir() {
                // Local: .samgraha/templates/
                let local_templates = self.runtime.context.repository_root
                    .join(".samgraha").join("templates");
                templates_copied = common::fs_sync::copy_dir_recursive(
                    &source_templates, &local_templates, &exclude_refs,
                )?;
                // Global: mcp_dir()/systems/<name>/templates/
                if !no_push {
                    let global_templates = common::env::mcp_dir()
                        .join("systems").join(&sys_name).join("templates");
                    common::fs_sync::copy_dir_recursive(
                        &source_templates, &global_templates, &exclude_refs,
                    )?;
                }
            }
        }

        // --- Audit/analysis assets (§7: seed-once) ---
        // Copied to global store like scripts/templates, but during sync
        // they're seed-once: first init copies, re-sync never overwrites.
        let mut seed_once_copied = 0usize;
        if !dry_run {
            for aa_dir_name in &assets.audit_analysis {
                let source_aa = path.join(aa_dir_name);
                if source_aa.is_dir() {
                    // Local: .samgraha/<dir_name>/
                    let local_aa = self.runtime.context.repository_root
                        .join(".samgraha").join(aa_dir_name);
                    seed_once_copied += common::fs_sync::copy_dir_recursive(
                        &source_aa, &local_aa, &exclude_refs,
                    )?;
                    // Global: mcp_dir()/systems/<name>/<dir_name>/
                    if !no_push {
                        let global_aa = common::env::mcp_dir()
                            .join("systems").join(&sys_name).join(aa_dir_name);
                        common::fs_sync::copy_dir_recursive(
                            &source_aa, &global_aa, &exclude_refs,
                        )?;
                    }
                }
            }
        }

        // --- Populate standard_assets + custom_data_tables catalogs ---
        // Writes metadata about declared scripts/prompts and custom tables
        // into the repo's knowledge.db for discoverability and name→path
        // resolution by the execution engine (§3.2, §5).
        let mut catalog_result = services::CatalogPopulateResult::default();
        if !dry_run {
            let knowledge_db = self.runtime.context.repository_root
                .join(".samgraha").join("knowledge.db");
            if knowledge_db.exists() {
                let catalog_std = system.unwrap_or_else(|| sys_name.as_str());

                // Map adapter-local CatalogEntry to shared CatalogEntry.
                let catalog_entries: Vec<services::CatalogEntry> = assets.catalog
                    .iter()
                    .map(|e| services::CatalogEntry {
                        name: e.name.clone(),
                        kind: e.kind.clone(),
                        path: e.path.clone(),
                        purpose: e.purpose.clone(),
                    })
                    .collect();

                // Map adapter-local DataTablesConfig to shared DataTablesConfig.
                let dt_config = assets.data_tables.as_ref().map(|dt| {
                    // Expand prefix glob into a single "table_name" entry for
                    // the catalog. The actual tables are created by owner_script;
                    // we just record the declared prefix for collision detection.
                    services::DataTablesConfig {
                        owner_script: dt.owner_script.clone(),
                        prefix: dt.prefix.clone(),
                        purpose: dt.purpose.clone(),
                        tables: vec![format!("{}*", dt.prefix)],
                    }
                });

                catalog_result = services::catalog::populate_standard_catalogs(
                    &knowledge_db,
                    catalog_std,
                    &catalog_entries,
                    dt_config.as_ref(),
                )?;
            }
        }

        if !dry_run && !no_push {
            let global_db = common::env::mcp_dir().join("standards.db");
            if let Some(parent) = global_db.parent() {
                std::fs::create_dir_all(parent)?;
            }
            {
                let check_conn = rusqlite::Connection::open(&local_db)?;
                let ok: String = check_conn.query_row(
                    "PRAGMA integrity_check", [], |row| row.get(0)
                )?;
                if ok != "ok" {
                    return Err(anyhow::anyhow!("Local standards DB failed integrity check: {}", ok));
                }
                standards::check_schema_version(&check_conn)?;
            }
            knowledge_publish::check_push_safe(&local_db, &global_db)?;
            std::fs::copy(&local_db, &global_db)?;
        }

        Ok(serde_json::json!({
            "success": true,
            "dry_run": dry_run,
            "db_path": local_db.display().to_string(),
            "pushed": !dry_run && !no_push,
            "scripts_copied": scripts_copied,
            "templates_copied": templates_copied,
            "custom_tables_cataloged": catalog_result.data_tables_inserted,
            "assets_cataloged": catalog_result.assets_inserted,
            "seed_once_copied": seed_once_copied,
            "loader_output": loader_output,
            "message": if dry_run {
                "Dry run complete — nothing written."
            } else if no_push {
                "Knowledge System registered locally. Use push_standards to push to global."
            } else {
                "Knowledge System registered locally and pushed to global."
            }
        }))
    }

    fn handle_push_standards(&self) -> Result<serde_json::Value> {
        let local_db = self.runtime.context.repository_root.join(".samgraha").join("standards.db");
        if !local_db.exists() {
            return Err(anyhow::anyhow!("No local .samgraha/standards.db — register a Knowledge System first"));
        }
        let global_db = common::env::mcp_dir().join("standards.db");
        if let Some(parent) = global_db.parent() {
            std::fs::create_dir_all(parent)?;
        }
        {
            let check_conn = rusqlite::Connection::open(&local_db)?;
            let ok: String = check_conn.query_row(
                "PRAGMA integrity_check", [], |row| row.get(0)
            )?;
            if ok != "ok" {
                return Err(anyhow::anyhow!("Local standards DB failed integrity check: {}", ok));
            }
            standards::check_schema_version(&check_conn)?;
        }
        knowledge_publish::check_push_safe(&local_db, &global_db)?;
        std::fs::copy(&local_db, &global_db)?;
        Ok(serde_json::json!({
            "success": true,
            "pushed_from": local_db.display().to_string(),
            "pushed_to": global_db.display().to_string(),
            "message": "Local standards.db pushed to global."
        }))
    }

    /// Flips which registered system is default in the shared `standards.db` —
    /// used by repos whose `samgraha.toml` doesn't set `standard_system`.
    fn handle_set_default_standard(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let system = parse_string(req, "system")?;
        let db_path = common::env::mcp_dir().join("standards.db");
        if !db_path.exists() {
            return Err(anyhow::anyhow!("No standards.db at {} — register a system first", db_path.display()));
        }
        let conn = rusqlite::Connection::open(&db_path)?;
        let exists: bool = conn.query_row(
            "SELECT 1 FROM systems WHERE name = ?",
            [&system],
            |_| Ok(true),
        ).unwrap_or(false);
        if !exists {
            return Err(anyhow::anyhow!("System '{}' not found in {}", system, db_path.display()));
        }
        conn.execute("UPDATE systems SET is_default = 0 WHERE is_default = 1", [])?;
        conn.execute("UPDATE systems SET is_default = 1 WHERE name = ?", [&system])?;
        Ok(serde_json::json!({ "success": true, "default_system": system }))
    }

    /// Pulls the repo's configured (or default) documentation-standard system,
    /// plus `help` content, from the mcp-adjacent `standards.db`/`help.db`
    /// into this repo's local `.samgraha/standards.db` / `knowledge.db`. Local
    /// DBs are the only thing queried at runtime after this — no MCP call
    /// switches between databases once a repo is synced.
    fn handle_sync_standards(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let force = req.params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let owned_root;
        let root: &Path = match req.params.get("repo_path").and_then(|v| v.as_str()) {
            Some(p) => {
                owned_root = std::path::PathBuf::from(p);
                &owned_root
            }
            None => &self.runtime.context.repository_root,
        };

        match services::init::sync_if_stale(root, force)? {
            None => Ok(serde_json::json!({
                "synced": false,
                "reason": "up_to_date",
            })),
            Some(result) => Ok(serde_json::json!({
                "synced": true,
                "standards_synced": result.standards_synced,
                "help_documents_synced": result.help_documents_synced,
                "scripts_synced": result.scripts_synced,
            })),
        }
    }

    fn handle_check_staleness(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let owned_root;
        let root: &Path = match req.params.get("repo_path").and_then(|v| v.as_str()) {
            Some(p) => {
                owned_root = std::path::PathBuf::from(p);
                &owned_root
            }
            None => &self.runtime.context.repository_root,
        };
        let status = services::init::check_knowledge_staleness(root)?;
        Ok(serde_json::json!({
            "status": match &status {
                services::init::StalenessStatus::NeverSynced => "never_synced",
                services::init::StalenessStatus::MissingLocal => "missing_local",
                services::init::StalenessStatus::UpToDate { .. } => "up_to_date",
                services::init::StalenessStatus::Stale { .. } => "stale",
                services::init::StalenessStatus::SourceMissing => "source_missing",
            },
            "detail": format!("{:?}", status),
        }))
    }

    // ── Phase 5: plan orchestration metadata ──────────────────────────────

    fn handle_get_plan_settings(&self) -> Result<serde_json::Value> {
        let settings = self.runtime.standard_registry.plan_settings();
        let items: Vec<serde_json::Value> = settings
            .iter()
            .map(|s| serde_json::json!({
                "threshold_rating": s.threshold_rating,
                "max_iterations": s.max_iterations,
                "fallback": s.fallback,
                "note": s.note,
            }))
            .collect();
        Ok(serde_json::json!({ "plan_settings": items, "total": items.len() }))
    }

    fn handle_get_plan_scenarios(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let tier_filter = req.params.get("tier").and_then(|v| v.as_str()).and_then(|t| t.parse::<i32>().ok());
        let scenarios = self.runtime.standard_registry.plan_scenarios();
        let items: Vec<serde_json::Value> = scenarios
            .iter()
            .filter(|s| tier_filter.map_or(true, |t| s.tier == t))
            .map(|s| serde_json::json!({
                "repo_state": s.repo_state,
                "doc_state": s.doc_state,
                "tier": s.tier,
                "step": s.step,
                "content": s.content,
            }))
            .collect();
        Ok(serde_json::json!({ "plan_scenarios": items, "total": items.len() }))
    }

    fn handle_list_script_checks(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain_filter = req.params.get("domain").and_then(|v| v.as_str());
        let checks = self.runtime.standard_registry.script_checks();
        let items: Vec<serde_json::Value> = checks
            .iter()
            .filter(|c| {
                domain_filter.map_or(true, |d| {
                    c.domain_id.as_deref() == Some(d)
                })
            })
            .map(|c| serde_json::json!({
                "check_name": c.check_name,
                "domain": c.domain_id,
                "category": c.category,
                "timeout_seconds": c.timeout_seconds,
                "requires_network": c.requires_network,
                "result_schema": c.result_schema,
                "description": c.description,
            }))
            .collect();
        Ok(serde_json::json!({ "script_checks": items, "total": items.len() }))
    }

    // ── Check runner ──────────────────────────────────────────────────────

    fn handle_run_check(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let check_name = parse_string(req, "name")?;
        let repo_root = req.params.get("repo_root")
            .and_then(|v| v.as_str())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| self.runtime.context.repository_root.clone());

        let config = load_repo_config(&repo_root);

        if let Some(source) = audit::check_runner::resolve_check(
            &check_name,
            &repo_root,
            Some(&config),
        ) {
            let result = audit::check_runner::execute_check(&source, &repo_root, &check_name);
            Ok(serde_json::json!({
                "result": result,
                "source": format!("{:?}", source),
            }))
        } else {
            Ok(serde_json::json!({
                "result": {
                    "check_name": check_name,
                    "status": "skip",
                    "message": format!("No implementation found for check '{}'", check_name),
                    "duration_ms": 0,
                },
                "source": null,
            }))
        }
    }

    /// Generic capability runner — accepts a `capability` param and dispatches
    /// through the 5-tier discovery chain. Shared core for the 6 dedicated tools.
    fn handle_run_system_script(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let cap_name = parse_string(req, "capability")?;
        let capability = audit::capability::Capability::from_name(&cap_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown capability '{}' — valid names: validate, calculate, report, scaffold, plan-generation, init", cap_name))?;
        self.run_capability_for(req, &capability)
    }

    /// Dedicated tool handler: capability is baked in by the tool name.
    fn handle_run_system_script_with_capability(
        &self,
        req: &McpRequest,
        capability: audit::capability::Capability,
    ) -> Result<serde_json::Value> {
        self.run_capability_for(req, &capability)
    }

    /// Shared implementation for all capability dispatch tools.
    fn run_capability_for(
        &self,
        req: &McpRequest,
        capability: &audit::capability::Capability,
    ) -> Result<serde_json::Value> {
        let repo_root = req.params.get("repo_root")
            .and_then(|v| v.as_str())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| self.runtime.context.repository_root.clone());

        let config = load_repo_config(&repo_root);

        let timeout_secs = req.params.get("timeout_secs")
            .and_then(|v| v.as_u64());

        // Optional phase_id: when provided, check prerequisites before
        // executing (§8.6). When omitted, skip the check (backward-compat).
        let phase_id = req.params.get("phase_id").and_then(|v| v.as_str());
        let system_name = req.params.get("system_name").and_then(|v| v.as_str());

        let runtime = self.runtime_for(req)?;
        let repo_fp = common::env::repo_fingerprint(&repo_root);
        let current_head = common::env::current_head_sha(&repo_root);
        // Best-effort: a repo with no standard registered yet can still run
        // a capability script purely off `resolve_capability`'s file-based
        // discovery (pre-existing behavior for `validate`) — only phase
        // gating and run-tracking need a resolved standard, so failure here
        // isn't fatal to the capability call itself.
        let standard_id: Option<i64> =
            resolve_standard_id(&runtime.registry.conn, system_name).ok();

        // ── Prerequisite gate (§8.6) ───────────────────────────────────
        // `gated_expiry` carries the resolved phase's expiry rule forward to
        // the run-tracking write below, so a successful run records the
        // rule its own plan declared, not a guess.
        let mut gated_expiry: Option<audit::capability::ExpiryRule> = None;
        if let (Some(pid), Some(sid)) = (phase_id, standard_id) {
            // Look up the phase from the normalized workflow tables.
            let phase_info: Option<(String, Option<String>)> = runtime.registry.conn.query_row(
                "SELECT wp.kind, wp.expiry_rule_json FROM workflow_phases wp
                 JOIN workflow_use_cases wuc ON wp.use_case_id = wuc.id
                 WHERE wuc.standard_id = ?1 AND wp.phase_id = ?2",
                rusqlite::params![sid, pid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).ok();

            if let Some((kind, expiry_json)) = phase_info {
                let check = audit::capability::check_phase_prerequisites(
                    &runtime.registry.conn,
                    sid,
                    &repo_fp,
                    pid,
                    current_head.as_deref(),
                );
                if check.blocked {
                    return Ok(serde_json::json!({
                        "blocked": true,
                        "reason": check.reason,
                        "phase_id": check.phase_id,
                        "phase_kind": kind,
                        "message": check.message,
                        "how_to_run": {
                            "tool": "run_system_script",
                            "args": {
                                "capability": capability.to_string(),
                                "phase_id": pid,
                            }
                        }
                    }));
                }
                if let Some(ej) = expiry_json {
                    if let Ok(rule) = serde_json::from_str::<audit::capability::ExpiryRule>(&ej) {
                        gated_expiry = Some(rule);
                    }
                }
            }
        }

        // ── Build input JSON payload ───────────────────────────────────
        let target_val = req.params.get("target").and_then(|v| v.as_str());
        let mut input_payload: serde_json::Value =
            match req.params.get("input_json").and_then(|v| v.as_str()) {
                Some(p) => {
                    let raw = std::fs::read_to_string(p)
                        .with_context(|| format!("Failed to read input_json at '{}'", p))?;
                    serde_json::from_str(&raw)
                        .with_context(|| format!("input_json at '{}' is not valid JSON", p))?
                }
                None => serde_json::json!({}),
            };
        if let Some(t) = target_val {
            input_payload["target"] = serde_json::Value::String(t.to_string());
        }

        let input_path = {
            let p = std::env::temp_dir()
                .join(format!("samgraha-cap-input-{}.json", uuid::Uuid::new_v4()));
            std::fs::write(&p, serde_json::to_string(&input_payload)?)
                .with_context(|| format!("Failed to write temp input JSON to {}", p.display()))?;
            p
        };

        match audit::capability::resolve_capability(capability, &repo_root, Some(&config)) {
            Some(source) => {
                let result = audit::capability::execute_capability(
                    &source,
                    capability,
                    &repo_root,
                    &input_path,
                    timeout_secs,
                );

                // ── Record the run (§8.5) ──────────────────────────────
                // Only on success — a failed run must not satisfy a
                // downstream phase's `depends_on`. Only when a standard
                // resolved — nothing to attribute the run to otherwise.
                if result.status == audit::capability::CapabilityStatus::Ok {
                    if let Some(sid) = standard_id {
                        let cap_name = capability.to_string();
                        let key = phase_id.unwrap_or(cap_name.as_str());
                        let expiry = gated_expiry.as_ref();
                        if let Err(e) = audit::capability::record_script_run(
                            &runtime.registry.conn,
                            sid,
                            &repo_fp,
                            capability,
                            key,
                            expiry,
                            current_head.as_deref(),
                        ) {
                            tracing::warn!("Failed to record script_runs row for '{}': {}", key, e);
                        }
                    }
                }

                Ok(serde_json::json!({
                    "result": result,
                    "source": format!("{:?}", source),
                }))
            }
            None => Ok(serde_json::json!({
                "result": {
                    "capability": capability.to_string(),
                    "status": "error",
                    "message": format!("No {} script found — searched: repo scripts/, .samgraha/scripts/, mcp global scripts/", capability),
                    "written": [],
                    "duration_ms": 0,
                },
                "source": null,
            })),
        }
    }

    /// Store a system's init plan output (§8.4) into normalized tables.
    /// Replaces the old JSON-blob approach: one row per use case, one row
    /// per phase, real edges for dependencies (§2.1).
    fn handle_store_system_plan(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let system_name = parse_string(req, "system_name")?;
        let plan_json_str = parse_string(req, "plan_json")?;

        let plan: audit::capability::InitPlan = serde_json::from_str(&plan_json_str)
            .with_context(|| "plan_json is not valid §8.4 InitPlan JSON")?;

        if plan.system != system_name {
            anyhow::bail!(
                "plan_json.system '{}' does not match system_name '{}'",
                plan.system,
                system_name
            );
        }

        let runtime = self.runtime_for(req)?;
        let standard_id = resolve_standard_id(&runtime.registry.conn, Some(&system_name))?;
        let db = &runtime.registry.conn;

        // Clear old data for this standard (system-level, not per-repo).
        db.execute(
            "DELETE FROM workflow_phase_dependencies WHERE phase_id IN (
                SELECT id FROM workflow_phases WHERE use_case_id IN (
                    SELECT id FROM workflow_use_cases WHERE standard_id = ?1
                )
            )",
            rusqlite::params![standard_id],
        )?;
        db.execute(
            "DELETE FROM workflow_phases WHERE use_case_id IN (
                SELECT id FROM workflow_use_cases WHERE standard_id = ?1
            )",
            rusqlite::params![standard_id],
        )?;
        db.execute(
            "DELETE FROM workflow_use_cases WHERE standard_id = ?1",
            rusqlite::params![standard_id],
        )?;

        let mut total_phases = 0usize;

        // First pass: insert use cases and phases.
        for (_uc_idx, use_case) in plan.use_cases.iter().enumerate() {
            db.execute(
                "INSERT INTO workflow_use_cases (standard_id, use_case_id, label)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![standard_id, use_case.id, use_case.label],
            )?;

            let uc_row_id: i64 = db.query_row(
                "SELECT id FROM workflow_use_cases WHERE standard_id = ?1 AND use_case_id = ?2",
                rusqlite::params![standard_id, use_case.id],
                |row| row.get(0),
            )?;

            for (phase_idx, phase) in use_case.phases.iter().enumerate() {
                let expiry_json = phase.expiry.as_ref()
                    .map(|e| serde_json::to_string(e).unwrap_or_default());

                db.execute(
                    "INSERT INTO workflow_phases
                        (use_case_id, phase_id, sort_order, kind, description,
                         script_name, pre_script, post_script, instruction,
                         expiry_rule_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![
                        uc_row_id, phase.id, phase_idx as i64, phase.kind,
                        phase.description, phase.script, phase.pre_script,
                        phase.post_script, phase.instruction, expiry_json
                    ],
                )?;
                total_phases += 1;
            }
        }

        // Second pass: insert dependency edges.
        for use_case in &plan.use_cases {
            let uc_row_id: i64 = db.query_row(
                "SELECT id FROM workflow_use_cases WHERE standard_id = ?1 AND use_case_id = ?2",
                rusqlite::params![standard_id, use_case.id],
                |row| row.get(0),
            )?;

            for phase in &use_case.phases {
                let phase_row_id: i64 = db.query_row(
                    "SELECT id FROM workflow_phases WHERE use_case_id = ?1 AND phase_id = ?2",
                    rusqlite::params![uc_row_id, phase.id],
                    |row| row.get(0),
                )?;

                for dep_id in &phase.depends_on {
                    let dep_row_id: Option<i64> = db.query_row(
                        "SELECT wp.id FROM workflow_phases wp
                         JOIN workflow_use_cases wuc ON wp.use_case_id = wuc.id
                         WHERE wuc.standard_id = ?1 AND wp.phase_id = ?2",
                        rusqlite::params![standard_id, dep_id],
                        |row| row.get(0),
                    ).ok();

                    if let Some(dep_rid) = dep_row_id {
                        db.execute(
                            "INSERT INTO workflow_phase_dependencies
                                (phase_id, depends_on_phase_id)
                             VALUES (?1, ?2)
                             ON CONFLICT(phase_id, depends_on_phase_id) DO NOTHING",
                            rusqlite::params![phase_row_id, dep_rid],
                        )?;
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "stored": true,
            "system_name": system_name,
            "use_cases": plan.use_cases.len(),
            "total_phases": total_phases,
        }))
    }

    /// Retrieve a stored init plan for a system, reconstructed from the
    /// normalized workflow_use_cases / workflow_phases /
    /// workflow_phase_dependencies tables.
    fn handle_get_system_plan(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let system_name = parse_string(req, "system_name")?;
        let runtime = self.runtime_for(req)?;
        let standard_id = resolve_standard_id(&runtime.registry.conn, Some(&system_name)).ok();

        let standard_id = match standard_id {
            Some(sid) => sid,
            None => return Ok(serde_json::json!({
                "system_name": system_name,
                "plan": null,
            })),
        };

        let db = &runtime.registry.conn;

        // Check if any use cases exist.
        let uc_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM workflow_use_cases WHERE standard_id = ?1",
            rusqlite::params![standard_id],
            |row| row.get(0),
        ).unwrap_or(0);

        if uc_count == 0 {
            return Ok(serde_json::json!({
                "system_name": system_name,
                "plan": null,
            }));
        }

        // Reconstruct InitPlan from normalized tables.
        let mut use_cases_json = Vec::new();

        let mut uc_stmt = db.prepare(
            "SELECT id, use_case_id, label FROM workflow_use_cases
             WHERE standard_id = ?1"
        )?;
        let uc_rows = uc_stmt.query_map(rusqlite::params![standard_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?;

        for uc_row in uc_rows {
            let (uc_db_id, use_case_id, label) = uc_row?;

            let mut phase_stmt = db.prepare(
                "SELECT id, phase_id, kind, description, script_name,
                        pre_script, post_script, instruction, expiry_rule_json
                 FROM workflow_phases
                 WHERE use_case_id = ?1
                 ORDER BY sort_order"
            )?;
            let phase_rows = phase_stmt.query_map(rusqlite::params![uc_db_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            })?;

            let mut phases_json = Vec::new();
            for pr in phase_rows {
                let (phase_db_id, phase_id, kind, desc, script, pre, post, instruction, expiry_json) = pr?;

                // Get dependency phase IDs (string IDs, not integer PKs).
                let mut deps = Vec::new();
                let mut dep_stmt = db.prepare(
                    "SELECT wp.phase_id FROM workflow_phase_dependencies wpd
                     JOIN workflow_phases wp ON wpd.depends_on_phase_id = wp.id
                     WHERE wpd.phase_id = ?1"
                )?;
                let dep_rows = dep_stmt.query_map(rusqlite::params![phase_db_id], |row| {
                    row.get::<_, String>(0)
                })?;
                for dr in dep_rows {
                    deps.push(dr?);
                }

                let mut phase_obj = serde_json::json!({
                    "id": phase_id,
                    "kind": kind,
                    "depends_on": deps,
                });
                if let Some(d) = desc { phase_obj["description"] = serde_json::json!(d); }
                if let Some(s) = script { phase_obj["script"] = serde_json::json!(s); }
                if let Some(p) = pre { phase_obj["pre_script"] = serde_json::json!(p); }
                if let Some(p) = post { phase_obj["post_script"] = serde_json::json!(p); }
                if let Some(i) = instruction { phase_obj["instruction"] = serde_json::json!(i); }
                if let Some(e) = expiry_json {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&e) {
                        phase_obj["expiry"] = val;
                    }
                }

                phases_json.push(phase_obj);
            }

            use_cases_json.push(serde_json::json!({
                "id": use_case_id,
                "label": label,
                "phases": phases_json,
            }));
        }

        Ok(serde_json::json!({
            "system_name": system_name,
            "plan": {
                "system": system_name,
                "use_cases": use_cases_json,
            },
        }))
    }

    /// Store a plan-generation semantic input (§8.3). Upserts by
    /// standard+repo+workflow+domain+instance using COALESCE virtual
    /// columns to handle NULL keys correctly (§3.0).
    fn handle_store_plan_generation_input(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let system_name = parse_string(req, "system_name")?;
        let workflow_id = parse_string(req, "workflow_id")?;
        let input_json = parse_string(req, "input_json")?;
        let domain_key: Option<String> = req.params.get("domain_key").and_then(|v| v.as_str()).map(String::from);
        let instance_key: Option<String> = req.params.get("instance_key").and_then(|v| v.as_str()).map(String::from);

        let runtime = self.runtime_for(req)?;
        let repo_fingerprint = common::env::repo_fingerprint(&runtime.context.repository_root);
        let standard_id = resolve_standard_id(&runtime.registry.conn, Some(&system_name))?;
        let db = &runtime.registry.conn;

        db.execute(
            "INSERT INTO plan_generation_inputs
                (standard_id, repo_fingerprint, workflow_id, domain_key,
                 instance_key, input_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
             ON CONFLICT(standard_id, repo_fingerprint, workflow_id,
                         domain_key_key, instance_key_key)
             DO UPDATE SET previous_input_json = plan_generation_inputs.input_json,
                           input_json = excluded.input_json,
                           created_at = excluded.created_at",
            rusqlite::params![standard_id, repo_fingerprint, workflow_id, domain_key, instance_key, input_json],
        )?;

        Ok(serde_json::json!({
            "stored": true,
            "system_name": system_name,
            "workflow_id": workflow_id,
        }))
    }

    /// Retrieve a stored plan-generation semantic input (§8.3).
    fn handle_get_plan_generation_input(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let system_name = parse_string(req, "system_name")?;
        let workflow_id = parse_string(req, "workflow_id")?;
        let domain_key: Option<String> = req.params.get("domain_key").and_then(|v| v.as_str()).map(String::from);
        let instance_key: Option<String> = req.params.get("instance_key").and_then(|v| v.as_str()).map(String::from);

        let runtime = self.runtime_for(req)?;
        let repo_fingerprint = common::env::repo_fingerprint(&runtime.context.repository_root);
        let standard_id = match resolve_standard_id(&runtime.registry.conn, Some(&system_name)) {
            Ok(sid) => sid,
            Err(_) => return Ok(serde_json::json!({ "input": null })),
        };

        let db = &runtime.registry.conn;
        let result: Option<(String, Option<String>)> = db.query_row(
            "SELECT input_json, previous_input_json FROM plan_generation_inputs
             WHERE standard_id = ?1 AND repo_fingerprint = ?2
               AND workflow_id = ?3
               AND domain_key_key = COALESCE(?4, '')
               AND instance_key_key = COALESCE(?5, '')",
            rusqlite::params![standard_id, repo_fingerprint, workflow_id, domain_key, instance_key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).ok();

        match result {
            Some((input, previous)) => Ok(serde_json::json!({
                "input_json": input,
                "previous_input_json": previous,
            })),
            None => Ok(serde_json::json!({ "input": null })),
        }
    }
}

#[cfg(test)]
mod repository_matrix_tests {
    use super::*;
    use crate::protocol::McpRequest;
    use common::config::SamgrahaConfig;
    use services::registry_client::FileRegistryClient;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempDir(std::path::PathBuf);
    impl TempDir {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!("samgraha-mcp-matrix-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self(root)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn adapter_with_kind(root: &Path, kind: RepositoryKind) -> McpAdapter {
        let mut config = SamgrahaConfig::default();
        config.repository.kind = kind;
        let runtime = Arc::new(KnowledgeRuntime::new(root, config).unwrap());
        let registry: Arc<dyn services::registry_client::RegistryClient> =
            Arc::new(FileRegistryClient::new(root));
        McpAdapter::new(runtime, registry)
    }

    fn request(method: &str) -> McpRequest {
        McpRequest {
            id: "1".to_string(),
            method: method.to_string(),
            params: Default::default(),
            repo: None,
        }
    }

    /// Gap 16 (via Gap 13's fix): every method in
    /// `KNOWLEDGE_REPO_BLOCKED_METHODS` must actually be rejected for a
    /// Knowledge Repository, through the real `handle_message` entry point —
    /// not just `search`, which is all the pre-fix code covered.
    #[test]
    fn knowledge_repository_blocks_every_matrix_restricted_method() {
        let tmp = TempDir::new();
        let adapter = adapter_with_kind(tmp.path(), RepositoryKind::Knowledge);

        for method in McpAdapter::KNOWLEDGE_REPO_BLOCKED_METHODS {
            let msg = adapter.handle_message(McpMessage::Request(request(method)));
            match msg {
                McpMessage::Error(e) => {
                    assert!(
                        e.message.contains("not available for Knowledge Repositories"),
                        "method '{}' errored for the wrong reason: {}", method, e.message
                    );
                }
                other => panic!("expected '{}' to be blocked for a Knowledge Repository, got {:?}", method, other),
            }
        }
    }

    /// Same methods must NOT be blocked for a plain Repository — the gate is
    /// kind-specific, not a blanket removal of these tools.
    #[test]
    fn plain_repository_does_not_block_matrix_restricted_methods() {
        let tmp = TempDir::new();
        let adapter = adapter_with_kind(tmp.path(), RepositoryKind::Repository);

        for method in McpAdapter::KNOWLEDGE_REPO_BLOCKED_METHODS {
            let msg = adapter.handle_message(McpMessage::Request(request(method)));
            if let McpMessage::Error(e) = msg {
                assert!(
                    !e.message.contains("not available for Knowledge Repositories"),
                    "method '{}' was wrongly blocked for a plain Repository: {}", method, e.message
                );
            }
        }
    }

    // ── Integration tests: init / sync / staleness via handle_message ────

    struct TempEnvGuard {
        key: String,
        old_val: Option<String>,
    }
    impl TempEnvGuard {
        fn new(key: &str, val: &std::path::Path) -> Self {
            let old_val = std::env::var(key).ok();
            std::env::set_var(key, val);
            Self { key: key.to_string(), old_val }
        }
    }
    impl Drop for TempEnvGuard {
        fn drop(&mut self) {
            match &self.old_val {
                Some(v) => std::env::set_var(&self.key, v),
                None => std::env::remove_var(&self.key),
            }
        }
    }

    fn setup_mock_global_store(dir: &Path, system_name: &str, version: &str) {
        let conn = rusqlite::Connection::open(dir.join("standards.db")).unwrap();
        conn.execute_batch(&format!(
            "PRAGMA user_version = 3;
             CREATE TABLE IF NOT EXISTS systems (
                 id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                 description TEXT, is_default INTEGER NOT NULL DEFAULT 1
             );
             CREATE TABLE IF NOT EXISTS standards (
                 id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                 name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                 generation_granularity TEXT NOT NULL DEFAULT 'section',
                 UNIQUE(system_id, name, version)
             );
             DELETE FROM systems;
             DELETE FROM standards;
             INSERT INTO systems (id, name, is_default) VALUES (1, '{system_name}', 1);
             INSERT INTO standards (id, system_id, name, version)
                 VALUES (1, 1, '{system_name}-std', '{version}');",
        ))
        .unwrap();
    }

    fn req_with_params(method: &str, params: serde_json::Value) -> McpRequest {
        McpRequest {
            id: "test-1".to_string(),
            method: method.to_string(),
            params: params.as_object()
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            repo: None,
        }
    }

    fn extract_json(msg: McpMessage) -> serde_json::Value {
        match msg {
            McpMessage::Response(r) => r.result,
            McpMessage::Error(e) => panic!("MCP error: {}", e.message),
            other => panic!("unexpected message type: {:?}", other),
        }
    }

    #[test]
    fn mcp_init_sync_staleness_lifecycle() {
        let tmp = TempDir::new();
        let adapter = adapter_with_kind(tmp.path(), RepositoryKind::Repository);

        let global = TempDir::new();
        setup_mock_global_store(global.path(), "my-sys", "1.0.0");
        let _guard = TempEnvGuard::new("SAMGRAHA_MCP_DIR", global.path());

        let root_param = tmp.path().to_str().unwrap().to_string();

        // 1. Check staleness before any sync — NeverSynced
        let result = extract_json(adapter.handle_message(McpMessage::Request(req_with_params(
            "check_knowledge_staleness",
            serde_json::json!({ "repo_path": &root_param }),
        ))));
        assert_eq!(result["status"], "never_synced");

        // 2. Init with sync
        let result = extract_json(adapter.handle_message(McpMessage::Request(req_with_params("init", serde_json::json!({
            "repo_path": &root_param,
            "standard_system": "my-sys",
            "auto_detect": false,
            "sync": true,
        })))));
        assert!(result["status"].as_str().unwrap().contains("Initialized"));
        assert_eq!(result["sync_result"]["standards_synced"], true);

        // 3. Sync again — should skip (up to date)
        let result = extract_json(adapter.handle_message(McpMessage::Request(req_with_params("sync_standards", serde_json::json!({
            "repo_path": &root_param,
            "force": false,
        })))));
        assert_eq!(result["synced"], false);
        assert_eq!(result["reason"], "up_to_date");

        // 4. Check staleness — UpToDate
        let result = extract_json(adapter.handle_message(McpMessage::Request(req_with_params(
            "check_knowledge_staleness",
            serde_json::json!({ "repo_path": &root_param }),
        ))));
        assert_eq!(result["status"], "up_to_date");

        // 5. Upgrade global
        setup_mock_global_store(global.path(), "my-sys", "2.0.0");

        // 6. Check staleness — Stale
        let result = extract_json(adapter.handle_message(McpMessage::Request(req_with_params(
            "check_knowledge_staleness",
            serde_json::json!({ "repo_path": &root_param }),
        ))));
        assert_eq!(result["status"], "stale");

        // 7. Sync with force — should re-sync
        let result = extract_json(adapter.handle_message(McpMessage::Request(req_with_params("sync_standards", serde_json::json!({
            "repo_path": &root_param,
            "force": true,
        })))));
        assert_eq!(result["synced"], true);

        // 8. Check staleness — UpToDate again
        let result = extract_json(adapter.handle_message(McpMessage::Request(req_with_params(
            "check_knowledge_staleness",
            serde_json::json!({ "repo_path": &root_param }),
        ))));
        assert_eq!(result["status"], "up_to_date");
    }
}
