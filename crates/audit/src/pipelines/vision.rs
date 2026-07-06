use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct VisionPipeline;

impl Pipeline for VisionPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Vision
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let vision_dir = ctx.project_root.join("docs").join("raw").join("vision");
        let docs = if vision_dir.exists() {
            scan_markdown_files(&vision_dir)
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

        // ── Vision Content (V1-V5) 35% ──────────────────────────────────

        let mut vc_passed = 0u32;
        let mut vc_total = 0u32;

        // V1: Purpose and Problem Defined
        vc_total += 1;
        if doc_count == 0 {
            findings.push(finding(
                "V1", Severity::Error,
                "No vision documents found — purpose and problem cannot be verified".into(),
                None,
            ));
        } else if has_heading(&["Purpose", "Overview", "Summary"]) && has_heading(&["Problem", "Problem Statement", "The Problem"]) {
            vc_passed += 1;
        } else {
            findings.push(finding(
                "V1", Severity::Error,
                "Purpose and/or Problem section missing — the Vision must clearly explain why the product exists".into(),
                None,
            ));
        }

        // V2: Long-term Direction Explicit
        vc_total += 1;
        if has_heading(&["Vision", "Long-Term Vision", "The Vision"]) {
            vc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "V2", Severity::Error,
                "No Vision statement section found — long-term direction is not explicit".into(),
                None,
            ));
        }

        // V3: Product Philosophy Documented
        vc_total += 1;
        if has_heading(&["Philosophy", "Product Philosophy", "Design Philosophy"]) {
            vc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "V3", Severity::Warning,
                "Product Philosophy section not found — values guiding product decisions should be documented".into(),
                None,
            ));
        }

        // V4: Guiding Principles Documented
        vc_total += 1;
        if has_heading(&["Guiding Principles", "Principles", "Core Principles"]) {
            vc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "V4", Severity::Warning,
                "Guiding Principles section not found — enduring principles should be documented".into(),
                None,
            ));
        }

        // V5: Target Audience Identified
        vc_total += 1;
        if has_heading(&["Target Audience", "Audience", "Who Is This For"]) {
            vc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "V5", Severity::Warning,
                "Target Audience section not found — the Vision should identify who benefits from the product".into(),
                None,
            ));
        }

        // ── Technology Independence (V6-V8) 30% ─────────────────────────

        let mut ti_passed = 0u32;
        let mut ti_total = 0u32;

        // V6: No Implementation Technologies. Negation-aware so "should
        // remain independent of frameworks and databases" — this repo's own
        // vision.md phrasing — doesn't get flagged as referencing them.
        ti_total += 1;
        let tech_keywords = ["react", "typescript", "javascript", "python", "rust", "sqlite", "postgres",
            "kubernetes", "docker", "aws", "azure", "gcp", "database", "framework", "cloud provider"];
        let tech_found = find_unnegated_keywords(&low, &tech_keywords);
        if tech_found.is_empty() {
            ti_passed += 1;
        } else {
            findings.push(finding(
                "V6", Severity::Error,
                format!("Implementation technologies referenced ({}) — Vision must remain technology independent", tech_found.join(", ")),
                None,
            ));
        }

        // V7: No Implementation Details. "```" dropped — code fences are
        // stripped from `all_text` before this point, so a fenced diagram no
        // longer counts as a code sample. "class " dropped: too short,
        // false-positives on "this class of systems".
        ti_total += 1;
        let impl_keywords = ["algorithm", "source code", "api endpoint", "config file", "function("];
        let impl_found = find_unnegated_keywords(&low, &impl_keywords);
        if impl_found.is_empty() {
            ti_passed += 1;
        } else {
            findings.push(finding(
                "V7", Severity::Error,
                format!("Implementation details detected ({}) — implementation belongs to downstream documentation", impl_found.join(", ")),
                None,
            ));
        }

        // V8: No Feature Specifications
        ti_total += 1;
        let feature_phrases = ["user story", "acceptance criteria", "user workflow", "ui layout", "button", "screen", "click"];
        let feature_found = find_unnegated_keywords(&low, &feature_phrases);
        if feature_found.is_empty() {
            ti_passed += 1;
        } else {
            findings.push(finding(
                "V8", Severity::Warning,
                format!("Feature-specification language detected ({}) — Features should derive from Vision, not appear within it", feature_found.join(", ")),
                None,
            ));
        }

        // ── Traceability and Consistency (V9-V11) 20% ───────────────────

        let mut tc_passed = 0u32;
        let mut tc_total = 0u32;

        // V9: Downstream Documentation Consistent — deep cross-doc semantic
        // comparison isn't implemented; this reports presence, not conflicts.
        tc_total += 1;
        let downstream_dirs = ["feature", "architecture", "engineering"];
        let has_downstream = downstream_dirs.iter().any(|d| ctx.project_root.join("docs/raw").join(d).exists());
        if !has_downstream {
            tc_passed += 1; // nothing downstream yet to conflict with
        } else {
            findings.push(finding(
                "V9", Severity::Suggestion,
                "Downstream documentation exists — cross-document contradiction detection requires semantic comparison, not yet implemented".into(),
                None,
            ));
            tc_passed += 1;
        }

        // V10: Vision Guides Feature Development
        tc_total += 1;
        if doc_count > 0 && (has_heading(&["Guiding Principles", "Principles", "Core Principles"]) || has_heading(&["Platform Pillars", "Pillars", "Foundations", "Core Pillars"])) {
            tc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "V10", Severity::Warning,
                "Neither Pillars nor Guiding Principles found — the Vision may not provide sufficient direction for feature definition".into(),
                None,
            ));
        }

        // V11: Stable and Future-Oriented
        tc_total += 1;
        let short_term_phrases = ["this sprint", "this quarter", "mvp", "release 1.0", "next release", "milestone 1"];
        let short_term_found = find_unnegated_keywords(&low, &short_term_phrases);
        if short_term_found.is_empty() {
            tc_passed += 1;
        } else {
            findings.push(finding(
                "V11", Severity::Warning,
                format!("Short-term / release-specific language detected ({}) — Vision should describe long-term direction, not milestones", short_term_found.join(", ")),
                None,
            ));
        }

        // ── Documentation Quality (V12) 15% ─────────────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // V12: Terminology Consistent
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
                        "V12", Severity::Warning,
                        "Limited shared terminology across vision documents — consistent product language improves clarity".into(),
                        None,
                    ));
                }
            } else {
                dq_passed += 1;
            }
        } else {
            dq_passed += 1; // single file or none, trivially consistent
        }

        // ── Category Scores ──────────────────────────────────────────────

        let vc_score = if vc_total > 0 { (vc_passed as f64 / vc_total as f64) * 100.0 } else { 100.0 };
        let ti_score = if ti_total > 0 { (ti_passed as f64 / ti_total as f64) * 100.0 } else { 100.0 };
        let tc_score = if tc_total > 0 { (tc_passed as f64 / tc_total as f64) * 100.0 } else { 100.0 };
        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Vision Content".into(), vc_score);
        cat_scores.insert("Technology Independence".into(), ti_score);
        cat_scores.insert("Traceability and Consistency".into(), tc_score);
        cat_scores.insert("Documentation Quality".into(), dq_score);

        // Weighted overall: 35/30/20/15
        let overall = vc_score * 0.35 + ti_score * 0.30 + tc_score * 0.20 + dq_score * 0.15;

        let mut report = make_report(PipelineKind::Vision, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, vc_score, ti_score));
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

