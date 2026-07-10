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

/// Phase 5 (docs/proposal.md §8): `build_pipeline_semantic_review` itself is
/// pipeline-agnostic (spec file path from `PipelineKind::as_str()`, parsing
/// from `audit_crate::spec_parser`, evidence from `get_documents_by_domain`)
/// — nothing pipeline-specific was needed for Phase 3-4 to already work on
/// all 20 real pipelines (every `PipelineKind` variant except the `Doc`
/// sentinel). This test proves that generality against every real
/// pipeline's real spec file in one pass, instead of one bespoke test per
/// pipeline.
///
/// Expected counts are cross-checked against `audit_crate::spec_parser`'s own
/// regression test (same source file, same counts) — `help` has no
/// `docs/raw/audit/help-audit.md` at all (confirmed: no such file exists),
/// so it's the one legitimate 0.
#[test]
fn every_pipeline_produces_a_semantic_review_bundle_without_erroring() {
    let root = workspace_root();
    let runtime = KnowledgeRuntime::new(&root, SamgrahaConfig::default()).unwrap();

    let expectations: &[(PipelineKind, usize)] = &[
        (PipelineKind::Architecture, 13),
        (PipelineKind::Build, 22),
        (PipelineKind::Security, 23),
        (PipelineKind::Consistency, 12),
        (PipelineKind::Coverage, 15),
        (PipelineKind::Dependency, 8),
        (PipelineKind::Design, 12),
        (PipelineKind::Readme, 12),
        (PipelineKind::Prototype, 15),
        (PipelineKind::ExternalContext, 12),
        (PipelineKind::Engineering, 12),
        (PipelineKind::Feature, 14),
        (PipelineKind::FeatureTechnical, 15),
        (PipelineKind::FeatureDesign, 15),
        (PipelineKind::DeterministicRuntime, 12),
        (PipelineKind::ExternalContextOwnership, 12),
        (PipelineKind::Implementation, 15),
        (PipelineKind::DocumentationStructure, 45),
        (PipelineKind::Vision, 12),
        // No docs/raw/audit/help-audit.md exists — legitimately 0 tasks,
        // not an error. See docs/proposal.md's Known gaps.
        (PipelineKind::Help, 0),
    ];
    assert_eq!(expectations.len(), 20, "expected every PipelineKind except Doc");

    for (kind, expected_task_count) in expectations {
        let bundle = runtime
            .build_pipeline_semantic_review(kind)
            .unwrap_or_else(|e| panic!("{:?} failed: {e}", kind));

        assert_eq!(
            bundle.tasks.len(),
            *expected_task_count,
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
