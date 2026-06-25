use anyhow::Result;
use schemas::audit::{AuditReport, QualityGate};
use schemas::document::Document;
use schemas::standard::StandardDefinition;
use audit::AuditFramework;

pub struct AuditService;

impl AuditService {
    pub fn execute(
        framework: &AuditFramework,
        domain: Option<&str>,
        documents: &[Document],
        standards: &[StandardDefinition],
        providers: &[String],
    ) -> Result<AuditReport> {
        framework.execute(domain, documents, standards, providers)
    }

    pub fn check_gate(report: &AuditReport, gate: &QualityGate) -> Result<bool> {
        AuditFramework::check_quality_gate(report, gate)
    }
}
