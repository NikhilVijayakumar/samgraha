use anyhow::{bail, Result};
use schemas::audit::{AuditFinding, PipelineKind, PipelineReport, ReadinessAssessment, Severity};
use schemas::document::Document;
use schemas::yaml_pipeline::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, info, warn};

// ── Pipeline Loader ───────────────────────────────────────────

/// Load a YAML pipeline definition from a file path.
pub fn load_pipeline_from_file(path: &Path) -> Result<YamlPipelineDef> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;
    load_pipeline_from_str(&content)
}

/// Load a YAML pipeline definition from a string.
pub fn load_pipeline_from_str(yaml: &str) -> Result<YamlPipelineDef> {
    let def: YamlPipelineDef = serde_yaml::from_str(yaml)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML pipeline: {}", e))?;
    validate_pipeline(&def)?;
    Ok(def)
}

/// Validate a pipeline definition for consistency.
fn validate_pipeline(def: &YamlPipelineDef) -> Result<()> {
    let stage_ids: HashSet<&str> = def.workflow.stages.iter().map(|s| s.id.as_str()).collect();

    // All depends_on must reference existing stages
    for stage_ref in &def.workflow.stages {
        for dep in &stage_ref.depends_on {
            if !stage_ids.contains(dep.as_str()) {
                bail!(
                    "Stage '{}' depends on '{}' which does not exist",
                    stage_ref.id,
                    dep
                );
            }
        }
    }

    // Detect cycles via topological sort
    if let Err(cycle) = topological_sort(&def.workflow.stages) {
        bail!("Pipeline has a dependency cycle: {}", cycle);
    }

    // All stage references in workflow must have matching stage definitions
    for stage_ref in &def.workflow.stages {
        if !def.stages.contains_key(&stage_ref.id) {
            warn!(
                "Workflow references stage '{}' but no definition found in stages map",
                stage_ref.id
            );
        }
    }

    Ok(())
}

// ── Workflow Engine ───────────────────────────────────────────

/// Topological sort of workflow stages. Returns error message if cycle detected.
fn topological_sort(stages: &[WorkflowStageRef]) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for stage in stages {
        in_degree.entry(stage.id.as_str()).or_insert(0);
        for dep in &stage.depends_on {
            dependents
                .entry(dep.as_str())
                .or_default()
                .push(stage.id.as_str());
        }
    }

    // Count in-degree: how many deps each stage has
    for stage in stages {
        for _dep in &stage.depends_on {
            *in_degree.entry(stage.id.as_str()).or_insert(0) += 1;
        }
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    let mut sorted: Vec<String> = Vec::new();

    while let Some(current) = queue.pop() {
        sorted.push(current.to_string());
        if let Some(deps) = dependents.get(current) {
            for &dep_id in deps {
                let deg = in_degree.get_mut(dep_id).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dep_id);
                }
            }
        }
    }

    if sorted.len() != stages.len() {
        let remaining: Vec<&str> = stages
            .iter()
            .filter(|s| !sorted.contains(&s.id))
            .map(|s| s.id.as_str())
            .collect();
        return Err(format!("Cycle involving: {:?}", remaining));
    }

    Ok(sorted)
}

/// Evaluate a condition against stage results.
fn evaluate_condition(condition: &ConditionDef, stage_results: &HashMap<String, StageResult>) -> bool {
    // Parse field path: "stage_id.score" or "stage_id.rule_id"
    let parts: Vec<&str> = condition.field.splitn(2, '.').collect();
    if parts.len() != 2 {
        warn!("Invalid condition field: {}", condition.field);
        return false;
    }

    let stage_id = parts[0];
    let field = parts[1];

    let Some(result) = stage_results.get(stage_id) else {
        debug!("Condition references unknown stage: {}", stage_id);
        return false;
    };

    let actual_value = match field {
        "score" => Some(result.score as f64),
        "passed" => Some(result.passed as f64),
        "total" => Some(result.total as f64),
        _ => None,
    };

    let Some(actual) = actual_value else {
        debug!("Condition references unknown field: {}", field);
        return false;
    };

    let expected = match &condition.value {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        _ => {
            warn!("Non-numeric condition value: {:?}", condition.value);
            return false;
        }
    };

    match condition.operator {
        ConditionOperator::Gte => actual >= expected,
        ConditionOperator::Lte => actual <= expected,
        ConditionOperator::Eq => (actual - expected).abs() < f64::EPSILON,
        ConditionOperator::Gt => actual > expected,
        ConditionOperator::Lt => actual < expected,
        ConditionOperator::Neq => (actual - expected).abs() >= f64::EPSILON,
        ConditionOperator::Contains => {
            let haystack = format!("{}", actual);
            let needle = format!("{}", expected);
            haystack.contains(&needle)
        }
        ConditionOperator::NotContains => {
            let haystack = format!("{}", actual);
            let needle = format!("{}", expected);
            !haystack.contains(&needle)
        }
    }
}

