use crate::pipeline::{finding, make_report, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;

pub struct BuildPipeline;

impl Pipeline for BuildPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Build
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();
        let mut config_passed = 0u32;
        let mut config_total = 0u32;
        let mut artifact_passed = 0u32;
        let mut artifact_total = 0u32;

        // BC1: Build Principles Realized
        config_total += 1;
        let cargo_path = ctx.project_root.join("Cargo.toml");
        if cargo_path.exists() {
            config_passed += 1;
        } else {
            findings.push(finding(
                "BC1", Severity::Error,
                "Cargo.toml not found — build principles cannot be verified".into(),
                Some(ctx.project_root.to_string_lossy().to_string()),
            ));
        }

        // BC2: Target Platforms
        config_total += 1;
        let ci_path = ctx.project_root.join(".github").join("workflows");
        if ci_path.exists() {
            config_passed += 1;
        } else {
            findings.push(finding(
                "BC2", Severity::Warning,
                "No CI workflow directory found — target platform verification limited".into(),
                Some(ctx.project_root.to_string_lossy().to_string()),
            ));
        }

        // BC3: Feature Completeness
        config_total += 1;
        if cargo_path.exists() {
            let content = std::fs::read_to_string(&cargo_path).unwrap_or_default();
            if content.contains("[features]") {
                config_passed += 1;
            } else {
                findings.push(finding(
                    "BC3", Severity::Suggestion,
                    "No [features] section in Cargo.toml".into(),
                    Some(cargo_path.to_string_lossy().to_string()),
                ));
            }
        }

        // BC4: Dependency Rationale — stub
        config_total += 1;
        findings.push(finding(
            "BC4", Severity::Suggestion,
            "Dependency rationale verification requires compiled knowledge base — not yet implemented".into(),
            None,
        ));

        // BC5: CI Platform Alignment — stub
        config_total += 1;
        findings.push(finding(
            "BC5", Severity::Suggestion,
            "CI platform alignment requires CI config parsing — not yet implemented".into(),
            None,
        ));

        // BC6: Output Completeness
        config_total += 1;
        if cargo_path.exists() {
            let content = std::fs::read_to_string(&cargo_path).unwrap_or_default();
            if content.contains("[[bin]]") {
                config_passed += 1;
            } else {
                findings.push(finding(
                    "BC6", Severity::Suggestion,
                    "No [[bin]] targets declared in Cargo.toml".into(),
                    Some(cargo_path.to_string_lossy().to_string()),
                ));
            }
        }

        // BC7: Build Config Self-Consistency — stub
        config_total += 1;
        findings.push(finding(
            "BC7", Severity::Suggestion,
            "Build config self-consistency check not yet implemented".into(),
            None,
        ));

        // BC8: External Context Applied — stub
        config_total += 1;
        findings.push(finding(
            "BC8", Severity::Suggestion,
            "External context verification not yet implemented".into(),
            None,
        ));

        // BC9: Artifact Contents Match Spec — artifact level
        if ctx.inspect_artifact {
            artifact_total += 1;
            findings.push(finding(
                "BC9", Severity::Suggestion,
                "Artifact contents inspection not yet implemented".into(),
                None,
            ));
        } else {
            artifact_total += 1;
            artifact_passed += 1; // skipped counts as pass
        }

        // BC10: Future Maintainability — config level
        config_total += 1;
        findings.push(finding(
            "BC10", Severity::Suggestion,
            "Future maintainability assessment not yet implemented".into(),
            None,
        ));

        let config_score = if config_total > 0 {
            (config_passed as f64 / config_total as f64) * 100.0
        } else {
            100.0
        };
        let artifact_score = if artifact_total > 0 {
            (artifact_passed as f64 / artifact_total as f64) * 100.0
        } else {
            100.0
        };

        cat_scores.insert("Config Checks".into(), config_score);
        cat_scores.insert("Artifact Checks".into(), artifact_score);

        let overall = (config_score + artifact_score) / 2.0;
        make_report(PipelineKind::Build, overall, cat_scores, findings)
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
            let root = std::env::temp_dir().join(format!("samgraha-build-test-{}-{}", std::process::id(), id));
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
    fn bc1_errors_when_no_cargo_toml() {
        let proj = TempProject::new();
        let report = BuildPipeline.run(&proj.ctx());
        let bc1 = report.findings.iter().find(|f| f.check_id == "BC1").unwrap();
        assert_eq!(bc1.severity, Severity::Error);
    }

    #[test]
    fn bc1_passes_when_cargo_toml_present() {
        let proj = TempProject::new();
        std::fs::write(proj.root.join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        let report = BuildPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "BC1"));
    }

    #[test]
    fn artifact_checks_skip_and_pass_when_not_inspecting() {
        let proj = TempProject::new();
        let ctx = proj.ctx(); // inspect_artifact defaults to false
        let report = BuildPipeline.run(&ctx);
        assert_eq!(report.categories.get("Artifact Checks"), Some(&100.0));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = BuildPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