fn readiness_label(overall: f64, vc: f64, ti: f64) -> String {
    if overall >= 90.0 && vc >= 80.0 && ti >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-vision-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_vision_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/vision");
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
    fn v1_errors_when_no_vision_docs() {
        let proj = TempProject::new();
        let report = VisionPipeline.run(&proj.ctx());
        let v1 = report.findings.iter().find(|f| f.check_id == "V1").unwrap();
        assert_eq!(v1.severity, Severity::Error);
    }

    #[test]
    fn v1_passes_with_purpose_and_problem() {
        let proj = TempProject::new()
            .with_vision_file("vision.md", "# Purpose\n\nWhy this exists.\n\n## Problem\n\nWhat's wrong today.");
        let report = VisionPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "V1"));
    }

    #[test]
    fn v6_errors_on_technology_leakage() {
        let proj = TempProject::new()
            .with_vision_file("vision.md", "# Purpose\n\nBuilt with React and a Postgres database.");
        let report = VisionPipeline.run(&proj.ctx());
        let v6 = report.findings.iter().find(|f| f.check_id == "V6").unwrap();
        assert_eq!(v6.severity, Severity::Error);
    }

    #[test]
    fn v6_passes_without_technology_references() {
        let proj = TempProject::new()
            .with_vision_file("vision.md", "# Purpose\n\nA platform for citizens to access services easily.");
        let report = VisionPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "V6"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = VisionPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