// ── Deterministic Evidence Executor ───────────────────────────

/// Execute deterministic evidence checks against documents.
pub fn execute_deterministic_rules(
    rules: &[YamlRuleDef],
    documents: &[Document],
    project_root: &Path,
) -> (Vec<AuditFinding>, usize, usize) {
    let mut findings = Vec::new();
    let total = rules.len();
    let mut passed = 0;

    for rule in rules {
        let rule_findings = execute_single_deterministic_rule(rule, documents, project_root);
        if rule_findings.is_empty() {
            passed += 1;
        }
        findings.extend(rule_findings);
    }

    (findings, passed, total)
}

fn execute_single_deterministic_rule(
    rule: &YamlRuleDef,
    documents: &[Document],
    project_root: &Path,
) -> Vec<AuditFinding> {
    match &rule.evidence {
        EvidenceDef::FilePresence(ev) => {
            check_file_presence(&ev.paths, project_root, rule)
        }
        EvidenceDef::FileAbsence(ev) => {
            check_file_absence(&ev.paths, project_root, rule)
        }
        EvidenceDef::ContentCheck(ev) => {
            check_content(ev, documents, rule)
        }
        EvidenceDef::KeywordAbsence(ev) => {
            check_keyword_absence(ev, documents, rule)
        }
        EvidenceDef::WordCount(ev) => {
            check_word_count(ev, documents, rule)
        }
        EvidenceDef::SectionPresence(ev) => {
            check_section_presence(ev, documents, rule)
        }
        EvidenceDef::CrossReference(ev) => {
            check_cross_reference(ev, documents, rule)
        }
        EvidenceDef::RegexMatch(ev) => {
            check_regex_match(ev, documents, rule)
        }
        EvidenceDef::GlobMatch(ev) => {
            check_glob_match(ev, project_root, rule)
        }
        EvidenceDef::JsonSchema(ev) => {
            check_json_schema(ev, documents, rule)
        }
        EvidenceDef::Script(ev) => {
            check_script_inline(ev, project_root, rule)
        }
        EvidenceDef::LlmJudgment(_) => {
            // Semantic evidence is handled by the semantic executor
            vec![]
        }
    }
}

fn make_finding(rule: &YamlRuleDef, message: String, location: Option<String>) -> AuditFinding {
    AuditFinding {
        check_id: rule.id.clone(),
        severity: Severity::from_str(&rule.severity),
        message,
        location,
        document_id: None,
        provider: "yaml-deterministic".into(),
        stage: None,
        section_id: None,
        confidence: None,
        evidence: None,
        status: None,
        strategy: None,
    }
}

fn check_file_presence(paths: &[String], root: &Path, rule: &YamlRuleDef) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for pattern in paths {
        if pattern.contains('*') {
            // Glob pattern
            let glob_pattern = root.join(pattern);
            if let Some(parent) = glob_pattern.parent() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    let has_match = entries.flatten().any(|e| {
                        let name = e.file_name();
                        let name_str = name.to_string_lossy();
                        glob_match(&name_str, pattern)
                    });
                    if !has_match {
                        findings.push(make_finding(
                            rule,
                            format!("{}: no file matching '{}' found", rule.name, pattern),
                            Some(pattern.to_string()),
                        ));
                    }
                }
            }
        } else {
            let path = root.join(pattern);
            if !path.exists() {
                findings.push(make_finding(
                    rule,
                    format!("{}: '{}' not found", rule.name, pattern),
                    Some(pattern.to_string()),
                ));
            }
        }
    }
    findings
}

fn check_file_absence(paths: &[String], root: &Path, rule: &YamlRuleDef) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for pattern in paths {
        let path = root.join(pattern);
        if path.exists() {
            findings.push(make_finding(
                rule,
                format!("{}: '{}' should not exist", rule.name, pattern),
                Some(pattern.to_string()),
            ));
        }
    }
    findings
}

