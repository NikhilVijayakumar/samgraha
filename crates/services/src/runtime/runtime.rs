use crate::audit::AuditService;
use crate::compilation::PipelineFactory;
use crate::package::{PackageFormat, PackageRequest, PackageResult, PackageService};
use crate::registry::{BoxedService, ServiceRegistry};
use crate::resolution::{KnowledgeResolver, ResolutionResult};
use crate::runtime::context::RuntimeContext;
use crate::runtime::fix_store::RegistryFixStore;
use crate::runtime::policy::RuntimePolicy;
use crate::search::SearchService;
use crate::workspace::{WorkspaceBuildResult, WorkspaceService};
use anyhow::{Context, Result};
use audit_crate::fix::orchestrator::{FixOrchestrator, FixStore};
use audit_crate::fix::planner::{BuildPlanner, ConfigPlanner, DocPlanner, FixPlanner, ImplPlanner, SecPlanner, TestPlanner};
use audit_crate::fix::planning_context::PlanningContextBuilder;
use audit_crate::fix::types::{FixPlan, FixSession, PlanType, SessionStatus};
use audit_crate::fix::verifier::Verifier;
use audit_crate::pipeline::PipelineContext;
use uuid::Uuid;
use audit_crate::pipelines::{architecture::ArchitecturePipeline, build::BuildPipeline, consistency::ConsistencyPipeline, coverage::CoveragePipeline, dependency::DependencyPipeline, design::DesignPipeline, deterministic_runtime::DeterministicRuntimePipeline, documentation_structure::DocumentationStructurePipeline, engineering::EngineeringPipeline, external_context::ExternalContextPipeline, external_context_ownership::ExternalContextOwnershipPipeline, feature::FeaturePipeline, feature_design::FeatureDesignPipeline, feature_technical::FeatureTechnicalPipeline, help::HelpPipeline, implementation::ImplementationPipeline, knowledge_system::KnowledgeSystemPipeline, philosophy::PhilosophyPipeline, prototype::PrototypePipeline, readme::ReadmePipeline, security::SecurityPipeline, vision::VisionPipeline};
use audit_crate::AuditFramework;
use common::config::SamgrahaConfig;
use registry::RegistryStore;
use schemas::audit::{AuditFinding, AuditReport, AuditStage, FindingStatus, GateResult, PipelineKind, PipelineReport, SectionChangedResult, SemanticReport, Severity};
use schemas::compilation::{CompilationRequest, CompilationResult};
use schemas::document::Document;
use schemas::manifest::RepositoryManifest;
use schemas::package::{PackageLayout, PackageProfile};
use schemas::search::{SearchQuery, SearchResponse, SectionQuery, SectionQueryResponse};
use serde::Serialize;
use standards::StandardRegistry;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct KnowledgeRuntime {
    pub context: RuntimeContext,
    pub registry: Arc<RegistryStore>,
    pub services: ServiceRegistry,
    pub standard_registry: Arc<StandardRegistry>,
    pub audit_framework: AuditFramework,
    pub policy: RuntimePolicy,
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

        let standard_registry = Arc::new(StandardRegistry::from_standards_db_and_overrides_with_system(
            &root,
            config.repository.documentation.standard_system.as_deref(),
        )?);
        let mut audit_framework = AuditFramework::new(Arc::clone(&standard_registry));
        let config_clone = config.clone();
        let root_clone = root.clone();
        let script_checks_clone = standard_registry.script_checks().to_vec();
        audit_framework.register_provider("deterministic", Arc::new(move |docs, rules, standard| {
            audit_crate::DeterministicAuditProvider::execute(docs, rules, standard, Some(&config_clone), Some(&root_clone), &script_checks_clone)
        }));
        audit_framework.register_provider("semantic", Arc::new(|docs, rules, standard| {
            providers::SemanticAuditProvider::execute(docs, rules, standard)
        }));
        let services = ServiceRegistry::new();
        let policy = RuntimePolicy::default();

        let context = RuntimeContext::new(root, registry_path, config);

        Ok(Self {
            context,
            registry,
            services,
            standard_registry,
            audit_framework,
            policy,
        })
    }

    pub fn register_service(&mut self, service: BoxedService) {
        self.services.register(service);
    }

    pub fn register_audit_provider<F>(&mut self, name: &str, provider: F)
    where
        F: Fn(
                &[Document],
                &[schemas::standard::AuditRuleDef],
                Option<&schemas::standard::StandardDefinition>,
            ) -> Vec<schemas::audit::AuditFinding>
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
        PipelineFactory::compile(
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
        execute: bool,
        dry_run: bool,
    ) -> Result<PipelineReport> {
        let ctx = PipelineContext::new(
            self.context.repository_root.clone(),
            self.context.config.clone(),
        )
        .with_inspect_artifact(inspect_artifact)
        .with_runtime(runtime_mode)
        .with_execute(execute)
        .with_dry_run(dry_run)
        .with_repository_metadata(self.registry.get_repository_metadata().unwrap_or_default());

        let report = match kind {
            PipelineKind::Build => AuditService::run_pipeline(&BuildPipeline, &ctx),
            PipelineKind::Security => AuditService::run_pipeline(&SecurityPipeline, &ctx),
            PipelineKind::Consistency => AuditService::run_pipeline(&ConsistencyPipeline, &ctx),
            PipelineKind::Coverage => AuditService::run_pipeline(&CoveragePipeline, &ctx),
            PipelineKind::Architecture => AuditService::run_pipeline(&ArchitecturePipeline, &ctx),
            PipelineKind::Vision => AuditService::run_pipeline(&VisionPipeline, &ctx),
            PipelineKind::Design => AuditService::run_pipeline(&DesignPipeline, &ctx),
            PipelineKind::Readme => AuditService::run_pipeline(&ReadmePipeline, &ctx),
            PipelineKind::Prototype => AuditService::run_pipeline(&PrototypePipeline, &ctx),
            PipelineKind::ExternalContext => AuditService::run_pipeline(&ExternalContextPipeline, &ctx),
            PipelineKind::Engineering => AuditService::run_pipeline(&EngineeringPipeline, &ctx),
            PipelineKind::Feature => AuditService::run_pipeline(&FeaturePipeline, &ctx),
            PipelineKind::FeatureTechnical => AuditService::run_pipeline(&FeatureTechnicalPipeline, &ctx),
            PipelineKind::FeatureDesign => AuditService::run_pipeline(&FeatureDesignPipeline, &ctx),
            PipelineKind::DeterministicRuntime => AuditService::run_pipeline(&DeterministicRuntimePipeline, &ctx),
            PipelineKind::ExternalContextOwnership => AuditService::run_pipeline(&ExternalContextOwnershipPipeline, &ctx),
            PipelineKind::Implementation => AuditService::run_pipeline(&ImplementationPipeline, &ctx),
            PipelineKind::DocumentationStructure => AuditService::run_pipeline(&DocumentationStructurePipeline, &ctx),
            PipelineKind::Dependency => AuditService::run_pipeline(&DependencyPipeline, &ctx),
            PipelineKind::Help => AuditService::run_pipeline(&HelpPipeline, &ctx),
            PipelineKind::KnowledgeSystem => AuditService::run_pipeline(&KnowledgeSystemPipeline, &ctx),
            PipelineKind::Philosophy => AuditService::run_pipeline(&PhilosophyPipeline, &ctx),
            PipelineKind::Doc => {
                anyhow::bail!("Use the standard audit() method for Documentation Audit");
            }
        };

        // Auto-store pipeline results to SQLite (Phase 8: per-audit storage)
        let session_id = Uuid::new_v4().to_string();
        if let Err(e) = self.store_pipeline_to_db(kind, &report, &session_id) {
            tracing::warn!("Failed to store pipeline report: {}", e);
        }

        Ok(report)
    }

    /// Run a pipeline and return both the report and its persisted report_id.
    /// Unlike `run_pipeline()`, this preserves the report_id needed by
    /// `apply_finding_fix()` / `generate_fix_plan()` in downstream phases.
    pub fn run_pipeline_with_id(
        &self,
        kind: &PipelineKind,
        inspect_artifact: bool,
        runtime_mode: bool,
        execute: bool,
        dry_run: bool,
    ) -> Result<(PipelineReport, i64)> {
        let ctx = PipelineContext::new(
            self.context.repository_root.clone(),
            self.context.config.clone(),
        )
        .with_inspect_artifact(inspect_artifact)
        .with_runtime(runtime_mode)
        .with_execute(execute)
        .with_dry_run(dry_run)
        .with_repository_metadata(self.registry.get_repository_metadata().unwrap_or_default());

        let report = match kind {
            PipelineKind::Build => AuditService::run_pipeline(&BuildPipeline, &ctx),
            PipelineKind::Security => AuditService::run_pipeline(&SecurityPipeline, &ctx),
            PipelineKind::Consistency => AuditService::run_pipeline(&ConsistencyPipeline, &ctx),
            PipelineKind::Coverage => AuditService::run_pipeline(&CoveragePipeline, &ctx),
            PipelineKind::Architecture => AuditService::run_pipeline(&ArchitecturePipeline, &ctx),
            PipelineKind::Vision => AuditService::run_pipeline(&VisionPipeline, &ctx),
            PipelineKind::Design => AuditService::run_pipeline(&DesignPipeline, &ctx),
            PipelineKind::Readme => AuditService::run_pipeline(&ReadmePipeline, &ctx),
            PipelineKind::Prototype => AuditService::run_pipeline(&PrototypePipeline, &ctx),
            PipelineKind::ExternalContext => AuditService::run_pipeline(&ExternalContextPipeline, &ctx),
            PipelineKind::Engineering => AuditService::run_pipeline(&EngineeringPipeline, &ctx),
            PipelineKind::Feature => AuditService::run_pipeline(&FeaturePipeline, &ctx),
            PipelineKind::FeatureTechnical => AuditService::run_pipeline(&FeatureTechnicalPipeline, &ctx),
            PipelineKind::FeatureDesign => AuditService::run_pipeline(&FeatureDesignPipeline, &ctx),
            PipelineKind::DeterministicRuntime => AuditService::run_pipeline(&DeterministicRuntimePipeline, &ctx),
            PipelineKind::ExternalContextOwnership => AuditService::run_pipeline(&ExternalContextOwnershipPipeline, &ctx),
            PipelineKind::Implementation => AuditService::run_pipeline(&ImplementationPipeline, &ctx),
            PipelineKind::DocumentationStructure => AuditService::run_pipeline(&DocumentationStructurePipeline, &ctx),
            PipelineKind::Dependency => AuditService::run_pipeline(&DependencyPipeline, &ctx),
            PipelineKind::Help => AuditService::run_pipeline(&HelpPipeline, &ctx),
            PipelineKind::KnowledgeSystem => AuditService::run_pipeline(&KnowledgeSystemPipeline, &ctx),
            PipelineKind::Philosophy => AuditService::run_pipeline(&PhilosophyPipeline, &ctx),
            PipelineKind::Doc => {
                anyhow::bail!("Use the standard audit() method for Documentation Audit");
            }
        };

        let session_id = Uuid::new_v4().to_string();
        let report_id = self.store_pipeline_to_db(kind, &report, &session_id)?;

        Ok((report, report_id))
    }

    /// Resolves `[report] dir` (env-substituted, falls back to `docs/raw/reports` under the
    /// repo root) — see `common::config::resolve_configured_dir`. Shared by `audit()`'s
    /// scorecard write and the semantic-report store methods' scorecard regeneration.
    fn reports_root(&self) -> PathBuf {
        common::config::resolve_configured_dir(
            &self.context.config.report.dir,
            &self.context.repository_root,
            "docs/raw/reports",
        )
    }

    fn report_templates_dir(&self) -> PathBuf {
        self.context.repository_root.join("docs/raw/report-templates")
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

        let mut result = AuditService::execute(&self.audit_framework, domain, &docs, providers)?;
        result.semantic_review = self.build_semantic_review(&docs, domain);

        self.registry.clear_audit_results()?;
        self.registry.insert_audit_findings(&result.findings)?;

        if let Some(d) = domain {
            if let Err(e) = crate::reporting::write_audit_scorecard(
                &self.reports_root(),
                &self.report_templates_dir(),
                d,
                &result,
                &[],
            ) {
                tracing::warn!("Failed to write audit scorecard for domain '{}': {}", d, e);
            }
        }

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

    /// Bundles per-section LLM review work into every domain audit — the
    /// deterministic providers above only check structural presence, not
    /// content quality. Skips a section silently if its domain has no
    /// audit-standards rubric for that semantic_type (nothing to judge it
    /// against yet).
    fn build_semantic_review(&self, docs: &[Document], domain: Option<&str>) -> schemas::audit::SemanticReviewBundle {
        let mut rubrics = std::collections::HashMap::new();
        let mut tasks = Vec::new();

        for doc in docs.iter().filter(|d| domain.map_or(true, |dom| d.standard == dom)) {
            let sections = self.registry.get_all_sections_for_document(doc.id).unwrap_or_default();
            for sec in sections {
                let key = format!("{}/{}", doc.standard, sec.semantic_type);
                if !rubrics.contains_key(&key) {
                    match self.get_audit_knowledge(&doc.standard, &sec.semantic_type) {
                        Ok(content) => {
                            rubrics.insert(key.clone(), content);
                        }
                        Err(_) => continue, // no rubric for this domain/section_type
                    }
                }
                tasks.push(schemas::audit::SemanticReviewTask {
                    document_id: sec.document_id,
                    section_id: sec.id,
                    document_title: sec.document_title,
                    document_path: sec.document_path,
                    domain: doc.standard.clone(),
                    semantic_type: sec.semantic_type,
                    content: sec.content,
                });
            }
        }

        let instruction = if tasks.is_empty() {
            String::new()
        } else {
            "Deterministic findings above only check structural presence (sections exist, \
             titles present, no obvious tech leakage) — they do not judge content quality. \
             For each task, look up its rubric in rubrics[\"{domain}/{semantic_type}\"] \
             (Engineering Intent / Audit Objectives / Expected Quality / Red Flags / Scoring \
             Criteria / Output Schema), judge the section's content against every row of that \
             rubric's Scoring Criteria table, then call store_section_report with a \
             SemanticReport (stage: \"section\", domain, document_id, section_id, score, \
             findings — one finding per criterion, in the rubric's Output Schema shape). Call \
             get_section_changed(section_id) first to skip sections unchanged since their last \
             stored report."
                .to_string()
        };

        schemas::audit::SemanticReviewBundle {
            instruction,
            rubrics,
            tasks,
        }
    }

    /// Spec-layer counterpart to `build_semantic_review` — one task per
    /// checklist item in `docs/raw/audit/{pipeline}-audit.md` (parsed by
    /// `audit_crate::spec_parser`), evidenced by the pipeline's matching
    /// domain's documents when one exists. See docs/proposal.md §6.1.
    ///
    /// Phase 3 scope: proven for pipelines with a 1:1 domain (e.g.
    /// `architecture`) only — pipelines with no matching domain (build,
    /// security, consistency, coverage, dependency, documentation-structure,
    /// deterministic-runtime, external-context-ownership, implementation)
    /// get an empty `evidence` map until their own evidence collection is
    /// built (docs/proposal.md §8, phase 5).
    pub fn build_pipeline_semantic_review(
        &self,
        pipeline: &PipelineKind,
    ) -> Result<schemas::audit::PipelineSemanticReviewBundle> {
        let pipeline_name = pipeline.as_str();
        let pctx = PlanningContextBuilder::new(self.context.repository_root.clone());
        let raw_spec = pctx.read_audit_spec(pipeline_name)?;
        let checks = audit_crate::spec_parser::parse_audit_spec_checks(&raw_spec);

        let tasks: Vec<schemas::audit::PipelineSemanticReviewTask> = checks
            .iter()
            .map(|c| schemas::audit::PipelineSemanticReviewTask {
                pipeline: pipeline_name.to_string(),
                check_id: c.id.clone(),
                title: c.title.clone(),
                audit_rule: c.audit_rule.clone(),
            })
            .collect();

        let evidence: std::collections::HashMap<String, String> = self
            .get_documents_by_domain(pipeline_name)
            .unwrap_or_default()
            .iter()
            .map(|doc| (doc.path.0.to_string_lossy().to_string(), doc.body.raw().to_string()))
            .collect();

        let instruction = if tasks.is_empty() {
            String::new()
        } else {
            "Deterministic findings above only check the mechanical heuristics implemented in \
             Rust — they do not judge whether this collection of documents coheres as one \
             system. For each task, judge `evidence` (every document in this pipeline's \
             matching domain, path → raw content) against that check's `audit_rule` (when \
             present) and `title`, then call store_pipeline_check_report with a score and \
             findings for that check_id."
                .to_string()
        };

        Ok(schemas::audit::PipelineSemanticReviewBundle {
            instruction,
            evidence,
            tasks,
        })
    }

    /// Local `.samgraha/knowledge.db` is the sole source at query time — no
    /// fallback to any other database. `help`-domain content included: run
    /// `samgraha standards sync` to pull it in from the binary-adjacent
    /// `help.db` first (see `crate::builtin::open_help_store` +
    /// `StandardsAction::Sync`).
    pub fn get_document(&self, id: i64) -> Result<Option<Document>> {
        self.registry.get_document(id)
    }

    pub fn get_document_by_path(&self, path: &str) -> Result<Option<Document>> {
        self.registry.get_document_by_path(path)
    }

    pub fn get_all_documents(&self) -> Result<Vec<Document>> {
        self.registry.get_all_documents()
    }

    // ── Semantic Audit Pass-Throughs ────────────────────────────────────────

    pub fn get_domains(&self) -> Result<Vec<String>> {
        self.registry.get_domains()
    }

    pub fn get_documents_by_domain(&self, domain: &str) -> Result<Vec<Document>> {
        self.registry.get_documents_by_domain(domain)
    }

    pub fn get_section_by_id(&self, section_id: i64) -> Result<Option<schemas::search::SemanticSection>> {
        self.registry.get_section_by_id(section_id)
    }

    pub fn get_audit_knowledge(&self, domain: &str, section_type: &str) -> Result<String> {
        self.standard_registry
            .get_audit_knowledge(domain, section_type)
            .map(|s| s.to_string())
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

    /// Re-renders the domain scorecard against every stored semantic report so far — called
    /// after each `store_*_report` lands, so pending tasks in the scorecard move to done
    /// without needing another `audit()` run. Best-effort: a failure here must not fail the
    /// store call that triggered it.
    fn regenerate_scorecard_for(&self, domain: &str) {
        match self.registry.get_semantic_reports_by_domain(domain) {
            Ok(semantic_results) => {
                if let Err(e) = crate::reporting::regenerate_audit_scorecard(
                    &self.reports_root(),
                    &self.report_templates_dir(),
                    domain,
                    &semantic_results,
                ) {
                    tracing::warn!("Failed to regenerate audit scorecard for domain '{}': {}", domain, e);
                }
            }
            Err(e) => tracing::warn!("Failed to load semantic reports for domain '{}': {}", domain, e),
        }
    }

    pub fn store_section_report(&self, report: &SemanticReport) -> Result<i64> {
        let id = self.registry.store_section_report(report)?;
        self.regenerate_scorecard_for(&report.domain);
        Ok(id)
    }

    pub fn store_document_report(&self, report: &SemanticReport) -> Result<i64> {
        let id = self.registry.store_document_report(report)?;
        self.regenerate_scorecard_for(&report.domain);
        Ok(id)
    }

    pub fn store_cross_domain_report(&self, report: &SemanticReport) -> Result<i64> {
        let id = self.registry.store_cross_domain_report(report)?;
        self.regenerate_scorecard_for(&report.domain);
        Ok(id)
    }

    pub fn store_pipeline_check_report(&self, report: &schemas::audit::PipelineCheckReport) -> Result<i64> {
        self.registry.store_pipeline_check_report(report)
    }

    pub fn get_pipeline_check_report(
        &self,
        pipeline: &str,
        check_id: &str,
    ) -> Result<Option<schemas::audit::PipelineCheckReport>> {
        self.registry.get_pipeline_check_report(pipeline, check_id)
    }

    pub fn check_pipeline_gate(&self, pipeline: &str) -> Result<GateResult> {
        self.registry.check_pipeline_gate(pipeline)
    }

    /// Rolls up whichever of the three audit layers are available for a
    /// target into one score + readiness verdict, persists the rollup, and
    /// returns it. See docs/proposal.md §4/§6.3.
    ///
    /// Deliberately does *not* read the 20 pipelines' own dedicated report
    /// tables (`architecture_reports`, `vision_reports`, ...) for the
    /// deterministic layer — there's no single accessor across their
    /// differently-shaped schemas yet (see docs/proposal.md's Known gaps),
    /// and every layer here is cheap to recompute live, which is also
    /// exactly what §4 "Verify without re-running" already tells callers to
    /// do. So this always re-runs the deterministic layer fresh rather than
    /// fetching a possibly-stale stored one — "build lazily on read", not
    /// "build lazily on read if stale", because there's no cheap way to
    /// tell if it's stale without already having recomputed it.
    pub fn get_summary_report(
        &self,
        target_type: &str,
        target_name: &str,
    ) -> Result<schemas::audit::SummaryReport> {
        let (deterministic_score, standard_score, spec_score) = match target_type {
            "domain" => {
                let det = self.audit(Some(target_name), &["deterministic".to_string()], None)?;
                let standard = self
                    .get_audit_report(target_name, None, None, AuditStage::CrossDomain)?
                    .map(|r| r.score as f64);
                (Some(det.score.overall), standard, None)
            }
            "pipeline" => {
                let kind = PipelineKind::from_str(target_name).ok_or_else(|| {
                    anyhow::anyhow!("Unknown pipeline '{}'", target_name)
                })?;
                if kind == PipelineKind::Doc {
                    anyhow::bail!(
                        "'doc' is not a real pipeline (it's the audit() sentinel for \"no pipeline\") — \
                         use target_type: \"domain\" instead"
                    );
                }
                let report = self.run_pipeline(&kind, false, false, false, false)?;
                let spec = self.registry.get_pipeline_spec_score(target_name)?;
                (Some(report.score), None, spec)
            }
            other => anyhow::bail!("Unknown target_type '{}': expected \"domain\" or \"pipeline\"", other),
        };

        let available: Vec<f64> = [deterministic_score, standard_score, spec_score]
            .into_iter()
            .flatten()
            .collect();
        let overall_score = available.iter().sum::<f64>() / available.len() as f64;

        let readiness = if overall_score >= 90.0 {
            schemas::audit::ReadinessAssessment::Production
        } else if overall_score >= 80.0 {
            schemas::audit::ReadinessAssessment::Implementation
        } else if overall_score >= 70.0 {
            schemas::audit::ReadinessAssessment::Engineering
        } else if overall_score >= 60.0 {
            schemas::audit::ReadinessAssessment::Design
        } else if overall_score >= 50.0 {
            schemas::audit::ReadinessAssessment::Architecture
        } else {
            schemas::audit::ReadinessAssessment::Product
        };

        let report = schemas::audit::SummaryReport {
            target_type: target_type.to_string(),
            target_name: target_name.to_string(),
            deterministic_score,
            standard_score,
            spec_score,
            overall_score,
            readiness,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        if let Err(e) = self.registry.store_summary_report(&report) {
            tracing::warn!("Failed to persist summary report: {}", e);
        }

        Ok(report)
    }

    pub fn update_finding_status(&self, report_id: i64, criterion_id: &str, status: FindingStatus) -> Result<()> {
        self.registry.update_finding_status(report_id, criterion_id, status)
    }

    // ── Pipeline Report Operations (Phase 8 — Per-Audit) ──────────────

    fn store_pipeline_to_db(
        &self,
        kind: &PipelineKind,
        report: &PipelineReport,
        session_id: &str,
    ) -> Result<i64> {
        match kind {
            PipelineKind::Build => self.store_build_report(
                report.score, session_id,
                None, None, None, None, None, None,
                &report.findings,
            ),
            PipelineKind::Security => self.store_security_report(
                report.score, session_id,
                0, 0, 0, 0, 0, None,
                &report.findings,
            ),
            PipelineKind::Consistency => self.store_consistency_report(
                report.score, session_id,
                false, false, None, None, 0,
                &report.findings,
            ),
            PipelineKind::Architecture => {
                let report_id = self.store_architecture_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Collection Integrity").copied(),
                    report.categories.get("Structural Integrity").copied(),
                    report.categories.get("Consistency").copied(),
                    report.categories.get("Cross-Repository Architecture").copied(),
                    None, None,
                    &report.findings,
                )?;
                // Generate recommendations from findings
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "architecture", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Vision => {
                let report_id = self.store_vision_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Vision Content").copied(),
                    report.categories.get("Technology Independence").copied(),
                    report.categories.get("Traceability and Consistency").copied(),
                    report.categories.get("Documentation Quality").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "vision", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Design => {
                let report_id = self.store_design_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Design System").copied(),
                    report.categories.get("Documentation Quality").copied(),
                    report.categories.get("Design Quality").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "design", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Readme => {
                let report_id = self.store_readme_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Repository Introduction").copied(),
                    report.categories.get("Documentation Navigation").copied(),
                    report.categories.get("Documentation Quality").copied(),
                    report.categories.get("Maintainability").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "readme", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Prototype => {
                let report_id = self.store_prototype_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Product Validation").copied(),
                    report.categories.get("Runtime Validation").copied(),
                    report.categories.get("Engineering Validation").copied(),
                    report.categories.get("Validation Quality").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "prototype", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::ExternalContext => {
                let report_id = self.store_external_context_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Document Quality").copied(),
                    report.categories.get("Content Completeness").copied(),
                    report.categories.get("Documentation Integrity").copied(),
                    report.categories.get("Collection Quality").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "external-context", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Engineering => {
                let report_id = self.store_engineering_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Engineering Coverage").copied(),
                    report.categories.get("Documentation Quality").copied(),
                    report.categories.get("Traceability and Consistency").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "engineering", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Feature => {
                let report_id = self.store_feature_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Feature Definition").copied(),
                    report.categories.get("Product Definition").copied(),
                    report.categories.get("Documentation Quality").copied(),
                    report.categories.get("Product Readiness").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "feature", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::FeatureTechnical => {
                let report_id = self.store_feature_technical_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Feature Mapping").copied(),
                    report.categories.get("Technical Realization").copied(),
                    report.categories.get("Documentation Quality").copied(),
                    report.categories.get("Implementation Readiness").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "feature-technical", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::FeatureDesign => {
                let report_id = self.store_feature_design_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Feature Mapping").copied(),
                    report.categories.get("User Experience").copied(),
                    report.categories.get("Documentation Quality").copied(),
                    report.categories.get("Design Readiness").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "feature-design", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::DeterministicRuntime => {
                let report_id = self.store_deterministic_runtime_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Runtime Model").copied(),
                    report.categories.get("Engineering Principles").copied(),
                    report.categories.get("Runtime Integrity").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "deterministic-runtime", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::ExternalContextOwnership => {
                let report_id = self.store_external_context_ownership_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Dependency Coverage").copied(),
                    report.categories.get("Documentation Integration").copied(),
                    report.categories.get("Consistency").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "external-context-ownership", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Implementation => {
                let report_id = self.store_implementation_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Architectural Conformance").copied(),
                    report.categories.get("Feature Conformance").copied(),
                    report.categories.get("Engineering Conformance").copied(),
                    report.categories.get("Documentation Integrity").copied(),
                    report.categories.get("Implementation Quality").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "implementation", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::DocumentationStructure => {
                let report_id = self.store_documentation_structure_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness").map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Structural Integrity").copied(),
                    report.categories.get("Mapping Consistency").copied(),
                    report.categories.get("Atomicity Enforcement").copied(),
                    report.categories.get("Cross-Document Alignment").copied(),
                    report.categories.get("Name Preservation").copied(),
                    report.categories.get("Implementation Traceability").copied(),
                    report.categories.get("Generation Compliance").copied(),
                    None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "documentation-structure", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Coverage => self.store_coverage_report(
                report.score, session_id,
                0, 0, None, None, None,
                &report.findings,
            ),
            PipelineKind::Dependency => {
                // Dependency pipeline stores to build_reports as a fallback
                self.store_build_report(
                    report.score, session_id,
                    None, None, None, None, None, None,
                    &report.findings,
                )
            }
            PipelineKind::Help => {
                let report_id = self.registry.insert_help_report(
                    report.score, session_id,
                    None, None,
                    report.metadata.get("engineering_readiness")
                        .map(|s| s.as_str()).unwrap_or("NOT_READY"),
                    report.categories.get("Coverage").copied(),
                    report.categories.get("Navigation").copied(),
                    report.categories.get("Quality").copied(),
                    report.categories.get("Accuracy").copied(),
                    None, None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "help", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::KnowledgeSystem => {
                // No dedicated report table — reuses build_reports the same
                // way Dependency's fallback above does, since a dedicated
                // schema isn't warranted for one more pipeline's findings.
                let report_id = self.store_build_report(
                    report.score, session_id,
                    None, None, None, None, None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "knowledge-system", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Philosophy => {
                let report_id = self.store_build_report(
                    report.score, session_id,
                    None, None, None, None, None, None,
                    &report.findings,
                )?;
                let recommendations = generate_recommendations(&report.findings);
                if !recommendations.is_empty() {
                    let _ = self.registry.insert_recommendations(
                        "philosophy", report_id, &recommendations,
                    );
                }
                Ok(report_id)
            }
            PipelineKind::Doc => anyhow::bail!("Doc pipeline uses audit() not run_pipeline()"),
        }
    }

    fn get_git_revision(&self) -> Option<String> {
        get_git_revision(&self.context.repository_root)
    }

    pub fn store_build_report(
        &self,
        score: f64,
        session_id: &str,
        contract_name: Option<&str>,
        declared_produces: Option<&str>,
        actual_artifacts: Option<&str>,
        artifact_freshness: Option<&str>,
        execution_success: Option<bool>,
        execution_output: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        let git_revision = self.get_git_revision();
        self.registry.insert_build_report(
            score, session_id, git_revision.as_deref(),
            contract_name, declared_produces, actual_artifacts, artifact_freshness,
            execution_success, execution_output, findings,
        )
    }

    pub fn store_security_report(
        &self,
        score: f64,
        session_id: &str,
        secrets_scanned: i64,
        secrets_found: i64,
        runtime_checks: i64,
        runtime_issues: i64,
        high_risk_findings: i64,
        threat_summary: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        let git_revision = self.get_git_revision();
        self.registry.insert_security_report(
            score, session_id, git_revision.as_deref(),
            secrets_scanned, secrets_found, runtime_checks, runtime_issues,
            high_risk_findings, threat_summary, findings,
        )
    }

    pub fn store_consistency_report(
        &self,
        score: f64,
        session_id: &str,
        vision_exists: bool,
        architecture_exists: bool,
        structure_score: Option<f64>,
        naming_issues: Option<&str>,
        cross_references: i64,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        let git_revision = self.get_git_revision();
        self.registry.insert_consistency_report(
            score, session_id, git_revision.as_deref(),
            vision_exists, architecture_exists, structure_score,
            naming_issues, cross_references, findings,
        )
    }

    pub fn store_coverage_report(
        &self,
        score: f64,
        session_id: &str,
        features_count: i64,
        src_files_count: i64,
        feature_coverage_pct: Option<f64>,
        uncovered_features: Option<&str>,
        doc_types_covered: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        let git_revision = self.get_git_revision();
        self.registry.insert_coverage_report(
            score, session_id, git_revision.as_deref(),
            features_count, src_files_count, feature_coverage_pct,
            uncovered_features, doc_types_covered, findings,
        )
    }

    pub fn query_build_sessions(&self, limit: usize) -> Result<Vec<registry::store::BuildSessionInfo>> {
        self.registry.query_build_sessions(limit)
    }

    pub fn query_security_sessions(&self, limit: usize) -> Result<Vec<registry::store::SecuritySessionInfo>> {
        self.registry.query_security_sessions(limit)
    }

    pub fn query_consistency_sessions(&self, limit: usize) -> Result<Vec<registry::store::ConsistencySessionInfo>> {
        self.registry.query_consistency_sessions(limit)
    }

    pub fn query_coverage_sessions(&self, limit: usize) -> Result<Vec<registry::store::CoverageSessionInfo>> {
        self.registry.query_coverage_sessions(limit)
    }

    pub fn get_build_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::BuildReportWithFindings>> {
        self.registry.get_build_report_with_findings(report_id)
    }

    pub fn get_security_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::SecurityReportWithFindings>> {
        self.registry.get_security_report_with_findings(report_id)
    }

    pub fn get_consistency_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::ConsistencyReportWithFindings>> {
        self.registry.get_consistency_report_with_findings(report_id)
    }

    pub fn get_coverage_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::CoverageReportWithFindings>> {
        self.registry.get_coverage_report_with_findings(report_id)
    }

    // ── Architecture (Phase 9) ──────────────────────────────────────────

    pub fn store_architecture_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        collection_integrity_score: Option<f64>,
        structural_integrity_score: Option<f64>,
        consistency_score: Option<f64>,
        cross_repo_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_architecture_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            collection_integrity_score, structural_integrity_score,
            consistency_score, cross_repo_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_architecture_sessions(&self, limit: usize) -> Result<Vec<registry::store::ArchitectureSessionInfo>> {
        self.registry.query_architecture_sessions(limit)
    }

    pub fn get_architecture_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::ArchitectureReportWithFindings>> {
        self.registry.get_architecture_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_vision_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        vision_content_score: Option<f64>,
        tech_independence_score: Option<f64>,
        traceability_consistency_score: Option<f64>,
        doc_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_vision_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            vision_content_score, tech_independence_score,
            traceability_consistency_score, doc_quality_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_vision_sessions(&self, limit: usize) -> Result<Vec<registry::store::VisionSessionInfo>> {
        self.registry.query_vision_sessions(limit)
    }

    pub fn get_vision_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::VisionReportWithFindings>> {
        self.registry.get_vision_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_design_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        design_system_score: Option<f64>,
        doc_quality_score: Option<f64>,
        design_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_design_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            design_system_score, doc_quality_score, design_quality_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_design_sessions(&self, limit: usize) -> Result<Vec<registry::store::DesignSessionInfo>> {
        self.registry.query_design_sessions(limit)
    }

    pub fn get_design_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::DesignReportWithFindings>> {
        self.registry.get_design_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_readme_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        repo_introduction_score: Option<f64>,
        doc_navigation_score: Option<f64>,
        doc_quality_score: Option<f64>,
        maintainability_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_readme_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            repo_introduction_score, doc_navigation_score, doc_quality_score, maintainability_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_readme_sessions(&self, limit: usize) -> Result<Vec<registry::store::ReadmeSessionInfo>> {
        self.registry.query_readme_sessions(limit)
    }

    pub fn get_readme_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::ReadmeReportWithFindings>> {
        self.registry.get_readme_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_help_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        coverage_score: Option<f64>,
        navigation_score: Option<f64>,
        quality_score: Option<f64>,
        accuracy_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_help_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            coverage_score, navigation_score, quality_score, accuracy_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_help_sessions(&self, limit: usize) -> Result<Vec<registry::store::HelpSessionInfo>> {
        self.registry.query_help_sessions(limit)
    }

    pub fn get_help_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::HelpReportWithFindings>> {
        self.registry.get_help_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_prototype_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        product_validation_score: Option<f64>,
        runtime_validation_score: Option<f64>,
        engineering_validation_score: Option<f64>,
        validation_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_prototype_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            product_validation_score, runtime_validation_score,
            engineering_validation_score, validation_quality_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_prototype_sessions(&self, limit: usize) -> Result<Vec<registry::store::PrototypeSessionInfo>> {
        self.registry.query_prototype_sessions(limit)
    }

    pub fn get_prototype_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::PrototypeReportWithFindings>> {
        self.registry.get_prototype_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_external_context_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        document_quality_score: Option<f64>,
        content_completeness_score: Option<f64>,
        documentation_integrity_score: Option<f64>,
        collection_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_external_context_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            document_quality_score, content_completeness_score,
            documentation_integrity_score, collection_quality_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_external_context_sessions(&self, limit: usize) -> Result<Vec<registry::store::ExternalContextSessionInfo>> {
        self.registry.query_external_context_sessions(limit)
    }

    pub fn get_external_context_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::ExternalContextReportWithFindings>> {
        self.registry.get_external_context_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_engineering_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        engineering_coverage_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        traceability_consistency_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_engineering_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            engineering_coverage_score, documentation_quality_score,
            traceability_consistency_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_engineering_sessions(&self, limit: usize) -> Result<Vec<registry::store::EngineeringSessionInfo>> {
        self.registry.query_engineering_sessions(limit)
    }

    pub fn get_engineering_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::EngineeringReportWithFindings>> {
        self.registry.get_engineering_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_feature_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        feature_definition_score: Option<f64>,
        product_definition_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        product_readiness_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_feature_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            feature_definition_score, product_definition_score,
            documentation_quality_score, product_readiness_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_feature_sessions(&self, limit: usize) -> Result<Vec<registry::store::FeatureSessionInfo>> {
        self.registry.query_feature_sessions(limit)
    }

    pub fn get_feature_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::FeatureReportWithFindings>> {
        self.registry.get_feature_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_feature_technical_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        feature_mapping_score: Option<f64>,
        technical_realization_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        implementation_readiness_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_feature_technical_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            feature_mapping_score, technical_realization_score,
            documentation_quality_score, implementation_readiness_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_feature_technical_sessions(&self, limit: usize) -> Result<Vec<registry::store::FeatureTechnicalSessionInfo>> {
        self.registry.query_feature_technical_sessions(limit)
    }

    pub fn get_feature_technical_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::FeatureTechnicalReportWithFindings>> {
        self.registry.get_feature_technical_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_feature_design_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        feature_mapping_score: Option<f64>,
        user_experience_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        design_readiness_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_feature_design_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            feature_mapping_score, user_experience_score,
            documentation_quality_score, design_readiness_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_feature_design_sessions(&self, limit: usize) -> Result<Vec<registry::store::FeatureDesignSessionInfo>> {
        self.registry.query_feature_design_sessions(limit)
    }

    pub fn get_feature_design_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::FeatureDesignReportWithFindings>> {
        self.registry.get_feature_design_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_deterministic_runtime_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        runtime_model_score: Option<f64>,
        engineering_principles_score: Option<f64>,
        runtime_integrity_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_deterministic_runtime_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            runtime_model_score, engineering_principles_score, runtime_integrity_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_deterministic_runtime_sessions(&self, limit: usize) -> Result<Vec<registry::store::DeterministicRuntimeSessionInfo>> {
        self.registry.query_deterministic_runtime_sessions(limit)
    }

    pub fn get_deterministic_runtime_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::DeterministicRuntimeReportWithFindings>> {
        self.registry.get_deterministic_runtime_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_external_context_ownership_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        dependency_coverage_score: Option<f64>,
        documentation_integration_score: Option<f64>,
        consistency_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_external_context_ownership_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            dependency_coverage_score, documentation_integration_score, consistency_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_external_context_ownership_sessions(&self, limit: usize) -> Result<Vec<registry::store::ExternalContextOwnershipSessionInfo>> {
        self.registry.query_external_context_ownership_sessions(limit)
    }

    pub fn get_external_context_ownership_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::ExternalContextOwnershipReportWithFindings>> {
        self.registry.get_external_context_ownership_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_implementation_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        architectural_conformance_score: Option<f64>,
        feature_conformance_score: Option<f64>,
        engineering_conformance_score: Option<f64>,
        documentation_integrity_score: Option<f64>,
        implementation_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_implementation_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            architectural_conformance_score, feature_conformance_score,
            engineering_conformance_score, documentation_integrity_score,
            implementation_quality_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_implementation_sessions(&self, limit: usize) -> Result<Vec<registry::store::ImplementationSessionInfo>> {
        self.registry.query_implementation_sessions(limit)
    }

    pub fn get_implementation_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::ImplementationReportWithFindings>> {
        self.registry.get_implementation_report_with_findings(report_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_documentation_structure_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        structural_integrity_score: Option<f64>,
        mapping_consistency_score: Option<f64>,
        atomicity_enforcement_score: Option<f64>,
        cross_document_alignment_score: Option<f64>,
        name_preservation_score: Option<f64>,
        implementation_traceability_score: Option<f64>,
        generation_compliance_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.registry.insert_documentation_structure_report(
            score, session_id, git_revision, previous_score, engineering_readiness,
            structural_integrity_score, mapping_consistency_score, atomicity_enforcement_score,
            cross_document_alignment_score, name_preservation_score,
            implementation_traceability_score, generation_compliance_score,
            doc_scores, validation_scores, None, findings,
        )
    }

    pub fn query_documentation_structure_sessions(&self, limit: usize) -> Result<Vec<registry::store::DocumentationStructureSessionInfo>> {
        self.registry.query_documentation_structure_sessions(limit)
    }

    pub fn get_documentation_structure_report_with_findings(&self, report_id: i64) -> Result<Option<registry::store::DocumentationStructureReportWithFindings>> {
        self.registry.get_documentation_structure_report_with_findings(report_id)
    }

    pub fn update_report_finding_status(&self, finding_id: i64, status: &str) -> Result<()> {
        self.registry.update_finding_status_by_id(finding_id, status)
    }

    // ── Per-audit dispatch helpers ────────────────────────────────────────────────
    // Used by CLI and MCP for the `report` subcommand which takes a string type.

    pub fn query_sessions_by_type(&self, audit_type: &str, limit: usize) -> Result<Vec<serde_json::Value>> {
        match audit_type {
            "build" => self.query_build_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "security" => self.query_security_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "consistency" => self.query_consistency_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "coverage" => self.query_coverage_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "architecture" => self.query_architecture_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "vision" => self.query_vision_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "design" => self.query_design_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "readme" => self.query_readme_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "prototype" => self.query_prototype_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "help" => self.query_help_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "documentation-structure" => self.query_documentation_structure_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "external-context" => self.query_external_context_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "engineering" => self.query_engineering_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "feature" => self.query_feature_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "feature-technical" => self.query_feature_technical_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "feature-design" => self.query_feature_design_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "deterministic-runtime" => self.query_deterministic_runtime_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "external-context-ownership" => self.query_external_context_ownership_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            "implementation" => self.query_implementation_sessions(limit)
                .map(|v| v.into_iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            _ => anyhow::bail!("Unknown audit type: {}", audit_type),
        }
    }

    pub fn get_report_with_findings_by_type(
        &self,
        audit_type: &str,
        report_id: i64,
    ) -> Result<Option<serde_json::Value>> {
        match audit_type {
            "build" => self.get_build_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "security" => self.get_security_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "consistency" => self.get_consistency_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "coverage" => self.get_coverage_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "architecture" => self.get_architecture_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "vision" => self.get_vision_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "design" => self.get_design_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "readme" => self.get_readme_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "prototype" => self.get_prototype_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "help" => self.get_help_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "external-context" => self.get_external_context_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "engineering" => self.get_engineering_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "feature" => self.get_feature_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "feature-technical" => self.get_feature_technical_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "feature-design" => self.get_feature_design_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "deterministic-runtime" => self.get_deterministic_runtime_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "external-context-ownership" => self.get_external_context_ownership_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            "implementation" => self.get_implementation_report_with_findings(report_id)
                .map(|v| v.map(|r| serde_json::to_value(r).unwrap())),
            _ => anyhow::bail!("Unknown audit type: {}", audit_type),
        }
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
            builtin_stores: {
                let mut stores = Vec::new();
                // help.db: compiled from product-guide source
                let help_path = common::env::mcp_dir().join("help.db");
                let help_status = if help_path.exists() { "available" } else { "not shipped" };
                stores.push(format!("help ({})", help_status));
                // knowledge.db: empty schema, populated by `standards register`
                let knowledge_path = common::env::mcp_dir().join("knowledge.db");
                let knowledge_status = if knowledge_path.exists() { "available (register a system)" } else { "not shipped" };
                stores.push(format!("knowledge ({})", knowledge_status));
                stores
            },
        }
    }

    // ── Audit-Fix Pipeline ─────────────────────────────────────────────────

    /// Reject domains the fix pipeline has no planner logic for yet.
    /// `DependencyPipeline` (docs/proposal.md Phase 7) now produces real
    /// findings (D1/D4/D5/D7) plus honest Suggestion-severity stubs for
    /// what's still deferred (D2/D3/D6/D8) — but no fix planner branch
    /// exists for any dependency check_id yet, so still refuse outright
    /// rather than silently no-op "succeeding".
    fn reject_stub_domain(domain: &str) -> Result<()> {
        if domain == "dependency" {
            anyhow::bail!(
                "Domain 'dependency' is excluded from the audit-fix pipeline: \
                 no fix planner logic exists for its checks yet"
            );
        }
        Ok(())
    }

    fn build_orchestrator(self: &Arc<Self>) -> Result<FixOrchestrator> {
        let db_path = self.context.registry_path.clone();
        let store: Box<dyn FixStore> = Box::new(RegistryFixStore::new(db_path)?);
        let planners: Vec<Box<dyn FixPlanner>> = vec![
            Box::new(DocPlanner),
            Box::new(ConfigPlanner),
            Box::new(ImplPlanner),
            Box::new(BuildPlanner),
            Box::new(SecPlanner),
            Box::new(TestPlanner),
        ];
        let runtime = Arc::clone(self);
        let check_runner = Box::new(move |domain: &str, check_id: &str| -> Result<f64> {
            runtime.run_single_check(domain, check_id)
        });
        let verifier = Verifier::new(check_runner);
        Ok(FixOrchestrator::new(
            self.context.repository_root.clone(),
            planners,
            verifier,
            store,
        ))
    }

    /// Run the full audit-fix pipeline: plan, execute, verify, retry.
    pub fn apply_finding_fix(
        self: &Arc<Self>,
        finding: &AuditFinding,
        domain: &str,
        report_id: i64,
        report_type: &str,
        target_path: &std::path::PathBuf,
    ) -> Result<FixSession> {
        Self::reject_stub_domain(domain)?;
        let orch = self.build_orchestrator()?;
        orch.execute(finding, domain, report_id, report_type, target_path)
    }

    /// Generate a fix plan without auto-execution.
    /// Returns the plan and its steps for human review.
    pub fn generate_fix_plan(
        &self,
        finding: &AuditFinding,
        domain: &str,
        report_id: i64,
        report_type: &str,
        target_path: &std::path::PathBuf,
    ) -> Result<FixPlan> {
        Self::reject_stub_domain(domain)?;
        let pctx = PlanningContextBuilder::new(self.context.repository_root.clone())
            .build(domain, target_path)
            .context("Failed to build planning context")?;
        // Kept in sync with FixOrchestrator::resolve_plan_type — same rules
        // must apply whether a finding goes through preview or full apply.
        let plan_type = match (domain, finding.check_id.as_str()) {
            ("coverage", "CV6") => PlanType::Test,
            ("implementation", "I8") => PlanType::Test,
            ("build", _) => PlanType::Build,
            ("security", _) => PlanType::Security,
            ("implementation", _) | ("deterministic-runtime", _) => PlanType::Implementation,
            _ => PlanType::Documentation,
        };
        let planners: Vec<Box<dyn FixPlanner>> = vec![
            Box::new(DocPlanner),
            Box::new(ConfigPlanner),
            Box::new(ImplPlanner),
            Box::new(BuildPlanner),
            Box::new(SecPlanner),
            Box::new(TestPlanner),
        ];
        let planner = planners
            .into_iter()
            .find(|p| p.plan_type() == plan_type)
            .context(format!("No planner for {:?}", plan_type))?;
        let session = FixSession {
            id: None,
            report_id,
            report_type: report_type.to_string(),
            criterion_id: finding.check_id.clone(),
            finding_json: serde_json::to_string(finding)?,
            domain: domain.to_string(),
            plan_type: plan_type.clone(),
            target_file: Some(target_path.to_string_lossy().to_string()),
            attempt_count: 0,
            max_attempts: 1,
            status: SessionStatus::InProgress,
            created_at: None,
            updated_at: None,
        };
        let intent = audit_crate::fix::types::Intent::restore_compliance(domain, &finding.check_id);
        let mut plan = planner.plan(&pctx, &intent, &session)?;

        // Persist so `audit_fix_plan_get(plan_id)` can retrieve this preview
        // later — same insert sequence FixOrchestrator::execute uses. No
        // FixSession exists for a preview (nothing has been verified yet),
        // so `session_id` stays empty; audit_fix_plan_get looks up by
        // plan_id directly and doesn't depend on it.
        plan.domain = domain.to_string();
        plan.criterion_id = finding.check_id.clone();
        plan.report_id = report_id;
        let plan_id = self.registry.insert_fix_plan(&plan)?;
        plan.id = Some(plan_id);
        for step in &mut plan.steps {
            step.plan_id = Some(plan_id);
            self.registry.insert_fix_plan_step(step)?;
        }

        Ok(plan)
    }

    /// Run a single check for a given domain and check_id by re-running that
    /// domain's full pipeline and looking up the check's own finding(s).
    /// No per-check pipeline API exists, so this re-runs the whole domain —
    /// coarse, but correct: a missing finding for `check_id` means it passed.
    pub fn run_single_check(&self, domain: &str, check_id: &str) -> Result<f64> {
        let kind = PipelineKind::from_str(domain)
            .ok_or_else(|| anyhow::anyhow!("Unknown audit domain '{}'", domain))?;
        let report = self
            .run_pipeline(&kind, false, true, false, false)
            .with_context(|| format!("Failed to run '{}' pipeline for check {}", domain, check_id))?;
        let worst = report
            .findings
            .iter()
            .filter(|f| f.check_id == check_id)
            .map(|f| severity_score(&f.severity))
            .fold(f64::INFINITY, f64::min);
        Ok(if worst.is_finite() { worst } else { 10.0 })
    }
}

/// Coarse severity → score mapping used by `run_single_check` — a finding
/// present for a check means it failed; severity sets how badly.
fn severity_score(severity: &Severity) -> f64 {
    match severity {
        Severity::Error => 0.0,
        Severity::Warning => 5.0,
        Severity::Suggestion => 7.0,
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

/// Get the current HEAD revision of the repository for traceability.
fn get_git_revision(root: &std::path::Path) -> Option<String> {
    let git_dir = root.join(".git");
    if !git_dir.exists() {
        return None;
    }
    let head_path = git_dir.join("HEAD");
    let head_content = std::fs::read_to_string(head_path).ok()?;
    let head = head_content.trim();
    if let Some(ref_path) = head.strip_prefix("ref: ") {
        // Resolve the ref
        let ref_file = git_dir.join(ref_path);
        std::fs::read_to_string(ref_file).ok().map(|s| s.trim().to_string())
    } else {
        // Detached HEAD — content is the hash directly
        Some(head.to_string())
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

/// Generate architecture recommendations from pipeline findings.
/// Maps finding severity to recommendation priority and category.
pub fn generate_recommendations(findings: &[schemas::audit::AuditFinding]) -> Vec<registry::store::ReportRecommendation> {
    let mut out = Vec::new();
    for f in findings {
        let (priority, category) = match f.severity {
            schemas::audit::Severity::Error => ("P1", "High Impact"),
            schemas::audit::Severity::Warning => ("P2", "Medium Impact"),
            schemas::audit::Severity::Suggestion => ("P3", "Low Impact"),
        };
        out.push(registry::store::ReportRecommendation {
            id: 0,
            priority: priority.to_string(),
            category: category.to_string(),
            description: format!("Address {}: {}", f.check_id, f.message),
            file_path: f.location.clone(),
        });
    }
    out
}
