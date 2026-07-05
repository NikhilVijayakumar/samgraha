use crate::audit::AuditService;
use crate::compilation::CompilationService;
use crate::package::{PackageFormat, PackageRequest, PackageResult, PackageService};
use crate::registry::{BoxedService, ServiceRegistry};
use crate::resolution::{KnowledgeResolver, ResolutionResult};
use crate::runtime::context::RuntimeContext;
use crate::runtime::policy::RuntimePolicy;
use crate::search::SearchService;
use crate::workspace::{WorkspaceBuildResult, WorkspaceService};
use anyhow::{Context, Result};
use audit_crate::pipeline::PipelineContext;
use audit_crate::pipelines::{build::BuildPipeline, consistency::ConsistencyPipeline, coverage::CoveragePipeline, security::SecurityPipeline};
use audit_crate::AuditFramework;
use common::config::SamgrahaConfig;
use registry::RegistryStore;
use schemas::audit::{AuditReport, AuditStage, FindingStatus, GateResult, PipelineKind, PipelineReport, SectionChangedResult, SemanticReport};
use schemas::compilation::{CompilationRequest, CompilationResult};
use schemas::document::Document;
use schemas::manifest::RepositoryManifest;
use schemas::package::{PackageLayout, PackageProfile};
use schemas::search::{SearchQuery, SearchResponse, SectionQuery, SectionQueryResponse};
use serde::Serialize;
use standards::StandardRegistry;
use std::path::Path;
use std::sync::Arc;

pub struct KnowledgeRuntime {
    pub context: RuntimeContext,
    pub registry: Arc<RegistryStore>,
    pub services: ServiceRegistry,
    pub standard_registry: Arc<StandardRegistry>,
    pub audit_framework: AuditFramework,
    pub policy: RuntimePolicy,
    /// Read-only built-in knowledge stores (help, standards) shipped next to the binary.
    pub builtin: Vec<(String, Arc<RegistryStore>)>,
}

impl KnowledgeRuntime {
    pub fn new<P: AsRef<Path>>(root: P, config: SamgrahaConfig) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let default_db = root.join(".samgraha").join("knowledge.db");
        let registry_path = config.repository.root.clone().unwrap_or(default_db);