fn check_content(
    ev: &ContentCheckEvidence,
    documents: &[Document],
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for doc in documents {
        let body_lower = doc.body.raw().to_lowercase();
        let doc_path = doc.path.as_str().to_string();

        match ev.mode.as_str() {
            "must_not_contain" => {
                for kw in &ev.keywords {
                    if contains_word(&body_lower, &kw.to_lowercase()) {
                        findings.push(make_finding(
                            rule,
                            format!("{}: '{}' found in {}", rule.name, kw, doc_path),
                            Some(doc_path.clone()),
                        ));
                    }
                }
            }
            _ => {
                // must_contain
                if ev.match_all {
                    for kw in &ev.keywords {
                        if !contains_word(&body_lower, &kw.to_lowercase()) {
                            findings.push(make_finding(
                                rule,
                                format!("{}: '{}' missing from {}", rule.name, kw, doc_path),
                                Some(doc_path.clone()),
                            ));
                        }
                    }
                } else {
                    if ev.keywords.iter().all(|kw| !contains_word(&body_lower, &kw.to_lowercase())) {
                        findings.push(make_finding(
                            rule,
                            format!("{}: none of {:?} found in {}", rule.name, ev.keywords, doc_path),
                            Some(doc_path),
                        ));
                    }
                }
            }
        }
    }
    findings
}

fn check_keyword_absence(
    ev: &KeywordAbsenceEvidence,
    documents: &[Document],
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for doc in documents {
        let body_lower = doc.body.raw().to_lowercase();
        let doc_path = doc.path.as_str().to_string();

        for kw in &ev.keywords {
            if contains_word(&body_lower, &kw.to_lowercase()) {
                findings.push(make_finding(
                    rule,
                    format!("{}: '{}' found in {}", rule.name, kw, doc_path),
                    Some(doc_path.clone()),
                ));
            }
        }
    }
    findings
}

fn check_word_count(
    ev: &WordCountEvidence,
    documents: &[Document],
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for doc in documents {
        let count = doc.body.raw().split_whitespace().count();
        let doc_path = doc.path.as_str().to_string();

        if let Some(min) = ev.min {
            if count < min {
                findings.push(make_finding(
                    rule,
                    format!("{}: {} words (min {})", rule.name, count, min),
                    Some(doc_path.clone()),
                ));
            }
        }
        if let Some(max) = ev.max {
            if count > max {
                findings.push(make_finding(
                    rule,
                    format!("{}: {} words (max {})", rule.name, count, max),
                    Some(doc_path),
                ));
            }
        }
    }
    findings
}

fn check_section_presence(
    ev: &SectionPresenceEvidence,
    documents: &[Document],
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for section in &ev.sections {
        let section_lower = section.to_lowercase().replace(' ', "_").replace('-', "_");
        let found = documents.iter().any(|doc| {
            let count = doc.quality.per_type.get(&section_lower).copied().unwrap_or(0);
            if count > 0 {
                return true;
            }
            let title_key = doc.title.to_lowercase().replace(' ', "_").replace('-', "_");
            title_key == section_lower
        });
        if !found {
            findings.push(make_finding(
                rule,
                format!("{}: section '{}' not found", rule.name, section),
                None,
            ));
        }
    }
    findings
}

fn check_cross_reference(
    ev: &CrossReferenceEvidence,
    documents: &[Document],
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for doc in documents {
        let body_lower = doc.body.raw().to_lowercase();
        let doc_path = doc.path.as_str().to_string();

        for target in &ev.target_patterns {
            if !contains_word(&body_lower, &target.to_lowercase()) {
                findings.push(make_finding(
                    rule,
                    format!("{}: '{}' not referenced in {}", rule.name, target, doc_path),
                    Some(doc_path.clone()),
                ));
            }
        }
    }
    findings
}

fn check_regex_match(
    ev: &RegexMatchEvidence,
    documents: &[Document],
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let re = match regex::Regex::new(&ev.pattern) {
        Ok(re) => re,
        Err(e) => {
            warn!("Invalid regex '{}': {}", ev.pattern, e);
            return vec![make_finding(
                rule,
                format!("{}: invalid regex pattern '{}'", rule.name, ev.pattern),
                None,
            )];
        }
    };

    for doc in documents {
        let doc_path = doc.path.as_str().to_string();
        if !re.is_match(doc.body.raw()) {
            findings.push(make_finding(
                rule,
                format!("{}: regex '{}' not matched in {}", rule.name, ev.pattern, doc_path),
                Some(doc_path),
            ));
        }
    }
    findings
}

