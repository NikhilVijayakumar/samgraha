use crate::pipeline::{finding, load_test_report, make_report, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;

pub struct CoveragePipeline;

impl Pipeline for CoveragePipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Coverage
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        // Forward coverage (doc→code)
        let docs_base = ctx.project_root.join("docs").join("raw");
        // Repository declares its own source location — never a hardcoded `src/`.
        let src_dir = common::config::resolve_configured_dir(
            &ctx.config.repository.implementation.dir,
            &ctx.project_root,
            "src",
        );
        let build_contract = ctx.config.pipelines.as_ref().and_then(|p| p.build.as_ref());

        // CV1: Documented Features Implemented
        let feature_dir = docs_base.join("feature");
        let has_features = feature_dir.exists();
        let has_src = src_dir.exists();
        let fwd_total: u32 = 7;
        let mut fwd_passed: u32 = 0;

        if has_features && has_src {
            fwd_passed += 1;
        } else if has_features {
            // feature docs exist but no src/ to implement them
            findings.push(finding(
                "CV1", Severity::Warning,
                "Feature documentation exists but no src/ directory to verify implementation".into(),
                None,
            ));
        } else {
            fwd_passed += 1; // no features = nothing to verify
            findings.push(finding(
                "CV1", Severity::Suggestion,
                "No feature documentation found — forward coverage cannot be verified".into(),
                None,
            ));
        }

        // CV2: Architecture Components Exist
        let arch_dir = docs_base.join("architecture");
        if arch_dir.exists() && has_src {
            fwd_passed += 1;
        } else if arch_dir.exists() {
            findings.push(finding(
                "CV2", Severity::Warning,
                "Architecture docs exist but no src/ directory to verify components".into(),
                None,
            ));
        } else {
            fwd_passed += 1;
        }

        // CV3: Documented APIs Available
        findings.push(finding(
            "CV3", Severity::Suggestion,
            "API coverage scanning requires compiled knowledge base — not yet implemented".into(),
            None,
        ));

        // CV4: Documented CLI Commands Work
        findings.push(finding(
            "CV4", Severity::Suggestion,
            "CLI command coverage requires CLI schema analysis — not yet implemented".into(),
            None,
        ));

        // CV5: Documented Config Keys Accepted
        findings.push(finding(
            "CV5", Severity::Suggestion,
            "Config key coverage requires config parser analysis — not yet implemented".into(),
            None,
        ));

        // CV6: Documented Capabilities Tested — real results from the
        // repo's declared [pipelines.test] contract when available, still
        // advisory (never touches fwd_passed/fwd_total — weight 0.0 per
        // docs/raw/audit/coverage-audit.md's Severity Weighting table).
        match load_test_report(ctx) {
            None => {
                findings.push(finding(
                    "CV6", Severity::Suggestion,
                    "No [pipelines.test] contract declared (or results not produced yet) — run with `--execute` to capture real test/coverage results".into(),
                    None,
                ));
            }
            Some(Err(e)) => {
                findings.push(finding(
                    "CV6", Severity::Warning,
                    format!("[pipelines.test] declared but results unreadable: {}", e),
                    None,
                ));
            }
            Some(Ok(report)) => {
                let mut failures: Vec<String> = Vec::new();
                failures.extend(report.unit.failures.iter().map(|f| format!("unit:{}", f.name)));
                failures.extend(report.e2e.failures.iter().map(|f| format!("e2e:{}", f.name)));
                let coverage_str = report
                    .coverage_percent
                    .map(|c| format!(", coverage {:.1}%", c))
                    .unwrap_or_default();
                if failures.is_empty() {
                    findings.push(finding(
                        "CV6", Severity::Suggestion,
                        format!(
                            "Tests passing: unit {}/{}, e2e {}/{}{}",
                            report.unit.passed, report.unit.total,
                            report.e2e.passed, report.e2e.total,
                            coverage_str,
                        ),
                        None,
                    ));
                } else {
                    findings.push(finding(
                        "CV6", Severity::Warning,
                        format!(
                            "Tests failing: unit {}/{} passed, e2e {}/{} passed{} — {}",
                            report.unit.passed, report.unit.total,
                            report.e2e.passed, report.e2e.total,
                            coverage_str,
                            failures.join(", "),
                        ),
                        Some(failures.join(", ")),
                    ));
                }
            }
        }

        // CV7: Documented Build Targets Exist
        if build_contract.is_some() {
            fwd_passed += 1;
        } else {
            findings.push(finding(
                "CV7", Severity::Warning,
                "No [pipelines.build] contract declared — build targets cannot be verified".into(),
                None,
            ));
        }

        // Reverse coverage (code→doc) — orphans
        let rev_total: u32 = 8;
        let mut rev_passed: u32 = 0;

        // CV8: No Orphan Source Components
        findings.push(finding(
            "CV8", Severity::Suggestion,
            "Orphan source component detection requires doc cross-reference analysis — not yet implemented".into(),
            None,
        ));

        // CV9: No Orphan APIs
        findings.push(finding(
            "CV9", Severity::Suggestion,
            "Orphan API detection requires function signature analysis — not yet implemented".into(),
            None,
        ));

        // CV10: No Orphan CLI Commands
        findings.push(finding(
            "CV10", Severity::Suggestion,
            "Orphan CLI command detection requires CLI parser analysis — not yet implemented".into(),
            None,
        ));

        // CV11: No Orphan Config Options
        findings.push(finding(
            "CV11", Severity::Suggestion,
            "Orphan config option detection requires config schema analysis — not yet implemented".into(),
            None,
        ));

        // CV12: No Orphan Dependencies
        if build_contract.is_some() {
            findings.push(finding(
                "CV12", Severity::Suggestion,
                "Orphan dependency detection requires dependency manifest analysis — not yet implemented".into(),
                None,
            ));
        } else {
            rev_passed += 1;
        }

        // CV13: No Orphan Features
        if build_contract.is_some() {
            findings.push(finding(
                "CV13", Severity::Suggestion,
                "Orphan feature detection requires build contract analysis — not yet implemented".into(),
                None,
            ));
        } else {
            rev_passed += 1;
        }

        // CV14: No Orphan Modules
        if has_src {
            findings.push(finding(
                "CV14", Severity::Suggestion,
                "Orphan module detection requires source tree analysis — not yet implemented".into(),
                None,
            ));
        } else {
            rev_passed += 1;
        }

        // CV15: No Orphan Security Mechanisms
        findings.push(finding(
            "CV15", Severity::Suggestion,
            "Orphan security mechanism detection requires code pattern analysis — not yet implemented".into(),
            None,
        ));

        let forward_score = if fwd_total > 0 {
            (fwd_passed as f64 / fwd_total as f64) * 100.0
        } else {
            100.0
        };
        let reverse_score = if rev_total > 0 {
            (rev_passed as f64 / rev_total as f64) * 100.0
        } else {
            100.0
        };

        cat_scores.insert("Forward Coverage".into(), forward_score);
        cat_scores.insert("Reverse Coverage".into(), reverse_score);

        let overall = (forward_score + reverse_score) / 2.0;
        make_report(PipelineKind::Coverage, overall, cat_scores, findings)
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
            let root = std::env::temp_dir().join(format!("samgraha-coverage-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn ctx(&self) -> PipelineContext {
            PipelineContext::new(self.root.clone(), common::config::SamgrahaConfig::default())
        }

        fn ctx_with_build_contract(&self) -> PipelineContext {
            let mut config = common::config::SamgrahaConfig::default();
            config.pipelines = Some(common::config::PipelineContractConfig {
                version: "1.0".to_string(),
                build: Some(common::config::ContractSpec {
                    command: vec!["echo".into(), "build".into()],
                    working_directory: "${PROJECT_ROOT}".to_string(),
                    artifacts: vec![],
                    success_exit_code: None,
                    timeout: None,
                    description: None,
                    produces: vec![],
                    consumes: vec![],
                }),
                test: None,
                package: None,
                deploy: None,
            });
            PipelineContext::new(self.root.clone(), config)
        }

        fn ctx_with_implementation_dir(&self, dir: &str) -> PipelineContext {
            let mut config = common::config::SamgrahaConfig::default();
            config.repository.implementation.dir = dir.to_string();
            PipelineContext::new(self.root.clone(), config)
        }

        fn ctx_with_test_report(&self, json: &str) -> PipelineContext {
            std::fs::write(self.root.join("results.json"), json).unwrap();
            let mut config = common::config::SamgrahaConfig::default();
            config.pipelines = Some(common::config::PipelineContractConfig {
                version: "1.0".to_string(),
                build: None,
                test: Some(common::config::ContractSpec {
                    command: vec!["true".into()],
                    working_directory: "${PROJECT_ROOT}".to_string(),
                    artifacts: vec!["${PROJECT_ROOT}/results.json".to_string()],
                    success_exit_code: None,
                    timeout: None,
                    description: None,
                    produces: vec![],
                    consumes: vec![],
                }),
                package: None,
                deploy: None,
            });
            PipelineContext::new(self.root.clone(), config)
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn cv7_passes_with_declared_build_contract_no_cargo_toml() {
        // regression: no Cargo.toml in this fixture at all — proves CV7 no
        // longer requires Rust/Cargo specifically.
        let proj = TempProject::new();
        let report = CoveragePipeline.run(&proj.ctx_with_build_contract());
        assert!(!report.findings.iter().any(|f| f.check_id == "CV7"));
    }

    #[test]
    fn cv7_warns_without_declared_build_contract() {
        let proj = TempProject::new();
        let report = CoveragePipeline.run(&proj.ctx());
        let cv7 = report.findings.iter().find(|f| f.check_id == "CV7").unwrap();
        assert_eq!(cv7.severity, Severity::Warning);
    }

    #[test]
    fn source_dir_honors_declared_implementation_dir_not_hardcoded_src() {
        // regression: source lives under "crates", not "src" — a workspace
        // shape this repo itself uses. CV1 must find it via the declared
        // implementation dir, not a hardcoded "src".
        let proj = TempProject::new();
        std::fs::create_dir_all(proj.root.join("docs/raw/feature")).unwrap();
        std::fs::create_dir_all(proj.root.join("crates")).unwrap();
        let report = CoveragePipeline.run(&proj.ctx_with_implementation_dir("crates"));
        assert!(!report.findings.iter().any(|f| f.check_id == "CV1"));
    }

    #[test]
    fn cv1_passes_when_features_and_src_both_present() {
        let proj = TempProject::new();
        std::fs::create_dir_all(proj.root.join("docs/raw/feature")).unwrap();
        std::fs::create_dir_all(proj.root.join("src")).unwrap();

        let report = CoveragePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "CV1"));
    }

    #[test]
    fn cv1_warns_when_features_exist_without_src() {
        let proj = TempProject::new();
        std::fs::create_dir_all(proj.root.join("docs/raw/feature")).unwrap();

        let report = CoveragePipeline.run(&proj.ctx());
        let cv1 = report.findings.iter().find(|f| f.check_id == "CV1").unwrap();
        assert_eq!(cv1.severity, Severity::Warning);
    }

    #[test]
    fn cv6_is_suggestion_stub_without_test_contract() {
        let proj = TempProject::new();
        let report = CoveragePipeline.run(&proj.ctx());
        let cv6 = report.findings.iter().find(|f| f.check_id == "CV6").unwrap();
        assert_eq!(cv6.severity, Severity::Suggestion);
        assert!(cv6.message.contains("--execute"));
    }

    #[test]
    fn cv6_warns_on_real_test_failures() {
        // Regression: this is the original "audit says 100/0 despite real
        // test failures" complaint — CV6 used to say the same static
        // "advisory in Phase 1" message no matter what.
        let proj = TempProject::new();
        let json = r#"{"unit":{"total":3,"passed":2,"failed":1,"skipped":0,"failures":[{"name":"test_foo","message":"boom"}]},"e2e":{"total":0,"passed":0,"failed":0,"skipped":0,"failures":[]},"coverage_percent":55.0}"#;
        let report = CoveragePipeline.run(&proj.ctx_with_test_report(json));
        let cv6 = report.findings.iter().find(|f| f.check_id == "CV6").unwrap();
        assert_eq!(cv6.severity, Severity::Warning);
        assert!(cv6.message.contains("test_foo"));
        assert!(cv6.message.contains("55.0%"));
    }

    #[test]
    fn cv6_is_suggestion_when_tests_pass() {
        let proj = TempProject::new();
        let json = r#"{"unit":{"total":3,"passed":3,"failed":0,"skipped":0,"failures":[]},"e2e":{"total":1,"passed":1,"failed":0,"skipped":0,"failures":[]},"coverage_percent":90.0}"#;
        let report = CoveragePipeline.run(&proj.ctx_with_test_report(json));
        let cv6 = report.findings.iter().find(|f| f.check_id == "CV6").unwrap();
        assert_eq!(cv6.severity, Severity::Suggestion);
        assert!(cv6.message.contains("3/3"));
    }

    #[test]
    fn cv8_is_a_suggestion_stub_not_a_false_positive_warning() {
        // Regression test: CV8 used to fire Warning whenever src/ existed,
        // which is true for every real project — a guaranteed false positive,
        // not a real orphan check. It must stay a Suggestion until orphan
        // cross-reference analysis actually exists.
        let proj = TempProject::new();
        std::fs::create_dir_all(proj.root.join("src")).unwrap();

        let report = CoveragePipeline.run(&proj.ctx());
        let cv8 = report.findings.iter().find(|f| f.check_id == "CV8").unwrap();
        assert_eq!(cv8.severity, Severity::Suggestion);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = CoveragePipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
