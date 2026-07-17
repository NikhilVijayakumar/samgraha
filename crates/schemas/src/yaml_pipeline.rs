use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level YAML pipeline definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YamlPipelineDef {
    pub pipeline: PipelineMeta,
    pub workflow: WorkflowDef,
    #[serde(default)]
    pub stages: HashMap<String, StageDef>,
    #[serde(default)]
    pub calculation: CalculationDef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineMeta {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_scope")]
    pub scope: AuditScope,
}

fn default_version() -> String {
    "1.0.0".into()
}

fn default_scope() -> AuditScope {
    AuditScope::Document
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditScope {
    Document,
    Session,
    Both,
}

/// Workflow defines execution order, dependencies, and error handling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowDef {
    pub stages: Vec<WorkflowStageRef>,
    #[serde(default = "default_on_failure")]
    pub on_failure: OnFailure,
    #[serde(default)]
    pub max_retries: u32,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_on_failure() -> OnFailure {
    OnFailure::Continue
}

fn default_timeout() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStageRef {
    pub id: String,
    #[serde(rename = "type")]
    pub stage_type: StageType,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub condition: Option<ConditionDef>,
    #[serde(default = "default_on_failure")]
    pub on_failure: OnFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageType {
    Deterministic,
    Script,
    Semantic,
    Calculation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OnFailure {
    Continue,
    Stop,
    SkipDownstream,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionDef {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Gte,
    Lte,
    Eq,
    Gt,
    Lt,
    Neq,
    Contains,
    NotContains,
}

/// A stage definition — one of deterministic, script, semantic, or calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StageDef {
    Deterministic(DeterministicStageDef),
    Script(ScriptStageDef),
    Semantic(SemanticStageDef),
    Calculation(CalculationStageDef),
}

/// Deterministic stage: rule-based checks with no LLM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeterministicStageDef {
    #[serde(default)]
    pub description: String,
    pub rules: Vec<YamlRuleDef>,
    #[serde(default = "default_scope")]
    pub scope: AuditScope,
}

/// Script stage: external command execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptStageDef {
    #[serde(default)]
    pub description: String,
    pub scripts: Vec<ScriptDef>,
}

/// Semantic stage: LLM-judged checks with rubrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticStageDef {
    #[serde(default)]
    pub description: String,
    pub rules: Vec<SemanticRuleDef>,
    #[serde(default = "default_scope")]
    pub scope: AuditScope,
}

/// Calculation stage: scoring formula (usually implicit, but can be explicit).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationStageDef {
    #[serde(default)]
    pub description: String,
}

// ── Rule Definitions ──────────────────────────────────────────

/// A single deterministic audit rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YamlRuleDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default)]
    pub mandatory: bool,
    #[serde(default = "default_weight")]
    pub weight: f64,
    pub evidence: EvidenceDef,
}

fn default_severity() -> String {
    "warning".into()
}

fn default_weight() -> f64 {
    1.0
}

/// Evidence definition — describes how to check a rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EvidenceDef {
    FilePresence(FilePresenceEvidence),
    FileAbsence(FileAbsenceEvidence),
    ContentCheck(ContentCheckEvidence),
    KeywordAbsence(KeywordAbsenceEvidence),
    WordCount(WordCountEvidence),
    SectionPresence(SectionPresenceEvidence),
    CrossReference(CrossReferenceEvidence),
    RegexMatch(RegexMatchEvidence),
    GlobMatch(GlobMatchEvidence),
    JsonSchema(JsonSchemaEvidence),
    Script(ScriptEvidence),
    LlmJudgment(LlmJudgmentEvidence),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilePresenceEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileAbsenceEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentCheckEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default = "default_true")]
    pub match_all: bool,
    #[serde(default = "default_must_contain")]
    pub mode: String,
}

fn default_true() -> bool {
    true
}

fn default_must_contain() -> String {
    "must_contain".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeywordAbsenceEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub word_boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordCountEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub min: Option<usize>,
    #[serde(default)]
    pub max: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SectionPresenceEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrossReferenceEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub source_paths: Vec<String>,
    pub target_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegexMatchEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobMatchEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub pattern: String,
    #[serde(default)]
    pub min_count: Option<usize>,
    #[serde(default)]
    pub max_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonSchemaEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub paths: Vec<String>,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_expected_exit")]
    pub expected_exit: i32,
    #[serde(default)]
    pub output_parsing: Option<OutputParsingDef>,
}

fn default_expected_exit() -> i32 {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputParsingDef {
    #[serde(rename = "type")]
    pub parsing_type: OutputParsingType,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub threshold: Option<f64>,
    #[serde(default)]
    pub max_lines: Option<usize>,
    #[serde(default)]
    pub max_matches: Option<usize>,
    #[serde(default)]
    pub finding_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputParsingType {
    ExitCode,
    LineCount,
    Regex,
    Json,
}

/// Semantic rule: LLM-judged check with a rubric.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticRuleDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
    pub evidence: LlmJudgmentEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmJudgmentEvidence {
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub rubric: String,
    #[serde(default)]
    pub target_sections: Vec<String>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_max_tokens() -> u32 {
    500
}

// ── Script Definition ─────────────────────────────────────────

/// A script to run in a script stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub script_path: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_expected_exit")]
    pub expected_exit: i32,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(default)]
    pub output_parsing: Option<OutputParsingDef>,
}

// ── Calculation Definition ────────────────────────────────────

/// Scoring calculation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CalculationDef {
    #[serde(default = "default_calc_method")]
    pub method: CalculationMethod,
    #[serde(default)]
    pub inputs: Vec<CalculationInputDef>,
    #[serde(default)]
    pub script: Option<CalculationScriptDef>,
    #[serde(default)]
    pub formula: Option<String>,
    #[serde(default)]
    pub bands: Vec<ScoreBandDef>,
    #[serde(default)]
    pub readiness: Option<ReadinessThresholds>,
}