fn check_glob_match(
    ev: &GlobMatchEvidence,
    root: &Path,
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let mut count = 0;

    if let Ok(entries) = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect::<Vec<_>>()
        .into_iter()
        .try_fold::<_, _, Result<usize, ()>>(0usize, |acc, entry| {
            let name = entry.file_name().to_string_lossy();
            if glob_match(&name, &ev.pattern) {
                Ok(acc + 1)
            } else {
                Ok(acc)
            }
        }) {
        count = entries;
    }

    if let Some(min) = ev.min_count {
        if count < min {
            findings.push(make_finding(
                rule,
                format!("{}: {} files matching '{}' (min {})", rule.name, count, ev.pattern, min),
                Some(ev.pattern.clone()),
            ));
        }
    }
    if let Some(max) = ev.max_count {
        if count > max {
            findings.push(make_finding(
                rule,
                format!("{}: {} files matching '{}' (max {})", rule.name, count, ev.pattern, max),
                Some(ev.pattern.clone()),
            ));
        }
    }
    findings
}

fn check_json_schema(
    _ev: &JsonSchemaEvidence,
    documents: &[Document],
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for doc in documents {
        let doc_path = doc.path.as_str().to_string();
        // Only check JSON files
        if !doc_path.ends_with(".json") {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(doc.body.raw()) {
            Ok(_) => {}
            Err(e) => {
                findings.push(make_finding(
                    rule,
                    format!("{}: invalid JSON in {}: {}", rule.name, doc_path, e),
                    Some(doc_path),
                ));
            }
        }
    }
    findings
}

fn check_script_inline(
    ev: &ScriptEvidence,
    root: &Path,
    rule: &YamlRuleDef,
) -> Vec<AuditFinding> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&ev.command)
        .current_dir(root)
        .output();

    match output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            if exit_code != ev.expected_exit {
                let stderr = String::from_utf8_lossy(&output.stderr);
                vec![make_finding(
                    rule,
                    format!(
                        "{}: command '{}' exited with {} (expected {}): {}",
                        rule.name, ev.command, exit_code, ev.expected_exit, stderr
                    ),
                    None,
                )]
            } else {
                vec![]
            }
        }
        Err(e) => vec![make_finding(
            rule,
            format!("{}: failed to run '{}': {}", rule.name, ev.command, e),
            None,
        )],
    }
}

// ── Script Evidence Executor ──────────────────────────────────

/// Execute a script definition and return findings.
pub fn execute_script_def(
    script: &ScriptDef,
    root: &Path,
) -> (Vec<AuditFinding>, bool) {
    let rule = YamlRuleDef {
        id: script.id.clone(),
        name: script.name.clone(),
        severity: script.severity.clone(),
        mandatory: false,
        weight: script.weight,
        evidence: EvidenceDef::Script(ScriptEvidence {
            evidence_type: "script".into(),
            command: script.command.clone(),
            timeout_seconds: script.timeout_seconds,
            expected_exit: script.expected_exit,
            output_parsing: script.output_parsing.clone(),
        }),
    };

    let findings = check_script_inline(
        &ScriptEvidence {
            evidence_type: "script".into(),
            command: script.command.clone(),
            timeout_seconds: script.timeout_seconds,
            expected_exit: script.expected_exit,
            output_parsing: script.output_parsing.clone(),
        },
        root,
        &rule,
    );

    let passed = findings.is_empty();
    (findings, passed)
}

// ── Semantic Evidence Executor ────────────────────────────────

