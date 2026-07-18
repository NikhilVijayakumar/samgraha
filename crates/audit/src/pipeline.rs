use schemas::audit::{PipelineKind, PipelineReport};
use schemas::TestRunReport;
use std::collections::HashMap;

pub trait Pipeline {
    fn name(&self) -> PipelineKind;
    fn run(&self, ctx: &PipelineContext) -> PipelineReport;
}

pub struct PipelineContext {
    pub project_root: std::path::PathBuf,
    pub config: common::config::SamgrahaConfig,
    pub inspect_artifact: bool,
    pub runtime_mode: bool,
    /// Run the declared Pipeline Contract (`[pipelines.build]`) instead of
    /// verify-only checking pre-existing artifacts. Build Audit only.
    pub execute: bool,
    /// Print the resolved command without running it. Build Audit only.
    pub dry_run: bool,
    /// The `repository_metadata` table's contents, if the caller has
    /// registry access to fetch it (`Pipeline::run` itself doesn't — no
    /// registry/DB handle is threaded through the trait). Empty unless a
    /// caller opts in via `with_repository_metadata`. Product Guide Audit's
    /// PA7 is currently the only consumer.
    pub repository_metadata: HashMap<String, String>,
}

impl PipelineContext {
    pub fn new(
        project_root: std::path::PathBuf,
        config: common::config::SamgrahaConfig,
    ) -> Self {
        Self {
            project_root,
            config,
            inspect_artifact: false,
            runtime_mode: false,
            execute: false,
            dry_run: false,
            repository_metadata: HashMap::new(),
        }
    }

    pub fn with_repository_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.repository_metadata = metadata;
        self
    }

    pub fn with_inspect_artifact(mut self, val: bool) -> Self {
        self.inspect_artifact = val;
        self
    }

    pub fn with_runtime(mut self, val: bool) -> Self {
        self.runtime_mode = val;
        self
    }

    pub fn with_execute(mut self, val: bool) -> Self {
        self.execute = val;
        self
    }

    pub fn with_dry_run(mut self, val: bool) -> Self {
        self.dry_run = val;
        self
    }
}

pub struct PipelineStage {
    pub name: String,
    pub check_ids: Vec<String>,
}

/// Loads real test/coverage results from a repo's declared `[pipelines.test]`
/// contract — shared by Coverage (CV6) and Implementation (I8) so the
/// contract-execution/JSON-parsing logic lives in exactly one place.
///
/// `None` means "nothing to report yet": no `[pipelines.test]` declared,
/// `ctx.dry_run` is set (informational-only — the calling pipeline handles
/// that message itself, same split as build.rs's BC6), or the results
/// artifact simply doesn't exist yet and `ctx.execute` wasn't passed to
/// produce one. `Some(Err(_))` means a contract exists but results couldn't
/// be obtained (execution failed, artifact missing after running, or
/// unparseable) — that's worth a finding, unlike the "not adopted yet" case.
pub fn load_test_report(ctx: &PipelineContext) -> Option<Result<TestRunReport, String>> {
    let contract = ctx.config.pipelines.as_ref().and_then(|p| p.test.as_ref())?;
    if ctx.dry_run {
        return None;
    }

    let resolved = match contract.resolve(&ctx.project_root) {
        Ok(r) => r,
        Err(e) => return Some(Err(format!("[pipelines.test] could not be resolved: {e}"))),
    };
    let Some(results_path) = resolved.artifacts.first() else {
        return Some(Err(
            "[pipelines.test] declares no `artifacts` — nowhere to read results from".into(),
        ));
    };

    if ctx.execute {
        let timeout_secs = contract
            .timeout
            .as_deref()
            .and_then(common::config::parse_ttl_duration);
        if let Err(e) = crate::contract::ContractRunner::execute(&resolved, &ctx.project_root, timeout_secs) {
            // A nonzero exit code alone isn't an error here — failing tests
            // are an expected, normal outcome the script should still have
            // written a results JSON for. Only spawn/timeout/escape failures
            // (the only things ContractRunner::execute itself returns Err
            // for) short-circuit before we even try reading the artifact.
            return Some(Err(format!("Test command execution failed: {e}")));
        }
    } else if !results_path.exists() {
        return None; // declared but never run — not adopted yet, not an error
    }

    let content = match std::fs::read_to_string(results_path) {
        Ok(c) => c,
        Err(e) => return Some(Err(format!("Could not read {}: {}", results_path.display(), e))),
    };
    // Strip a leading UTF-8 BOM — several common script runtimes (Windows
    // PowerShell 5.1's `Set-Content -Encoding utf8` among them) write one by
    // default, and serde_json treats it as invalid leading bytes rather than
    // skipping it.
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content);
    match serde_json::from_str::<TestRunReport>(content) {
        Ok(report) => Some(Ok(report)),
        Err(e) => Some(Err(format!(
            "Could not parse {} as test results: {}",
            results_path.display(),
            e
        ))),
    }
}

