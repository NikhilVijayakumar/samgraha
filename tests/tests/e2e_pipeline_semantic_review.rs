use common::config::SamgrahaConfig;
use schemas::audit::PipelineKind;
use services::KnowledgeRuntime;

// PlanningContextBuilder reads docs/raw/audit/<pipeline>-audit.md straight off
// disk (same as e2e_fix.rs's coverage_cv6 test), so this needs the real
// workspace root, not cargo test's crate-dir cwd.
fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn architecture_pipeline_semantic_review_parses_real_spec_and_evidences_real_docs() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let bundle = runtime
        .build_pipeline_semantic_review(&PipelineKind::Architecture)
        .unwrap();

    // architecture-audit.md defines A1-A13 (audit_crate::spec_parser's own
    // regression test cross-checks this against README.md's Authority Chain
    // table) — one task per check.
    assert_eq!(bundle.tasks.len(), 13);
    assert!(bundle
        .tasks
        .iter()
        .any(|t| t.check_id == "A1" && t.title == "Modular Architecture"));
    assert!(bundle.tasks.iter().all(|t| t.pipeline == "architecture"));
    assert!(bundle.tasks.iter().any(|t| t.audit_rule.is_some()));

    // Evidence must be the real compiled docs/raw/architecture/* documents,
    // not empty — this pipeline has a 1:1 matching domain.
    assert!(
        !bundle.evidence.is_empty(),
        "expected real architecture documents as evidence"
    );
    assert!(bundle
        .evidence
        .keys()
        .any(|path| path.replace('\\', "/").contains("architecture")));

    assert!(!bundle.instruction.is_empty());
}

#[test]
fn pipeline_with_no_matching_domain_still_parses_checks_with_empty_evidence() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let bundle = runtime
        .build_pipeline_semantic_review(&PipelineKind::Build)
        .unwrap();

    // build-audit.md defines B1-B12 + BC1-BC10 = 22 checks (spec_parser's
    // regression test confirms this count). There is no "build" domain, so
    // evidence collection (Phase 3 scope: domain-matched pipelines only, see
    // docs/proposal.md §8 phase 5) legitimately comes back empty — that's
    // the documented gap, not a bug.
    assert_eq!(bundle.tasks.len(), 22);
    assert!(bundle.evidence.is_empty());
}

#[test]
fn run_pipeline_alone_does_not_populate_semantic_review() {
    // Phase 4: semantic_review is opt-in (providers: ["semantic"] at the MCP
    // layer) — a plain run_pipeline() call must keep returning the default
    // empty bundle so existing callers see no behavior change.
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let report = runtime
        .run_pipeline(&PipelineKind::Architecture, false, false, false, false)
        .unwrap();

    assert!(report.semantic_review.tasks.is_empty());
    assert!(report.semantic_review.evidence.is_empty());
    assert!(report.semantic_review.instruction.is_empty());
}
