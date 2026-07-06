use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct DesignPipeline;

impl Pipeline for DesignPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Design
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let design_dir = ctx.project_root.join("docs").join("raw").join("design");
        let docs = if design_dir.exists() {
            scan_markdown_files(&design_dir)
        } else {
            Vec::new()
        };
        let doc_count = docs.len();

        let found_headings: HashSet<String> = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .flat_map(|content| extract_headings(&content))
            .collect();
        let has_heading = |names: &[&str]| {
            found_headings.iter().any(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
        };

        let all_text: String = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let low = all_text.to_lowercase();

        // ── Design System (D1-D4) 35% ───────────────────────────────────

        let mut ds_passed = 0u32;
        let mut ds_total = 0u32;

        // D1: Reusable Design Principles
        ds_total += 1;
        if doc_count == 0 {
            findings.push(finding(
                "D1", Severity::Error,
                "No design documents found — reusable design principles cannot be verified".into(),
                None,
            ));
        } else if has_heading(&["Design Principles", "Principles", "Core Design"]) {
            ds_passed += 1;
        } else {
            findings.push(finding(
                "D1", Severity::Error,
                "Design Principles section missing — design principles must be documented and reusable".into(),
                None,
            ));
        }

        // D2: Design Philosophy Defined
        ds_total += 1;
        let has_philosophy_words = low.contains("philosophy") || low.contains("design goal");
        if has_philosophy_words {
            ds_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "D2", Severity::Warning,
                "No design philosophy found — design decisions should derive from a documented philosophy".into(),
                None,
            ));
        }

        // D3: Design System Completeness
        ds_total += 1;
        let required_concerns = ["visual language", "interaction", "navigation", "accessibility",
            "localization", "responsive", "information hierarchy", "feedback"];
        let missing_concerns: Vec<&str> = required_concerns.iter()
            .copied()
            .filter(|c| !low.contains(c))
            .collect();
        if missing_concerns.is_empty() {
            ds_passed += 1;
        } else if missing_concerns.len() <= 3 {
            ds_passed += 1;
            findings.push(finding(
                "D3", Severity::Warning,
                format!("Design system completeness: missing {} — consider adding these concerns", missing_concerns.join(", ")),
                None,
            ));
        } else {
            findings.push(finding(
                "D3", Severity::Error,
                format!("Design system incomplete: missing {}", missing_concerns.join(", ")),
                None,
            ));
        }

        // D4: Technology Independence
        ds_total += 1;
        let tech_keywords = ["css", "html", "javascript", "react", "vue", "angular",
            "tailwind", "bootstrap", "component library", "rendering technology"];
        let tech_found = find_unnegated_keywords(&low, &tech_keywords);
        if tech_found.is_empty() {
            ds_passed += 1;
        } else {
            findings.push(finding(
                "D4", Severity::Error,
                format!("Implementation technologies referenced ({}) — Design must remain technology independent", tech_found.join(", ")),
                None,
            ));
        }

        // ── Documentation Quality (D5-D8) 30% ───────────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // D5: Modular Documentation
        dq_total += 1;
        if doc_count >= 2 {
            dq_passed += 1;
        } else if doc_count == 1 {
            let content = fs::read_to_string(&docs[0]).unwrap_or_default();
            let h2_count = content.matches("\n## ").count();
            if h2_count >= 3 {
                dq_passed += 1;
            } else {
                findings.push(finding(
                    "D5", Severity::Warning,
                    "Single design document with few sections — consider decomposing by design concern".into(),
                    Some(docs[0].to_string_lossy().to_string()),
                ));
            }
        } else {
            dq_passed += 1;
        }

        // D6: Responsibility Separation — duplicate title check
        dq_total += 1;
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
            dq_passed += 1;
        } else {
            findings.push(finding(
                "D6", Severity::Error,
                "Duplicate design documents detected — each design concern must be documented once".into(),
                None,
            ));
        }

        // D7: Feature Independence
        dq_total += 1;
        let feature_phrases = ["user story", "acceptance criteria", "feature requirement", "business rule", "this screen", "this workflow"];
        let feature_found = find_unnegated_keywords(&low, &feature_phrases);
        if feature_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "D7", Severity::Warning,
                format!("Feature-specific language detected ({}) — feature-specific design belongs to Feature Design", feature_found.join(", ")),
                None,
            ));
        }

        // D8: Cross-Repository Reuse
        dq_total += 1;
        if low.contains("shared") || low.contains("cross-repo") || low.contains("cross repo") || low.contains("external design") {
            dq_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "D8", Severity::Suggestion,
                "Cross-repository reuse not mentioned — if design is shared across repos, reference rather than duplicate it".into(),
                None,
            ));
        } else {
            dq_passed += 1;
        }

        // ── Design Quality (D9-D12) 35% ─────────────────────────────────

        let mut dqu_passed = 0u32;
        let mut dqu_total = 0u32;

        // D9: Accessibility Guidance
        dqu_total += 1;
        if has_heading(&["Accessibility", "A11y", "Accessibility Standards"]) {
            dqu_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "D9", Severity::Warning,
                "No Accessibility section found — accessibility guidance should be documented where applicable".into(),
                None,
            ));
        }

        // D10: Localization Guidance
        dqu_total += 1;
        if low.contains("localization") || low.contains("rtl") || low.contains("internationalization") {
            dqu_passed += 1;
        } else {
            dqu_passed += 1; // optional — absence alone isn't a failure
            findings.push(finding(
                "D10", Severity::Suggestion,
                "No localization guidance found — document if applicable to this product".into(),
                None,
            ));
        }

        // D11: Consistency
        dqu_total += 1;
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
                    dqu_passed += 1;
                } else {
                    findings.push(finding(
                        "D11", Severity::Warning,
                        "Limited shared terminology across design documents".into(),
                        None,
                    ));
                }
            } else {
                dqu_passed += 1;
            }
        } else {
            dqu_passed += 1;
        }

        // D12: Future Maintainability — stub, matches honest-stub pattern elsewhere
        dqu_total += 1;
        findings.push(finding(
            "D12", Severity::Suggestion,
            "Future maintainability assessment requires historical comparison — not yet implemented".into(),
            None,
        ));
        dqu_passed += 1;

        // ── Category Scores ──────────────────────────────────────────────

        let ds_score = if ds_total > 0 { (ds_passed as f64 / ds_total as f64) * 100.0 } else { 100.0 };
        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };
        let dqu_score = if dqu_total > 0 { (dqu_passed as f64 / dqu_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Design System".into(), ds_score);
        cat_scores.insert("Documentation Quality".into(), dq_score);
        cat_scores.insert("Design Quality".into(), dqu_score);

        // Weighted overall: 35/30/35
        let overall = ds_score * 0.35 + dq_score * 0.30 + dqu_score * 0.35;

        let mut report = make_report(PipelineKind::Design, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, ds_score));
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

fn readiness_label(overall: f64, design_system: f64) -> String {
    if overall >= 90.0 && design_system >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-design-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_design_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/design");
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
    fn d1_errors_when_no_design_docs() {
        let proj = TempProject::new();
        let report = DesignPipeline.run(&proj.ctx());
        let d1 = report.findings.iter().find(|f| f.check_id == "D1").unwrap();
        assert_eq!(d1.severity, Severity::Error);
    }

    #[test]
    fn d1_passes_with_design_principles_heading() {
        let proj = TempProject::new()
            .with_design_file("principles.md", "# Design Principles\n\nConsistency, simplicity.");
        let report = DesignPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "D1"));
    }

    #[test]
    fn d4_errors_on_technology_leakage() {
        let proj = TempProject::new()
            .with_design_file("principles.md", "# Design Principles\n\nBuilt with React and Tailwind CSS.");
        let report = DesignPipeline.run(&proj.ctx());
        let d4 = report.findings.iter().find(|f| f.check_id == "D4").unwrap();
        assert_eq!(d4.severity, Severity::Error);
    }

    #[test]
    fn d4_does_not_flag_negated_technology_mention() {
        let proj = TempProject::new()
            .with_design_file("principles.md", "# Design Principles\n\nGuidance should remain independent of CSS and any rendering technology.");
        let report = DesignPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "D4"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = DesignPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
