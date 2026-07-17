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
    KnowledgeSystem,
    Philosophy,
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
            Self::KnowledgeSystem => "knowledge-system",
            Self::Philosophy => "philosophy",
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
            "knowledge-system" => Some(Self::KnowledgeSystem),
            "philosophy" => Some(Self::Philosophy),
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
    /// Spec-layer (docs/raw/audit/*.md checklist) LLM review work, populated
    /// only when the caller requests `providers: ["semantic"]` — mirrors
    /// `AuditReport::semantic_review`. See docs/proposal.md.
    #[serde(default)]
    pub semantic_review: PipelineSemanticReviewBundle,
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

/// A judged Spec-layer check (one A1/V1/BC10/... item), the pipeline
/// counterpart to `SemanticReport`. No `stage`/`document_id`/`section_id` —
/// Spec-layer checks judge a whole collection, not one section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineCheckReport {
    pub report_id: String,
    pub pipeline: String,
    pub check_id: String,
    pub score: i64,
    pub findings: Vec<AuditFinding>,
    pub git_revision: Option<String>,
    pub created_at: String,
}

/// One archived run of a standard-driven (YAML pipeline) audit. `model` is
/// self-reported by the calling agent on the `audit` MCP tool call — samgraha
/// has no other way to learn which LLM is driving the client. `report` is the
/// full `PipelineExecutionResult`/`PipelineReport` JSON, kept as a blob (same
/// convention `semantic_reports.findings` uses) rather than normalized, since
/// nothing queries into it yet beyond `score`/`model`/`standard`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandardAuditRun {
    pub id: i64,
    pub standard: String,
    pub pipeline: String,
    pub model: Option<String>,
    pub score: f64,
    pub report: String,
    pub git_revision: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditScore {
    pub overall: f64,
    pub categories: std::collections::HashMap<String, f64>,
    pub documents_checked: usize,
    pub documents_passed: usize,
    pub findings_count: usize,
    /// Band rating resolved from score_bands (e.g. "Excellent", "Good").
    #[serde(default)]
    pub rating: String,
    /// Per-bucket scores (e.g. "deterministic_whole" → 92.5).
    #[serde(default)]
    pub bucket_scores: std::collections::HashMap<String, f64>,
}

/// A scoring calculation rule — one per bucket defined by a standard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationRule {
    pub bucket: String,
    pub calculation_method: String,
    pub formula: String,
}

/// A weighted input to a calculation rule (e.g. final_score's 25/25/25/25).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationInput {
    pub name: String,
    pub weight: f64,
}

/// A rating band threshold (e.g. Excellent 95-100).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreBand {
    pub rating: String,
    pub min_score: f64,
    pub max_score: f64,
}

/// One of a standard's own scoring-pipeline integrity checks
/// (calculation/validation/scoring_validation.yaml — weight sums, score
/// bounds, domain counts, ...). `rule` is prose describing the check, same
/// relationship `AuditRuleDef.condition` has to its `evidence` — documentation
/// for a human/LLM, not an expression this (or any) Rust code evaluates.
/// No evaluator exists yet; these are loaded and exposed, not enforced.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationRule {
    pub check_key: String,
    pub name: String,
    pub description: Option<String>,
    pub rule: String,
    pub severity: Option<String>,
    pub invalidate_audit: bool,
}

/// Full scoring configuration loaded from calculation_rules + calculation_inputs + score_bands + validation_rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ScoringConfig {
    pub calculation_rules: Vec<CalculationRule>,
    pub calculation_inputs: Vec<CalculationInput>,
    pub score_bands: Vec<ScoreBand>,
    pub validation_rules: Vec<ValidationRule>,
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

/// On-disk scorecard snapshot written by `audit()` and kept in sync by
/// `store_section_report`/`store_document_report`/`store_cross_domain_report` — `report` is
/// frozen at the last deterministic audit run, `semantic_results` is refreshed each time a
/// semantic review lands so the rendered Markdown can show pending vs. done without
/// re-running the audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditScorecard {
    pub report: AuditReport,
    pub semantic_results: Vec<SemanticReport>,
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

/// Pipeline-level (Spec-layer) counterpart to `SemanticReviewBundle` — one
/// task per checklist item (A1, V1, BC10, ...) in a pipeline's
/// `docs/raw/audit/{pipeline}-audit.md`, judged against the whole document
/// collection rather than one section. See docs/proposal.md — "Three-Layer
/// Audit Model".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PipelineSemanticReviewBundle {
    pub instruction: String,
    /// Document path → raw content, for every document in this pipeline's
    /// matching domain. Empty for pipelines with no 1:1 domain (build,
    /// security, consistency, coverage, dependency, documentation-structure,
    /// deterministic-runtime, external-context-ownership, implementation) —
    /// those need their own evidence collection, not yet built (see
    /// docs/proposal.md §8, phase 5).
    pub evidence: std::collections::HashMap<String, String>,
    pub tasks: Vec<PipelineSemanticReviewTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineSemanticReviewTask {
    pub pipeline: String,
    pub check_id: String,
    pub title: String,
    pub audit_rule: Option<String>,
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

/// Rolls up whichever of the three audit layers ran for a target (a domain
/// name or a pipeline name) into one score + readiness verdict. Any of the
/// three `*_score` fields may be `None` — a domain never has `spec_score`
/// (that layer belongs to pipelines only, see docs/proposal.md §3), and a
/// target with only the deterministic layer run still gets a summary with
/// the other two `None`. See docs/proposal.md — "Three-Layer Audit Model".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummaryReport {
    pub target_type: String,
    pub target_name: String,
    pub deterministic_score: Option<f64>,
    pub standard_score: Option<f64>,
    pub spec_score: Option<f64>,
    pub overall_score: f64,
    pub readiness: ReadinessAssessment,
    pub created_at: String,
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
