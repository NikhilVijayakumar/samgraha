use schemas::{
    PhaseStatus, PhaseType, ProjectCase, ProjectPhase, ProjectPlan, ProjectPlanWithPhases, PlanStatus,
};
use uuid::Uuid;

use super::context::ProjectContext;
#[cfg(test)]
use super::context::StandardWorkflowContext;

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
    "feature-technical", "prototype", "consistency", "coverage", "philosophy",
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

// ── StandardWorkflowPlanner ───────────────────────────────────────────────

/// Builds phases from a registered standard's own `plan_scenarios` (tier x
/// repo_state x doc_state x step content — loader Pass 8) instead of a
/// hardcoded domain list, the way the other 4 planners do. No file is read
/// here — `ctx.standard` is already fully populated from `StandardRegistry`
/// by `ProjectContext::detect_with_registry`.
pub struct StandardWorkflowPlanner;

impl ProjectPlanner for StandardWorkflowPlanner {
    fn case(&self) -> ProjectCase { ProjectCase::Standard }

    fn generate_phases(&self, plan_id: &str, ctx: &ProjectContext) -> Vec<ProjectPhase> {
        let Some(sw) = &ctx.standard else { return vec![] };
        let doc_state = if ctx.has_docs { "has_documentation" } else { "no_documentation" };

        // Group this repo's matching scenarios by tier (ascending), steps
        // within a tier always considered generation -> audit -> fix
        // regardless of the DB rows' own order.
        let mut by_tier: std::collections::BTreeMap<i32, Vec<&schemas::standard::PlanScenario>> =
            std::collections::BTreeMap::new();
        for scenario in &sw.plan_scenarios {
            if scenario.repo_state == ctx.repo_state && scenario.doc_state == doc_state {
                by_tier.entry(scenario.tier).or_default().push(scenario);
            }
        }

        const STEP_ORDER: [&str; 3] = ["generation", "audit", "fix"];
        let mut phases = Vec::new();
        let mut number: u32 = 0;
        // tier_gate (plan_settings): every domain in a tier must clear before
        // the next tier starts. Within a tier, domains are one flat group
        // (parallel) unless domain_relationships declares enforce_order
        // between two of them (e.g. base_dev's documented exception:
        // External Context before Engineering, both tier 2) — those split
        // the tier into topological layers instead. Chains across
        // tiers/layers the same way either case: each new group's first
        // phase depends on the previous group's last phase.
        let mut prev_group_last_number: Option<u32> = None;

        for (tier, scenarios) in &by_tier {
            let domains: Vec<String> = sw.domains_by_tier.get(tier).cloned().unwrap_or_default();
            let layers = topological_layers(&domains, &sw.relationships);

            for layer in &layers {
                let layer_domains: Vec<&str> = layer.iter().map(|s| s.as_str()).collect();
                let mut prev_step_number: Option<u32> = None;

                for step in STEP_ORDER {
                    let Some(scenario) = scenarios.iter().find(|s| s.step == step) else { continue };
                    number += 1;
                    let phase_type = match scenario.step.as_str() {
                        "generation" => PhaseType::Generate,
                        "audit" => PhaseType::Audit,
                        "fix" => PhaseType::Fix,
                        _ => PhaseType::Verify,
                    };
                    let deps: Vec<u32> = prev_step_number
                        .or(prev_group_last_number)
                        .into_iter()
                        .collect();
                    let name = if layers.len() > 1 {
                        format!("Tier {} [{}] — {}", tier, layer.join(", "), step)
                    } else {
                        format!("Tier {} — {}", tier, step)
                    };
                    // pipeline_ids intentionally empty — these domains aren't
                    // PipelineKind variants (AuditPhaseExecutor/FixPhaseExecutor
                    // fall back to runtime.audit(domain) per phase.domains entry
                    // when pipeline_ids is empty).
                    phases.push(make_phase(plan_id, number, &name, phase_type, layer_domains.clone(), vec![], &deps));
                    prev_step_number = Some(number);
                }
                if let Some(last) = prev_step_number {
                    prev_group_last_number = Some(last);
                }
            }
        }
        phases
    }
}

