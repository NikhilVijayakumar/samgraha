use crate::pipeline::{finding, make_report, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;

pub struct ConsistencyPipeline;

impl Pipeline for ConsistencyPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Consistency
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();
        let mut alignment_passed = 0u32;
        let mut alignment_total = 0u32;
        let mut integrity_passed = 0u32;
        let mut integrity_total = 0u32;

        // --- Layer Alignment (C1-C7) ---

        // C1: Vision→Architecture Alignment
        alignment_total += 1;
        let vision_dir = ctx.project_root.join("docs").join("raw").join("vision");
        let arch_dir = ctx.project_root.join("docs").join("raw").join("architecture");
        if vision_dir.exists() && arch_dir.exists() {
            alignment_passed += 1;
        } else if vision_dir.exists() {
            findings.push(finding(
                "C1", Severity::Warning,
                "Vision docs exist but no Architecture docs to verify alignment".into(),
                None,
            ));
        } else {
            alignment_passed += 1; // no vision = nothing to align
        }

        // C2: Architecture→Feature Alignment
        alignment_total += 1;
        let feature_dir = ctx.project_root.join("docs").join("raw").join("feature");
        if arch_dir.exists() && feature_dir.exists() {
            alignment_passed += 1;
        } else {
            findings.push(finding(
                "C2", Severity::Suggestion,
                "Architecture→Feature alignment requires both directories".into(),
                None,
            ));
        }

        // C3: Feature→Feature Technical Alignment
        alignment_total += 1;
        let ft_dir = ctx.project_root.join("docs").join("raw").join("feature-technical");
        if feature_dir.exists() && ft_dir.exists() {
            alignment_passed += 1;
        } else {
            findings.push(finding(
                "C3", Severity::Suggestion,
                "Feature→Feature Technical alignment requires both directories".into(),
                None,
            ));
        }

        // C4: Feature Technical→Engineering Alignment
        alignment_total += 1;
        let eng_dir = ctx.project_root.join("docs").join("raw").join("engineering");
        if ft_dir.exists() && eng_dir.exists() {
            alignment_passed += 1;
        } else {
            findings.push(finding(
                "C4", Severity::Suggestion,
                "Feature Technical→Engineering alignment requires both directories".into(),
                None,
            ));
        }

        // C5: Engineering→Implementation Alignment — repository declares its
        // own source location, never a hardcoded `src/`
        alignment_total += 1;
        let src_dir = common::config::resolve_configured_dir(
            &ctx.config.repository.implementation.dir,
            &ctx.project_root,
            "src",
        );
        if eng_dir.exists() && src_dir.exists() {
            alignment_passed += 1;
        } else {
            findings.push(finding(
                "C5", Severity::Suggestion,
                "Engineering→Implementation alignment requires both docs and the declared implementation directory".into(),
                None,
            ));
        }

        // C6: Build→Implementation Alignment (stub — needs cross-ref with Build Audit)
        alignment_total += 1;
        findings.push(finding(
            "C6", Severity::Suggestion,
            "Build→Implementation alignment requires Build Audit evidence — stub".into(),
            None,
        ));

        // C7: Security→Implementation Alignment (stub)
        alignment_total += 1;
        findings.push(finding(
            "C7", Severity::Suggestion,
            "Security→Implementation alignment requires Security Audit evidence — stub".into(),
            None,
        ));

        // --- Cross-Layer Integrity (C8-C12) ---

        // C8: No Layer Skip
        integrity_total += 1;
        findings.push(finding(
            "C8", Severity::Suggestion,
            "Layer skip detection requires document cross-reference analysis — not yet implemented".into(),
            None,
        ));

        // C9: Cross-Document Terminology Consistency
        integrity_total += 1;
        findings.push(finding(
            "C9", Severity::Suggestion,
            "Terminology consistency check requires NLP or keyword extraction — not yet implemented".into(),
            None,
        ));

        // C10: Constraint Propagation
        integrity_total += 1;
        findings.push(finding(
            "C10", Severity::Suggestion,
            "Constraint propagation verification not yet implemented".into(),
            None,
        ));

        // C11: No Contradiction
        integrity_total += 1;
        findings.push(finding(
            "C11", Severity::Suggestion,
            "Contradiction detection not yet implemented".into(),
            None,
        ));

        // C12: Traceability Complete
        integrity_total += 1;
        if vision_dir.exists() || arch_dir.exists() || feature_dir.exists() {
            integrity_passed += 1;
        } else {
            findings.push(finding(
                "C12", Severity::Warning,
                "No documentation layers found — traceability cannot be verified".into(),
                None,
            ));
        }

        let alignment_score = if alignment_total > 0 {
            (alignment_passed as f64 / alignment_total as f64) * 100.0
        } else {
            100.0
        };
        let integrity_score = if integrity_total > 0 {
            (integrity_passed as f64 / integrity_total as f64) * 100.0
        } else {
            100.0
        };

        cat_scores.insert("Layer Alignment".into(), alignment_score);
        cat_scores.insert("Cross-Layer Integrity".into(), integrity_score);

        let overall = (alignment_score + integrity_score) / 2.0;
        make_report(PipelineKind::Consistency, overall, cat_scores, findings)
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
            let root = std::env::temp_dir().join(format!("samgraha-consistency-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
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
    fn c1_warns_when_vision_exists_without_architecture() {
        let proj = TempProject::new();
        std::fs::create_dir_all(proj.root.join("docs/raw/vision")).unwrap();
        let report = ConsistencyPipeline.run(&proj.ctx());
        let c1 = report.findings.iter().find(|f| f.check_id == "C1").unwrap();
        assert_eq!(c1.severity, Severity::Warning);
    }

    #[test]
    fn c1_passes_when_vision_and_architecture_both_present() {
        let proj = TempProject::new();
        std::fs::create_dir_all(proj.root.join("docs/raw/vision")).unwrap();
        std::fs::create_dir_all(proj.root.join("docs/raw/architecture")).unwrap();
        let report = ConsistencyPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "C1"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = ConsistencyPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