fn default_calc_method() -> CalculationMethod {
    CalculationMethod::WeightedAverage
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalculationMethod {
    #[default]
    WeightedAverage,
    Script,
    Formula,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationInputDef {
    pub name: String,
    pub weight: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationScriptDef {
    pub command: String,
    #[serde(default = "default_json_format")]
    pub input_format: String,
    #[serde(default = "default_json_format")]
    pub output_format: String,
}

fn default_json_format() -> String {
    "json".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreBandDef {
    pub min: f64,
    pub label: String,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadinessThresholds {
    #[serde(default = "default_readiness_production")]
    pub production: f64,
    #[serde(default = "default_readiness_implementation")]
    pub implementation: f64,
    #[serde(default = "default_readiness_engineering")]
    pub engineering: f64,
    #[serde(default = "default_readiness_design")]
    pub design: f64,
    #[serde(default = "default_readiness_architecture")]
    pub architecture: f64,
}

fn default_readiness_production() -> f64 {
    90.0
}
fn default_readiness_implementation() -> f64 {
    80.0
}
fn default_readiness_engineering() -> f64 {
    70.0
}
fn default_readiness_design() -> f64 {
    60.0
}
fn default_readiness_architecture() -> f64 {
    50.0
}

// ── Stage Execution Result ────────────────────────────────────

/// Result of executing a single stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageResult {
    pub stage_id: String,
    pub score: f64,
    pub passed: usize,
    pub total: usize,
    pub findings: Vec<crate::audit::AuditFinding>,
    pub skipped: bool,
    pub error: Option<String>,
}

impl Default for StageResult {
    fn default() -> Self {
        Self {
            stage_id: String::new(),
            score: 100.0,
            passed: 0,
            total: 0,
            findings: Vec::new(),
            skipped: false,
            error: None,
        }
    }
}

/// Full pipeline execution result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineExecutionResult {
    pub pipeline_name: String,
    pub overall_score: f64,
    pub rating: String,
    pub stage_results: Vec<StageResult>,
    pub findings: Vec<crate::audit::AuditFinding>,
    pub readiness: crate::audit::ReadinessAssessment,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_pipeline() {
        let yaml = r#"
pipeline:
  name: test
workflow:
  stages:
    - id: det
      type: deterministic
stages:
  det:
    rules:
      - id: T-1
        evidence:
          type: file_presence
          paths: ["test.txt"]
calculation:
  method: weighted_average
  inputs:
    - name: det
      weight: 100
      source: det
"#;
        let def: YamlPipelineDef = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(def.pipeline.name, "test");
        assert_eq!(def.workflow.stages.len(), 1);
        assert_eq!(def.workflow.stages[0].id, "det");
    }

    #[test]
    fn deserialize_full_pipeline() {
        let yaml = r#"
pipeline:
  name: infrastructure
  version: "1.0.0"
  description: "Infrastructure checks"
  scope: both
workflow:
  stages:
    - id: deterministic_checks
      type: deterministic
    - id: script_validation
      type: script
      depends_on: [deterministic_checks]
      condition:
        field: "deterministic_checks.score"
        operator: gte
        value: 50
    - id: semantic_review
      type: semantic
      depends_on: [deterministic_checks]
    - id: final_calculation
      type: calculation
      depends_on: [deterministic_checks, script_validation, semantic_review]
  on_failure: continue
  max_retries: 0
  timeout_seconds: 300
stages:
  deterministic_checks:
    description: "File checks"
    scope: document
    rules:
      - id: INFRA-1
        name: "Dockerfile exists"
        severity: error
        mandatory: true
        weight: 10
        evidence:
          type: file_presence
          paths: ["Dockerfile"]
      - id: INFRA-2
        name: "No secrets"
        severity: error
        weight: 8
        evidence:
          type: keyword_absence
          paths: ["*.py"]
          keywords: ["password =", "secret ="]
          word_boundary: true
  script_validation:
    description: "Lint check"
    scripts:
      - id: LINT-1
        name: "Ruff lint"
        command: "ruff check ."
        timeout_seconds: 60
        expected_exit: 0
        severity: error
        weight: 10
  semantic_review:
    description: "LLM review"
    scope: document
    rules:
      - id: SEM-1
        name: "Documentation quality"
        severity: warning
        weight: 5
        evidence:
          type: llm_judgment
          rubric: "Evaluate documentation quality"
          target_sections: ["all"]
          max_tokens: 500
calculation:
  method: weighted_average
  inputs:
    - name: det
      weight: 40
      source: deterministic_checks
    - name: scripts
      weight: 30
      source: script_validation
    - name: sem
      weight: 30
      source: semantic_review
  bands:
    - min: 90
      label: "Excellent"
    - min: 70
      label: "Good"
    - min: 0
      label: "Critical"
  readiness:
    production: 90
    implementation: 80
    engineering: 70
    design: 60
    architecture: 50
"#;
        let def: YamlPipelineDef = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(def.pipeline.scope, AuditScope::Both);
        assert_eq!(def.workflow.stages.len(), 4);
        assert!(def.workflow.stages[1].condition.is_some());
        assert_eq!(def.calculation.bands.len(), 3);
    }
}