/// Build semantic review tasks from YAML semantic rules.
/// These are returned as findings with provider="yaml-semantic" for the
/// calling agent to judge against the rubric.
pub fn build_semantic_tasks(
    rules: &[SemanticRuleDef],
    _documents: &[Document],
) -> (Vec<AuditFinding>, usize, usize) {
    let mut findings = Vec::new();
    let total = rules.len();
    let mut passed = 0;

    for rule in rules {
        // Create a finding that signals this rule needs LLM review
        let target_desc = if rule.evidence.target_sections.contains(&"all".to_string()) {
            "all sections".to_string()
        } else {
            rule.evidence.target_sections.join(", ")
        };

        findings.push(AuditFinding {
            check_id: rule.id.clone(),
            severity: Severity::from_str(&rule.severity),
            message: format!(
                "{}: LLM review required for {} [rubric: {}]",
                rule.name, target_desc, rule.evidence.rubric
            ),
            location: None,
            document_id: None,
            provider: "yaml-semantic".into(),
            stage: None,
            section_id: None,
            confidence: None,
            evidence: None,
            status: None,
            strategy: Some(rule.evidence.rubric.clone()),
        });
        // Semantic rules are always "passed" from deterministic perspective
        // The actual judgment happens via LLM
        passed += 1;
    }

    (findings, passed, total)
}

// ── Calculation System ────────────────────────────────────────

/// Calculate the final score from stage results using the pipeline's calculation config.
pub fn calculate_score(
    calc: &CalculationDef,
    stage_results: &HashMap<String, StageResult>,
) -> (f64, String) {
    let score = match calc.method {
        CalculationMethod::WeightedAverage => {
            calculate_weighted_average(&calc.inputs, stage_results)
        }
        CalculationMethod::Formula => {
            // For now, fall back to weighted average if formula parsing isn't implemented
            warn!("Formula calculation not yet implemented, falling back to weighted_average");
            calculate_weighted_average(&calc.inputs, stage_results)
        }
        CalculationMethod::Script => {
            // Script-based calculation would need external execution
            warn!("Script calculation not yet implemented, falling back to weighted_average");
            calculate_weighted_average(&calc.inputs, stage_results)
        }
    };

    let rating = resolve_rating(score, &calc.bands);
    (score, rating)
}

fn calculate_weighted_average(
    inputs: &[CalculationInputDef],
    stage_results: &HashMap<String, StageResult>,
) -> f64 {
    if inputs.is_empty() {
        return 100.0;
    }

    let total_weight: f64 = inputs.iter().map(|i| i.weight).sum();
    if total_weight <= 0.0 {
        return 100.0;
    }

    let weighted_sum: f64 = inputs
        .iter()
        .map(|input| {
            let stage_score = stage_results
                .get(&input.source)
                .map(|r| r.score)
                .unwrap_or(100.0);
            (stage_score / 100.0) * input.weight
        })
        .sum();

    weighted_sum
}

fn resolve_rating(score: f64, bands: &[ScoreBandDef]) -> String {
    if bands.is_empty() {
        return "Unknown".to_string();
    }

    // Bands are sorted by min descending (highest first)
    let mut sorted_bands = bands.to_vec();
    sorted_bands.sort_by(|a, b| b.min.partial_cmp(&a.min).unwrap_or(std::cmp::Ordering::Equal));

    for band in &sorted_bands {
        if score >= band.min {
            return band.label.clone();
        }
    }

    sorted_bands.last().map(|b| b.label.clone()).unwrap_or_else(|| "Unknown".into())
}

fn resolve_readiness(score: f64, thresholds: Option<&ReadinessThresholds>) -> ReadinessAssessment {
    let t = thresholds.unwrap_or(&ReadinessThresholds {
        production: 90.0,
        implementation: 80.0,
        engineering: 70.0,
        design: 60.0,
        architecture: 50.0,
    });

    if score >= t.production {
        ReadinessAssessment::Production
    } else if score >= t.implementation {
        ReadinessAssessment::Implementation
    } else if score >= t.engineering {
        ReadinessAssessment::Engineering
    } else if score >= t.design {
        ReadinessAssessment::Design
    } else if score >= t.architecture {
        ReadinessAssessment::Architecture
    } else {
        ReadinessAssessment::Product
    }
}

// ── YamlPipeline (Pipeline trait impl) ───────────────────────

/// A data-driven pipeline that executes YAML-defined rules.
pub struct YamlPipeline {
    pub def: YamlPipelineDef,
}

impl YamlPipeline {
    pub fn new(def: YamlPipelineDef) -> Self {
        Self { def }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let def = load_pipeline_from_file(path)?;
        Ok(Self::new(def))
    }

    pub fn from_str(yaml: &str) -> Result<Self> {
        let def = load_pipeline_from_str(yaml)?;
        Ok(Self::new(def))
    }
}

