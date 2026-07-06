use crate::KnowledgeRuntime;
use registry::RegistryStore;
use schemas::{PhaseStatus, PlanStatus, ProjectPhase, ProjectPlan, ProjectPlanWithPhases};
use std::sync::Arc;

use super::context::ProjectContext;
use anyhow::Result;
use super::executors::resolve_executor;
use super::planners::resolve_planner;

/// Orchestrates plan creation, loading, and phase execution.
pub struct PlanOrchestrator {
    runtime: Arc<KnowledgeRuntime>,
    registry: Arc<RegistryStore>,
}

impl PlanOrchestrator {
    pub fn new(runtime: Arc<KnowledgeRuntime>, registry: Arc<RegistryStore>) -> Self {
        Self { runtime, registry }
    }

    fn persist_plan(&self, plan: &ProjectPlan, phases: &[ProjectPhase]) -> Result<()> {
        self.registry.insert_plan(plan)?;
        for phase in phases {
            self.registry.insert_phase(phase)?;
        }
        Ok(())
    }

    /// Create a new plan for a given case.
    pub fn create_plan(&self, case: &schemas::ProjectCase, title: &str) -> Result<ProjectPlanWithPhases> {
        let ctx = ProjectContext::detect(&self.runtime.context.repository_root, case)?;
        let planner = resolve_planner(case);
        let result = planner.generate_plan(&ctx, title);
        self.persist_plan(&result.plan, &result.phases)?;
        Ok(result)
    }

    /// Get a plan with its phases.
    pub fn get_plan(&self, id: &str) -> Result<Option<ProjectPlanWithPhases>> {
        let plan = self.registry.get_plan(id)?;
        match plan {
            Some(p) => {
                let phases = self.registry.get_phases(id)?;
                Ok(Some(ProjectPlanWithPhases { plan: p, phases }))
            }
            None => Ok(None),
        }
    }

    /// List all plans.
    pub fn list_plans(&self) -> Result<Vec<ProjectPlan>> {
        self.registry.list_plans()
    }

    /// Get progress for a plan.
    pub fn get_progress(&self, id: &str) -> Result<Option<schemas::PlanProgress>> {
        let plan = self.registry.get_plan(id)?;
        match plan {
            Some(p) => {
                let phases = self.registry.get_phases(id)?;
                let total = phases.len();
                let completed = phases.iter().filter(|ph| ph.status == PhaseStatus::Completed).count();
                let mut by_status = std::collections::HashMap::new();
                for ph in &phases {
                    let key = format!("{:?}", ph.status);
                    *by_status.entry(key).or_insert(0) += 1;
                }
                Ok(Some(schemas::PlanProgress {
                    plan_id: p.id,
                    title: p.title,
                    case_type: p.case_type,
                    status: p.status,
                    total_phases: total,
                    completed_phases: completed,
                    current_phase: p.current_phase,
                    phases_by_status: by_status,
                }))
            }
            None => Ok(None),
        }
    }

    /// Abort a plan (mark as failed).
    pub fn abort_plan(&self, id: &str, _reason: &str) -> Result<()> {
        self.registry.update_plan_status(id, &PlanStatus::Failed)
    }

    /// Execute the next pending phase, or a specific phase.
    pub fn execute_phase(
        &self,
        plan_id: &str,
        phase_number: Option<u32>,
    ) -> Result<serde_json::Value> {
        let plan = self.registry.get_plan(plan_id)?
            .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

        if plan.status != PlanStatus::Active {
            anyhow::bail!("Plan is not active (status: {:?})", plan.status);
        }

        let phases = self.registry.get_phases(plan_id)?;

        let phase = if let Some(num) = phase_number {
            phases.iter().find(|p| p.phase_number == num)
                .ok_or_else(|| anyhow::anyhow!("Phase {} not found", num))?
        } else {
            phases.iter().find(|p| p.status == PhaseStatus::Pending)
                .ok_or_else(|| anyhow::anyhow!("No pending phases"))?
        };

        // Check dependencies
        for dep_id in &phase.dependencies {
            let dep_phase = phases.iter().find(|p| p.id == *dep_id);
            match dep_phase {
                Some(dep) if dep.status != PhaseStatus::Completed => {
                    anyhow::bail!(
                        "Dependency phase '{}' ({}) not completed",
                        dep.name, dep.id
                    );
                }
                None => {
                    anyhow::bail!("Dependency phase '{}' not found in plan", dep_id);
                }
                _ => {}
            }
        }

        // Mark in-progress — atomic conditional UPDATE so two overlapping
        // execute_phase calls on the same plan can't both start this phase.
        if !self.registry.try_start_phase(&phase.id)? {
            anyhow::bail!(
                "Phase '{}' ({}) is no longer pending — already started by a concurrent call",
                phase.name, phase.id
            );
        }
        self.registry.update_plan_current_phase(plan_id, &phase.id)?;

        // Execute
        let executor = resolve_executor(&phase.phase_type);
        let result = executor.execute(phase, &self.runtime, &self.registry);

        match result {
            Ok(value) => {
                let result_json = serde_json::to_string(&value).unwrap_or_default();
                self.registry.update_phase_status(&phase.id, &PhaseStatus::Completed)?;
                self.registry.update_phase_result(&phase.id, &result_json)?;

                let updated_phases = self.registry.get_phases(plan_id)?;
                let all_done = updated_phases.iter().all(|p| {
                    p.status == PhaseStatus::Completed || p.status == PhaseStatus::Failed
                });
                if all_done {
                    self.registry.update_plan_status(plan_id, &PlanStatus::Completed)?;
                }

                Ok(serde_json::json!({
                    "status": "completed",
                    "phase": phase.phase_number,
                    "phase_name": phase.name,
                    "result": value,
                }))
            }
            Err(e) => {
                self.registry.update_phase_status(&phase.id, &PhaseStatus::Failed)?;

                for p in &phases {
                    if p.dependencies.contains(&phase.id) {
                        self.registry.update_phase_status(&p.id, &PhaseStatus::Blocked)?;
                    }
                }

                Ok(serde_json::json!({
                    "status": "failed",
                    "phase": phase.phase_number,
                    "phase_name": phase.name,
                    "error": e.to_string(),
                }))
            }
        }
    }
}
