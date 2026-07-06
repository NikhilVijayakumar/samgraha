use crate::fix::executor::executor_for_plan;
use crate::fix::planning_context::PlanningContextBuilder;
use crate::fix::types::{
    ExecutorKind, FixAttempt, FixPlan, FixPlanStatus, FixSession, Intent, PlanType, SessionStatus,
};
use crate::fix::verifier::Verifier;
use anyhow::{Context, Result};
use schemas::audit::AuditFinding;
use std::path::PathBuf;

type StoreMethods = Box<dyn FixStore>;

pub trait FixStore: Send {
    fn insert_session(&self, session: &FixSession) -> Result<i64>;
    fn update_session(&self, session: &FixSession) -> Result<()>;
    fn get_session(&self, id: i64) -> Result<Option<FixSession>>;
    fn insert_plan(&self, plan: &FixPlan) -> Result<i64>;
    fn get_plan(&self, id: i64) -> Result<Option<FixPlan>>;
    fn update_plan_status(&self, plan_id: i64, status: &FixPlanStatus) -> Result<()>;
    fn insert_step(&self, step: &crate::fix::types::PlanStep) -> Result<i64>;
    fn update_step(&self, step: &crate::fix::types::PlanStep) -> Result<()>;
    fn get_steps(&self, plan_id: i64) -> Result<Vec<crate::fix::types::PlanStep>>;
    fn insert_attempt(&self, attempt: &FixAttempt) -> Result<i64>;
    fn get_attempts(&self, session_id: i64) -> Result<Vec<FixAttempt>>;
}

pub struct FixOrchestrator {
    context_builder: PlanningContextBuilder,
    planners: Vec<Box<dyn crate::fix::planner::FixPlanner>>,
    verifier: Verifier,
    store: StoreMethods,
    max_attempts: i32,
}

impl FixOrchestrator {
    pub fn new(
        repo_root: PathBuf,
        planners: Vec<Box<dyn crate::fix::planner::FixPlanner>>,
        verifier: Verifier,
        store: StoreMethods,
    ) -> Self {
        Self {
            context_builder: PlanningContextBuilder::new(repo_root),
            planners,
            verifier,
            store,
            max_attempts: 3,
        }
    }

