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

        let build_contract = ctx.config.pipelines.as_ref().and_then(|p| p.build.as_ref());

        // BC1: Build Principles Realized
        config_total += 1;
        if build_contract.is_some() {
            config_passed += 1;
        } else {
            findings.push(finding(
                "BC1", Severity::Error,
                "No [pipelines.build] contract declared in samgraha.toml — build principles cannot be verified".into(),
                Some(ctx.project_root.to_string_lossy().to_string()),
            ));
        }

        // BC2: Target Platforms — CI presence, checked generically (not tied to one CI vendor)
        config_total += 1;
        let has_ci = [".github/workflows", ".gitlab-ci.yml", ".circleci/config.yml", "azure-pipelines.yml"]
            .iter()
            .any(|p| ctx.project_root.join(p).exists());
        if has_ci {
            config_passed += 1;
        } else {
            findings.push(finding(
                "BC2", Severity::Warning,
                "No recognized CI configuration found — target platform verification limited".into(),
                Some(ctx.project_root.to_string_lossy().to_string()),
            ));
        }

        // BC3: Feature Completeness — declared `produces` on the build contract,
        // not any single build system's own feature-flag syntax
        config_total += 1;
        match build_contract {
            Some(c) if !c.produces.is_empty() => config_passed += 1,
            Some(_) => findings.push(finding(
                "BC3", Severity::Suggestion,
                "[pipelines.build] declares no `produces` — feature completeness cannot be verified".into(),
                None,
            )),
            None => {} // already covered by BC1
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

        // BC6: Output Completeness — declared `artifacts` on the build contract exist on disk
        config_total += 1;
        match build_contract.map(|c| (c, c.resolve(&ctx.project_root))) {
            Some((contract, Ok(resolved))) if ctx.dry_run => {
                config_passed += 1; // dry-run is informational, not a failure
                findings.push(finding(
                    "BC6", Severity::Suggestion,
                    format!("Dry run — would execute: {}", resolved.command.join(" ")),
                    None,
                ));
                let _ = contract; // command already captured above
            }
            Some((contract, Ok(resolved))) if ctx.execute => {
                let timeout_secs = contract
                    .timeout
                    .as_deref()
                    .and_then(common::config::parse_ttl_duration);
                match crate::contract::ContractRunner::execute(&resolved, &ctx.project_root, timeout_secs) {
                    Ok(result) if !result.matched_expected_exit_code => {
                        findings.push(finding(
                            "BC6", Severity::Warning,
                            format!("Build command exited {} (expected {})", result.exit_code, resolved.success_exit_code),
                            None,
                        ));
                    }
                    Ok(_) => {
                        let missing: Vec<_> = resolved.artifacts.iter().filter(|a| !a.exists()).collect();
                        if missing.is_empty() {
                            config_passed += 1;
                        } else {
                            for m in missing {
                                findings.push(finding(
                                    "BC6", Severity::Suggestion,
                                    format!("Declared artifact not found after build: {}", m.display()),
                                    Some(m.to_string_lossy().to_string()),
                                ));
                            }
                        }
                    }
                    Err(e) => findings.push(finding(
                        "BC6", Severity::Error,
                        format!("Build execution failed: {}", e),
                        None,
                    )),
                }
            }
            Some((_, Ok(resolved))) if resolved.artifacts.is_empty() => {
                findings.push(finding(
                    "BC6", Severity::Suggestion,
                    "[pipelines.build] declares no `artifacts` — output completeness cannot be verified".into(),
                    None,
                ));
            }
            Some((_, Ok(resolved))) => {
                let missing: Vec<_> = resolved.artifacts.iter().filter(|a| !a.exists()).collect();
                if missing.is_empty() {
                    config_passed += 1;
                } else {
                    for m in missing {
                        findings.push(finding(
                            "BC6", Severity::Suggestion,
                            format!("Declared artifact not found: {}", m.display()),
                            Some(m.to_string_lossy().to_string()),
                        ));
                    }
                }
            }
            Some((_, Err(e))) => findings.push(finding(
                "BC6", Severity::Warning,
                format!("[pipelines.build] artifacts could not be resolved: {}", e),
                None,
            )),
            None => {} // already covered by BC1
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

        fn ctx_with_build_contract(&self, contract: common::config::ContractSpec) -> PipelineContext {
            let mut config = common::config::SamgrahaConfig::default();
            config.pipelines = Some(common::config::PipelineContractConfig {
                version: "1.0".to_string(),
                build: Some(contract),
                test: None,
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

    fn bare_contract() -> common::config::ContractSpec {
        common::config::ContractSpec {
            command: vec!["echo".into(), "build".into()],
            working_directory: "${PROJECT_ROOT}".to_string(),
            artifacts: vec![],
            success_exit_code: None,
            timeout: None,
            description: None,
            produces: vec![],
            consumes: vec![],
        }
    }

    #[test]
    fn bc1_errors_when_no_pipelines_build_contract_declared() {
        let proj = TempProject::new();
        let report = BuildPipeline.run(&proj.ctx());
        let bc1 = report.findings.iter().find(|f| f.check_id == "BC1").unwrap();
        assert_eq!(bc1.severity, Severity::Error);
    }

    #[test]
    fn bc1_passes_when_pipelines_build_contract_declared() {
        // regression: no Cargo.toml anywhere in this fixture — proves BuildPipeline
        // no longer requires Rust/Cargo specifically, any declared contract works.
        let proj = TempProject::new();
        let report = BuildPipeline.run(&proj.ctx_with_build_contract(bare_contract()));
        assert!(!report.findings.iter().any(|f| f.check_id == "BC1"));
    }

    #[test]
    fn bc3_passes_when_contract_declares_produces() {
        let proj = TempProject::new();
        let mut contract = bare_contract();
        contract.produces = vec!["release binary".into()];
        let report = proj_bc3(&proj, contract);
        assert!(!report.findings.iter().any(|f| f.check_id == "BC3"));
    }

    #[test]
    fn bc3_suggests_when_contract_declares_no_produces() {
        let proj = TempProject::new();
        let report = proj_bc3(&proj, bare_contract());
        let bc3 = report.findings.iter().find(|f| f.check_id == "BC3").unwrap();
        assert_eq!(bc3.severity, Severity::Suggestion);
    }

    fn proj_bc3(proj: &TempProject, contract: common::config::ContractSpec) -> schemas::audit::PipelineReport {
        BuildPipeline.run(&proj.ctx_with_build_contract(contract))
    }

    #[test]
    fn bc6_passes_when_declared_artifact_exists() {
        let proj = TempProject::new();
        std::fs::write(proj.root.join("out.bin"), b"x").unwrap();
        let mut contract = bare_contract();
        contract.artifacts = vec!["${PROJECT_ROOT}/out.bin".to_string()];
        let report = BuildPipeline.run(&proj.ctx_with_build_contract(contract));
        assert!(!report.findings.iter().any(|f| f.check_id == "BC6"));
    }

    #[test]
    fn bc6_warns_when_declared_artifact_missing() {
        let proj = TempProject::new();
        let mut contract = bare_contract();
        contract.artifacts = vec!["${PROJECT_ROOT}/missing.bin".to_string()];
        let report = BuildPipeline.run(&proj.ctx_with_build_contract(contract));
        assert!(report.findings.iter().any(|f| f.check_id == "BC6"));
    }

    #[test]
    fn bc6_dry_run_reports_command_without_running_it() {
        let proj = TempProject::new();
        let mut contract = bare_contract();
        contract.artifacts = vec!["${PROJECT_ROOT}/out.bin".to_string()];
        contract.command = vec!["some-marker-that-would-fail-if-run".to_string()];
        let ctx = proj.ctx_with_build_contract(contract).with_dry_run(true);
        let report = BuildPipeline.run(&ctx);
        let bc6 = report.findings.iter().find(|f| f.check_id == "BC6").unwrap();
        assert!(bc6.message.contains("Dry run"));
        assert!(!proj.root.join("out.bin").exists()); // nothing was executed
    }

    #[test]
    fn bc6_execute_runs_command_and_verifies_fresh_artifact() {
        let proj = TempProject::new();
        let mut contract = bare_contract();
        // portable "create the declared artifact" command
        let (cmd, args): (&str, &[&str]) = if cfg!(windows) {
            ("cmd", &["/C", "echo x > out.bin"])
        } else {
            ("sh", &["-c", "echo x > out.bin"])
        };
        contract.command = std::iter::once(cmd.to_string())
            .chain(args.iter().map(|s| s.to_string()))
            .collect();
        contract.artifacts = vec!["${PROJECT_ROOT}/out.bin".to_string()];
        let ctx = proj.ctx_with_build_contract(contract).with_execute(true);
        let report = BuildPipeline.run(&ctx);
        assert!(!report.findings.iter().any(|f| f.check_id == "BC6"));
        assert!(proj.root.join("out.bin").exists());
    }

    #[test]
    fn bc6_execute_refuses_contract_escaping_project_root() {
        let proj = TempProject::new();
        let mut contract = bare_contract();
        contract.working_directory = "${PROJECT_ROOT}/../".to_string();
        let ctx = proj.ctx_with_build_contract(contract).with_execute(true);
        let report = BuildPipeline.run(&ctx);
        let bc6 = report.findings.iter().find(|f| f.check_id == "BC6").unwrap();
        assert_eq!(bc6.severity, Severity::Error);
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
