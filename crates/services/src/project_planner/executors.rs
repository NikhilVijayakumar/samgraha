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
        Ok(serde_json::json!({ "pipelines": results }))
    }
}

// ── FixPhaseExecutor ──────────────────────────────────────────────────────

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

        // Planners now always set explicit pipeline_ids on Verify phases;
        // this is a defensive fallback only, sharing the same canonical
        // ordering as the planners instead of a second hand-typed list.
        let pipelines: Vec<String> = if phase.pipeline_ids.is_empty() {
            super::planners::all_pipelines().into_iter().map(String::from).collect()
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
                    let passed = report.score >= Self::PASS_THRESHOLD;
                    if !passed {
                        all_passed = false;
                    }
                    results.push(serde_json::json!({
                        "pipeline": pipeline_str,
                        "score": report.score,
                        "passed": passed,
                        "threshold": Self::PASS_THRESHOLD,
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
            "threshold": Self::PASS_THRESHOLD,
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