#[cfg(test)]
mod load_test_report_tests {
    use super::*;
    use common::config::{ContractSpec, PipelineContractConfig, SamgrahaConfig};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("samgraha-load-test-report-{}-{}", std::process::id(), id));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn noop_command() -> Vec<String> {
        if cfg!(windows) {
            vec!["cmd".into(), "/C".into(), "exit".into(), "0".into()]
        } else {
            vec!["true".into()]
        }
    }

    fn config_with_test_contract(artifact_rel: &str) -> SamgrahaConfig {
        let mut config = SamgrahaConfig::default();
        config.pipelines = Some(PipelineContractConfig {
            version: "1.0".to_string(),
            build: None,
            test: Some(ContractSpec {
                command: noop_command(),
                working_directory: "${PROJECT_ROOT}".to_string(),
                artifacts: vec![format!("${{PROJECT_ROOT}}/{}", artifact_rel)],
                success_exit_code: None,
                timeout: None,
                description: None,
                produces: vec![],
                consumes: vec![],
            }),
            package: None,
            deploy: None,
        });
        config
    }

    fn valid_report_json() -> &'static str {
        r#"{"unit":{"total":3,"passed":2,"failed":1,"skipped":0,"failures":[{"name":"test_foo","message":"boom"}]},"e2e":{"total":1,"passed":1,"failed":0,"skipped":0,"failures":[]},"coverage_percent":80.0}"#
    }

    #[test]
    fn no_contract_declared_is_none() {
        let root = temp_root();
        let ctx = PipelineContext::new(root.clone(), SamgrahaConfig::default());
        assert!(load_test_report(&ctx).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn dry_run_is_none_even_with_contract() {
        let root = temp_root();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config).with_dry_run(true);
        assert!(load_test_report(&ctx).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn declared_but_never_run_and_no_artifact_is_none() {
        let root = temp_root();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        assert!(load_test_report(&ctx).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn preexisting_artifact_is_read_without_execute() {
        let root = temp_root();
        std::fs::write(root.join("results.json"), valid_report_json()).unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        let report = load_test_report(&ctx).unwrap().unwrap();
        assert_eq!(report.unit.failed, 1);
        assert_eq!(report.coverage_percent, Some(80.0));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn execute_runs_command_then_reads_artifact() {
        let root = temp_root();
        // Simulates a script that already wrote its results — the noop
        // command just proves the execute path re-reads after running.
        std::fs::write(root.join("results.json"), valid_report_json()).unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config).with_execute(true);
        let report = load_test_report(&ctx).unwrap().unwrap();
        assert_eq!(report.unit.total, 3);
        assert_eq!(report.e2e.passed, 1);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn utf8_bom_prefixed_artifact_still_parses() {
        // Regression: Windows PowerShell 5.1's `Set-Content -Encoding utf8`
        // writes a UTF-8 BOM by default (this repo's own scripts/test-coverage.ps1
        // hit exactly this before switching to a BOM-less writer).
        let root = temp_root();
        let mut bytes = vec![0xEFu8, 0xBB, 0xBF];
        bytes.extend_from_slice(valid_report_json().as_bytes());
        std::fs::write(root.join("results.json"), bytes).unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        let report = load_test_report(&ctx).unwrap().unwrap();
        assert_eq!(report.unit.total, 3);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn malformed_artifact_is_some_err() {
        let root = temp_root();
        std::fs::write(root.join("results.json"), "not json").unwrap();
        let config = config_with_test_contract("results.json");
        let ctx = PipelineContext::new(root.clone(), config);
        assert!(load_test_report(&ctx).unwrap().is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn no_artifacts_declared_is_some_err() {
        let root = temp_root();
        let mut config = config_with_test_contract("results.json");
        config.pipelines.as_mut().unwrap().test.as_mut().unwrap().artifacts = vec![];
        let ctx = PipelineContext::new(root.clone(), config).with_execute(true);
        assert!(load_test_report(&ctx).unwrap().is_err());
        std::fs::remove_dir_all(&root).ok();
    }
}


