use common::config::SamgrahaConfig;
use services::KnowledgeRuntime;

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn domain_summary_has_deterministic_and_no_spec_score() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let summary = runtime.get_summary_report("domain", "vision").unwrap();

    assert_eq!(summary.target_type, "domain");
    assert_eq!(summary.target_name, "vision");
    assert!(summary.deterministic_score.is_some());
    // Spec layer belongs to pipelines only (docs/proposal.md §3) — a domain
    // summary must never have one, even for names that are also a pipeline.
    assert!(summary.spec_score.is_none());
    assert!((0.0..=100.0).contains(&summary.overall_score));
}

#[test]
fn pipeline_summary_fails_clearly_with_no_validate_script() {
    // Renamed from "...has_deterministic_and_no_standard_score" —
    // get_summary_report("pipeline", ...) computes deterministic_score via
    // run_pipeline() (runtime.rs:684), which now requires a system-provided
    // `validate` script for the "architecture" kind (the 22 hardcoded Rust
    // pipelines were deleted, no fallback, by design). No system here ships
    // one yet, so the whole call fails before standard_score/overall_score
    // are even reachable — same as run_single_check's regression test.
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let err = runtime.get_summary_report("pipeline", "architecture").unwrap_err();
    let full_chain = err.chain().map(|e| e.to_string()).collect::<Vec<_>>().join(" | ");
    assert!(
        full_chain.contains("No validate script found"),
        "expected the new missing-script error, got: {}",
        full_chain
    );
}

#[test]
fn pipeline_summary_spec_score_is_unreachable_without_a_validate_script() {
    // Renamed from "...picks_up_spec_score_once_a_check_is_judged" — this
    // test's whole point was proving spec_score updates once a check gets
    // judged, but get_summary_report("pipeline", ...) always computes
    // deterministic_score (via run_pipeline) first, unconditionally, and
    // that now fails outright with no validate script present. spec_score
    // behavior is real but currently unreachable through this call path —
    // storing the check report first doesn't change that, since
    // get_summary_report never gets past the deterministic_score step to
    // read it. Proving that honestly (both calls fail the same way) instead
    // of pretending the original scenario still applies.
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    assert!(runtime.get_summary_report("pipeline", "architecture").is_err());

    runtime
        .store_pipeline_check_report(&schemas::audit::PipelineCheckReport {
            report_id: format!("e2e-summary-{}", uuid::Uuid::new_v4()),
            pipeline: "architecture".into(),
            check_id: "A1".into(),
            score: 100,
            findings: vec![],
            git_revision: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();

    // Storing the check report doesn't unblock it — deterministic_score is
    // still computed first, unconditionally, and still has no script.
    assert!(runtime.get_summary_report("pipeline", "architecture").is_err());
}

#[test]
fn doc_is_rejected_as_a_pipeline_target() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let err = runtime.get_summary_report("pipeline", "doc").unwrap_err();
    assert!(err.to_string().contains("not a real pipeline"));
}

#[test]
fn unknown_target_type_is_rejected() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let err = runtime.get_summary_report("nonsense", "vision").unwrap_err();
    assert!(err.to_string().contains("Unknown target_type"));
}