impl crate::pipeline::Pipeline for YamlPipeline {
    fn name(&self) -> PipelineKind {
        // Map to a custom pipeline kind or use Doc as fallback
        // Since PipelineKind is an enum, we use Doc for YAML pipelines
        // The actual name comes from the YAML definition
        PipelineKind::Doc
    }

    fn run(&self, ctx: &crate::pipeline::PipelineContext) -> PipelineReport {
        let result = execute_yaml_pipeline(&self.def, ctx);
        build_pipeline_report(&self.def, &result)
    }
}

/// Execute a full YAML pipeline.
pub fn execute_yaml_pipeline(
    def: &YamlPipelineDef,
    ctx: &crate::pipeline::PipelineContext,
) -> PipelineExecutionResult {
    info!("Executing YAML pipeline: {}", def.pipeline.name);

    let sorted_stages = match topological_sort(&def.workflow.stages) {
        Ok(s) => s,
        Err(e) => {
            warn!("Pipeline dependency error: {}", e);
            return PipelineExecutionResult {
                pipeline_name: def.pipeline.name.clone(),
                overall_score: 0.0,
                rating: "Error".into(),
                stage_results: vec![],
                findings: vec![AuditFinding {
                    check_id: "pipeline-error".into(),
                    severity: Severity::Error,
                    message: format!("Pipeline dependency error: {}", e),
                    location: None,
                    document_id: None,
                    provider: "yaml-pipeline".into(),
                    stage: None,
                    section_id: None,
                    confidence: None,
                    evidence: None,
                    status: None,
                    strategy: None,
                }],
                readiness: ReadinessAssessment::Product,
            };
        }
    };

    let mut stage_results: HashMap<String, StageResult> = HashMap::new();
    let mut all_findings: Vec<AuditFinding> = Vec::new();
    let mut _pipeline_failed = false;

    for stage_id in &sorted_stages {
        // Find the workflow ref
        let workflow_ref = def.workflow.stages.iter().find(|s| s.id.as_str() == stage_id.as_str());
        let Some(wf_ref) = workflow_ref else {
            continue;
        };

        // Check dependencies are satisfied
        let deps_satisfied = wf_ref.depends_on.iter().all(|dep| {
            stage_results
                .get(dep)
                .map(|r| !r.skipped && r.error.is_none())
                .unwrap_or(false)
        });

        if !deps_satisfied {
            debug!("Skipping stage '{}' due to failed dependencies", stage_id);
            stage_results.insert(
                stage_id.clone(),
                StageResult {
                    stage_id: stage_id.clone(),
                    skipped: true,
                    ..Default::default()
                },
            );
            continue;
        }

        // Check condition
        if let Some(ref condition) = wf_ref.condition {
            if !evaluate_condition(condition, &stage_results) {
                debug!("Skipping stage '{}' due to condition", stage_id);
                stage_results.insert(
                    stage_id.clone(),
                    StageResult {
                        stage_id: stage_id.clone(),
                        skipped: true,
                        ..Default::default()
                    },
                );
                continue;
            }
        }

        // Get the stage definition
        let Some(stage_def) = def.stages.get(stage_id) else {
            warn!("No definition for stage '{}'", stage_id);
            continue;
        };

        // Execute the stage
        let result = match stage_def {
            StageDef::Deterministic(det_def) => {
                execute_deterministic_stage(det_def, &ctx.project_root)
            }
            StageDef::Script(script_def) => {
                execute_script_stage(script_def, &ctx.project_root)
            }
            StageDef::Semantic(sem_def) => {
                execute_semantic_stage(sem_def, &ctx.project_root)
            }
            StageDef::Calculation(_) => {
                // Calculation stages are handled after all other stages
                StageResult {
                    stage_id: stage_id.clone(),
                    score: 100.0,
                    passed: 0,
                    total: 0,
                    findings: vec![],
                    skipped: false,
                    error: None,
                }
            }
        };

        all_findings.extend(result.findings.clone());
        stage_results.insert(stage_id.clone(), result);

        // Check on_failure policy
        if let Some(result) = stage_results.get(stage_id) {
            if result.error.is_some() || (!result.skipped && result.score < 50.0) {
                match wf_ref.on_failure {
                    OnFailure::Stop => {
                        warn!("Pipeline stopped due to stage '{}' failure", stage_id);
                        _pipeline_failed = true;
                        break;
                    }
                    OnFailure::SkipDownstream => {
                        // Mark all downstream stages as skipped
                        // (handled in next iterations via dependency check)
                    }
                    OnFailure::Continue => {}
                }
            }
        }
    }

    // Run calculation stage if present
    let (overall_score, rating) = calculate_score(&def.calculation, &stage_results);
    let readiness = resolve_readiness(overall_score, def.calculation.readiness.as_ref());

    info!(
        "Pipeline '{}' complete: score={:.1}, rating={}",
        def.pipeline.name, overall_score, rating
    );

    PipelineExecutionResult {
        pipeline_name: def.pipeline.name.clone(),
        overall_score,
        rating,
        stage_results: stage_results.into_values().collect(),
        findings: all_findings,
        readiness,
    }
}

