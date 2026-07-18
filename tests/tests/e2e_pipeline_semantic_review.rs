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
fn architecture_pipeline_semantic_review_degrades_gracefully_without_spec_on_disk() {
    // docs/raw/audit/architecture-audit.md no longer exists (documentation-
    // cleanup-proposal.md — spec content is the owning system's concern
    // now). Renamed and rewritten from
    // "...parses_real_spec_and_evidences_real_docs": that spec is gone
    // permanently, not temporarily missing, so "parses real spec" is no
    // longer a real scenario to test. What's actually worth proving now:
    // no spec on disk means zero tasks, not an error — same graceful-
    // degradation contract planning_context.rs's get_or_load establishes
    // for the fix pipeline (e2e_fix.rs's preview_plan_is_persisted test).
    //
    // Evidence is a separate concern from tasks — it comes from real
    // compiled docs/raw/architecture/* documents via get_documents_by_domain,
    // not from the (now-deleted) spec file, so it's unaffected by the spec
    // being gone. First draft of this rewrite incorrectly assumed evidence
    // would also be empty; fixed to assert on the same "matches a real
    // domain with compiled docs" self-consistency check the original test
    // used, instead of a hardcoded emptiness guess either way.
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let bundle = runtime
        .build_pipeline_semantic_review(&PipelineKind::Architecture)
        .unwrap();

    assert!(bundle.tasks.is_empty(), "expected no tasks with no spec on disk, got {:?}", bundle.tasks);

    let has_domain_docs = !runtime
        .get_documents_by_domain("architecture")
        .unwrap_or_default()
        .is_empty();
    assert_eq!(
        !bundle.evidence.is_empty(),
        has_domain_docs,
        "evidence emptiness disagrees with get_documents_by_domain"
    );
}

#[test]
fn pipeline_with_no_spec_and_no_matching_domain_has_empty_evidence_too() {
    // Renamed from "...still_parses_checks_with_empty_evidence" — build-
    // audit.md is also gone, so there are no checks to parse either now.
    // Still proving the same original point about evidence (no "build"
    // domain means no evidence to collect), just without the spec-parsing
    // half that no longer applies.
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let bundle = runtime
        .build_pipeline_semantic_review(&PipelineKind::Build)
        .unwrap();

    assert!(bundle.tasks.is_empty());
    assert!(bundle.evidence.is_empty());
}

#[test]
fn run_pipeline_alone_fails_clearly_with_no_validate_script() {
    // Renamed from "...does_not_populate_semantic_review" — the 22 hardcoded
    // Rust pipelines were deleted (codebase-refactoring-proposal.md §10
    // Phase 4), no fallback, by design. run_pipeline() can no longer
    // succeed at all for a kind with no system-provided `validate` script,
    // so "call it and check semantic_review stayed empty" isn't a
    // reachable scenario anymore — the call fails before there's any
    // report to check. What's still true and worth asserting: it fails
    // with the new clear message, not silently or with a stub error.
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let err = runtime
        .run_pipeline(&PipelineKind::Architecture, false, false, false, false)
        .unwrap_err();
    let full_chain = err.chain().map(|e| e.to_string()).collect::<Vec<_>>().join(" | ");
    assert!(
        full_chain.contains("No validate script found"),
        "expected the new missing-script error, got: {}",
        full_chain
    );
}
