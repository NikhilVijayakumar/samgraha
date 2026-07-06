use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

/// Domains that may reference External Context, per
/// `docs/raw/audit/external-context-ownership-audit.md`'s Scope section.
const REFERENCING_DOMAINS: &[&str] = &[
    "vision", "architecture", "design", "feature", "feature-design",
    "feature-technical", "engineering", "prototype",
];

pub struct ExternalContextOwnershipPipeline;

impl Pipeline for ExternalContextOwnershipPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::ExternalContextOwnership
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let ec_dir = ctx.project_root.join("docs").join("raw").join("external-context");
        let docs = if ec_dir.exists() { scan_markdown_files(&ec_dir) } else { Vec::new() };
        let doc_count = docs.len();

        let per_doc_headings: Vec<HashSet<String>> = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| extract_headings(&strip_code_fences(&c)))
            .collect();
        let found_headings: HashSet<String> = per_doc_headings.iter().flatten().cloned().collect();
        let has_heading = |names: &[&str]| {
            found_headings.iter().any(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
        };

        let ec_text: String = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let ec_low = ec_text.to_lowercase();

        let referencing_text: String = REFERENCING_DOMAINS.iter()
            .map(|d| ctx.project_root.join("docs").join("raw").join(d))
            .filter(|d| d.exists())
            .flat_map(|d| scan_markdown_files(&d))
            .filter_map(|p| fs::read_to_string(&p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let referencing_low = referencing_text.to_lowercase();

        // ── Dependency Coverage (EC1-EC4) 35% ───────────────────────────

        let mut dc_passed = 0u32;
        let mut dc_total = 0u32;

        // EC1: External Dependencies Complete — stub, exhaustive cross-repo
        // dependency discovery requires parsing every domain's technology
        // references and matching against Cargo.toml/package manifests.
        dc_total += 1;
        dc_passed += 1;
        findings.push(finding(
            "EC1", Severity::Suggestion,
            "Exhaustive external-dependency discovery across the whole documentation ecosystem requires manifest parsing and semantic matching — not yet implemented".into(),
            None,
        ));

        // EC2: One Dependency Per Document — duplicate title check
        dc_total += 1;
        if doc_count == 0 {
            dc_passed += 1;
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
                dc_passed += 1;
            } else {
                findings.push(finding(
                    "EC2", Severity::Error,
                    "Duplicate External Context document titles detected — each document must describe exactly one dependency".into(),
                    None,
                ));
            }
        }

        // EC3: Dependency Purpose Explicit
        dc_total += 1;
        if doc_count == 0 || has_heading(&["Purpose", "Overview", "Summary"]) {
            dc_passed += 1;
        } else {
            findings.push(finding(
                "EC3", Severity::Error,
                "No Purpose section found — every dependency must explain what it is, why it exists, and its architectural/engineering role".into(),
                None,
            ));
        }

        // EC4: Constraints Documented
        dc_total += 1;
        if doc_count == 0 || has_heading(&["Constraints", "Limitations", "Non-Functional Requirements"]) {
            dc_passed += 1;
        } else {
            findings.push(finding(
                "EC4", Severity::Warning,
                "No Constraints section found — document platform limitations, performance constraints, licensing, or API limitations for each dependency".into(),
                None,
            ));
        }

        // ── Documentation Integration (EC5-EC8) 35% ─────────────────────

        let mut di_passed = 0u32;
        let mut di_total = 0u32;

        // EC5: Downstream Application — stub, requires verifying that
        // referencing domains actually apply the documented constraints.
        di_total += 1;
        di_passed += 1;
        findings.push(finding(
            "EC5", Severity::Suggestion,
            "Verifying that referencing domains apply (not just mention) documented External Context constraints requires semantic comparison — not yet implemented".into(),
            None,
        ));

        // EC6: Referenced Rather Than Duplicated — stub, requires
        // semantic comparison between External Context text and referencing
        // domains' text.
        di_total += 1;
        di_passed += 1;
        findings.push(finding(
            "EC6", Severity::Suggestion,
            "Duplication-vs-reference detection across domains requires semantic comparison — not yet implemented".into(),
            None,
        ));

        // EC7: Repository Relevance
        di_total += 1;
        if doc_count == 0 || ec_low.contains("this repository") || ec_low.contains("this project") || ec_low.contains("this repo") {
            di_passed += 1;
        } else {
            di_passed += 1;
            findings.push(finding(
                "EC7", Severity::Suggestion,
                "Repository-specific relevance not obviously stated — explain why each dependency matters to this repository, not just what it is generically".into(),
                None,
            ));
        }

        // EC8: Repository Isolation
        di_total += 1;
        let leakage_keywords = ["internal architecture", "our component model", "our build configuration", "implementation detail"];
        let leakage_found = find_unnegated_keywords(&ec_low, &leakage_keywords);
        if leakage_found.is_empty() {
            di_passed += 1;
        } else {
            findings.push(finding(
                "EC8", Severity::Error,
                "Internal repository references found in External Context — these documents must describe only external systems, not this repository's own architecture or implementation".into(),
                None,
            ));
        }

        // ── Consistency (EC9-EC12) 30% ──────────────────────────────────

        let mut cs_passed = 0u32;
        let mut cs_total = 0u32;

        // EC9: Terminology Consistency
        cs_total += 1;
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
                    cs_passed += 1;
                } else {
                    findings.push(finding(
                        "EC9", Severity::Warning,
                        "Limited shared terminology across External Context documents".into(),
                        None,
                    ));
                }
            } else {
                cs_passed += 1;
            }
        } else {
            cs_passed += 1;
        }

        // EC10: Architecture Alignment
        cs_total += 1;
        let arch_dir = ctx.project_root.join("docs").join("raw").join("architecture");
        if doc_count == 0 || !arch_dir.exists() || referencing_low.contains("external context") {
            cs_passed += 1;
        } else {
            findings.push(finding(
                "EC10", Severity::Warning,
                "External Context documents exist but Architecture never references External Context — dependencies should reinforce architectural decisions, not sit unreferenced".into(),
                None,
            ));
        }

        // EC11: Engineering Alignment
        cs_total += 1;
        let eng_dir = ctx.project_root.join("docs").join("raw").join("engineering");
        if doc_count == 0 || !eng_dir.exists() {
            cs_passed += 1;
        } else {
            let eng_text: String = scan_markdown_files(&eng_dir).iter()
                .filter_map(|p| fs::read_to_string(p).ok())
                .map(|c| strip_code_fences(&c))
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase();
            if eng_text.contains("external context") {
                cs_passed += 1;
            } else {
                findings.push(finding(
                    "EC11", Severity::Warning,
                    "External Context documents exist but Engineering never references External Context — Engineering should not introduce undocumented external dependencies".into(),
                    None,
                ));
            }
        }

        // EC12: Future Maintainability
        cs_total += 1;
        if doc_count >= 2 || doc_count == 0 {
            cs_passed += 1;
        } else {
            cs_passed += 1;
            findings.push(finding(
                "EC12", Severity::Suggestion,
                "Only one External Context document exists — verify each dependency gets its own document as the collection grows".into(),
                None,
            ));
        }

        // ── Category Scores ──────────────────────────────────────────────

        let dc_score = if dc_total > 0 { (dc_passed as f64 / dc_total as f64) * 100.0 } else { 100.0 };
        let di_score = if di_total > 0 { (di_passed as f64 / di_total as f64) * 100.0 } else { 100.0 };
        let cs_score = if cs_total > 0 { (cs_passed as f64 / cs_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Dependency Coverage".into(), dc_score);
        cat_scores.insert("Documentation Integration".into(), di_score);
        cat_scores.insert("Consistency".into(), cs_score);

        // Weighted overall: 35/35/30
        let overall = dc_score * 0.35 + di_score * 0.35 + cs_score * 0.30;

        let mut report = make_report(PipelineKind::ExternalContextOwnership, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, dc_score, di_score));
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

fn readiness_label(overall: f64, dc: f64, di: f64) -> String {
    if overall >= 90.0 && dc >= 80.0 && di >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-eco-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_ec_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/external-context");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), content).unwrap();
            self
        }

        fn with_arch_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/architecture");
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
    fn ec2_errors_on_duplicate_titles() {
        let proj = TempProject::new()
            .with_ec_file("a.md", "# Stripe\n\n## Purpose\n\nPayments.")
            .with_ec_file("b.md", "# Stripe\n\nDifferent content.");
        let report = ExternalContextOwnershipPipeline.run(&proj.ctx());
        let ec2 = report.findings.iter().find(|f| f.check_id == "EC2").unwrap();
        assert_eq!(ec2.severity, Severity::Error);
    }

    #[test]
    fn ec8_errors_on_internal_leakage() {
        let proj = TempProject::new()
            .with_ec_file("stripe.md", "# Stripe\n\n## Purpose\n\nOur component model routes payments through internal architecture.");
        let report = ExternalContextOwnershipPipeline.run(&proj.ctx());
        let ec8 = report.findings.iter().find(|f| f.check_id == "EC8").unwrap();
        assert_eq!(ec8.severity, Severity::Error);
    }

    #[test]
    fn ec10_warns_when_architecture_never_references_external_context() {
        let proj = TempProject::new()
            .with_ec_file("stripe.md", "# Stripe\n\n## Purpose\n\nPayment processing for this repository.")
            .with_arch_file("system.md", "# System Overview\n\nComponents communicate via events.");
        let report = ExternalContextOwnershipPipeline.run(&proj.ctx());
        let ec10 = report.findings.iter().find(|f| f.check_id == "EC10").unwrap();
        assert_eq!(ec10.severity, Severity::Warning);
    }

    #[test]
    fn ec10_passes_when_architecture_references_external_context() {
        let proj = TempProject::new()
            .with_ec_file("stripe.md", "# Stripe\n\n## Purpose\n\nPayment processing for this repository.")
            .with_arch_file("system.md", "# System Overview\n\nSee External Context for payment provider constraints.");
        let report = ExternalContextOwnershipPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "EC10"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = ExternalContextOwnershipPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
