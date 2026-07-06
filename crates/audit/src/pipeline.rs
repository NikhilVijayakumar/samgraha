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


