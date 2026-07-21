use common::config::SamgrahaConfig;
use schemas::{PhaseStatus, PlanStatus, ProjectCase};
use services::{KnowledgeRuntime, PlanOrchestrator};
use std::sync::Arc;

fn real_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Regression test for the bug where NewProjectPlanner phases had
/// dependencies stored as plain numbers ("1") while phase ids were random
/// UUIDs — execute_phase's dependency check matched on `p.id == dep_id` and
/// always failed to find phase "1", permanently deadlocking every plan after
/// its first phase. This drives real create_plan + execute_phase calls
/// end-to-end through a real registry DB, which is the only place the old
/// bug actually manifested (unit tests on the planner's phase list alone
/// couldn't see it).
#[test]
fn new_project_plan_advances_past_first_phase() {
    let root = real_root();
    let config = SamgrahaConfig::default();
    let runtime = Arc::new(KnowledgeRuntime::new(&root, config).unwrap());
    let orchestrator = PlanOrchestrator::new(Arc::clone(&runtime), Arc::clone(&runtime.registry));

    let created = orchestrator.create_plan(&ProjectCase::NewProject, "e2e-new-project").unwrap();
    assert!(created.phases.len() >= 2, "expected a multi-phase plan");
    let plan_id = created.plan.id.clone();

    // Phase 1 (Generate or Audit, depending on whether docs already exist).
    let phase1 = orchestrator.execute_phase(&plan_id, Some(1)).unwrap();
    assert_eq!(phase1["status"], "completed", "phase 1 should complete: {phase1:?}");

    // Phase 2 must be able to resolve its dependency on phase 1 by id, not
    // fail with "Dependency phase '1' not found in plan".
    let phase2 = orchestrator.execute_phase(&plan_id, Some(2)).unwrap();
    assert_eq!(phase2["status"], "completed", "phase 2 should complete: {phase2:?}");

    let plan = orchestrator.get_plan(&plan_id).unwrap().unwrap();
    assert_eq!(plan.phases[0].status, PhaseStatus::Completed);
    assert_eq!(plan.phases[1].status, PhaseStatus::Completed);
}

/// A second execute_phase call for a phase already started must be rejected,
/// not silently re-execute it — the atomic try_start_phase guard.
#[test]
fn concurrent_execute_on_same_phase_is_rejected() {
    let root = real_root();
    let config = SamgrahaConfig::default();
    let runtime = Arc::new(KnowledgeRuntime::new(&root, config).unwrap());
    let orchestrator = PlanOrchestrator::new(Arc::clone(&runtime), Arc::clone(&runtime.registry));

    let created = orchestrator.create_plan(&ProjectCase::BuildAudit, "e2e-concurrency").unwrap();
    let plan_id = created.plan.id.clone();

    // Manually re-mark phase 1 back to pending after a hand start, to
    // simulate two overlapping callers racing on the same pending phase:
    // the first call's try_start_phase should win, so directly asserting via
    // the registry is the deterministic way to exercise the guard without
    // real threads.
    assert!(runtime.registry.try_start_phase(&created.phases[0].id).unwrap());
    assert!(!runtime.registry.try_start_phase(&created.phases[0].id).unwrap());

    // Plan status has to stay active for this assertion.
    assert_eq!(orchestrator.get_plan(&plan_id).unwrap().unwrap().plan.status, PlanStatus::Active);
}