    pub fn execute(
        &self,
        finding: &AuditFinding,
        domain: &str,
        report_id: i64,
        report_type: &str,
        target_path: &PathBuf,
    ) -> Result<FixSession> {
        let mut pctx = self
            .context_builder
            .build(domain, target_path)
            .context("Failed to build planning context")?;

        let intent = Intent::restore_compliance(domain, &finding.check_id);

        let mut session = FixSession {
            id: None,
            report_id,
            report_type: report_type.to_string(),
            criterion_id: finding.check_id.clone(),
            finding_json: serde_json::to_string(finding)?,
            domain: domain.to_string(),
            plan_type: Self::resolve_plan_type(domain, &finding.check_id),
            target_file: Some(target_path.to_string_lossy().to_string()),
            attempt_count: 0,
            max_attempts: self.max_attempts,
            status: SessionStatus::InProgress,
            created_at: None,
            updated_at: None,
        };

        let session_id = self.store.insert_session(&session)?;
        session.id = Some(session_id);

        let planner = self
            .planners
            .iter()
            .find(|p| p.plan_type() == session.plan_type)
            .context(format!(
                "No planner found for plan type {:?}",
                session.plan_type
            ))?;

        for attempt in 0..self.max_attempts {
            let mut plan = planner
                .plan(&pctx, &intent, &session)
                .context("Failed to generate fix plan")?;
            // Planners don't know the session/finding IDs — fill them in
            // here so the persisted row (and executor markers keyed on
            // criterion_id) identify the right finding.
            plan.session_id = session_id.to_string();
            plan.criterion_id = finding.check_id.clone();

            let plan_id = self.store.insert_plan(&plan)?;
            plan.id = Some(plan_id);

            for step in &mut plan.steps {
                step.plan_id = Some(plan_id);
                self.store.insert_step(step)?;
            }

            let executor = executor_for_plan(&session.plan_type);
            let _exec_result = executor.execute(&plan)?;

            let check_ids: Vec<String> = plan.expected_checks.clone();
            let verdict = self
                .verifier
                .verify(domain, &if check_ids.is_empty() {
                    vec![finding.check_id.clone()]
                } else {
                    check_ids
                })
                .context("Verification failed")?;

            let attempt_record = FixAttempt {
                id: None,
                session_id,
                attempt,
                plan_id: Some(plan_id),
                plan_type: session.plan_type.clone(),
                score: Some(verdict.score),
                check_scores: Some(verdict.check_scores.clone()),
                passed: verdict.passed,
                error_message: if verdict.passed {
                    None
                } else {
                    Some(format!(
                        "Score {:.1}/10 — details: {}",
                        verdict.score,
                        verdict
                            .details
                            .iter()
                            .map(|d| format!("{}: {:.1}", d.check_id, d.score))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                },
                created_at: None,
            };
            self.store.insert_attempt(&attempt_record)?;

            if verdict.passed {
                session.status = SessionStatus::Passed;
                session.attempt_count = attempt + 1;
                self.store.update_session(&session)?;
                return Ok(session);
            }

            if executor.kind() == ExecutorKind::Doc || executor.kind() == ExecutorKind::Config {
                self.context_builder.invalidate(target_path);
                pctx = self
                    .context_builder
                    .build(domain, target_path)?;
            }

            for detail in &verdict.details {
                pctx.add_feedback(detail.clone());
            }

            session.attempt_count = attempt + 1;
        }

        session.status = SessionStatus::NeedsHumanReview;
        self.store.update_session(&session)?;
        Ok(session)
    }

    fn resolve_plan_type(domain: &str, check_id: &str) -> PlanType {
        // "dependency" intentionally absent: that pipeline is a stub (no real
        // checks), so callers reject it before a session is ever created —
        // see `KnowledgeRuntime::reject_stub_domain`.
        //
        // "coverage"+"CV6" is the one real Test-plan trigger: per
        // docs/raw/audit/coverage-audit.md, CV6 ("Documented Capabilities
        // Tested") is the only Coverage check whose fix is adding a test —
        // CV1-CV5/CV7 need code (out of scope) and CV8-CV15 are orphan
        // checks fixed by documenting, correctly staying on DocPlanner.
        match (domain, check_id) {
            ("coverage", "CV6") => PlanType::Test,
            ("build", _) => PlanType::Build,
            ("security", _) => PlanType::Security,
            ("implementation", _) | ("deterministic-runtime", _) => PlanType::Implementation,
            _ => PlanType::Documentation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_cv6_routes_to_test_plan() {
        assert_eq!(FixOrchestrator::resolve_plan_type("coverage", "CV6"), PlanType::Test);
    }

    #[test]
    fn coverage_other_checks_stay_documentation() {
        for cid in ["CV1", "CV5", "CV7", "CV8", "CV15"] {
            assert_eq!(
                FixOrchestrator::resolve_plan_type("coverage", cid),
                PlanType::Documentation,
                "check {} should stay Documentation",
                cid
            );
        }
    }

    #[test]
    fn other_domain_routing_unchanged() {
        assert_eq!(FixOrchestrator::resolve_plan_type("build", "BC1"), PlanType::Build);
        assert_eq!(FixOrchestrator::resolve_plan_type("security", "SC1"), PlanType::Security);
        assert_eq!(FixOrchestrator::resolve_plan_type("implementation", "I1"), PlanType::Implementation);
        assert_eq!(FixOrchestrator::resolve_plan_type("deterministic-runtime", "S1"), PlanType::Implementation);
        assert_eq!(FixOrchestrator::resolve_plan_type("architecture", "A1"), PlanType::Documentation);
    }
}