        // Ensure parent directory exists
        if let Some(parent) = registry_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                tracing::warn!("Cannot create registry directory: {}", e)
            });
        }

        let registry = Arc::new(
            RegistryStore::open(&registry_path).context("Failed to open knowledge registry")?,
        );

        let standard_registry = Arc::new(StandardRegistry::with_builtins());
        let audit_framework = AuditFramework::new(Arc::clone(&standard_registry));
        let services = ServiceRegistry::new();
        let policy = RuntimePolicy::default();

        let context = RuntimeContext::new(root, registry_path, config);
        let builtin = crate::builtin::load_builtin_stores();

        Ok(Self {
            context,
            registry,
            services,
            standard_registry,
            audit_framework,
            policy,
            builtin,
        })
    }

    /// All documents from built-in stores (help, standards), flattened.
    fn builtin_documents(&self) -> Vec<Document> {
        self.builtin
            .iter()
            .flat_map(|(_, store)| store.get_all_documents().unwrap_or_default())
            .collect()
    }

    pub fn register_service(&mut self, service: BoxedService) {
        self.services.register(service);
    }

    pub fn register_audit_provider<F>(&mut self, name: &str, provider: F)
    where
        F: Fn(&[Document], &[schemas::standard::AuditRuleDef]) -> Vec<schemas::audit::AuditFinding>
            + Send
            + Sync
            + 'static,
    {
        self.audit_framework
            .register_provider(name, Arc::new(provider));
    }

    pub fn compile(&self, request: &CompilationRequest) -> Result<CompilationResult> {
        if request.scope == schemas::compilation::CompilationScope::Workspace {
            if let Some((ws_root, ws_config)) =
                WorkspaceService::discover(&self.context.repository_root)
            {
                let ws_result = WorkspaceService::compile(&ws_root, &ws_config, request)?;
                return Ok(workspace_result_to_compilation(ws_result));
            }
        }
        CompilationService::execute(
            &self.context.repository_root,
            &self.context.config,
            request,
            &self.standard_registry,
            Arc::clone(&self.registry),
        )
    }

    pub fn compile_workspace(
        &self,
        request: &CompilationRequest,
    ) -> Result<WorkspaceBuildResult> {
        let (ws_root, ws_config) =
            WorkspaceService::discover(&self.context.repository_root)
                .ok_or_else(|| anyhow::anyhow!("No samgraha-workspace.toml found"))?;
        WorkspaceService::compile(&ws_root, &ws_config, request)
    }

    pub fn search_workspace(
        &self,
        query: &schemas::search::SearchQuery,
    ) -> Result<schemas::search::SearchResponse> {
        let (ws_root, ws_config) =
            WorkspaceService::discover(&self.context.repository_root)
                .ok_or_else(|| anyhow::anyhow!("No samgraha-workspace.toml found"))?;
        WorkspaceService::search(&ws_root, &ws_config, query)
    }

    pub fn search(&self, query: &SearchQuery) -> Result<SearchResponse> {
        let docs = self.get_all_documents()?;
        SearchService::search(&docs, query)
    }

    pub fn get_sections(&self, query: &SectionQuery) -> Result<SectionQueryResponse> {
        self.registry.get_sections_by_type(query)
    }

    pub fn run_pipeline(
        &self,
        kind: &PipelineKind,
        inspect_artifact: bool,
        runtime_mode: bool,
    ) -> Result<PipelineReport> {
        let ctx = PipelineContext::new(
            self.context.repository_root.clone(),
            self.context.config.clone(),
        )
        .with_inspect_artifact(inspect_artifact)
        .with_runtime(runtime_mode);

        let report = match kind {
            PipelineKind::Build => AuditService::run_pipeline(&BuildPipeline, &ctx),
            PipelineKind::Security => AuditService::run_pipeline(&SecurityPipeline, &ctx),
            PipelineKind::Consistency => AuditService::run_pipeline(&ConsistencyPipeline, &ctx),
            PipelineKind::Coverage => AuditService::run_pipeline(&CoveragePipeline, &ctx),
            PipelineKind::Dependency => {
                let mut findings = Vec::new();
                let mut cats = std::collections::HashMap::new();
                findings.push(schemas::audit::AuditFinding {
                    check_id: "D0".into(),
                    severity: schemas::audit::Severity::Suggestion,
                    message: "Dependency Governance is specification only — automated checks not yet implemented".into(),
                    location: None,
                    document_id: None,
                    provider: "pipeline".into(),
                    stage: None,
                    section_id: None,
                    confidence: None,
                    evidence: None,
                    status: None,
                    strategy: None,
                });
                cats.insert("Governance".into(), 100.0);
                schemas::audit::PipelineReport {
                    pipeline: PipelineKind::Dependency,
                    score: 100.0,
                    categories: cats,
                    findings,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    metadata: std::collections::HashMap::new(),
                }
            }
            PipelineKind::Doc => {
                anyhow::bail!("Use the standard audit() method for Documentation Audit");
            }
        };
        Ok(report)
    }

    pub fn audit(
        &self,
        domain: Option<&str>,
        providers: &[String],
        documents: Option<&[Document]>,
    ) -> Result<AuditReport> {
        let docs = match documents {
            Some(d) => d.to_vec(),
            None => self.registry.get_all_documents()?,
        };

        let result = AuditService::execute(&self.audit_framework, domain, &docs, providers)?;

        self.registry.clear_audit_results()?;
        self.registry.insert_audit_findings(&result.findings)?;

        // Phase F4: Update manifest audit status after audit run.
        let audit_status = if result
            .findings
            .iter()
            .any(|f| matches!(f.severity, schemas::audit::Severity::Error))
        {
            "FAIL"
        } else {
            "PASS"
        };
        let now = chrono::Utc::now().to_rfc3339(); // keep fully-qualified, avoid unused import

        let manifest_path = self
            .context
            .repository_root
            .join(".samgraha")
            .join("manifest.json");
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(mut manifest) = serde_json::from_str::<RepositoryManifest>(&content)
            {
                manifest.audit.status = audit_status.to_string();
                manifest.audit.last_audit = Some(now);
                if let Ok(json) = serde_json::to_string_pretty(&manifest) {
                    let _ = std::fs::write(&manifest_path, &json);
                }
            }
        }

        Ok(result)
    }

    /// Checks the primary store first; falls back to built-in stores.
    /// Note: ids are only unique per-store, so an id collision across a
    /// primary store and a built-in store would resolve to the primary's
    /// document — acceptable since built-in stores are a distinct, small,
    /// non-user-facing id space.
    pub fn get_document(&self, id: i64) -> Result<Option<Document>> {
        if let Some(doc) = self.registry.get_document(id)? {
            return Ok(Some(doc));
        }
        for (_, store) in &self.builtin {
            if let Some(doc) = store.get_document(id)? {
                return Ok(Some(doc));
            }
        }
        Ok(None)
    }

    pub fn get_document_by_path(&self, path: &str) -> Result<Option<Document>> {
        if let Some(doc) = self.registry.get_document_by_path(path)? {
            return Ok(Some(doc));
        }
        for (_, store) in &self.builtin {
            if let Some(doc) = store.get_document_by_path(path)? {
                return Ok(Some(doc));
            }
        }
        Ok(None)
    }

    pub fn get_all_documents(&self) -> Result<Vec<Document>> {
        let mut docs = self.registry.get_all_documents()?;
        docs.extend(self.builtin_documents());
        Ok(docs)
    }

    // ── Semantic Audit Pass-Throughs ────────────────────────────────────────

    pub fn get_domains(&self) -> Result<Vec<String>> {
        let mut domains = self.registry.get_domains()?;
        domains.extend(self.builtin.iter().map(|(d, _)| d.clone()));
        domains.sort();
        domains.dedup();
        Ok(domains)
    }

    pub fn get_documents_by_domain(&self, domain: &str) -> Result<Vec<Document>> {
        let mut docs = self.registry.get_documents_by_domain(domain)?;
        if let Some((_, store)) = self.builtin.iter().find(|(d, _)| d == domain) {
            docs.extend(store.get_all_documents()?);
        }
        Ok(docs)
    }

    pub fn get_section_by_id(&self, section_id: i64) -> Result<Option<schemas::search::SemanticSection>> {
        self.registry.get_section_by_id(section_id)
    }

    pub fn get_audit_knowledge(&self, domain: &str, section_type: &str) -> Result<String> {
        self.registry.get_audit_knowledge(domain, section_type)
    }

    pub fn get_section_changed(&self, section_id: i64) -> Result<SectionChangedResult> {
        self.registry.get_section_changed(section_id)
    }

    pub fn check_gate(&self, stage: AuditStage, document_id: Option<i64>) -> Result<GateResult> {
        self.registry.check_gate(stage, document_id)
    }

    pub fn get_audit_report(
        &self,
        domain: &str,
        document_id: Option<i64>,
        section_id: Option<i64>,
        stage: AuditStage,
    ) -> Result<Option<SemanticReport>> {
        self.registry.get_audit_report(domain, document_id, section_id, stage)
    }

    pub fn store_section_report(&self, report: &SemanticReport) -> Result<i64> {
        self.registry.store_section_report(report)
    }

    pub fn store_document_report(&self, report: &SemanticReport) -> Result<i64> {
        self.registry.store_document_report(report)
    }

    pub fn store_cross_domain_report(&self, report: &SemanticReport) -> Result<i64> {
        self.registry.store_cross_domain_report(report)
    }

    pub fn update_finding_status(&self, report_id: i64, criterion_id: &str, status: FindingStatus) -> Result<()> {
        self.registry.update_finding_status(report_id, criterion_id, status)
    }

    // ── Typed accessors ──────────────────────────────────────────────────────────

    pub fn documents_by_standard(&self, standard: &str) -> Result<Vec<Document>> {
        Ok(self
            .get_all_documents()?
            .into_iter()
            .filter(|d| d.standard == standard)
            .collect())
    }

    pub fn features(&self) -> Result<Vec<Document>> {
        self.documents_by_standard("feature")
    }

    pub fn architecture_docs(&self) -> Result<Vec<Document>> {
        self.documents_by_standard("architecture")
    }

    pub fn design_docs(&self) -> Result<Vec<Document>> {
        self.documents_by_standard("design")
    }

    pub fn engineering_docs(&self) -> Result<Vec<Document>> {
        self.documents_by_standard("engineering")
    }

    pub fn vision_docs(&self) -> Result<Vec<Document>> {
        self.documents_by_standard("vision")
    }

    pub fn feature_technical_docs(&self) -> Result<Vec<Document>> {
        self.documents_by_standard("feature-technical")
    }

    pub fn package(
        &self,
        output_path: std::path::PathBuf,
        profile: PackageProfile,
        format: PackageFormat,
    ) -> Result<PackageResult> {
        let repo_name = self.context.repository_name();
        let registry_path = self.context.registry_path.clone();
        let request = PackageRequest {
            output_path,
            profile,
            repository_name: repo_name,
            format,
            layout: PackageLayout::Physical,
            primary_root: Some(self.context.repository_root.to_string_lossy().to_string()),
        };
        PackageService::generate(
            Arc::clone(&self.registry),
            Some(&registry_path),
            &request,
            &[],
        )
    }

    pub fn resolve(
        &self,
        profile: PackageProfile,
        output_path: std::path::PathBuf,
        format: PackageFormat,
        layout: PackageLayout,
    ) -> Result<ResolutionResult> {
        KnowledgeResolver::resolve(
            &self.context.repository_root,
            &self.context.config,
            Arc::clone(&self.registry),
            &self.context.registry_path,
            profile,
            output_path,
            format,
            layout,
        )
    }

    pub fn info(&self) -> RuntimeInfo {
        let repo_name = self.context.repository_name();
        let doc_count = self.registry.document_count().unwrap_or(0);
        let declared = &self.context.config.repository.documentation.domain;
        let excluded = &self.context.config.repository.documentation.domain_exclusion;
        RuntimeInfo {
            repository: repo_name,
            registry_path: self.context.registry_path.display().to_string(),
            document_count: doc_count,
            standards: self
                .standard_registry
                .domains()
                .into_iter()
                .map(|s| s.to_string())
                .filter(|d| (declared.is_empty() || declared.contains(d)) && !excluded.contains(d))
                .collect(),
            services: self
                .services
                .all()
                .iter()
                .map(|s| s.name().to_string())
                .collect(),
            policy: self.policy.clone(),
            builtin_stores: crate::builtin::BUILTIN_DOMAINS
                .iter()
                .map(|(domain, _)| {
                    let loaded = self.builtin.iter().any(|(d, _)| d == domain);
                    format!("{} ({})", domain, if loaded { "loaded" } else { "missing" })
                })
                .collect(),
        }
    }
}

fn workspace_result_to_compilation(ws: WorkspaceBuildResult) -> CompilationResult {
    let all_errors: Vec<schemas::compilation::CompilationError> = ws
        .repository_results
        .iter()
        .flat_map(|(_, r)| r.errors.iter().cloned())
        .collect();
    let duration_ms = ws
        .repository_results
        .iter()
        .map(|(_, r)| r.duration_ms)
        .sum();
    CompilationResult {
        success: ws.total_errors == 0,
        documents_found: ws.total_documents,
        documents_processed: ws.total_documents,
        documents_failed: ws.total_errors,
        documents_skipped: 0,
        errors: all_errors,
        warnings: Vec::new(),
        diagnostics: Vec::new(),
        quality: None,
        duration_ms,
        registry_path: None,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeInfo {
    pub repository: String,
    pub registry_path: String,
    pub document_count: usize,
    pub standards: Vec<String>,
    pub services: Vec<String>,
    pub policy: RuntimePolicy,
    /// Built-in knowledge store status, e.g. "help (loaded)", "standards (missing)".
    pub builtin_stores: Vec<String>,
}
