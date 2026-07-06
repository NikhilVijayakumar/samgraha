use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct FeaturePipeline;

impl Pipeline for FeaturePipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Feature
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let feature_dir = ctx.project_root.join("docs").join("raw").join("feature");
        let docs = if feature_dir.exists() {
            scan_markdown_files(&feature_dir)
        } else {
            Vec::new()
        };
        let doc_count = docs.len();

        let per_doc_headings: Vec<HashSet<String>> = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| extract_headings(&strip_code_fences(&c)))
            .collect();
        let found_headings: HashSet<String> = per_doc_headings.iter().flatten().cloned().collect();
        let has_heading = |names: &[&str]| {
            found_headings.iter().any(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
        };

        let all_text: String = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let low = all_text.to_lowercase();

        // ── Feature Definition (F1-F4) 30% ──────────────────────────────

        let mut fd_passed = 0u32;
        let mut fd_total = 0u32;

        // F1: Atomic Features — duplicate title check, same pattern as
        // architecture/design/external-context's responsibility-separation check
        fd_total += 1;
        if doc_count == 0 {
            fd_passed += 1;
        } else {
            let mut titles_seen: HashSet<String> = HashSet::new();
            let mut dupes_found = false;
            for p in &docs {
                if let Some(title) = fs::read_to_string(p).ok()
                    .and_then(|c| c.lines().next().map(|l| l.trim_start_matches("# ").to_lowercase()))
                {
                    if !titles_seen.insert(title) {
                        dupes_found = true;
                    }
                }
            }
            if !dupes_found {
                fd_passed += 1;
            } else {
                findings.push(finding(
                    "F1", Severity::Error,
                    "Duplicate feature document titles detected — each document must describe exactly one feature".into(),
                    None,
                ));
            }
        }

        // F2: Responsibilities Defined
        fd_total += 1;
        if doc_count == 0 || has_heading(&["Purpose", "Overview", "Summary"]) {
            fd_passed += 1;
        } else {
            findings.push(finding(
                "F2", Severity::Error,
                "No Purpose section found — required by the Feature Standard".into(),
                None,
            ));
        }

        // F3: Product Scope Complete
        fd_total += 1;
        if doc_count > 0 {
            fd_passed += 1;
        } else {
            findings.push(finding(
                "F3", Severity::Error,
                "No feature documents found — the Feature collection cannot describe the product".into(),
                None,
            ));
        }

        // F4: Technology Independence
        fd_total += 1;
        let tech_keywords = ["framework", "database", "programming language", "rest api", "graphql"];
        let tech_found = find_unnegated_keywords(&low, &tech_keywords);
        if tech_found.is_empty() {
            fd_passed += 1;
        } else {
            findings.push(finding(
                "F4", Severity::Warning,
                format!("Technology references found ({}) — Feature Documentation should remain technology independent; technology belongs to Engineering", tech_found.join(", ")),
                None,
            ));
        }

        // ── Product Definition (F5-F8) 35% ──────────────────────────────

        let mut pd_passed = 0u32;
        let mut pd_total = 0u32;

        // F5: Business Rules Complete
        pd_total += 1;
        if doc_count == 0 || has_heading(&["Business Rules", "Rules", "Business Logic"]) {
            pd_passed += 1;
        } else {
            pd_passed += 1;
            findings.push(finding(
                "F5", Severity::Suggestion,
                "No Business Rules section found in any feature — add if the feature enforces domain logic or policy".into(),
                None,
            ));
        }

        // F6: Acceptance Criteria Complete
        pd_total += 1;
        if doc_count == 0 || has_heading(&["Acceptance Criteria", "Success Criteria", "Definition of Done", "Criteria"]) {
            pd_passed += 1;
        } else {
            findings.push(finding(
                "F6", Severity::Error,
                "No Acceptance Criteria section found — required by the Feature Standard".into(),
                None,
            ));
        }

        // F7: Product Constraints Documented
        pd_total += 1;
        if doc_count == 0 || has_heading(&["Constraints", "Limitations", "Non-Functional Requirements"]) {
            pd_passed += 1;
        } else {
            pd_passed += 1;
            findings.push(finding(
                "F7", Severity::Suggestion,
                "No Constraints section found in any feature — document permissions, limits, or workflow constraints if applicable".into(),
                None,
            ));
        }

        // F8: User Value Clear
        pd_total += 1;
        if doc_count == 0 || low.contains("value") || low.contains("benefit") || has_heading(&["Purpose", "Overview", "Summary"]) {
            pd_passed += 1;
        } else {
            findings.push(finding(
                "F8", Severity::Warning,
                "User value not clearly stated — explain who benefits and why the feature exists".into(),
                None,
            ));
        }

        // ── Documentation Quality (F9-F12) 20% ──────────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // F9: Vision Traceability
        dq_total += 1;
        if doc_count == 0 || has_heading(&["Traceability", "Traces To", "Derived From"]) || low.contains("vision") {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "F9", Severity::Warning,
                "No traceability to Vision found — every feature should trace to one or more Vision objectives".into(),
                None,
            ));
        }

        // F10: Independent Understanding — stub, requires semantic
        // cross-feature-dependency analysis.
        dq_total += 1;
        dq_passed += 1;
        findings.push(finding(
            "F10", Severity::Suggestion,
            "Independent understanding requires semantic cross-feature-dependency analysis — not yet implemented".into(),
            None,
        ));

        // F11: No Design or Engineering Leakage
        dq_total += 1;
        let leakage_keywords = ["```", "class ", "fn(", "ui component", "architecture"];
        let leakage_found = find_unnegated_keywords(&low, &leakage_keywords);
        if leakage_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "F11", Severity::Warning,
                "Design or engineering content detected — Feature Documentation describes product capability, not implementation or architecture".into(),
                None,
            ));
        }

        // F12: Terminology Consistency
        dq_total += 1;
        if doc_count >= 2 {
            let all_words: Vec<HashSet<String>> = docs.iter()
                .filter_map(|p| fs::read_to_string(p).ok())
                .map(|c| {
                    c.split_whitespace()
                        .filter(|w| w.len() > 3 && w.chars().all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '-'))
                        .map(|w| w.to_lowercase())
                        .collect()
                })
                .collect();
            if all_words.len() >= 2 {
                let intersection: HashSet<&String> = all_words[0].iter()
                    .filter(|w| all_words[1..].iter().all(|set| set.contains(*w)))
                    .collect();
                if intersection.len() >= 3 {
                    dq_passed += 1;
                } else {
                    findings.push(finding(
                        "F12", Severity::Warning,
                        "Limited shared terminology across Feature documents".into(),
                        None,
                    ));
                }
            } else {
                dq_passed += 1;
            }
        } else {
            dq_passed += 1;
        }

        // ── Product Readiness (F13-F14) 15% ─────────────────────────────

        let mut pr_passed = 0u32;
        let mut pr_total = 0u32;

        // F13: Downstream Readiness
        pr_total += 1;
        if doc_count == 0 || (has_heading(&["Functional Requirements", "Requirements", "FRs", "Functional Reqs", "Feature Requirements"])
            && has_heading(&["Acceptance Criteria", "Success Criteria", "Definition of Done", "Criteria"]))
        {
            pr_passed += 1;
        } else {
            findings.push(finding(
                "F13", Severity::Error,
                "Functional Requirements or Acceptance Criteria missing — Feature Design cannot proceed without both".into(),
                None,
            ));
        }

        // F14: Future Maintainability
        pr_total += 1;
        if doc_count == 0 || has_heading(&["Future Extensions", "Future Work", "Roadmap", "Non-Goals", "Non Goals", "Out of Scope"]) {
            pr_passed += 1;
        } else {
            pr_passed += 1;
            findings.push(finding(
                "F14", Severity::Suggestion,
                "No Future Extensions or Non-Goals section found — document scope boundaries to prevent future scope creep".into(),
                None,
            ));
        }

        // ── Category Scores ──────────────────────────────────────────────

        let fd_score = if fd_total > 0 { (fd_passed as f64 / fd_total as f64) * 100.0 } else { 100.0 };
        let pd_score = if pd_total > 0 { (pd_passed as f64 / pd_total as f64) * 100.0 } else { 100.0 };
        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };
        let pr_score = if pr_total > 0 { (pr_passed as f64 / pr_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Feature Definition".into(), fd_score);
        cat_scores.insert("Product Definition".into(), pd_score);
        cat_scores.insert("Documentation Quality".into(), dq_score);
        cat_scores.insert("Product Readiness".into(), pr_score);

        // Weighted overall: 30/35/20/15
        let overall = fd_score * 0.30 + pd_score * 0.35 + dq_score * 0.20 + pr_score * 0.15;

        let mut report = make_report(PipelineKind::Feature, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, fd_score, pd_score));
        report
    }
}

