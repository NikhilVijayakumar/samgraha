use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;
use std::fs;

pub struct ReadmePipeline;

impl Pipeline for ReadmePipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Readme
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        // README is a single repo-root file, not a docs/raw/<domain>/
        // collection — every other domain pipeline scans a directory of
        // documents; this one scans exactly one file.
        let readme_path = ctx.project_root.join("README.md");
        let exists = readme_path.exists();
        let raw_content = if exists {
            fs::read_to_string(&readme_path).unwrap_or_default()
        } else {
            String::new()
        };
        let content = strip_code_fences(&raw_content);
        let low = content.to_lowercase();
        let headings = extract_headings(&content);
        let has_heading = |names: &[&str]| {
            headings.iter().any(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
        };

        // ── Repository Introduction (R1-R3) 30% ─────────────────────────

        let mut ri_passed = 0u32;
        let mut ri_total = 0u32;

        // R1: Purpose Immediately Understandable
        ri_total += 1;
        if !exists {
            findings.push(finding(
                "R1", Severity::Error,
                "No README.md found at repository root — project purpose cannot be verified".into(),
                None,
            ));
        } else if content.split_whitespace().count() >= 20 {
            ri_passed += 1;
        } else {
            findings.push(finding(
                "R1", Severity::Error,
                "README is too short to establish project purpose within the opening section".into(),
                Some("README.md".into()),
            ));
        }

        // R2: Repository Responsibilities Defined
        ri_total += 1;
        if has_heading(&["Purpose", "Overview", "About", "Key Capabilities"]) {
            ri_passed += 1;
        } else if exists {
            findings.push(finding(
                "R2", Severity::Warning,
                "No Purpose/Overview section found — repository responsibilities should be explained".into(),
                Some("README.md".into()),
            ));
        }

        // R3: Repository Identity Consistent — cross-doc check not
        // implemented (requires comparing terminology against Vision/
        // Architecture); reports presence of those docs as a proxy signal.
        ri_total += 1;
        let has_vision = ctx.project_root.join("docs/raw/vision").exists();
        if has_vision {
            findings.push(finding(
                "R3", Severity::Suggestion,
                "Vision documentation exists — cross-document identity consistency requires semantic comparison, not yet implemented".into(),
                None,
            ));
        }
        ri_passed += 1;

        // ── Documentation Navigation (R4-R6) 30% ────────────────────────

        let mut dn_passed = 0u32;
        let mut dn_total = 0u32;

        // R4: Documentation Navigation Complete
        dn_total += 1;
        let nav_targets = ["vision", "architecture", "design", "feature", "engineering", "prototype", "external context", "external-context"];
        let nav_found = nav_targets.iter().filter(|t| low.contains(*t)).count();
        if nav_found >= 2 {
            dn_passed += 1;
        } else if exists {
            findings.push(finding(
                "R4", Severity::Warning,
                "README references fewer than 2 documentation domains — navigation to detailed docs may be incomplete".into(),
                Some("README.md".into()),
            ));
        }

        // R5: Repository Structure Explained
        dn_total += 1;
        if has_heading(&["Repository Structure", "Structure", "Project Structure"]) || low.contains("docs/") {
            dn_passed += 1;
        } else if exists {
            findings.push(finding(
                "R5", Severity::Suggestion,
                "No repository structure explanation found — introduce major directories (docs/, src/, tests/)".into(),
                Some("README.md".into()),
            ));
        }

        // R6: Ecosystem Relationships Explained — optional, presence-only
        dn_total += 1;
        dn_passed += 1;
        if !(low.contains("upstream") || low.contains("downstream") || low.contains("ecosystem")) {
            findings.push(finding(
                "R6", Severity::Suggestion,
                "No ecosystem relationships mentioned — document if this repository depends on or is depended on by others".into(),
                None,
            ));
        }

        // ── Documentation Quality (R7-R10) 25% ──────────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // R7: No Documentation Duplication
        dq_total += 1;
        let duplication_signals = ["architectural decision record", "component model", "acceptance criteria", "sql schema"];
        let dup_found = find_unnegated_keywords(&low, &duplication_signals);
        if dup_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "R7", Severity::Warning,
                format!("README may duplicate detailed documentation ({}) — summarize and link instead", dup_found.join(", ")),
                Some("README.md".into()),
            ));
        }

        // R8: Links Accurate — verifying link resolution requires fetching/
        // filesystem cross-checks per link; not yet implemented.
        dq_total += 1;
        dq_passed += 1;
        findings.push(finding(
            "R8", Severity::Suggestion,
            "Link resolution checking not yet implemented — verify links manually".into(),
            None,
        ));

        // R9: README Scope Controlled
        dq_total += 1;
        let word_count = content.split_whitespace().count();
        if word_count <= 2000 {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "R9", Severity::Warning,
                format!("README is {} words — consider summarizing and linking to detailed docs instead of growing the README itself", word_count),
                Some("README.md".into()),
            ));
        }

        // R10: Installation and Quick Start Appropriate
        dq_total += 1;
        if has_heading(&["Getting Started", "Installation", "Quick Start", "Quickstart"]) {
            dq_passed += 1;
        } else if exists {
            findings.push(finding(
                "R10", Severity::Warning,
                "No Getting Started/Installation section found".into(),
                Some("README.md".into()),
            ));
        }

        // ── Maintainability (R11-R12) 15% ───────────────────────────────

        let mut mt_passed = 0u32;
        let mut mt_total = 0u32;

        // R11: Documentation Synchronization — requires comparing README
        // claims against current doc state; not yet implemented.
        mt_total += 1;
        mt_passed += 1;
        findings.push(finding(
            "R11", Severity::Suggestion,
            "Documentation synchronization check requires cross-document comparison — not yet implemented".into(),
            None,
        ));

        // R12: Future Maintainability — stub, matches honest-stub pattern
        mt_total += 1;
        mt_passed += 1;
        findings.push(finding(
            "R12", Severity::Suggestion,
            "Future maintainability assessment requires historical comparison — not yet implemented".into(),
            None,
        ));

        // ── Category Scores ──────────────────────────────────────────────

        let ri_score = if ri_total > 0 { (ri_passed as f64 / ri_total as f64) * 100.0 } else { 100.0 };
        let dn_score = if dn_total > 0 { (dn_passed as f64 / dn_total as f64) * 100.0 } else { 100.0 };
        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };
        let mt_score = if mt_total > 0 { (mt_passed as f64 / mt_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Repository Introduction".into(), ri_score);
        cat_scores.insert("Documentation Navigation".into(), dn_score);
        cat_scores.insert("Documentation Quality".into(), dq_score);
        cat_scores.insert("Maintainability".into(), mt_score);

        // Weighted overall: 30/30/25/15
        let overall = ri_score * 0.30 + dn_score * 0.30 + dq_score * 0.25 + mt_score * 0.15;

        let mut report = make_report(PipelineKind::Readme, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), if exists { "1" } else { "0" }.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, ri_score, dn_score));
        report
    }
}

