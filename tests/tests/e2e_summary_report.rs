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
fn pipeline_summary_has_deterministic_and_no_standard_score() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let summary = runtime.get_summary_report("pipeline", "architecture").unwrap();

    assert_eq!(summary.target_type, "pipeline");
    assert_eq!(summary.target_name, "architecture");
    assert!(summary.deterministic_score.is_some());
    // Standard (rubric) layer is per-section domain content, not a thing a
    // pipeline has (docs/proposal.md §2's matrix).
    assert!(summary.standard_score.is_none());
    assert!((0.0..=100.0).contains(&summary.overall_score));
}

#[test]
fn pipeline_summary_picks_up_spec_score_once_a_check_is_judged() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let before = runtime.get_summary_report("pipeline", "architecture").unwrap();

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

    let after = runtime.get_summary_report("pipeline", "architecture").unwrap();
    assert!(before.spec_score.is_none() || after.spec_score.is_some());
    assert_eq!(after.spec_score, Some(100.0));
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