fn scan_markdown_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn extract_headings(content: &str) -> HashSet<String> {
    content.lines()
        .filter(|l| l.starts_with("# ") || l.starts_with("## "))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .collect()
}

fn readiness_label(overall: f64, fd: f64, pd: f64) -> String {
    if overall >= 90.0 && fd >= 80.0 && pd >= 80.0 {
        "READY".to_string()
    } else if overall >= 70.0 {
        "NEEDS_WORK".to_string()
    } else {
        "NOT_READY".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempProject {
        root: std::path::PathBuf,
    }

    impl TempProject {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!("samgraha-feature-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_feature_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/feature");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), content).unwrap();
            self
        }

        fn ctx(&self) -> PipelineContext {
            PipelineContext::new(self.root.clone(), common::config::SamgrahaConfig::default())
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn f3_errors_when_no_feature_docs() {
        let proj = TempProject::new();
        let report = FeaturePipeline.run(&proj.ctx());
        let f3 = report.findings.iter().find(|f| f.check_id == "F3").unwrap();
        assert_eq!(f3.severity, Severity::Error);
    }

    #[test]
    fn f1_errors_on_duplicate_titles() {
        let proj = TempProject::new()
            .with_feature_file("a.md", "# Authentication\n\n## Purpose\n\nLog users in.")
            .with_feature_file("b.md", "# Authentication\n\n## Purpose\n\nDifferent content.");
        let report = FeaturePipeline.run(&proj.ctx());
        let f1 = report.findings.iter().find(|f| f.check_id == "F1").unwrap();
        assert_eq!(f1.severity, Severity::Error);
    }

    #[test]
    fn f6_errors_when_acceptance_criteria_missing() {
        let proj = TempProject::new()
            .with_feature_file("auth.md", "# Authentication\n\n## Purpose\n\nLog users in.\n\n## Functional Requirements\n\nFR1. System shall authenticate users.");
        let report = FeaturePipeline.run(&proj.ctx());
        let f6 = report.findings.iter().find(|f| f.check_id == "F6").unwrap();
        assert_eq!(f6.severity, Severity::Error);
    }

    #[test]
    fn f6_passes_with_acceptance_criteria_heading() {
        let proj = TempProject::new()
            .with_feature_file("auth.md", "# Authentication\n\n## Purpose\n\nLog users in.\n\n## Acceptance Criteria\n\nGiven valid credentials, when submitted, then session is created.");
        let report = FeaturePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "F6"));
    }

    #[test]
    fn f4_warns_on_technology_leakage() {
        let proj = TempProject::new()
            .with_feature_file("auth.md", "# Authentication\n\n## Purpose\n\nThis feature uses a REST API and a relational database.");
        let report = FeaturePipeline.run(&proj.ctx());
        let f4 = report.findings.iter().find(|f| f.check_id == "F4").unwrap();
        assert_eq!(f4.severity, Severity::Warning);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = FeaturePipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
