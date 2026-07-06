use schemas::{
    PhaseStatus, PhaseType, ProjectCase, ProjectPhase, ProjectPlan, ProjectPlanWithPhases, PlanStatus,
};
use uuid::Uuid;

use super::context::ProjectContext;

pub trait ProjectPlanner: Send + Sync {
    fn case(&self) -> ProjectCase;

    /// Generate this plan's phases. `plan_id` is deterministic-embedded into
    /// each phase id (`{plan_id}-{number}`) so dependency ids always resolve
    /// against the phases actually persisted for this plan.
    fn generate_phases(&self, plan_id: &str, ctx: &ProjectContext) -> Vec<ProjectPhase>;

    /// Build the plan header + phases together so callers never have to
    /// regenerate or hand-duplicate the phase list.
    fn generate_plan(&self, ctx: &ProjectContext, title: &str) -> ProjectPlanWithPhases {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let phases = self.generate_phases(&id, ctx);
        let plan = ProjectPlan {
            id: id.clone(),
            title: title.to_string(),
            case_type: self.case(),
            status: PlanStatus::Active,
            current_phase: phases.first().map(|p| p.id.clone()),
            created_at: now.clone(),
            updated_at: now,
        };
        ProjectPlanWithPhases { plan, phases }
    }
}

/// Build a phase whose id is `{plan_id}-{number}` and whose `dependencies`
/// reference other phases in the same plan by number — resolving to real
/// phase ids, never orphaned strings like `"1"` that no phase.id equals.
fn make_phase(
    plan_id: &str,
    number: u32,
    name: &str,
    phase_type: PhaseType,
    domains: Vec<&str>,
    pipeline_ids: Vec<&str>,
    dep_numbers: &[u32],
) -> ProjectPhase {
    ProjectPhase {
        id: format!("{}-{}", plan_id, number),
        plan_id: plan_id.to_string(),
        phase_number: number,
        name: name.to_string(),
        phase_type,
        domains: domains.into_iter().map(|s| s.to_string()).collect(),
        pipeline_ids: pipeline_ids.into_iter().map(|s| s.to_string()).collect(),
        dependencies: dep_numbers.iter().map(|n| format!("{}-{}", plan_id, n)).collect(),
        status: PhaseStatus::Pending,
        started_at: None,
        completed_at: None,
        result_json: None,
    }
}

// ── Doc-domain ordering ───────────────────────────────────────────────────

pub(crate) const DOC_DOMAINS: &[&str] = &[
    "vision", "architecture", "engineering", "readme", "external-context",
    "external-context-ownership", "design", "feature", "feature-design",
    "feature-technical", "prototype", "consistency", "coverage",
];

pub(crate) const DOC_PIPELINES: &[&str] = DOC_DOMAINS;

pub(crate) const IMPL_DOMAINS: &[&str] = &[
    "implementation", "deterministic-runtime", "security",
];

pub(crate) const IMPL_PIPELINES: &[&str] = IMPL_DOMAINS;

/// Every pipeline this planner suite ever audits, in canonical Domain
/// Ordering — used by full-lifecycle Verify phases so there's one source of
/// truth instead of a hand-typed list drifting out of sync (see executors.rs).
pub(crate) fn all_pipelines() -> Vec<&'static str> {
    DOC_PIPELINES.iter().chain(IMPL_PIPELINES.iter()).chain(["build"].iter()).copied().collect()
}

// ── NewProjectPlanner ─────────────────────────────────────────────────────

pub struct NewProjectPlanner;

impl ProjectPlanner for NewProjectPlanner {
    fn case(&self) -> ProjectCase { ProjectCase::NewProject }

    fn generate_phases(&self, plan_id: &str, ctx: &ProjectContext) -> Vec<ProjectPhase> {
        // Skip the Generate phase entirely when docs already exist — there is
        // no template-scaffolding capability to run for a truly blank
        // project (see proposal's "Missing Prerequisite Capabilities" #3),
        // and re-compiling existing docs from a no-op "Generate" phase is
        // wasted work.
        let mut specs: Vec<(&str, PhaseType, Vec<&str>, Vec<&str>)> = Vec::new();
        if !ctx.has_docs {
            specs.push(("Generate docs from templates", PhaseType::Generate, vec!["*"], vec![]));
        }
        specs.push(("Audit doc domains", PhaseType::Audit, DOC_DOMAINS.to_vec(), DOC_PIPELINES.to_vec()));
        specs.push(("Fix doc findings", PhaseType::Fix, DOC_DOMAINS.to_vec(), DOC_PIPELINES.to_vec()));
        specs.push(("Audit implementation + security", PhaseType::Audit, IMPL_DOMAINS.to_vec(), IMPL_PIPELINES.to_vec()));
        specs.push(("Fix implementation + security findings", PhaseType::Fix, IMPL_DOMAINS.to_vec(), IMPL_PIPELINES.to_vec()));
        specs.push(("Audit build", PhaseType::Audit, vec!["build"], vec!["build"]));
        specs.push(("Fix build findings", PhaseType::Fix, vec!["build"], vec!["build"]));
        specs.push(("Final verify", PhaseType::Verify, vec!["*"], all_pipelines()));

        specs
            .into_iter()
            .enumerate()
            .map(|(i, (name, phase_type, domains, pipelines))| {
                let number = (i + 1) as u32;
                let deps: Vec<u32> = if number > 1 { vec![number - 1] } else { vec![] };
                make_phase(plan_id, number, name, phase_type, domains, pipelines, &deps)
            })
            .collect()
    }
}

