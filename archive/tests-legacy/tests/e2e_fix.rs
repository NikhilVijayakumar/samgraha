use common::config::SamgrahaConfig;
use schemas::audit::{AuditFinding, Severity};
use schemas::fix::PlanType;
use services::KnowledgeRuntime;
use std::sync::Arc;

fn dummy_finding(check_id: &str) -> AuditFinding {
    AuditFinding {
        check_id: check_id.into(),
        severity: Severity::Error,
        message: "test finding".into(),
        location: None,
        document_id: None,
        provider: "test".into(),
        stage: None,
        section_id: None,
        confidence: None,
        evidence: None,
        status: None,
        strategy: None,
    }
}

#[test]
fn run_single_check_reports_missing_script_not_stub_error() {
    // The 22 hardcoded Rust pipelines were deleted (codebase-refactoring-
    // proposal.md §10 Phase 4) — no Rust-native fallback, by design.
    // A pipeline kind with no system-provided `validate` script now fails
    // with a clear "register a script" message, not the old stub error
    // ("not yet implemented") and not a silently-run Rust pipeline.
    let config = SamgrahaConfig::default();
    let root = std::env::current_dir().unwrap();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();

    // anyhow's `Display`/`to_string()` only shows the outermost `with_context`
    // message ("Failed to run 'architecture' pipeline...") — the actual
    // missing-script text is a lower link in the chain, so check the full
    // chain, not just the top message.
    let err = runtime.run_single_check("architecture", "A1").unwrap_err();
    let full_chain = err.chain().map(|e| e.to_string()).collect::<Vec<_>>().join(" | ");
    assert!(
        full_chain.contains("No validate script found"),
        "expected the new missing-script error, got: {}",
        full_chain
    );

    // Unknown domain fails earlier and differently — never reaches the
    // capability-dispatch path at all.
    let err = runtime.run_single_check("not-a-real-domain", "X1").unwrap_err();
    assert!(err.to_string().contains("Unknown audit domain"));
}

#[test]
fn dependency_domain_is_rejected_by_fix_pipeline() {
    let config = SamgrahaConfig::default();
    let root = std::env::current_dir().unwrap();
    let runtime = Arc::new(KnowledgeRuntime::new(&root, config).unwrap());
    let finding = dummy_finding("D1");
    let target = root.join("Cargo.toml");

    let plan_err = runtime
        .generate_fix_plan(&finding, "dependency", 1, "pipeline", &target)
        .unwrap_err();
    assert!(plan_err.to_string().contains("excluded"));

    let apply_err = runtime
        .apply_finding_fix(&finding, "dependency", 1, "pipeline", &target)
        .unwrap_err();
    assert!(apply_err.to_string().contains("excluded"));
}

#[test]
fn coverage_cv6_routes_to_test_plan_via_preview_path() {
    // PlanningContextBuilder reads docs/raw/audit/<domain>-audit.md straight
    // off disk (unlike pipelines, which query the compiled knowledge.db), so
    // this needs the real workspace root, not cargo test's crate-dir cwd.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let config = SamgrahaConfig::default();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();
    let target = root.join("Cargo.toml");

    let cv6_plan = runtime
        .generate_fix_plan(&dummy_finding("CV6"), "coverage", 1, "pipeline", &target)
        .unwrap();
    assert_eq!(cv6_plan.plan_type, PlanType::Test);

    let cv1_plan = runtime
        .generate_fix_plan(&dummy_finding("CV1"), "coverage", 1, "pipeline", &target)
        .unwrap();
    assert_eq!(cv1_plan.plan_type, PlanType::Documentation);
}

#[test]
fn preview_plan_is_persisted_and_retrievable() {
    // docs/raw/audit/architecture-audit.md no longer exists (documentation-
    // cleanup-proposal.md — that content is the owning system's concern
    // now, not samgraha's). PlanningContextBuilder degrades gracefully
    // instead of hard-failing (planning_context.rs's get_or_load) — a plan
    // still gets generated, just with planner.rs's generic fallback
    // rationale instead of a check-specific one quoted from the spec.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let config = SamgrahaConfig::default();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();
    let target = root.join("Cargo.toml");

    let plan = runtime
        .generate_fix_plan(&dummy_finding("A1"), "architecture", 42, "pipeline", &target)
        .unwrap();

    // No spec on disk to quote from — planner.rs's generic fallback wins,
    // proving degradation is graceful (a plan is still produced) rather
    // than the spec's absence silently producing an empty/wrong rationale.
    assert!(plan.steps[0].rationale.contains("Document standard requires sections missing"));

    let plan_id = plan.id.expect("preview plan must be persisted and carry an id");

    // Same store methods audit_fix_plan_get calls — proves the MCP path
    // that previously returned nothing for a preview-only plan now works.
    let fetched = runtime.registry.get_fix_plan(plan_id).unwrap()
        .expect("persisted plan must be retrievable by id");
    assert_eq!(fetched.criterion_id, "A1");
    assert_eq!(fetched.domain, "architecture");

    let fetched_steps = runtime.registry.get_fix_plan_steps(plan_id).unwrap();
    assert_eq!(fetched_steps.len(), plan.steps.len());
    assert!(!fetched_steps.is_empty());
}