/// Split `domains` into topological layers using only the `enforce_order`
/// edges whose *both* endpoints are in `domains` (an edge to/from a domain
/// in a different tier doesn't constrain phase ordering within this one).
/// Domains with no such edge between them land in the same layer — order
/// among them is unconstrained, not implied. Falls back to one layer
/// holding every domain (today's behavior for every tier but one, across
/// both real standards) when there's nothing to order. A cycle (shouldn't
/// happen — `knowledge-hub-loader.py` doesn't validate against inserting
/// one, so defensive, not just decorative) dumps whatever's left into a
/// final layer rather than looping forever.
fn topological_layers(domains: &[String], relationships: &[schemas::standard::StandardRelationship]) -> Vec<Vec<String>> {
    use std::collections::{HashMap, HashSet};

    let domain_set: HashSet<&str> = domains.iter().map(|s| s.as_str()).collect();
    let edges: Vec<(&str, &str)> = relationships
        .iter()
        .filter(|r| r.enforce_order)
        .filter(|r| domain_set.contains(r.from_domain.as_str()) && domain_set.contains(r.to_domain.as_str()))
        .map(|r| (r.from_domain.as_str(), r.to_domain.as_str()))
        .collect();

    if edges.is_empty() {
        return vec![domains.to_vec()];
    }

    let mut in_degree: HashMap<&str, usize> = domains.iter().map(|d| (d.as_str(), 0)).collect();
    let mut successors: HashMap<&str, Vec<&str>> = HashMap::new();
    for (from, to) in &edges {
        *in_degree.entry(to).or_insert(0) += 1;
        successors.entry(from).or_default().push(to);
    }

    let mut layers: Vec<Vec<String>> = Vec::new();
    let mut remaining: HashSet<&str> = domain_set.clone();
    while !remaining.is_empty() {
        let mut layer: Vec<&str> = remaining
            .iter()
            .filter(|d| in_degree.get(*d).copied().unwrap_or(0) == 0)
            .copied()
            .collect();
        if layer.is_empty() {
            // Cycle — no zero-in-degree node left. Dump the rest as one
            // final layer instead of looping forever.
            layer = remaining.iter().copied().collect();
        }
        layer.sort();
        for d in &layer {
            remaining.remove(d);
            if let Some(succs) = successors.get(d) {
                for s in succs {
                    if let Some(deg) = in_degree.get_mut(s) {
                        *deg = deg.saturating_sub(1);
                    }
                }
            }
        }
        layers.push(layer.into_iter().map(String::from).collect());
    }
    layers
}