// ── DocAuditPlanner ───────────────────────────────────────────────────────

pub struct DocAuditPlanner;

impl ProjectPlanner for DocAuditPlanner {
    fn case(&self) -> ProjectCase { ProjectCase::DocAudit }

    fn generate_phases(&self, plan_id: &str, _ctx: &ProjectContext) -> Vec<ProjectPhase> {
        vec![
            make_phase(plan_id, 1, "Audit all doc domains", PhaseType::Audit,
                DOC_DOMAINS.to_vec(), DOC_PIPELINES.to_vec(), &[]),
            make_phase(plan_id, 2, "Fix doc findings", PhaseType::Fix,
                DOC_DOMAINS.to_vec(), DOC_PIPELINES.to_vec(), &[1]),
            make_phase(plan_id, 3, "Re-audit fixed domains", PhaseType::Verify,
                DOC_DOMAINS.to_vec(), DOC_PIPELINES.to_vec(), &[2]),
        ]
    }
}

// ── ImplTestAuditPlanner ──────────────────────────────────────────────────

pub struct ImplTestAuditPlanner;

impl ProjectPlanner for ImplTestAuditPlanner {
    fn case(&self) -> ProjectCase { ProjectCase::ImplTestAudit }

    fn generate_phases(&self, plan_id: &str, _ctx: &ProjectContext) -> Vec<ProjectPhase> {
        vec![
            make_phase(plan_id, 1, "Audit implementation, security, runtime", PhaseType::Audit,
                IMPL_DOMAINS.to_vec(), IMPL_PIPELINES.to_vec(), &[]),
            make_phase(plan_id, 2, "Fix implementation findings", PhaseType::Fix,
                vec!["implementation"], vec!["implementation"], &[1]),
            make_phase(plan_id, 3, "Fix security findings", PhaseType::Fix,
                vec!["security"], vec!["security"], &[1]),
            make_phase(plan_id, 4, "Run test/coverage pipeline", PhaseType::Audit,
                vec!["coverage"], vec!["coverage"], &[2, 3]),
            make_phase(plan_id, 5, "Fix test/coverage findings", PhaseType::Fix,
                vec!["coverage"], vec!["coverage"], &[4]),
            make_phase(plan_id, 6, "Re-audit fixed domains", PhaseType::Verify,
                vec!["*"], vec!["implementation", "deterministic-runtime", "security", "coverage"], &[5]),
        ]
    }
}

// ── BuildAuditPlanner ─────────────────────────────────────────────────────

pub struct BuildAuditPlanner;

impl ProjectPlanner for BuildAuditPlanner {
    fn case(&self) -> ProjectCase { ProjectCase::BuildAudit }

    fn generate_phases(&self, plan_id: &str, _ctx: &ProjectContext) -> Vec<ProjectPhase> {
        vec![
            make_phase(plan_id, 1, "Audit build configuration", PhaseType::Audit,
                vec!["build"], vec!["build"], &[]),
            make_phase(plan_id, 2, "Fix build findings", PhaseType::Fix,
                vec!["build"], vec!["build"], &[1]),
            make_phase(plan_id, 3, "Re-audit build", PhaseType::Verify,
                vec!["build"], vec!["build"], &[2]),
        ]
    }
}

