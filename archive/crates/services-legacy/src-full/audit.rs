use anyhow::Result;
use audit_crate::pipeline::{Pipeline, PipelineContext};
use audit_crate::AuditFramework;
use schemas::audit::{AuditReport, PipelineReport, QualityGate};
use schemas::document::Document;

pub struct AuditService;

impl AuditService {
    pub fn execute(
        framework: &AuditFramework,
        domain: Option<&str>,
        documents: &[Document],
        providers: &[String],
    ) -> Result<AuditReport> {
        framework.execute(domain, documents, providers)
    }

    pub fn check_gate(report: &AuditReport, gate: &QualityGate) -> Result<bool> {
        AuditFramework::check_quality_gate(report, gate)
    }

    pub fn run_pipeline(
        pipeline: &dyn Pipeline,
        ctx: &PipelineContext,
    ) -> PipelineReport {
        pipeline.run(ctx)
    }
}
