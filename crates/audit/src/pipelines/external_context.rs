use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct ExternalContextPipeline;

impl Pipeline for ExternalContextPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::ExternalContext
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let ec_dir = ctx.project_root.join("docs").join("raw").join("external-context");
        let docs = if ec_dir.exists() {
            scan_markdown_files(&ec_dir)
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

        // ── Document Quality (EC1-EC4) 30% ──────────────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // EC1: One Document Per Dependency — duplicate title check, same
        // pattern as architecture/design's responsibility-separation check
        dq_total += 1;
        if doc_count == 0 {
            dq_passed += 1; // nothing to check — zero dependencies is valid per the standard
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
                dq_passed += 1;
            } else {
                findings.push(finding(
                    "EC1", Severity::Error,
                    "Duplicate External Context document titles detected — each document must describe exactly one external dependency".into(),
                    None,
                ));
            }
        }

        // EC2: Dependency Necessity Justified
        dq_total += 1;
        if doc_count == 0 || low.contains("materially") || low.contains("necessary") || low.contains("project-specific") {
            dq_passed += 1;
        } else {
            dq_passed += 1;
            findings.push(finding(
                "EC2", Severity::Suggestion,
                "No explicit necessity justification found — document why each dependency requires External Context, not just widely-known libraries".into(),
                None,
            ));
        }

        // EC3: Dependency Purpose Clearly Explained
        dq_total += 1;
        if doc_count == 0 || has_heading(&["Purpose", "Overview", "Summary"]) {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "EC3", Severity::Error,
                "No Purpose section found — required by the External Context Standard".into(),
                None,
            ));
        }

        // EC4: Repository Relevance Explicit
        dq_total += 1;
        if doc_count == 0 || low.contains("repository") || low.contains("this project") || low.contains("this repo") {
            dq_passed += 1;
        } else {
            dq_passed += 1;
            findings.push(finding(
                "EC4", Severity::Suggestion,
                "Repository-specific relevance not obviously stated".into(),
                None,
            ));
        }

        // ── Content Completeness (EC5-EC7) 30% ──────────────────────────

        let mut cc_passed = 0u32;
        let mut cc_total = 0u32;

        // EC5: Constraints Documented
        cc_total += 1;
        if doc_count == 0 || has_heading(&["Constraints", "Limitations", "Non-Functional Requirements"]) {
            cc_passed += 1;
        } else {
            findings.push(finding(
                "EC5", Severity::Warning,
                "No Constraints section found — document API limitations, version compatibility, or behavioral constraints this dependency introduces".into(),
                None,
            ));
        }

        // EC6: Usage Context Explained
        cc_total += 1;
        if doc_count == 0 || low.contains("used for") || low.contains("used to") || low.contains("integration") {
            cc_passed += 1;
        } else {
            findings.push(finding(
                "EC6", Severity::Warning,
                "Usage context not clearly explained — describe which capabilities are used and which are excluded".into(),
                None,
            ));
        }

        // EC7: External Documentation Referenced
        cc_total += 1;
        if doc_count == 0 || low.contains("http://") || low.contains("https://") || has_heading(&["Integration Contract", "Contract", "API Contract", "Interface"]) {
            cc_passed += 1;
        } else {
            findings.push(finding(
                "EC7", Severity::Warning,
                "No authoritative external documentation referenced — link to the vendor's own docs rather than reproducing them".into(),
                None,
            ));
        }

        // ── Documentation Integrity (EC8-EC10) 25% ──────────────────────

        let mut di_passed = 0u32;
        let mut di_total = 0u32;

        // EC8: No Internal Architecture Leakage
        di_total += 1;
        let leakage_keywords = ["internal architecture", "our component model", "our build configuration"];
        let leakage_found = find_unnegated_keywords(&low, &leakage_keywords);
        if leakage_found.is_empty() {
            di_passed += 1;
        } else {
            findings.push(finding(
                "EC8", Severity::Error,
                "Internal architecture references found — External Context must describe the dependency, not this repository's own architecture".into(),
                None,
            ));
        }

        // EC9: No Duplication of External Documentation — proxy: very long
        // documents risk reproducing vendor docs rather than summarizing.
        di_total += 1;
        let long_docs: Vec<_> = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .filter(|c| c.split_whitespace().count() > 3000)
            .collect();
        if long_docs.is_empty() {
            di_passed += 1;
        } else {
            findings.push(finding(
                "EC9", Severity::Suggestion,
                format!("{} document(s) exceed 3000 words — verify they summarize rather than reproduce external documentation", long_docs.len()),
                None,
            ));
            di_passed += 1;
        }

        // EC10: Implementation Independence
        di_total += 1;
        let impl_keywords = ["```", "function(", "class ", "fn("];
        let impl_found = find_unnegated_keywords(&low, &impl_keywords);
        if impl_found.is_empty() {
            di_passed += 1;
        } else {
            findings.push(finding(
                "EC10", Severity::Warning,
                "Implementation code detected — External Context describes the knowledge dependency, not how it's implemented in source".into(),
                None,
            ));
        }

        // ── Collection Quality (EC11-EC12) 15% ──────────────────────────

        let mut clq_passed = 0u32;
        let mut clq_total = 0u32;

        // EC11: Collection Completeness — stub, requires cross-referencing
        // actual dependency manifests (Cargo.toml etc.) against documented ones.
        clq_total += 1;
        clq_passed += 1;
        findings.push(finding(
            "EC11", Severity::Suggestion,
            "Collection completeness requires cross-referencing dependency manifests against documented External Context — not yet implemented".into(),
            None,
        ));

        // EC12: Consistent Terminology
        clq_total += 1;
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
                    clq_passed += 1;
                } else {
                    findings.push(finding(
                        "EC12", Severity::Warning,
                        "Limited shared terminology across External Context documents".into(),
                        None,
                    ));
                }
            } else {
                clq_passed += 1;
            }
        } else {
            clq_passed += 1;
        }

        // ── Category Scores ──────────────────────────────────────────────

        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };
        let cc_score = if cc_total > 0 { (cc_passed as f64 / cc_total as f64) * 100.0 } else { 100.0 };
        let di_score = if di_total > 0 { (di_passed as f64 / di_total as f64) * 100.0 } else { 100.0 };
        let clq_score = if clq_total > 0 { (clq_passed as f64 / clq_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Document Quality".into(), dq_score);
        cat_scores.insert("Content Completeness".into(), cc_score);
        cat_scores.insert("Documentation Integrity".into(), di_score);
        cat_scores.insert("Collection Quality".into(), clq_score);

        // Weighted overall: 30/30/25/15
        let overall = dq_score * 0.30 + cc_score * 0.30 + di_score * 0.25 + clq_score * 0.15;

        let mut report = make_report(PipelineKind::ExternalContext, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, dq_score, di_score));
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

fn readiness_label(overall: f64, dq: f64, di: f64) -> String {
    if overall >= 90.0 && dq >= 80.0 && di >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-ec-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_ec_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/external-context");
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
    fn ec1_passes_with_zero_dependencies() {
        let proj = TempProject::new();
        let report = ExternalContextPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "EC1"));
    }

    #[test]
    fn ec1_errors_on_duplicate_titles() {
        let proj = TempProject::new()
            .with_ec_file("a.md", "# Stripe\n\nPurpose: payments.")
            .with_ec_file("b.md", "# Stripe\n\nDifferent content.");
        let report = ExternalContextPipeline.run(&proj.ctx());
        let ec1 = report.findings.iter().find(|f| f.check_id == "EC1").unwrap();
        assert_eq!(ec1.severity, Severity::Error);
    }

    #[test]
    fn ec3_errors_when_purpose_missing() {
        let proj = TempProject::new()
            .with_ec_file("stripe.md", "# Stripe\n\n## Constraints\n\nRate limited.");
        let report = ExternalContextPipeline.run(&proj.ctx());
        let ec3 = report.findings.iter().find(|f| f.check_id == "EC3").unwrap();
        assert_eq!(ec3.severity, Severity::Error);
    }

    #[test]
    fn ec3_passes_with_purpose_heading() {
        let proj = TempProject::new()
            .with_ec_file("stripe.md", "# Stripe\n\n## Purpose\n\nPayment processing for this repository's checkout flow.");
        let report = ExternalContextPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "EC3"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = ExternalContextPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