/// Resolve the planner for a given case.
pub fn resolve_planner(case: &ProjectCase) -> Box<dyn ProjectPlanner> {
    match case {
        ProjectCase::NewProject => Box::new(NewProjectPlanner),
        ProjectCase::DocAudit => Box::new(DocAuditPlanner),
        ProjectCase::ImplTestAudit => Box::new(ImplTestAuditPlanner),
        ProjectCase::BuildAudit => Box::new(BuildAuditPlanner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx(has_docs: bool) -> ProjectContext {
        ProjectContext {
            case: ProjectCase::DocAudit,
            has_docs,
            compiled_domains: vec![],
            existing_scores: std::collections::HashMap::new(),
        }
    }

    /// Every `dependencies` entry must equal some other phase's `id` in the
    /// same plan — this is the regression test for the bug where
    /// dependencies were plain numbers ("1", "2") that never matched the
    /// UUID phase ids the orchestrator matches against, permanently
    /// deadlocking execution after phase 1.
    fn assert_deps_resolve(phases: &[ProjectPhase]) {
        for phase in phases {
            for dep in &phase.dependencies {
                assert!(
                    phases.iter().any(|p| &p.id == dep),
                    "phase {} depends on '{}' which matches no phase id in the plan",
                    phase.phase_number, dep
                );
            }
        }
    }

    #[test]
    fn test_new_project_planner_creates_8_phases() {
        let planner = NewProjectPlanner;
        let ctx = test_ctx(false);
        let result = planner.generate_plan(&ctx, "test-new");
        assert_eq!(result.phases.len(), 8);
        assert_eq!(result.phases[0].phase_type, PhaseType::Generate);
        assert_eq!(result.phases[1].phase_type, PhaseType::Audit);
        assert_eq!(result.phases[2].phase_type, PhaseType::Fix);
        assert_eq!(result.phases[7].phase_type, PhaseType::Verify);
        assert!(!result.phases[7].pipeline_ids.is_empty(), "final verify must not rely on an executor-side fallback list");
        assert_deps_resolve(&result.phases);
    }

    #[test]
    fn test_new_project_planner_skips_generate_when_docs_exist() {
        let planner = NewProjectPlanner;
        let ctx = test_ctx(true);
        let result = planner.generate_plan(&ctx, "test-existing");
        assert_eq!(result.phases.len(), 7);
        assert_eq!(result.phases[0].phase_type, PhaseType::Audit);
        assert_deps_resolve(&result.phases);
    }

    #[test]
    fn test_doc_audit_planner() {
        let planner = DocAuditPlanner;
        let ctx = test_ctx(false);
        let result = planner.generate_plan(&ctx, "test-docs");
        assert_eq!(result.plan.case_type, ProjectCase::DocAudit);
        assert_eq!(result.plan.status, PlanStatus::Active);
        assert_eq!(result.phases.len(), 3);
        assert_deps_resolve(&result.phases);
    }

    #[test]
    fn test_build_audit_planner_creates_3_phases() {
        let planner = BuildAuditPlanner;
        let ctx = test_ctx(false);
        let result = planner.generate_plan(&ctx, "test-build");
        assert_eq!(result.plan.case_type, ProjectCase::BuildAudit);
        assert_eq!(result.plan.status, PlanStatus::Active);
        assert_eq!(result.phases.len(), 3);
        assert_deps_resolve(&result.phases);
    }

    #[test]
    fn test_impl_test_audit_planner() {
        let planner = ImplTestAuditPlanner;
        let ctx = test_ctx(false);
        let result = planner.generate_plan(&ctx, "test-impl");
        assert_eq!(result.plan.case_type, ProjectCase::ImplTestAudit);
        assert_eq!(result.phases.len(), 6);
        // Phase 4 (test/coverage) depends on both fix phases (2 and 3).
        assert_eq!(result.phases[3].dependencies.len(), 2);
        assert_deps_resolve(&result.phases);
    }

    #[test]
    fn test_resolve_planner() {
        let p = resolve_planner(&ProjectCase::NewProject);
        assert_eq!(p.case(), ProjectCase::NewProject);
        let p = resolve_planner(&ProjectCase::DocAudit);
        assert_eq!(p.case(), ProjectCase::DocAudit);
        let p = resolve_planner(&ProjectCase::BuildAudit);
        assert_eq!(p.case(), ProjectCase::BuildAudit);
        let p = resolve_planner(&ProjectCase::ImplTestAudit);
        assert_eq!(p.case(), ProjectCase::ImplTestAudit);
    }

    #[test]
    fn test_phase_dependencies_are_sequential() {
        let planner = NewProjectPlanner;
        let ctx = test_ctx(false);
        let result = planner.generate_plan(&ctx, "test-dep");
        for (i, phase) in result.phases.iter().enumerate() {
            if i > 0 {
                assert!(!phase.dependencies.is_empty(), "Phase {} has no deps", phase.phase_number);
            }
        }
    }

    #[test]
    fn test_all_planner_dependencies_resolve() {
        for case in [ProjectCase::NewProject, ProjectCase::DocAudit, ProjectCase::ImplTestAudit, ProjectCase::BuildAudit] {
            let planner = resolve_planner(&case);
            let result = planner.generate_plan(&test_ctx(false), "test");
            assert_deps_resolve(&result.phases);
        }
    }
}
