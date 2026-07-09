use schemas::audit::{
    AuditFinding, PipelineKind, PipelineReport, Severity,
};
use schemas::TestRunReport;
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
    /// Run the declared Pipeline Contract (`[pipelines.build]`) instead of
    /// verify-only checking pre-existing artifacts. Build Audit only.
    pub execute: bool,
    /// Print the resolved command without running it. Build Audit only.
    pub dry_run: bool,
    /// The `repository_metadata` table's contents, if the caller has
    /// registry access to fetch it (`Pipeline::run` itself doesn't — no
    /// registry/DB handle is threaded through the trait). Empty unless a
    /// caller opts in via `with_repository_metadata`. Product Guide Audit's
    /// PA7 is currently the only consumer.
    pub repository_metadata: HashMap<String, String>,
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
            execute: false,
            dry_run: false,
            repository_metadata: HashMap::new(),
        }
    }

    pub fn with_repository_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.repository_metadata = metadata;
        self
    }

    pub fn with_inspect_artifact(mut self, val: bool) -> Self {
        self.inspect_artifact = val;
        self
    }

    pub fn with_runtime(mut self, val: bool) -> Self {
        self.runtime_mode = val;
        self
    }

    pub fn with_execute(mut self, val: bool) -> Self {
        self.execute = val;
        self
    }

    pub fn with_dry_run(mut self, val: bool) -> Self {
        self.dry_run = val;
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

/// Loads real test/coverage results from a repo's declared `[pipelines.test]`
/// contract — shared by Coverage (CV6) and Implementation (I8) so the
/// contract-execution/JSON-parsing logic lives in exactly one place.
///
/// `None` means "nothing to report yet": no `[pipelines.test]` declared,
/// `ctx.dry_run` is set (informational-only — the calling pipeline handles
/// that message itself, same split as build.rs's BC6), or the results
/// artifact simply doesn't exist yet and `ctx.execute` wasn't passed to
/// produce one. `Some(Err(_))` means a contract exists but results couldn't
/// be obtained (execution failed, artifact missing after running, or
/// unparseable) — that's worth a finding, unlike the "not adopted yet" case.
pub fn load_test_report(ctx: &PipelineContext) -> Option<Result<TestRunReport, String>> {
    let contract = ctx.config.pipelines.as_ref().and_then(|p| p.test.as_ref())?;
    if ctx.dry_run {
        return None;
    }

    let resolved = match contract.resolve(&ctx.project_root) {
        Ok(r) => r,
        Err(e) => return Some(Err(format!("[pipelines.test] could not be resolved: {e}"))),
    };
    let Some(results_path) = resolved.artifacts.first() else {
        return Some(Err(
            "[pipelines.test] declares no `artifacts` — nowhere to read results from".into(),
        ));
    };

    if ctx.execute {
        let timeout_secs = contract
            .timeout
            .as_deref()
            .and_then(common::config::parse_ttl_duration);
        if let Err(e) = crate::contract::ContractRunner::execute(&resolved, &ctx.project_root, timeout_secs) {
            // A nonzero exit code alone isn't an error here — failing tests
            // are an expected, normal outcome the script should still have
            // written a results JSON for. Only spawn/timeout/escape failures
            // (the only things ContractRunner::execute itself returns Err
            // for) short-circuit before we even try reading the artifact.
            return Some(Err(format!("Test command execution failed: {e}")));
        }
    } else if !results_path.exists() {
        return None; // declared but never run — not adopted yet, not an error
    }

    let content = match std::fs::read_to_string(results_path) {
        Ok(c) => c,
        Err(e) => return Some(Err(format!("Could not read {}: {}", results_path.display(), e))),
    };
    // Strip a leading UTF-8 BOM — several common script runtimes (Windows
    // PowerShell 5.1's `Set-Content -Encoding utf8` among them) write one by
    // default, and serde_json treats it as invalid leading bytes rather than
    // skipping it.
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content);
    match serde_json::from_str::<TestRunReport>(content) {
        Ok(report) => Some(Ok(report)),
        Err(e) => Some(Err(format!(
            "Could not parse {} as test results: {}",
            results_path.display(),
            e
        ))),
    }
}

/// Strip fenced code blocks (```...```) before running keyword/heuristic
/// text scans. ASCII-art diagrams and prose fences (` ```text `) are
/// legitimate architecture/vision content, not implementation leakage —
/// without this, a diagram inside a text fence gets flagged the same as a
/// real code sample.
pub(crate) fn strip_code_fences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Phrases that, found shortly before a keyword match, mean the keyword is
/// being disclaimed rather than actually present. Without this,
/// "should remain independent of databases" flags "databases" as a real
/// technology reference — the sentence is claiming the opposite.
const NEGATION_CUES: &[&str] = &[
    "independent of", "regardless of", "without", "avoid", "avoids", "avoiding",
    "not ", "no ", "never", "free of", "free from", "rather than", "instead of",
    "reverse engineer", "should not", "must not", "does not", "doesn't",
];