fn extract_headings(content: &str) -> std::collections::HashSet<String> {
    content.lines()
        .filter(|l| l.starts_with("# ") || l.starts_with("## "))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .collect()
}

fn readiness_label(overall: f64, ri: f64, dn: f64) -> String {
    if overall >= 90.0 && ri >= 80.0 && dn >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-readme-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_readme(self, content: &str) -> Self {
            std::fs::write(self.root.join("README.md"), content).unwrap();
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
    fn r1_errors_when_no_readme() {
        let proj = TempProject::new();
        let report = ReadmePipeline.run(&proj.ctx());
        let r1 = report.findings.iter().find(|f| f.check_id == "R1").unwrap();
        assert_eq!(r1.severity, Severity::Error);
    }

    #[test]
    fn r1_passes_with_substantial_readme() {
        let proj = TempProject::new().with_readme(
            "# My Project\n\nThis project does many interesting and useful things for developers who need a reliable tool for their daily engineering work.",
        );
        let report = ReadmePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "R1"));
    }

    #[test]
    fn r10_passes_with_getting_started_heading() {
        let proj = TempProject::new().with_readme(
            "# My Project\n\nDescription here with enough words to pass the opening check easily.\n\n## Getting Started\n\nRun `install.sh`.",
        );
        let report = ReadmePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "R10"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = ReadmePipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
