use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;
use std::fs;

pub struct DeterministicRuntimePipeline;

impl Pipeline for DeterministicRuntimePipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::DeterministicRuntime
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let arch_dir = ctx.project_root.join("docs").join("raw").join("architecture");
        let eng_dir = ctx.project_root.join("docs").join("raw").join("engineering");
        let arch_docs = if arch_dir.exists() { scan_markdown_files(&arch_dir) } else { Vec::new() };
        let eng_docs = if eng_dir.exists() { scan_markdown_files(&eng_dir) } else { Vec::new() };
        let doc_count = arch_docs.len() + eng_docs.len();

        let arch_text: String = arch_docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let eng_text: String = eng_docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let arch_low = arch_text.to_lowercase();
        let eng_low = eng_text.to_lowercase();
        let all_low = format!("{}\n{}", arch_low, eng_low);

        let found_headings: std::collections::HashSet<String> = arch_docs.iter().chain(eng_docs.iter())
            .filter_map(|p| fs::read_to_string(p).ok())
            .flat_map(|c| extract_headings(&strip_code_fences(&c)))
            .collect();
        let has_heading = |names: &[&str]| {
            found_headings.iter().any(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
        };

        // ── Runtime Model (S1-S4) 40% ───────────────────────────────────

        let mut rm_passed = 0u32;
        let mut rm_total = 0u32;

        // S1: Explicit Stage Contracts
        rm_total += 1;
        if doc_count == 0 || has_heading(&["Component Interactions", "Communication Paths", "Integration Points", "Interactions", "Communication"]) || all_low.contains("contract") {
            rm_passed += 1;
        } else {
            findings.push(finding(
                "S1", Severity::Warning,
                "No stage contract documentation found — pipeline stages should document inputs, outputs, ownership, and responsibilities explicitly".into(),
                None,
            ));
        }

        // S2: Communication Paths Deterministic
        rm_total += 1;
        if doc_count == 0 || all_low.contains("deterministic") {
            rm_passed += 1;
        } else {
            findings.push(finding(
                "S2", Severity::Warning,
                "No mention of deterministic communication paths — pipeline execution and sequencing should be documented as predictable and reproducible".into(),
                None,
            ));
        }

        // S3: Stateless Stage Design
        rm_total += 1;
        if doc_count == 0 || all_low.contains("stateless") {
            rm_passed += 1;
        } else {
            findings.push(finding(
                "S3", Severity::Warning,
                "No mention of stateless stage design — pipeline stages should be documented as stateless transformations".into(),
                None,
            ));
        }

        // S4: Reproducible Execution
        rm_total += 1;
        if doc_count == 0 || all_low.contains("reproducib") {
            rm_passed += 1;
        } else {
            findings.push(finding(
                "S4", Severity::Warning,
                "No mention of reproducible execution — identical inputs should be documented as producing identical outputs".into(),
                None,
            ));
        }

        // ── Engineering Principles (S5-S8) 30% ──────────────────────────

        let mut ep_passed = 0u32;
        let mut ep_total = 0u32;

        // S5: Determinism Documented
        ep_total += 1;
        if doc_count == 0 || eng_low.contains("determinis") {
            ep_passed += 1;
        } else {
            findings.push(finding(
                "S5", Severity::Warning,
                "Determinism not documented as an Engineering principle — explain why determinism matters and how it is preserved".into(),
                None,
            ));
        }

        // S6: Statelessness Rationale
        ep_total += 1;
        if doc_count == 0 || eng_low.contains("stateless") {
            ep_passed += 1;
        } else {
            findings.push(finding(
                "S6", Severity::Warning,
                "No statelessness rationale found in Engineering docs — explain the engineering benefits and trade-offs of stateless execution".into(),
                None,
            ));
        }

        // S7: Cache Strategy Documented
        ep_total += 1;
        if doc_count == 0 || eng_low.contains("cache") {
            ep_passed += 1;
        } else {
            ep_passed += 1;
            findings.push(finding(
                "S7", Severity::Suggestion,
                "No cache strategy documented — document cache ownership, invalidation, and reproducibility guarantees if caching is used".into(),
                None,
            ));
        }

        // S8: Artifact Lifecycle Defined
        ep_total += 1;
        if doc_count == 0 || eng_low.contains("artifact") {
            ep_passed += 1;
        } else {
            ep_passed += 1;
            findings.push(finding(
                "S8", Severity::Suggestion,
                "No artifact lifecycle documented — define ownership, regeneration, and disposal strategy for generated artifacts if applicable".into(),
                None,
            ));
        }

        // ── Runtime Integrity (S9-S12) 30% ──────────────────────────────

        let mut ri_passed = 0u32;
        let mut ri_total = 0u32;

        // S9: No Hidden State
        ri_total += 1;
        let hidden_state_keywords = ["global state", "hidden state", "hidden cache", "implicit configuration"];
        let hidden_state_found = find_unnegated_keywords(&all_low, &hidden_state_keywords);
        if hidden_state_found.is_empty() {
            ri_passed += 1;
        } else {
            findings.push(finding(
                "S9", Severity::Error,
                format!("Hidden runtime state referenced ({}) — all runtime state should be explicit and documented", hidden_state_found.join(", ")),
                None,
            ));
        }

        // S10: External Dependencies Controlled — stub, requires semantic
        // judgment of whether referenced external systems are deterministic
        // or optional.
        ri_total += 1;
        ri_passed += 1;
        findings.push(finding(
            "S10", Severity::Suggestion,
            "External dependency control (deterministic or optional) requires semantic judgment — verify manually against External Context".into(),
            None,
        ));

        // S11: Runtime Consistency — flag if only one of the two domains
        // documents determinism/statelessness (potential contradiction risk).
        ri_total += 1;
        let arch_mentions = arch_low.contains("determinis") || arch_low.contains("stateless");
        let eng_mentions = eng_low.contains("determinis") || eng_low.contains("stateless");
        if !arch_dir.exists() || !eng_dir.exists() || arch_mentions == eng_mentions {
            ri_passed += 1;
        } else {
            findings.push(finding(
                "S11", Severity::Warning,
                "Architecture and Engineering documentation disagree on whether the runtime model is deterministic/stateless — reconcile the two domains".into(),
                None,
            ));
        }

        // S12: Future Maintainability
        ri_total += 1;
        if doc_count >= 2 {
            ri_passed += 1;
        } else {
            ri_passed += 1;
            findings.push(finding(
                "S12", Severity::Suggestion,
                "Very little runtime documentation exists — verify new pipeline stages will integrate through existing contracts as the collection grows".into(),
                None,
            ));
        }

        // ── Category Scores ──────────────────────────────────────────────

        let rm_score = if rm_total > 0 { (rm_passed as f64 / rm_total as f64) * 100.0 } else { 100.0 };
        let ep_score = if ep_total > 0 { (ep_passed as f64 / ep_total as f64) * 100.0 } else { 100.0 };
        let ri_score = if ri_total > 0 { (ri_passed as f64 / ri_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Runtime Model".into(), rm_score);
        cat_scores.insert("Engineering Principles".into(), ep_score);
        cat_scores.insert("Runtime Integrity".into(), ri_score);

        // Weighted overall: 40/30/30
        let overall = rm_score * 0.40 + ep_score * 0.30 + ri_score * 0.30;

        let mut report = make_report(PipelineKind::DeterministicRuntime, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, ri_score));
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

fn extract_headings(content: &str) -> std::collections::HashSet<String> {
    content.lines()
        .filter(|l| l.starts_with("# ") || l.starts_with("## "))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .collect()
}

fn readiness_label(overall: f64, runtime_integrity: f64) -> String {
    if overall >= 90.0 && runtime_integrity >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-runtime-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_arch_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/architecture");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), content).unwrap();
            self
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
    fn s9_errors_on_hidden_state_reference() {
        let proj = TempProject::new()
            .with_eng_file("runtime.md", "# Runtime\n\nThe pipeline relies on global state for caching.");
        let report = DeterministicRuntimePipeline.run(&proj.ctx());
        let s9 = report.findings.iter().find(|f| f.check_id == "S9").unwrap();
        assert_eq!(s9.severity, Severity::Error);
    }

    #[test]
    fn s9_passes_without_hidden_state() {
        let proj = TempProject::new()
            .with_eng_file("runtime.md", "# Runtime\n\nEach stage is deterministic and stateless.");
        let report = DeterministicRuntimePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "S9"));
    }

    #[test]
    fn s11_warns_on_runtime_model_disagreement() {
        let proj = TempProject::new()
            .with_arch_file("pipeline.md", "# Pipeline\n\nStages are deterministic and stateless.")
            .with_eng_file("build.md", "# Build Standards\n\nCI runs tests on every commit.");
        let report = DeterministicRuntimePipeline.run(&proj.ctx());
        let s11 = report.findings.iter().find(|f| f.check_id == "S11").unwrap();
        assert_eq!(s11.severity, Severity::Warning);
    }

    #[test]
    fn s11_passes_when_both_domains_agree() {
        let proj = TempProject::new()
            .with_arch_file("pipeline.md", "# Pipeline\n\nStages are deterministic and stateless.")
            .with_eng_file("build.md", "# Build Standards\n\nDeterministic, stateless builds are required.");
        let report = DeterministicRuntimePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "S11"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = DeterministicRuntimePipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