fn execute_deterministic_stage(
    def: &DeterministicStageDef,
    project_root: &Path,
) -> StageResult {
    // For document scope, we'd need documents passed in context
    // For now, use empty documents (rules that need docs will be skipped)
    let (findings, passed, total) = execute_deterministic_rules(&def.rules, &[], project_root);
    let score = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    StageResult {
        stage_id: String::new(), // filled by caller
        score,
        passed,
        total,
        findings,
        skipped: false,
        error: None,
    }
}

fn execute_script_stage(def: &ScriptStageDef, project_root: &Path) -> StageResult {
    let mut all_findings = Vec::new();
    let mut passed = 0;
    let total = def.scripts.len();

    for script in &def.scripts {
        let (findings, script_passed) = execute_script_def(script, project_root);
        if script_passed {
            passed += 1;
        }
        all_findings.extend(findings);
    }

    let score = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    StageResult {
        stage_id: String::new(),
        score,
        passed,
        total,
        findings: all_findings,
        skipped: false,
        error: None,
    }
}

fn execute_semantic_stage(def: &SemanticStageDef, _project_root: &Path) -> StageResult {
    let (findings, passed, total) = build_semantic_tasks(&def.rules, &[]);
    let score = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    StageResult {
        stage_id: String::new(),
        score,
        passed,
        total,
        findings,
        skipped: false,
        error: None,
    }
}

fn build_pipeline_report(def: &YamlPipelineDef, result: &PipelineExecutionResult) -> PipelineReport {
    let mut categories: HashMap<String, f64> = HashMap::new();
    for sr in &result.stage_results {
        if !sr.skipped {
            categories.insert(sr.stage_id.clone(), sr.score);
        }
    }

    PipelineReport {
        pipeline: PipelineKind::Doc,
        score: result.overall_score,
        categories,
        findings: result.findings.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: {
            let mut m = HashMap::new();
            m.insert("pipeline_name".into(), def.pipeline.name.clone());
            m.insert("pipeline_version".into(), def.pipeline.version.clone());
            m.insert("rating".into(), result.rating.clone());
            m
        },
        semantic_review: Default::default(),
    }
}

// ── Utility Functions ─────────────────────────────────────────

/// Word-boundary-aware substring match.
fn contains_word(text: &str, keyword: &str) -> bool {
    fn joins_word(c: char) -> bool {
        c.is_alphanumeric() || c == '-' || c == '_'
    }
    let kw_lower = keyword.to_lowercase();
    let mut start = 0;
    while let Some(pos) = text[start..].find(&kw_lower) {
        let abs = start + pos;
        let before_ok = !joins_word(kw_lower.chars().next().unwrap_or('x'))
            || text[..abs].chars().next_back().is_none_or(|c| !joins_word(c));
        let end = abs + kw_lower.len();
        let after_ok = !joins_word(kw_lower.chars().last().unwrap_or('x'))
            || text[end..].chars().next().is_none_or(|c| !joins_word(c));
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
    }
    false
}

