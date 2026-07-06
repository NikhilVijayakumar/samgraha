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
fn run_single_check_dispatches_to_real_pipeline() {
    let config = SamgrahaConfig::default();
    let root = std::env::current_dir().unwrap();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();

    // Must actually run the architecture pipeline and return a score —
    // not the old hardcoded "not yet implemented" stub error.
    let score = runtime.run_single_check("architecture", "A1").unwrap();
    assert!((0.0..=10.0).contains(&score));

    // Unknown domain still fails, distinctly from the old stub message.
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

    // Proves the check-specific extraction (Phase 1) is actually wired
    // through the real repo's audit spec, not just the test fixtures —
    // A1 in docs/raw/audit/architecture-audit.md is "Modular Architecture".
    assert!(plan.steps[0].rationale.contains("Modular Architecture"));
    assert!(!plan.steps[0].rationale.contains("Document standard requires sections missing"));

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
