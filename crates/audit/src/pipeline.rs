use schemas::audit::{
    AuditFinding, PipelineKind, PipelineReport, Severity,
};
use std::collections::HashMap;

pub trait Pipeline {
    fn name(&self) -> PipelineKind;
    fn run(&self, ctx: &PipelineContext) -> PipelineReport;
}

pub struct PipelineContext {
    pub project_root: std::path::PathBuf,
    pub config: common::config::SamgrahaConfig,
    pub inspect_artifact: bool,
    pub runtime_mode: bool,
}

impl PipelineContext {
    pub fn new(
        project_root: std::path::PathBuf,
        config: common::config::SamgrahaConfig,
    ) -> Self {
        Self {
            project_root,
            config,
            inspect_artifact: false,
            runtime_mode: false,
        }
    }

    pub fn with_inspect_artifact(mut self, val: bool) -> Self {
        self.inspect_artifact = val;
        self
    }

    pub fn with_runtime(mut self, val: bool) -> Self {
        self.runtime_mode = val;
        self
    }
}

pub struct PipelineStage {
    pub name: String,
    pub check_ids: Vec<String>,
}

pub(crate) fn make_report(
    pipeline: PipelineKind,
    score: f64,
    categories: HashMap<String, f64>,
    findings: Vec<AuditFinding>,
) -> PipelineReport {
    PipelineReport {
        pipeline,
        score,
        categories,
        findings,
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: HashMap::new(),
    }
}

pub(crate) fn finding(
    check_id: &str,
    severity: Severity,
    message: String,
    location: Option<String>,
) -> AuditFinding {
    AuditFinding {
        check_id: check_id.to_string(),
        severity,
        message,
        location,
        document_id: None,
        provider: "pipeline".into(),
        stage: None,
        section_id: None,
        confidence: None,
        evidence: None,
        status: None,
        strategy: None,
    }
}


