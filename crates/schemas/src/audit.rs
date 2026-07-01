use serde::{Deserialize, Serialize};

pub type AuditCheckId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Suggestion,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Suggestion => write!(f, "suggestion"),
        }
    }
}

impl Severity {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => Self::Error,
            "warning" => Self::Warning,
            _ => Self::Suggestion,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditFinding {
    pub check_id: AuditCheckId,
    pub severity: Severity,
    pub message: String,
    pub location: Option<String>,
    pub document_id: Option<i64>,
    pub provider: String,

    // semantic audit extensions
    pub stage: Option<AuditStage>,
    pub section_id: Option<i64>,
    pub confidence: Option<f64>,
    pub evidence: Option<Evidence>,
    pub status: Option<FindingStatus>,
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Evidence {
    pub section_id: i64,
    pub paragraph_index: usize,
    pub sentence: Option<String>,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingStatus {
    Open,
    Fixed,
    Accepted,
    Ignored,
    FalsePositive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditStage {
    Deterministic,
    Section,
    Document,
    CrossDomain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateResult {
    pub blocked: bool,
    pub reason: Option<String>,
    pub blocking_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SectionChangedResult {
    pub changed: bool,
    pub previous_report_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticReport {
    pub report_id: String,
    pub stage: AuditStage,
    pub domain: String,
    pub document_id: Option<i64>,
    pub section_id: Option<i64>,
    pub strategy: Option<String>,
    pub score: i64,
    pub findings: Vec<AuditFinding>,
    pub created_at: String,
    pub document_revision: Option<i64>,
    pub document_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditScore {
    pub overall: f64,
    pub categories: std::collections::HashMap<String, f64>,
    pub documents_checked: usize,
    pub documents_passed: usize,
    pub findings_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditReport {
    pub id: String,
    pub domain: Option<String>,
    pub timestamp: String,
    pub provider: String,
    pub score: AuditScore,
    pub findings: Vec<AuditFinding>,
    pub readiness: ReadinessAssessment,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReadinessAssessment {
    Production,
    Implementation,
    Engineering,
    Design,
    Architecture,
    Product,
    None,
}

impl std::fmt::Display for ReadinessAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Production => write!(f, "production"),
            Self::Implementation => write!(f, "implementation"),
            Self::Engineering => write!(f, "engineering"),
            Self::Design => write!(f, "design"),
            Self::Architecture => write!(f, "architecture"),
            Self::Product => write!(f, "product"),
            Self::None => write!(f, "none"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityGate {
    pub enabled: bool,
    pub min_score: Option<f64>,
    pub min_readiness: Option<ReadinessAssessment>,
    pub required_domains: Vec<String>,
}

impl Default for QualityGate {
    fn default() -> Self {
        Self {
            enabled: false,
            min_score: None,
            min_readiness: None,
            required_domains: Vec::new(),
        }
    }
}
