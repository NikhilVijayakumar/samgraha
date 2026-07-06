use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct EngineeringPipeline;

impl Pipeline for EngineeringPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Engineering
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let eng_dir = ctx.project_root.join("docs").join("raw").join("engineering");
        let docs = if eng_dir.exists() {
            scan_markdown_files(&eng_dir)
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

        // ── Engineering Coverage (E1-E7) 40% ────────────────────────────

        let mut ec_passed = 0u32;
        let mut ec_total = 0u32;

        // E1: Engineering Principles Documented
        ec_total += 1;
        if doc_count == 0 {
            findings.push(finding(
                "E1", Severity::Error,
                "No engineering documents found — engineering principles cannot be verified".into(),
                None,
            ));
        } else if has_heading(&["Engineering Principles", "Principles", "Core Principles"]) {
            ec_passed += 1;
        } else {
            findings.push(finding(
                "E1", Severity::Error,
                "Engineering Principles section missing — required by the Engineering Standard".into(),
                None,
            ));
        }

        // E2: Repository Structure Declared — MANDATORY per the audit spec;
        // score 0 here blocks implementation-audit, so this is Error, not Warning.
        ec_total += 1;
        if has_heading(&["Repository Structure"]) {
            ec_passed += 1;
        } else {
            findings.push(finding(
                "E2", Severity::Error,
                "Repository Structure section missing — this is the authoritative source for implementation-audit; its absence blocks that audit entirely".into(),
                None,
            ));
        }

        // E3: Technology Selection Documented with Rationale
        ec_total += 1;
        if has_heading(&["Technology Selection", "Technology Choices", "Technology Rationale", "Why"]) {
            ec_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "E3", Severity::Warning,
                "No Technology Selection section found — technology choices should include rationale".into(),
                None,
            ));
        }

        // E4: Build Engineering Coverage
        ec_total += 1;
        if has_heading(&["Build Standards", "Build", "Build Process", "CI/CD"]) {
            ec_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "E4", Severity::Warning,
                "No Build Standards section found — build-audit needs this coverage to run against".into(),
                None,
            ));
        }

        // E5: Security Engineering Coverage
        ec_total += 1;
        if has_heading(&["Security Standards", "Security"]) || low.contains("secret") || low.contains("authentication") {
            ec_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "E5", Severity::Warning,
                "No security engineering coverage found — security-audit needs this coverage to run against".into(),
                None,
            ));
        }

        // E6: Runtime Engineering Coverage
        ec_total += 1;
        if low.contains("determinis") || low.contains("stateless") {
            ec_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "E6", Severity::Warning,
                "No determinism/stateless runtime coverage found — deterministic-runtime-audit needs this coverage to run against".into(),
                None,
            ));
        }

        // E7: Prototype Engineering Coverage
        ec_total += 1;
        if low.contains("prototype") {
            ec_passed += 1;
        } else {
            ec_passed += 1; // optional — this repo may not use prototypes
            findings.push(finding(
                "E7", Severity::Suggestion,
                "No prototype engineering assumptions documented — add if this repository uses prototypes".into(),
                None,
            ));
        }

        // ── Documentation Quality (E8-E10) 30% ──────────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // E8: Modular Documentation
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
                    "E8", Severity::Warning,
                    "Single engineering document with few sections — consider decomposing by engineering concern".into(),
                    Some(docs[0].to_string_lossy().to_string()),
                ));
            }
        } else {
            dq_passed += 1;
        }

        // E9: Implementation Independence
        dq_total += 1;
        let impl_keywords = ["```", "function(", "impl ", "fn(", "shell command"];
        let impl_found = find_unnegated_keywords(&low, &impl_keywords);
        if impl_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "E9", Severity::Warning,
                format!("Implementation content detected ({}) — Engineering Documentation explains decisions, not implementation itself", impl_found.join(", ")),
                None,
            ));
        }

        // E10: Responsibilities Do Not Overlap
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
                "E10", Severity::Error,
                "Duplicate engineering document titles detected — each concern must be documented once".into(),
                None,
            ));
        }

        // ── Traceability and Consistency (E11-E12) 30% ──────────────────

        let mut tc_passed = 0u32;
        let mut tc_total = 0u32;

        // E11: Architecture Alignment — stub, requires semantic comparison
        // against Architecture docs.
        tc_total += 1;
        let has_architecture_docs = ctx.project_root.join("docs/raw/architecture").exists();
        if !has_architecture_docs {
            tc_passed += 1;
        } else {
            tc_passed += 1;
            findings.push(finding(
                "E11", Severity::Suggestion,
                "Architecture documentation exists — alignment verification requires semantic comparison, not yet implemented".into(),
                None,
            ));
        }

        // E12: External Context Applied
        tc_total += 1;
        let has_external_context_docs = ctx.project_root.join("docs/raw/external-context").exists();
        if !has_external_context_docs || low.contains("external context") {
            tc_passed += 1;
        } else {
            findings.push(finding(
                "E12", Severity::Suggestion,
                "External Context documentation exists but is not referenced — reference relevant dependencies rather than duplicating them".into(),
                None,
            ));
        }

        // ── Category Scores ──────────────────────────────────────────────

        let ec_score = if ec_total > 0 { (ec_passed as f64 / ec_total as f64) * 100.0 } else { 100.0 };
        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };
        let tc_score = if tc_total > 0 { (tc_passed as f64 / tc_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Engineering Coverage".into(), ec_score);
        cat_scores.insert("Documentation Quality".into(), dq_score);
        cat_scores.insert("Traceability and Consistency".into(), tc_score);

        // Weighted overall: 40/30/30
        let overall = ec_score * 0.40 + dq_score * 0.30 + tc_score * 0.30;

        let mut report = make_report(PipelineKind::Engineering, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, ec_score));
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

fn readiness_label(overall: f64, engineering_coverage: f64) -> String {
    if overall >= 90.0 && engineering_coverage >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-eng-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_eng_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/engineering");
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
    fn e2_errors_when_repository_structure_missing() {
        let proj = TempProject::new()
            .with_eng_file("principles.md", "# Engineering Principles\n\nDeterministic builds.");
        let report = EngineeringPipeline.run(&proj.ctx());
        let e2 = report.findings.iter().find(|f| f.check_id == "E2").unwrap();
        assert_eq!(e2.severity, Severity::Error);
    }

    #[test]
    fn e2_passes_with_repository_structure_heading() {
        let proj = TempProject::new()
            .with_eng_file("structure.md", "# Repository Structure\n\nImplementation lives under `crates/`.");
        let report = EngineeringPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "E2"));
    }

    #[test]
    fn e1_errors_when_no_engineering_docs() {
        let proj = TempProject::new();
        let report = EngineeringPipeline.run(&proj.ctx());
        let e1 = report.findings.iter().find(|f| f.check_id == "E1").unwrap();
        assert_eq!(e1.severity, Severity::Error);
    }

    #[test]
    fn e9_errors_on_implementation_leakage() {
        let proj = TempProject::new()
            .with_eng_file("principles.md", "# Engineering Principles\n\nSee `impl Foo` for details on how this works.");
        let report = EngineeringPipeline.run(&proj.ctx());
        let e9 = report.findings.iter().find(|f| f.check_id == "E9").unwrap();
        assert_eq!(e9.severity, Severity::Warning);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = EngineeringPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
