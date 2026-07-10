use serde::{Deserialize, Serialize};

pub type AuditCheckId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PipelineKind {
    Doc,
    Build,
    Security,
    Consistency,
    Coverage,
    Architecture,
    Vision,
    Design,
    Readme,
    Prototype,
    ExternalContext,
    Engineering,
    Feature,
    FeatureTechnical,
    FeatureDesign,
    DeterministicRuntime,
    ExternalContextOwnership,
    Implementation,
    Dependency,
    Help,
    DocumentationStructure,
}

impl PipelineKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Doc => "doc",
            Self::Build => "build",
            Self::Security => "security",
            Self::Consistency => "consistency",
            Self::Coverage => "coverage",
            Self::Architecture => "architecture",
            Self::Vision => "vision",
            Self::Design => "design",
            Self::Readme => "readme",
            Self::Prototype => "prototype",
            Self::ExternalContext => "external-context",
            Self::Engineering => "engineering",
            Self::Feature => "feature",
            Self::FeatureTechnical => "feature-technical",
            Self::FeatureDesign => "feature-design",
            Self::DeterministicRuntime => "deterministic-runtime",
            Self::ExternalContextOwnership => "external-context-ownership",
            Self::Implementation => "implementation",
            Self::Dependency => "dependency",
            Self::Help => "help",
            Self::DocumentationStructure => "documentation-structure",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "doc" => Some(Self::Doc),
            "build" => Some(Self::Build),
            "security" => Some(Self::Security),
            "consistency" => Some(Self::Consistency),
            "coverage" => Some(Self::Coverage),
            "architecture" => Some(Self::Architecture),
            "vision" => Some(Self::Vision),
            "design" => Some(Self::Design),
            "readme" => Some(Self::Readme),
            "prototype" => Some(Self::Prototype),
            "external-context" => Some(Self::ExternalContext),
            "engineering" => Some(Self::Engineering),
            "feature" => Some(Self::Feature),
            "feature-technical" => Some(Self::FeatureTechnical),
            "feature-design" => Some(Self::FeatureDesign),
            "deterministic-runtime" => Some(Self::DeterministicRuntime),
            "external-context-ownership" => Some(Self::ExternalContextOwnership),
            "implementation" => Some(Self::Implementation),
            "dependency" => Some(Self::Dependency),
            "help" => Some(Self::Help),
            "documentation-structure" => Some(Self::DocumentationStructure),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineReport {
    pub pipeline: PipelineKind,
    pub score: f64,
    pub categories: std::collections::HashMap<String, f64>,
    pub findings: Vec<AuditFinding>,
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, String>,
}

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
    /// LLM review work generated by this same audit call — the deterministic
    /// findings above only check structural presence, not content quality.
    /// The calling agent is expected to judge each task against its rubric
    /// and call store_section_report; see `instruction`.
    #[serde(default)]
    pub semantic_review: SemanticReviewBundle,
}

/// Section-level LLM review work bundled into every domain `AuditReport`.
/// `rubrics` is keyed `"{domain}/{semantic_type}"` — the same content
/// `get_audit_knowledge(domain, semantic_type)` would return, inlined here so
/// the calling agent doesn't need a round trip per section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SemanticReviewBundle {
    pub instruction: String,
    pub rubrics: std::collections::HashMap<String, String>,
    pub tasks: Vec<SemanticReviewTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticReviewTask {
    pub document_id: i64,
    pub section_id: i64,
    pub document_title: String,
    pub document_path: String,
    pub domain: String,
    pub semantic_type: String,
    pub content: String,
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
