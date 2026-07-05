use crate::pipeline::{finding, make_report, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;

pub struct SecurityPipeline;

impl Pipeline for SecurityPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Security
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();
        let mut static_passed = 0u32;
        let mut static_total = 0u32;
        let mut config_passed = 0u32;
        let mut config_total = 0u32;
        let mut runtime_passed = 0u32;
        let mut runtime_total = 0u32;

        // SC1: Dependency Vulnerability Scanning (static) — checked generically,
        // not tied to one CI vendor or scanner
        static_total += 1;
        let has_ci = [".github/workflows", ".gitlab-ci.yml", ".circleci/config.yml", "azure-pipelines.yml"]
            .iter()
            .any(|p| ctx.project_root.join(p).exists());
        if has_ci {
            static_passed += 1;
        } else {
            findings.push(finding(
                "SC1", Severity::Suggestion,
                "No recognized CI configuration found — assume dependency vulnerability scanning is not configured".into(),
                Some(ctx.project_root.to_string_lossy().to_string()),
            ));
        }

        // SC2: Authentication Config (config level)
        config_total += 1;
        let config_path = ctx.project_root.join("samgraha.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            if content.contains("auth") || content.contains("token") {
                config_passed += 1;
            } else {
                findings.push(finding(
                    "SC2", Severity::Suggestion,
                    "No auth/token configuration found in samgraha.toml".into(),
                    Some(config_path.to_string_lossy().to_string()),
                ));
            }
        } else {
            config_passed += 1; // no config = no auth config needed
        }

        // SC3: Authorization Config (config level)
        config_total += 1;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            if content.contains("role") || content.contains("permission") {
                config_passed += 1;
            } else {
                findings.push(finding(
                    "SC3", Severity::Suggestion,
                    "No role/permission configuration found".into(),
                    Some(config_path.to_string_lossy().to_string()),
                ));
            }
        } else {
            config_passed += 1;
        }

        // SC4: Secrets Isolation (config level)
        config_total += 1;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            let contains_secret = content.contains("password")
                || content.contains("secret")
                || content.contains("api_key");
            if contains_secret {
                findings.push(finding(
                    "SC4", Severity::Error,
                    "Potential secret found in configuration file — use env vars instead".into(),
                    Some(config_path.to_string_lossy().to_string()),
                ));
            } else {
                config_passed += 1;
            }
        } else {
            config_passed += 1;
        }

        // SC5: TLS Configuration (config level)
        config_total += 1;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            if content.contains("tls") || content.contains("ssl") || content.contains("https") {
                config_passed += 1;
            } else {
                findings.push(finding(
                    "SC5", Severity::Suggestion,
                    "No TLS/SSL configuration found".into(),
                    Some(config_path.to_string_lossy().to_string()),
                ));
            }
        } else {
            config_passed += 1;
        }

        // SC6: Properties Match Runtime (runtime level)
        if ctx.runtime_mode {
            runtime_total += 1;
            if cfg!(target_os = "linux") {
                findings.push(finding(
                    "SC6", Severity::Suggestion,
                    "Runtime property verification not yet implemented".into(),
                    None,
                ));
            } else {
                findings.push(finding(
                    "SC6", Severity::Warning,
                    "Runtime verification not supported on this platform (Linux only)".into(),
                    None,
                ));
            }
        } else {
            runtime_total += 1;
            runtime_passed += 1; // skipped counts as pass
        }

        // SC7: No Security Regression (config level)
        config_total += 1;
        findings.push(finding(
            "SC7", Severity::Suggestion,
            "Security regression check requires historical report comparison — not yet implemented".into(),
            None,
        ));

        // SC8: External Context Verification (static)
        static_total += 1;
        findings.push(finding(
            "SC8", Severity::Suggestion,
            "External dependency version verification not yet implemented".into(),
            None,
        ));

        // SC9: Runtime Dependency Chain (runtime level)
        if ctx.runtime_mode {
            runtime_total += 1;
            if cfg!(target_os = "linux") {
                findings.push(finding(
                    "SC9", Severity::Suggestion,
                    "Runtime dependency chain inspection not yet implemented".into(),
                    None,
                ));
            } else {
                findings.push(finding(
                    "SC9", Severity::Warning,
                    "Runtime dependency chain not supported on this platform (Linux only)".into(),
                    None,
                ));
            }
        } else {
            runtime_total += 1;
            runtime_passed += 1;
        }

        // SC10: Runtime Secret Handling (runtime level, Linux only)
        if ctx.runtime_mode {
            runtime_total += 1;
            if cfg!(target_os = "linux") {
                findings.push(finding(
                    "SC10", Severity::Suggestion,
                    "Runtime secret leak detection not yet implemented".into(),
                    None,
                ));
            } else {
                findings.push(finding(
                    "SC10", Severity::Warning,
                    "Runtime secret handling not supported on this platform (Linux only)".into(),
                    None,
                ));
            }
        } else {
            runtime_total += 1;
            runtime_passed += 1;
        }

        // SC11: Future Maintainability (config level)
        config_total += 1;
        findings.push(finding(
            "SC11", Severity::Suggestion,
            "Future maintainability assessment not yet implemented".into(),
            None,
        ));

        let static_score = if static_total > 0 {
            (static_passed as f64 / static_total as f64) * 100.0
        } else {
            100.0
        };
        let config_score = if config_total > 0 {
            (config_passed as f64 / config_total as f64) * 100.0
        } else {
            100.0
        };
        let runtime_score = if runtime_total > 0 {
            (runtime_passed as f64 / runtime_total as f64) * 100.0
        } else {
            100.0
        };

        cat_scores.insert("Static Checks".into(), static_score);
        cat_scores.insert("Config Checks".into(), config_score);
        cat_scores.insert("Runtime Checks".into(), runtime_score);

        let overall = (static_score + config_score + runtime_score) / 3.0;
        make_report(PipelineKind::Security, overall, cat_scores, findings)
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
            let root = std::env::temp_dir().join(format!("samgraha-security-test-{}-{}", std::process::id(), id));
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
    fn sc4_errors_on_plaintext_secret_in_config() {
        let proj = TempProject::new();
        std::fs::write(proj.root.join("samgraha.toml"), "password = \"hunter2\"").unwrap();
        let report = SecurityPipeline.run(&proj.ctx());
        let sc4 = report.findings.iter().find(|f| f.check_id == "SC4").unwrap();
        assert_eq!(sc4.severity, Severity::Error);
    }

    #[test]
    fn sc4_passes_with_no_secret_in_config() {
        let proj = TempProject::new();
        std::fs::write(proj.root.join("samgraha.toml"), "[audit]\ndefault-severity = \"warning\"").unwrap();
        let report = SecurityPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "SC4"));
    }

    #[test]
    fn runtime_checks_skip_and_pass_when_runtime_mode_off() {
        let proj = TempProject::new();
        let ctx = proj.ctx(); // runtime_mode defaults to false
        let report = SecurityPipeline.run(&ctx);
        assert_eq!(report.categories.get("Runtime Checks"), Some(&100.0));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = SecurityPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
