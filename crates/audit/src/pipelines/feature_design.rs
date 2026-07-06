use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct FeatureDesignPipeline;

impl Pipeline for FeatureDesignPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::FeatureDesign
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let fd_dir = ctx.project_root.join("docs").join("raw").join("feature-design");
        let docs = if fd_dir.exists() {
            scan_markdown_files(&fd_dir)
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

        // ── Feature Mapping (FD1-FD4) 25% ───────────────────────────────

        let mut fm_passed = 0u32;
        let mut fm_total = 0u32;

        // FD1: One-to-One Mapping — duplicate title check
        fm_total += 1;
        if doc_count == 0 {
            fm_passed += 1;
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
                fm_passed += 1;
            } else {
                findings.push(finding(
                    "FD1", Severity::Error,
                    "Duplicate Feature Design titles detected — each document must map to exactly one Feature Specification".into(),
                    None,
                ));
            }
        }

        // FD2: Feature Coverage Complete — compare filenames against
        // docs/raw/feature/
        fm_total += 1;
        let feature_dir = ctx.project_root.join("docs").join("raw").join("feature");
        if feature_dir.exists() {
            let feature_names: HashSet<String> = scan_markdown_files(&feature_dir).iter()
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
                .collect();
            let fd_names: HashSet<String> = docs.iter()
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
                .collect();
            let missing: Vec<&String> = feature_names.difference(&fd_names).collect();
            if missing.is_empty() {
                fm_passed += 1;
            } else {
                findings.push(finding(
                    "FD2", Severity::Warning,
                    format!("{} Feature Specification(s) have no corresponding Feature Design document: {}", missing.len(), missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                    None,
                ));
            }
        } else {
            fm_passed += 1;
        }

        // FD3: Design System Applied
        fm_total += 1;
        if doc_count == 0 || low.contains("design") {
            fm_passed += 1;
        } else {
            findings.push(finding(
                "FD3", Severity::Warning,
                "No reference to shared Design Documentation found — Feature Design should apply shared design principles".into(),
                None,
            ));
        }

        // FD4: Relevant External Context Applied
        fm_total += 1;
        if doc_count == 0 || low.contains("external context") {
            fm_passed += 1;
        } else {
            fm_passed += 1;
            findings.push(finding(
                "FD4", Severity::Suggestion,
                "No reference to External Context found — reference relevant external UX constraints if any influence this feature".into(),
                None,
            ));
        }

        // ── User Experience (FD5-FD9) 40% ───────────────────────────────

        let mut ux_passed = 0u32;
        let mut ux_total = 0u32;

        // FD5: User Workflows Complete
        ux_total += 1;
        if doc_count == 0 || has_heading(&["Workflow", "User Workflow", "Flow"]) {
            ux_passed += 1;
        } else {
            findings.push(finding(
                "FD5", Severity::Error,
                "No Workflow section found — required by the Feature Design Standard".into(),
                None,
            ));
        }

        // FD6: User Experience Complete
        ux_total += 1;
        if doc_count == 0 || has_heading(&["User Experience", "UX", "User Flow"]) {
            ux_passed += 1;
        } else {
            findings.push(finding(
                "FD6", Severity::Error,
                "No User Experience section found — required by the Feature Design Standard".into(),
                None,
            ));
        }

        // FD7: Accessibility Considered
        ux_total += 1;
        if doc_count == 0 || low.contains("accessib") {
            ux_passed += 1;
        } else {
            ux_passed += 1;
            findings.push(finding(
                "FD7", Severity::Suggestion,
                "No accessibility considerations found — document accessibility requirements where applicable".into(),
                None,
            ));
        }

        // FD8: Localization Considered
        ux_total += 1;
        if doc_count == 0 || low.contains("localiz") || low.contains("internationaliz") {
            ux_passed += 1;
        } else {
            ux_passed += 1;
            findings.push(finding(
                "FD8", Severity::Suggestion,
                "No localization considerations found — document localization requirements where applicable".into(),
                None,
            ));
        }

        // FD9: External Constraints Reflected
        ux_total += 1;
        if doc_count == 0 || has_heading(&["Constraints", "Limitations", "Non-Functional Requirements"]) {
            ux_passed += 1;
        } else {
            ux_passed += 1;
            findings.push(finding(
                "FD9", Severity::Suggestion,
                "No Constraints section found — document user-facing constraints from External Context if applicable".into(),
                None,
            ));
        }

        // ── Documentation Quality (FD10-FD13) 20% ───────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // FD10: Technology Independence
        dq_total += 1;
        let tech_keywords = ["ui framework", "component library", "html", "css", "rest api", "graphql"];
        let tech_found = find_unnegated_keywords(&low, &tech_keywords);
        if tech_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "FD10", Severity::Warning,
                format!("Implementation technology references found ({}) — Feature Design should remain technology independent; technology belongs to Engineering", tech_found.join(", ")),
                None,
            ));
        }

        // FD11: No Architecture or Engineering Leakage
        dq_total += 1;
        let leakage_keywords = ["```", "runtime behavior", "communication pattern", "persistence", "engineering rationale"];
        let leakage_found = find_unnegated_keywords(&low, &leakage_keywords);
        if leakage_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "FD11", Severity::Warning,
                "Architecture or engineering content detected — Feature Design describes user experience, not implementation or architecture".into(),
                None,
            ));
        }

        // FD12: References Rather Than Duplication — stub, requires
        // semantic comparison against Design/External Context text.
        dq_total += 1;
        dq_passed += 1;
        findings.push(finding(
            "FD12", Severity::Suggestion,
            "Duplication-vs-reference detection requires semantic comparison against Design and External Context documents — not yet implemented".into(),
            None,
        ));

        // FD13: UX Consistency — shared terminology check
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
                        "FD13", Severity::Warning,
                        "Limited shared terminology across Feature Design documents".into(),
                        None,
                    ));
                }
            } else {
                dq_passed += 1;
            }
        } else {
            dq_passed += 1;
        }

        // ── Design Readiness (FD14-FD15) 15% ────────────────────────────

        let mut dr_passed = 0u32;
        let mut dr_total = 0u32;

        // FD14: Feature Technical Design Readiness
        dr_total += 1;
        if doc_count == 0 || (
            has_heading(&["User Experience", "UX", "User Flow"])
            && has_heading(&["Workflow", "User Workflow", "Flow"])
            && has_heading(&["States", "UI States", "Application States", "State Transitions"])
        ) {
            dr_passed += 1;
        } else {
            findings.push(finding(
                "FD14", Severity::Error,
                "User Experience, Workflow, or States missing — Feature Technical Design cannot proceed without all three".into(),
                None,
            ));
        }

        // FD15: Future Maintainability
        dr_total += 1;
        if doc_count == 0 || doc_count >= 2 {
            dr_passed += 1;
        } else {
            dr_passed += 1;
            findings.push(finding(
                "FD15", Severity::Suggestion,
                "Only one Feature Design document exists — verify each feature gets its own document as the collection grows".into(),
                None,
            ));
        }

        // ── Category Scores ──────────────────────────────────────────────

        let fm_score = if fm_total > 0 { (fm_passed as f64 / fm_total as f64) * 100.0 } else { 100.0 };
        let ux_score = if ux_total > 0 { (ux_passed as f64 / ux_total as f64) * 100.0 } else { 100.0 };
        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };
        let dr_score = if dr_total > 0 { (dr_passed as f64 / dr_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Feature Mapping".into(), fm_score);
        cat_scores.insert("User Experience".into(), ux_score);
        cat_scores.insert("Documentation Quality".into(), dq_score);
        cat_scores.insert("Design Readiness".into(), dr_score);

        // Weighted overall: 25/40/20/15
        let overall = fm_score * 0.25 + ux_score * 0.40 + dq_score * 0.20 + dr_score * 0.15;

        let mut report = make_report(PipelineKind::FeatureDesign, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, ux_score, dr_score));
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

fn readiness_label(overall: f64, ux: f64, dr: f64) -> String {
    if overall >= 90.0 && ux >= 80.0 && dr >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-fd-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_fd_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/feature-design");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), content).unwrap();
            self
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
    fn fd5_errors_when_workflow_missing() {
        let proj = TempProject::new()
            .with_fd_file("auth.md", "# Authentication\n\n## User Experience\n\nUser logs in with one click.");
        let report = FeatureDesignPipeline.run(&proj.ctx());
        let fd5 = report.findings.iter().find(|f| f.check_id == "FD5").unwrap();
        assert_eq!(fd5.severity, Severity::Error);
    }

    #[test]
    fn fd5_passes_with_workflow_heading() {
        let proj = TempProject::new()
            .with_fd_file("auth.md", "# Authentication\n\n## User Experience\n\nOne-click login.\n\n## Workflow\n\n1. User clicks Login.");
        let report = FeatureDesignPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "FD5"));
    }

    #[test]
    fn fd2_warns_when_feature_has_no_design() {
        let proj = TempProject::new()
            .with_feature_file("authentication.md", "# Authentication\n\n## Purpose\n\nLog users in.")
            .with_fd_file("localization.md", "# Localization\n\n## User Experience\n\nTranslate strings.");
        let report = FeatureDesignPipeline.run(&proj.ctx());
        let fd2 = report.findings.iter().find(|f| f.check_id == "FD2").unwrap();
        assert_eq!(fd2.severity, Severity::Warning);
    }

    #[test]
    fn fd14_errors_when_design_readiness_incomplete() {
        let proj = TempProject::new()
            .with_fd_file("auth.md", "# Authentication\n\n## User Experience\n\nOne-click login.");
        let report = FeatureDesignPipeline.run(&proj.ctx());
        let fd14 = report.findings.iter().find(|f| f.check_id == "FD14").unwrap();
        assert_eq!(fd14.severity, Severity::Error);
    }

    #[test]
    fn score_is_within_bounds_with_no_docs() {
        let proj = TempProject::new();
        let report = FeatureDesignPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