/// Resolve the planner for a given case.
pub fn resolve_planner(case: &ProjectCase) -> Box<dyn ProjectPlanner> {
    match case {
        ProjectCase::NewProject => Box::new(NewProjectPlanner),
        ProjectCase::DocAudit => Box::new(DocAuditPlanner),
        ProjectCase::ImplTestAudit => Box::new(ImplTestAuditPlanner),
        ProjectCase::BuildAudit => Box::new(BuildAuditPlanner),
        ProjectCase::Standard => Box::new(StandardWorkflowPlanner),
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
            standard: None,
            repo_state: "new".to_string(),
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

    fn scenario(repo_state: &str, doc_state: &str, tier: i32, step: &str) -> schemas::standard::PlanScenario {
        schemas::standard::PlanScenario {
            repo_state: repo_state.into(),
            doc_state: doc_state.into(),
            tier,
            step: step.into(),
            content: format!("tier {} {} content", tier, step),
        }
    }

    fn standard_ctx(
        repo_state: &str,
        has_docs: bool,
        scenarios: Vec<schemas::standard::PlanScenario>,
        domains_by_tier: std::collections::HashMap<i32, Vec<String>>,
    ) -> ProjectContext {
        standard_ctx_with_relationships(repo_state, has_docs, scenarios, domains_by_tier, vec![])
    }

    fn standard_ctx_with_relationships(
        repo_state: &str,
        has_docs: bool,
        scenarios: Vec<schemas::standard::PlanScenario>,
        domains_by_tier: std::collections::HashMap<i32, Vec<String>>,
        relationships: Vec<schemas::standard::StandardRelationship>,
    ) -> ProjectContext {
        ProjectContext {
            case: ProjectCase::Standard,
            has_docs,
            compiled_domains: vec![],
            existing_scores: std::collections::HashMap::new(),
            repo_state: repo_state.to_string(),
            standard: Some(StandardWorkflowContext {
                plan_settings: None,
                plan_scenarios: scenarios,
                domains_by_tier,
                relationships,
            }),
        }
    }

    #[test]
    fn standard_workflow_planner_orders_tiers_and_steps() {
        let scenarios = vec![
            scenario("new", "no_documentation", 1, "generation"),
            scenario("new", "no_documentation", 1, "audit"),
            scenario("new", "no_documentation", 1, "fix"),
            scenario("new", "no_documentation", 2, "generation"),
            scenario("new", "no_documentation", 2, "audit"),
            scenario("new", "no_documentation", 2, "fix"),
            // Wrong repo_state/doc_state — must not leak into the result.
            scenario("existing", "has_documentation", 1, "generation"),
        ];
        let mut domains_by_tier = std::collections::HashMap::new();
        domains_by_tier.insert(1, vec!["infrastructure".to_string()]);
        domains_by_tier.insert(2, vec!["engineering".to_string()]);

        let ctx = standard_ctx("new", false, scenarios, domains_by_tier);
        let result = StandardWorkflowPlanner.generate_plan(&ctx, "test-standard");

        assert_eq!(result.phases.len(), 6, "3 steps x 2 tiers, mismatched scenario excluded");
        assert_eq!(result.phases[0].phase_type, PhaseType::Generate);
        assert_eq!(result.phases[1].phase_type, PhaseType::Audit);
        assert_eq!(result.phases[2].phase_type, PhaseType::Fix);
        assert_eq!(result.phases[0].domains, vec!["infrastructure".to_string()]);
        assert_eq!(result.phases[3].domains, vec!["engineering".to_string()]);
        assert!(result.phases[3].pipeline_ids.is_empty(), "domains, not PipelineKind pipeline_ids");
        // Tier 2's first phase gates on tier 1's last phase completing.
        assert_eq!(result.phases[3].dependencies, vec![result.phases[2].id.clone()]);
        assert_deps_resolve(&result.phases);
    }

    fn relationship(from: &str, to: &str, enforce_order: bool) -> schemas::standard::StandardRelationship {
        schemas::standard::StandardRelationship {
            from_domain: from.into(),
            to_domain: to.into(),
            relationship: "informs".into(),
            enforce_order,
            tier_gating_strict: false,
        }
    }

    #[test]
    fn standard_workflow_planner_splits_tier_on_enforce_order() {
        // base_dev's real documented exception: external-context before
        // engineering, both tier 2. documentation has no ordering
        // constraint with either and should land in the same layer as
        // whichever domain has no edge to it (external-context here, since
        // it has in-degree 0 same as documentation).
        let scenarios = vec![
            scenario("new", "no_documentation", 2, "audit"),
        ];
        let mut domains_by_tier = std::collections::HashMap::new();
        domains_by_tier.insert(2, vec!["external-context".to_string(), "engineering".to_string(), "documentation".to_string()]);
        let relationships = vec![relationship("external-context", "engineering", true)];

        let ctx = standard_ctx_with_relationships("new", false, scenarios, domains_by_tier, relationships);
        let result = StandardWorkflowPlanner.generate_plan(&ctx, "test-enforce-order");

        assert_eq!(result.phases.len(), 2, "2 layers x 1 step (audit)");
        let mut layer0 = result.phases[0].domains.clone();
        layer0.sort();
        assert_eq!(layer0, vec!["documentation".to_string(), "external-context".to_string()]);
        assert_eq!(result.phases[1].domains, vec!["engineering".to_string()]);
        // engineering's phase gates on the external-context/documentation layer.
        assert_eq!(result.phases[1].dependencies, vec![result.phases[0].id.clone()]);
        assert_deps_resolve(&result.phases);
    }

    #[test]
    fn standard_workflow_planner_ignores_enforce_order_edges_outside_this_tier() {
        // An enforce_order edge between two domains that aren't both in the
        // tier being planned must not affect that tier's layering at all.
        let scenarios = vec![scenario("new", "no_documentation", 1, "audit")];
        let mut domains_by_tier = std::collections::HashMap::new();
        domains_by_tier.insert(1, vec!["infrastructure".to_string(), "security".to_string()]);
        // "engineering" isn't in tier 1 at all — this edge shouldn't apply.
        let relationships = vec![relationship("infrastructure", "engineering", true)];

        let ctx = standard_ctx_with_relationships("new", false, scenarios, domains_by_tier, relationships);
        let result = StandardWorkflowPlanner.generate_plan(&ctx, "test-cross-tier-edge");

        assert_eq!(result.phases.len(), 1, "one layer — the edge's target isn't in this tier");
        let mut domains = result.phases[0].domains.clone();
        domains.sort();
        assert_eq!(domains, vec!["infrastructure".to_string(), "security".to_string()]);
    }

    #[test]
    fn topological_layers_handles_a_cycle_without_looping_forever() {
        let domains = vec!["a".to_string(), "b".to_string()];
        let relationships = vec![relationship("a", "b", true), relationship("b", "a", true)];
        let layers = topological_layers(&domains, &relationships);
        let total: usize = layers.iter().map(|l| l.len()).sum();
        assert_eq!(total, 2, "both domains must still appear exactly once");
    }

    #[test]
    fn standard_workflow_planner_skips_steps_absent_for_this_tier() {
        // A tier with only audit+fix content (no generation scenario row).
        let scenarios = vec![
            scenario("existing", "has_documentation", 1, "audit"),
            scenario("existing", "has_documentation", 1, "fix"),
        ];
        let ctx = standard_ctx("existing", true, scenarios, std::collections::HashMap::new());
        let result = StandardWorkflowPlanner.generate_plan(&ctx, "test-standard-skip");
        assert_eq!(result.phases.len(), 2);
        assert_eq!(result.phases[0].phase_type, PhaseType::Audit);
        assert_eq!(result.phases[1].phase_type, PhaseType::Fix);
    }

    #[test]
    fn standard_workflow_planner_empty_without_standard_context() {
        let ctx = test_ctx(false); // ctx.standard is None
        let result = StandardWorkflowPlanner.generate_plan(&ctx, "test-empty");
        assert!(result.phases.is_empty());
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