/// Case-insensitive keyword scan (`haystack_lower` must already be
/// lowercased) that only reports a keyword if at least one occurrence isn't
/// preceded by a negation cue within the last 40 characters. A keyword that
/// only ever appears negated ("we avoid X") is not returned.
pub(crate) fn find_unnegated_keywords(haystack_lower: &str, keywords: &[&str]) -> Vec<String> {
    const WINDOW: usize = 40;
    let mut found = Vec::new();
    for kw in keywords {
        let has_unnegated = haystack_lower.match_indices(kw).any(|(pos, _)| {
            if !has_word_boundaries(haystack_lower, pos, kw) {
                return false; // e.g. "rust" must not match inside "trust"/"trusted"
            }
            let mut start = pos.saturating_sub(WINDOW);
            while start > 0 && !haystack_lower.is_char_boundary(start) {
                start -= 1;
            }
            let preceding = &haystack_lower[start..pos];
            !NEGATION_CUES.iter().any(|cue| preceding.contains(cue))
        });
        if has_unnegated {
            found.push(kw.to_string());
        }
    }
    found
}

/// Whether the match of `kw` at byte offset `pos` in `haystack` is a real
/// word, not a substring of a longer word. Only checks the side(s) where the
/// keyword itself doesn't already end in a non-alphanumeric delimiter — a
/// keyword like `"impl "` or `"fn("` is already self-delimiting on its right
/// (checking past the trailing space/paren would reject valid matches like
/// "impl Foo"), so only its left edge is checked.
fn has_word_boundaries(haystack: &str, pos: usize, kw: &str) -> bool {
    // Hyphens and underscores join words the same way letters do
    // ("first-class", "snake_case") — treat them as part of the word so they
    // don't create a false boundary that lets "class " match inside
    // "first-class ".
    fn joins_word(c: char) -> bool {
        c.is_alphanumeric() || c == '-' || c == '_'
    }

    let first_alnum = kw.chars().next().is_some_and(joins_word);
    let last_alnum = kw.chars().last().is_some_and(joins_word);

    let before_ok = !first_alnum
        || haystack[..pos].chars().next_back().is_none_or(|c| !joins_word(c));
    let end = pos + kw.len();
    let after_ok = !last_alnum
        || haystack[end..].chars().next().is_none_or(|c| !joins_word(c));
    before_ok && after_ok
}

#[cfg(test)]
mod text_scan_tests {
    use super::*;

    #[test]
    fn find_unnegated_keywords_does_not_match_substring_inside_longer_word() {
        // regression: this exact false positive showed up live — "rust"
        // matched inside "trust"/"trusted" in this repo's own vision.md.
        let text = "engineering knowledge should be verified before it is trusted".to_lowercase();
        let found = find_unnegated_keywords(&text, &["rust"]);
        assert!(found.is_empty(), "expected no match, found {:?}", found);
    }

    #[test]
    fn find_unnegated_keywords_still_matches_standalone_word() {
        let text = "built using rust and typescript".to_lowercase();
        let found = find_unnegated_keywords(&text, &["rust"]);
        assert_eq!(found, vec!["rust".to_string()]);
    }

    #[test]
    fn find_unnegated_keywords_does_not_match_inside_hyphenated_compound() {
        // regression: this exact false positive showed up live — "class "
        // matched inside "first-class " in this repo's own
        // docs/raw/feature-technical/knowledge-runtime.md.
        let text = "section-type operations are a first-class runtime capability".to_lowercase();
        let found = find_unnegated_keywords(&text, &["class "]);
        assert!(found.is_empty(), "expected no match, found {:?}", found);
    }

    #[test]
    fn find_unnegated_keywords_boundary_check_does_not_break_self_delimited_keywords() {
        // "impl " already ends in a space — the boundary check must not
        // reject a valid match just because a real word follows the space.
        let text = "impl Foo for Bar".to_lowercase();
        let found = find_unnegated_keywords(&text, &["impl "]);
        assert_eq!(found, vec!["impl ".to_string()]);
    }

    #[test]
    fn strip_code_fences_removes_fenced_content_and_markers() {
        let input = "before\n```text\ndiagram here\n```\nafter";
        let stripped = strip_code_fences(input);
        assert!(stripped.contains("before"));
        assert!(stripped.contains("after"));
        assert!(!stripped.contains("diagram here"));
        assert!(!stripped.contains("```"));
    }

    #[test]
    fn find_unnegated_keywords_skips_negated_mention() {
        // regression: this exact sentence shape was a real false positive on
        // this repo's own docs/raw/vision/vision.md.
        let text = "documentation methodology should remain independent of frameworks and databases".to_lowercase();
        let found = find_unnegated_keywords(&text, &["framework", "database"]);
        assert!(found.is_empty(), "expected no unnegated keywords, found {:?}", found);
    }

