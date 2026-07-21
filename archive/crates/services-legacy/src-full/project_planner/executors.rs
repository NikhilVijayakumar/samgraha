use crate::KnowledgeRuntime;
use anyhow::{Context, Result};
use registry::RegistryStore;
use schemas::{
    AuditFinding, CompilationScope, PhaseType, PipelineKind, ProjectPhase,
};
use std::sync::Arc;

/// Trait for executing a single phase.
/// Takes `Arc<KnowledgeRuntime>` so fix executors can clone the Arc for
/// `apply_finding_fix` (which requires `&Arc<Self>`).
pub trait PhaseExecutor: Send + Sync {
    fn phase_type(&self) -> PhaseType;
    fn execute(
        &self,
        phase: &ProjectPhase,
        runtime: &Arc<KnowledgeRuntime>,
        registry: &RegistryStore,
    ) -> Result<serde_json::Value>;
}

/// Resolve a finding's target path from document_id or location field.
pub fn resolve_finding_path(
    finding: &AuditFinding,
    registry: &RegistryStore,
) -> Result<std::path::PathBuf> {
    if let Some(doc_id) = finding.document_id {
        if let Ok(Some(doc)) = registry.get_document(doc_id) {
            return Ok(std::path::PathBuf::from(doc.path.as_str()));
        }
    }
    if let Some(ref loc) = finding.location {
        return Ok(std::path::PathBuf::from(loc));
    }
    anyhow::bail!(
        "Cannot resolve target path for finding {}: no document_id or location",
        finding.check_id
    );
}

// ── GeneratePhaseExecutor ─────────────────────────────────────────────────

pub struct GeneratePhaseExecutor;

impl PhaseExecutor for GeneratePhaseExecutor {
    fn phase_type(&self) -> PhaseType { PhaseType::Generate }

    fn execute(
        &self,
        _phase: &ProjectPhase,
        runtime: &Arc<KnowledgeRuntime>,
        _registry: &RegistryStore,
    ) -> Result<serde_json::Value> {
        let result = runtime.compile(
            &schemas::compilation::CompilationRequest {
                scope: CompilationScope::Repository,
                force: false,
                watch: false,
            },
        )?;
        Ok(serde_json::json!({
            "compilation": {
                "documents_found": result.documents_found,
                "documents_processed": result.documents_processed,
                "success": result.success,
            }
        }))
    }
}

// ── AuditPhaseExecutor ────────────────────────────────────────────────────

pub struct AuditPhaseExecutor;

impl PhaseExecutor for AuditPhaseExecutor {
    fn phase_type(&self) -> PhaseType { PhaseType::Audit }

    fn execute(
        &self,
        phase: &ProjectPhase,
        runtime: &Arc<KnowledgeRuntime>,
        _registry: &RegistryStore,
    ) -> Result<serde_json::Value> {
        let mut results = Vec::new();
        for pipeline_str in &phase.pipeline_ids {
            let kind = PipelineKind::from_str(pipeline_str)
                .ok_or_else(|| anyhow::anyhow!("Unknown pipeline: {}", pipeline_str))?;
            let (report, report_id) = runtime
                .run_pipeline_with_id(&kind, false, true, false, false)
                .with_context(|| format!("Pipeline '{}' failed", pipeline_str))?;
            results.push(serde_json::json!({
                "pipeline": pipeline_str,
                "score": report.score,
                "report_id": report_id,
                "findings_count": report.findings.len(),
            }));
        }

        // StandardWorkflowPlanner-generated phases name domains but leave
        // pipeline_ids empty (these domains aren't PipelineKind variants) —
        // route those through runtime.audit() (StandardRegistry.audit_rules)
        // instead. Gated on pipeline_ids being empty so this never
        // double-runs for the other 4 planners, whose domains/pipeline_ids
        // both list the same built-in pipeline names.
        if phase.pipeline_ids.is_empty() {
            for domain in &phase.domains {
                let report = runtime
                    .audit(Some(domain.as_str()), &["deterministic".to_string()], None)
                    .with_context(|| format!("Domain audit '{}' failed", domain))?;
                results.push(serde_json::json!({
                    "domain": domain,
                    "score": report.score.overall,
                    "findings_count": report.findings.len(),
                }));
            }
        }

        Ok(serde_json::json!({ "pipelines": results }))
    }
}

// ── FixPhaseExecutor ──────────────────────────────────────────────────────
//
// No domain-audit fallback here (unlike AuditPhaseExecutor above) — deliberately
// not built yet, and checked deeper than the first pass at this comment found:
//
// - report_id: i64 turned out NOT to be the real blocker. fix_sessions.report_id
//   has no FK constraint (`crates/registry/src/migration.rs` V28) — it's pure
//   bookkeeping, nothing joins against it expecting a real row. A synthetic id
//   (e.g. a timestamp) would be honest and safe to pass here.
// - The actual blocker: `resolve_finding_path` (above) requires either
//   `finding.document_id` (looked up via `registry.get_document`) or
//   `finding.location` — a domain-driven standard's `file_presence`/`glob_match`
//   findings (python_hackathon's actual shape: "Dockerfile missing") have
//   neither. There's no document these findings are "in"; they're about a
//   file's *absence*, not its content.
// - Even with a path, every existing FixPlanner (`crate::fix::planner::*`)
//   generates edits to existing document content. None of them know how to
//   scaffold a missing file from nothing (a Dockerfile, a docker-compose.yml).
//   That's a materially different capability, not a missing plumbing step —
//   building it means a new FixPlanner variant with real judgment about what
//   a generated Dockerfile should contain, not something to improvise here.
//
// A StandardWorkflowPlanner-generated Fix phase currently just no-ops (empty
// pipeline_ids, so the loop below never iterates).