/// Simple glob matching (supports * wildcard).
fn glob_match(name: &str, pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return name == pattern;
    }

    let mut remaining = name;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !remaining.starts_with(part) {
                return false;
            }
            remaining = &remaining[part.len()..];
        } else if i == parts.len() - 1 {
            if !remaining.ends_with(part) {
                return false;
            }
        } else {
            if let Some(pos) = remaining.find(part) {
                remaining = &remaining[pos + part.len()..];
            } else {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort_linear() {
        let stages = vec![
            WorkflowStageRef {
                id: "a".into(),
                stage_type: StageType::Deterministic,
                depends_on: vec![],
                condition: None,
                on_failure: OnFailure::Continue,
            },
            WorkflowStageRef {
                id: "b".into(),
                stage_type: StageType::Script,
                depends_on: vec!["a".into()],
                condition: None,
                on_failure: OnFailure::Continue,
            },
            WorkflowStageRef {
                id: "c".into(),
                stage_type: StageType::Calculation,
                depends_on: vec!["b".into()],
                condition: None,
                on_failure: OnFailure::Continue,
            },
        ];
        let sorted = topological_sort(&stages).unwrap();
        assert_eq!(sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_topological_sort_diamond() {
        let stages = vec![
            WorkflowStageRef {
                id: "a".into(),
                stage_type: StageType::Deterministic,
                depends_on: vec![],
                condition: None,
                on_failure: OnFailure::Continue,
            },
            WorkflowStageRef {
                id: "b".into(),
                stage_type: StageType::Script,
                depends_on: vec!["a".into()],
                condition: None,
                on_failure: OnFailure::Continue,
            },
            WorkflowStageRef {
                id: "c".into(),
                stage_type: StageType::Semantic,
                depends_on: vec!["a".into()],
                condition: None,
                on_failure: OnFailure::Continue,
            },
            WorkflowStageRef {
                id: "d".into(),
                stage_type: StageType::Calculation,
                depends_on: vec!["b".into(), "c".into()],
                condition: None,
                on_failure: OnFailure::Continue,
            },
        ];
        let sorted = topological_sort(&stages).unwrap();
        assert_eq!(sorted.len(), 4);
        assert!(sorted.iter().position(|s| s == "a").unwrap() < sorted.iter().position(|s| s == "b").unwrap());
        assert!(sorted.iter().position(|s| s == "a").unwrap() < sorted.iter().position(|s| s == "c").unwrap());
        assert!(sorted.iter().position(|s| s == "b").unwrap() < sorted.iter().position(|s| s == "d").unwrap());
        assert!(sorted.iter().position(|s| s == "c").unwrap() < sorted.iter().position(|s| s == "d").unwrap());
    }

    #[test]
    fn test_topological_sort_cycle() {
        let stages = vec![
            WorkflowStageRef {
                id: "a".into(),
                stage_type: StageType::Deterministic,
                depends_on: vec!["b".into()],
                condition: None,
                on_failure: OnFailure::Continue,
            },
            WorkflowStageRef {
                id: "b".into(),
                stage_type: StageType::Script,
                depends_on: vec!["a".into()],
                condition: None,
                on_failure: OnFailure::Continue,
            },
        ];
        assert!(topological_sort(&stages).is_err());
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("test.txt", "*.txt"));
        assert!(glob_match("Dockerfile", "Dockerfile"));
        assert!(!glob_match("test.txt", "*.rs"));
        assert!(glob_match("foo_bar.rs", "foo_*.rs"));
    }

    #[test]
    fn test_calculate_weighted_average() {
        let inputs = vec![
            CalculationInputDef {
                name: "det".into(),
                weight: 40.0,
                source: "deterministic".into(),
            },
            CalculationInputDef {
                name: "scripts".into(),
                weight: 30.0,
                source: "script_validation".into(),
            },
        ];

        let mut results = HashMap::new();
        results.insert(
            "deterministic".into(),
            StageResult {
                stage_id: "deterministic".into(),
                score: 80.0,
                ..Default::default()
            },
        );
        results.insert(
            "script_validation".into(),
            StageResult {
                stage_id: "script_validation".into(),
                score: 100.0,
                ..Default::default()
            },
        );

        let score = calculate_weighted_average(&inputs, &results);
        // (80/100 * 40) + (100/100 * 30) = 32 + 30 = 62
        assert!((score - 62.0).abs() < 0.01);
    }

    #[test]
    fn test_resolve_rating() {
        let bands = vec![
            ScoreBandDef { min: 90.0, label: "Excellent".into(), color: None },
            ScoreBandDef { min: 70.0, label: "Good".into(), color: None },
            ScoreBandDef { min: 0.0, label: "Critical".into(), color: None },
        ];
        assert_eq!(resolve_rating(95.0, &bands), "Excellent");
        assert_eq!(resolve_rating(80.0, &bands), "Good");
        assert_eq!(resolve_rating(50.0, &bands), "Critical");
    }
}