    #[test]
    fn find_unnegated_keywords_flags_real_reference() {
        let text = "built using react and a postgres database".to_lowercase();
        let found = find_unnegated_keywords(&text, &["react", "database"]);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn find_unnegated_keywords_handles_multibyte_text_without_panicking() {
        // Saṃgraha contains a multi-byte UTF-8 char ('ṃ') — the negation
        // window must not slice mid-character.
        let text = "saṃgraha should remain independent of any database".to_lowercase();
        let found = find_unnegated_keywords(&text, &["database"]);
        assert!(found.is_empty());
    }

    #[test]
    fn find_unnegated_keywords_flags_if_any_occurrence_is_unnegated() {
        let text = "we avoid databases in most cases, but this uses a database directly".to_lowercase();
        let found = find_unnegated_keywords(&text, &["database"]);
        assert_eq!(found, vec!["database".to_string()]);
    }
}

#[cfg(test)]
mod load_test_report_tests {
    use super::*;
    use common::config::{ContractSpec, PipelineContractConfig, SamgrahaConfig};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("samgraha-load-test-report-{}-{}", std::process::id(), id));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn noop_command() -> Vec<String> {
        if cfg!(windows) {
            vec!["cmd".into(), "/C".into(), "exit".into(), "0".into()]
        } else {
            vec!["true".into()]
        }
    }

    fn config_with_test_contract(artifact_rel: &str) -> SamgrahaConfig {
        let mut config = SamgrahaConfig::default();
        config.pipelines = Some(PipelineContractConfig {
            version: "1.0".to_string(),
            build: None,
            test: Some(ContractSpec {
                command: noop_command(),
                working_directory: "${PROJECT_ROOT}".to_string(),
                artifacts: vec![format!("${{PROJECT_ROOT}}/{}", artifact_rel)],
                success_exit_code: None,
                timeout: None,
                description: None,
                produces: vec![],
                consumes: vec![],
            }),
            package: None,
            deploy: None,
        });
        config
    }

    fn valid_report_json() -> &'static str {
        r#"{"unit":{"total":3,"passed":2,"failed":1,"skipped":0,"failures":[{"name":"test_foo","message":"boom"}]},"e2e":{"total":1,"passed":1,"failed":0,"skipped":0,"failures":[]},"coverage_percent":80.0}"#
    }

    #[test]
    fn no_contract_declared_is_none() {
        let root = temp_root();
        let ctx = PipelineContext::new(root.clone(), SamgrahaConfig::default());
        assert!(load_test_report(&ctx).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn dry_run_is_none_even_with_contract() {
        let root = temp_root();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config).with_dry_run(true);
        assert!(load_test_report(&ctx).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn declared_but_never_run_and_no_artifact_is_none() {
        let root = temp_root();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        assert!(load_test_report(&ctx).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn preexisting_artifact_is_read_without_execute() {
        let root = temp_root();
        std::fs::write(root.join("results.json"), valid_report_json()).unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        let report = load_test_report(&ctx).unwrap().unwrap();
        assert_eq!(report.unit.failed, 1);
        assert_eq!(report.coverage_percent, Some(80.0));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn execute_runs_command_then_reads_artifact() {
        let root = temp_root();
        // Simulates a script that already wrote its results — the noop
        // command just proves the execute path re-reads after running.
        std::fs::write(root.join("results.json"), valid_report_json()).unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config).with_execute(true);
        let report = load_test_report(&ctx).unwrap().unwrap();
        assert_eq!(report.unit.total, 3);
        assert_eq!(report.e2e.passed, 1);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn utf8_bom_prefixed_artifact_still_parses() {
        // Regression: Windows PowerShell 5.1's `Set-Content -Encoding utf8`
        // writes a UTF-8 BOM by default (this repo's own scripts/test-coverage.ps1
        // hit exactly this before switching to a BOM-less writer).
        let root = temp_root();
        let mut bytes = vec![0xEFu8, 0xBB, 0xBF];
        bytes.extend_from_slice(valid_report_json().as_bytes());
        std::fs::write(root.join("results.json"), bytes).unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        let report = load_test_report(&ctx).unwrap().unwrap();
        assert_eq!(report.unit.total, 3);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn malformed_artifact_is_some_err() {
        let root = temp_root();
        std::fs::write(root.join("results.json"), "not json").unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        assert!(load_test_report(&ctx).unwrap().is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn no_artifacts_declared_is_some_err() {
        let root = temp_root();
        let mut config = config_with_test_contract("results.json");
        config.pipelines.as_mut().unwrap().test.as_mut().unwrap().artifacts = vec![];
        let ctx = PipelineContext::new(root.clone(), config).with_execute(true);
        assert!(load_test_report(&ctx).unwrap().is_err());
        std::fs::remove_dir_all(&root).ok();
    }
}