pub struct FixPhaseExecutor;

impl PhaseExecutor for FixPhaseExecutor {
    fn phase_type(&self) -> PhaseType { PhaseType::Fix }

    fn execute(
        &self,
        phase: &ProjectPhase,
        runtime: &Arc<KnowledgeRuntime>,
        registry: &RegistryStore,
    ) -> Result<serde_json::Value> {
        let mut fix_sessions = Vec::new();

        for pipeline_str in &phase.pipeline_ids {
            let kind = PipelineKind::from_str(pipeline_str)
                .ok_or_else(|| anyhow::anyhow!("Unknown pipeline: {}", pipeline_str))?;

            // Re-run pipeline to get latest findings
            let (report, report_id) = runtime
                .run_pipeline_with_id(&kind, false, true, false, false)
                .with_context(|| format!("Pipeline '{}' failed for fix phase", pipeline_str))?;

            for finding in &report.findings {
                let target_path = match resolve_finding_path(finding, registry) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("Skipping finding {}: {}", finding.check_id, e);
                        continue;
                    }
                };

                match runtime.apply_finding_fix(
                    finding,
                    pipeline_str,
                    report_id,
                    pipeline_str,
                    &target_path,
                ) {
                    Ok(session) => {
                        fix_sessions.push(serde_json::json!({
                            "check_id": finding.check_id,
                            "session_id": session.id,
                            "status": format!("{:?}", session.status),
                        }));
                    }
                    Err(e) => {
                        tracing::warn!("Fix failed for {}: {}", finding.check_id, e);
                        fix_sessions.push(serde_json::json!({
                            "check_id": finding.check_id,
                            "error": e.to_string(),
                        }));
                    }
                }
            }
        }

        Ok(serde_json::json!({ "fix_sessions": fix_sessions }))
    }
}

// ── VerifyPhaseExecutor ───────────────────────────────────────────────────

pub struct VerifyPhaseExecutor;

impl VerifyPhaseExecutor {
    /// Score threshold matching the "Acceptable" rating band.
    pub const PASS_THRESHOLD: f64 = 70.0;

    /// Resolve the score threshold for a given pipeline, checking config
    /// gates first, then falling back to the compile-time default.
    fn threshold_for(pipeline: &str, runtime: &KnowledgeRuntime) -> f64 {
        runtime
            .context
            .config
            .audit
            .gates
            .get(pipeline)
            .and_then(|g| g.min_score)
            .unwrap_or(Self::PASS_THRESHOLD)
    }
}

impl PhaseExecutor for VerifyPhaseExecutor {
    fn phase_type(&self) -> PhaseType { PhaseType::Verify }

    fn execute(
        &self,
        phase: &ProjectPhase,
        runtime: &Arc<KnowledgeRuntime>,
        _registry: &RegistryStore,
    ) -> Result<serde_json::Value> {
        let mut results = Vec::new();
        let mut all_passed = true;

        // When pipeline_ids is empty, fall back to the phase's own domains.
        // StandardWorkflowPlanner sets pipeline_ids to empty and relies on
        // runtime.audit(domain) per phase.domains entry.
        let pipelines: Vec<String> = if phase.pipeline_ids.is_empty() {
            phase.domains.clone()
        } else {
            phase.pipeline_ids.clone()
        };

        for pipeline_str in &pipelines {
            let kind = match PipelineKind::from_str(pipeline_str) {
                Some(k) => k,
                None => continue,
            };

            match runtime.run_pipeline_with_id(&kind, false, true, false, false) {
                Ok((report, _report_id)) => {
                    let threshold = Self::threshold_for(pipeline_str, runtime);
                    let passed = report.score >= threshold;
                    if !passed {
                        all_passed = false;
                    }
                    results.push(serde_json::json!({
                        "pipeline": pipeline_str,
                        "score": report.score,
                        "passed": passed,
                        "threshold": threshold,
                    }));
                }
                Err(e) => {
                    all_passed = false;
                    results.push(serde_json::json!({
                        "pipeline": pipeline_str,
                        "error": e.to_string(),
                        "passed": false,
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "all_passed": all_passed,
            "default_threshold": Self::PASS_THRESHOLD,
            "results": results,
        }))
    }
}

/// Resolve the executor for a given phase type.
pub fn resolve_executor(phase_type: &PhaseType) -> Box<dyn PhaseExecutor> {
    match phase_type {
        PhaseType::Generate => Box::new(GeneratePhaseExecutor),
        PhaseType::Audit => Box::new(AuditPhaseExecutor),
        PhaseType::Fix => Box::new(FixPhaseExecutor),
        PhaseType::Verify => Box::new(VerifyPhaseExecutor),
    }
}
