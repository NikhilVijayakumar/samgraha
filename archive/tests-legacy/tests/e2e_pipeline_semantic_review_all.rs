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

/// Originally proved `build_pipeline_semantic_review`'s generality against
/// every real pipeline's real `docs/raw/audit/{pipeline}-audit.md` spec file,
/// with per-pipeline task counts cross-checked against
/// `audit_crate::spec_parser`'s own regression test.
///
/// `docs/raw/audit/` no longer exists (documentation-cleanup-proposal.md —
/// spec content is the owning system's concern now, not samgraha's). Every
/// pipeline's expected count is now 0, same as `help`'s always was (it
/// never had a spec file either) — this test now proves the *graceful*
/// half of that generality claim: no spec anywhere means zero tasks
/// everywhere, consistently, not an error for some pipelines and a crash
/// for others. Real per-pipeline task-count parsing no longer has
/// anything to test against.
#[test]
fn every_pipeline_produces_an_empty_semantic_review_bundle_without_erroring() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let kinds: &[PipelineKind] = &[
        PipelineKind::Architecture,
        PipelineKind::Build,
        PipelineKind::Security,
        PipelineKind::Consistency,
        PipelineKind::Coverage,
        PipelineKind::Dependency,
        PipelineKind::Design,
        PipelineKind::Readme,
        PipelineKind::Prototype,
        PipelineKind::ExternalContext,
        PipelineKind::Engineering,
        PipelineKind::Feature,
        PipelineKind::FeatureTechnical,
        PipelineKind::FeatureDesign,
        PipelineKind::DeterministicRuntime,
        PipelineKind::ExternalContextOwnership,
        PipelineKind::Implementation,
        PipelineKind::DocumentationStructure,
        PipelineKind::Vision,
        PipelineKind::Help,
    ];
    assert_eq!(kinds.len(), 20, "expected every PipelineKind except Doc");

    for kind in kinds {
        let expected_task_count = 0usize;
        let bundle = runtime
            .build_pipeline_semantic_review(kind)
            .unwrap_or_else(|e| panic!("{:?} failed: {e}", kind));

        assert_eq!(
            bundle.tasks.len(),
            expected_task_count,
            "{:?}: expected {expected_task_count} tasks, got {} ({:?})",
            kind,
            bundle.tasks.len(),
            bundle.tasks.iter().map(|t| t.check_id.as_str()).collect::<Vec<_>>()
        );
        assert!(bundle.tasks.iter().all(|t| &t.pipeline == kind.as_str()));

        // Evidence non-emptiness must exactly match whether this pipeline's
        // name is also a real document domain — self-consistency check
        // against get_documents_by_domain rather than a hardcoded guess.
        let has_domain_docs = !runtime
            .get_documents_by_domain(kind.as_str())
            .unwrap_or_default()
            .is_empty();
        assert_eq!(
            !bundle.evidence.is_empty(),
            has_domain_docs,
            "{:?}: evidence emptiness disagrees with get_documents_by_domain",
            kind
        );
    }
}
