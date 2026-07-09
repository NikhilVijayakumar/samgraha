use crate::fix::types::{
    FixPlan, FixPlanStatus, FixSession, Intent, PlanStep, PlanType, PlanningContext,
};
use anyhow::Result;
use schemas::audit::{AuditFinding, PipelineKind};

pub trait FixPlanner {
    fn domain(&self) -> PipelineKind;
    fn plan_type(&self) -> PlanType;
    fn plan(
        &self,
        ctx: &PlanningContext,
        intent: &Intent,
        session: &FixSession,
    ) -> Result<FixPlan>;
}

pub struct DocPlanner;

impl FixPlanner for DocPlanner {
    fn domain(&self) -> PipelineKind {
        PipelineKind::Doc
    }

    fn plan_type(&self) -> PlanType {
        PlanType::Documentation
    }

    // ponytail: deterministic extraction, not content synthesis — this
    // quotes and structures the real check requirement so the plan is
    // finding-specific, but it does not author compliant prose. A human
    // still writes the actual content for qualitative checks. Upgrade path
    // is LLM-based generation if auto-pass rates matter later.
    fn plan(
        &self,
        ctx: &PlanningContext,
        intent: &Intent,
        _session: &FixSession,
    ) -> Result<FixPlan> {
        let target_path = ctx.target_path.to_string_lossy().to_string();
        let target_content = &ctx.target_content;
        let requirement = ctx.check_requirement(intent.check_id());

        let mut steps = Vec::new();
        steps.push(PlanStep {
            id: None,
            plan_id: None,
            step_order: 1,
            action: "add_section".into(),
            target: target_path.clone(),
            rationale: requirement.clone().unwrap_or_else(|| {
                "Document standard requires sections missing from target".into()
            }),
            detail: match &requirement {
                Some(req) => format!(
                    "Satisfy this requirement in {}:\n\n{}\n\nCurrent content:\n{}",
                    target_path,
                    req,
                    truncate(target_content, 2000)
                ),
                None => format!(
                    "Review audit standard and doc standard for required sections, \
                     then add or update sections in {} to match.\n\n\
                     Current content:\n{}",
                    target_path,
                    truncate(target_content, 2000)
                ),
            },
            verification: "Re-run audit checks for this domain".into(),
            rollback: Some("git checkout the file before the fix".into()),
            status: crate::fix::types::FixStepStatus::Pending,
            verified_at: None,
            score: None,
        });
        steps.push(PlanStep {
            id: None,
            plan_id: None,
            step_order: 2,
            action: "verify_sections".into(),
            target: target_path.clone(),
            rationale: "Ensure all required sections are present with correct content".into(),
            detail: "Verify each required section header and content block exists".into(),
            verification: "Re-run audit checks for this domain".into(),
            rollback: None,
            status: crate::fix::types::FixStepStatus::Pending,
            verified_at: None,
            score: None,
        });

        let plan_type = self.plan_type();
        Ok(FixPlan {
            id: None,
            session_id: String::new(),
            report_id: 0,
            criterion_id: String::new(),
            domain: ctx.domain().to_string(),
            plan_type,
            title: format!("Documentation fix for {}", ctx.domain()),
            summary: format!(
                "Apply documentation standard requirements to {}",
                ctx.domain()
            ),
            prerequisites: vec![format!("Target file {} is writable", target_path)],
            steps,
            rollback_instructions: Some("Use git checkout to revert changes".into()),
            expected_checks: Vec::new(),
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        })
    }
}

pub struct ConfigPlanner;

impl FixPlanner for ConfigPlanner {
    fn domain(&self) -> PipelineKind {
        PipelineKind::Dependency
    }

    fn plan_type(&self) -> PlanType {
        PlanType::Configuration
    }

    fn plan(
        &self,
        ctx: &PlanningContext,
        intent: &Intent,
        _session: &FixSession,
    ) -> Result<FixPlan> {
        let target_path = ctx.target_path.to_string_lossy().to_string();
        let requirement = ctx.check_requirement(intent.check_id());

        let steps = vec![PlanStep {
            id: None,
            plan_id: None,
            step_order: 1,
            action: "modify_value".into(),
            target: target_path.clone(),
            rationale: requirement.clone().unwrap_or_else(|| {
                "Configuration key missing or incorrect per audit finding".into()
            }),
            detail: match &requirement {
                Some(req) => format!("Satisfy this requirement:\n\n{}", req),
                None => "Update the target config file to match documented standards".into(),
            },
            verification: "Re-run audit checks for this domain".into(),
            rollback: Some("git checkout the file before the fix".into()),
            status: crate::fix::types::FixStepStatus::Pending,
            verified_at: None,
            score: None,
        }];

        let plan_type = self.plan_type();
        Ok(FixPlan {
            id: None,
            session_id: String::new(),
            report_id: 0,
            criterion_id: String::new(),
            domain: ctx.domain().to_string(),
            plan_type,
            title: format!("Configuration fix for {}", ctx.domain()),
            summary: format!(
                "Apply configuration standard requirements to {}",
                ctx.domain()
            ),
            prerequisites: vec![format!("Target file {} is writable", target_path)],
            steps,
            rollback_instructions: Some("Use git checkout to revert changes".into()),
            expected_checks: Vec::new(),
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        })
    }
}

pub struct ImplPlanner;

impl FixPlanner for ImplPlanner {
    fn domain(&self) -> PipelineKind {
        PipelineKind::Implementation
    }

    fn plan_type(&self) -> PlanType {
        PlanType::Implementation
    }

    fn plan(
        &self,
        ctx: &PlanningContext,
        intent: &Intent,
        _session: &FixSession,
    ) -> Result<FixPlan> {
        let target_path = ctx.target_path.to_string_lossy().to_string();
        let plan_type = self.plan_type();
        let requirement = ctx.check_requirement(intent.check_id());
        Ok(FixPlan {
            id: None,
            session_id: String::new(),
            report_id: 0,
            criterion_id: String::new(),
            domain: ctx.domain().to_string(),
            plan_type,
            title: format!("Implementation fix for {}", ctx.domain()),
            summary: format!(
                "Phasewise implementation plan for {}",
                ctx.domain()
            ),
            prerequisites: vec![
                "Source code is checked out and writable".into(),
                format!("Target: {}", target_path),
            ],
            steps: vec![PlanStep {
                id: None,
                plan_id: None,
                step_order: 1,
                action: "implement_function".into(),
                target: target_path,
                rationale: requirement.clone().unwrap_or_else(|| {
                    "Finding indicates missing implementation per domain standards".into()
                }),
                detail: match &requirement {
                    Some(req) => format!("Implement the required code changes to satisfy:\n\n{}", req),
                    None => "Implement the required code changes as described in the audit finding".into(),
                },
                verification: "Re-run audit checks for this domain".into(),
                rollback: Some("git checkout before changes".into()),
                status: crate::fix::types::FixStepStatus::Pending,
                verified_at: None,
                score: None,
            }],
            rollback_instructions: Some("Use git checkout to revert changes".into()),
            expected_checks: Vec::new(),
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        })
    }
}

pub struct BuildPlanner;

impl FixPlanner for BuildPlanner {
    fn domain(&self) -> PipelineKind {
        PipelineKind::Build
    }

    fn plan_type(&self) -> PlanType {
        PlanType::Build
    }

    fn plan(
        &self,
        ctx: &PlanningContext,
        intent: &Intent,
        _session: &FixSession,
    ) -> Result<FixPlan> {
        let target_path = ctx.target_path.to_string_lossy().to_string();
        let plan_type = self.plan_type();
        let requirement = ctx.check_requirement(intent.check_id());
        Ok(FixPlan {
            id: None,
            session_id: String::new(),
            report_id: 0,
            criterion_id: String::new(),
            domain: ctx.domain().to_string(),
            plan_type,
            title: format!("Build config fix for {}", ctx.domain()),
            summary: format!("Phasewise build configuration plan for {}", ctx.domain()),
            prerequisites: vec![
                "Build configuration files are writable".into(),
                format!("Target: {}", target_path),
            ],
            steps: vec![PlanStep {
                id: None,
                plan_id: None,
                step_order: 1,
                action: "modify_build_config".into(),
                target: target_path,
                rationale: requirement.clone().unwrap_or_else(|| {
                    "Build audit finding requires configuration change".into()
                }),
                detail: match &requirement {
                    Some(req) => format!("Update build configuration to satisfy:\n\n{}", req),
                    None => "Update build configuration to satisfy audit requirements".into(),
                },
                verification: "Re-run build audit checks".into(),
                rollback: Some("git checkout before changes".into()),
                status: crate::fix::types::FixStepStatus::Pending,
                verified_at: None,
                score: None,
            }],
            rollback_instructions: Some("Use git checkout to revert changes".into()),
            expected_checks: Vec::new(),
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        })
    }
}

pub struct SecPlanner;

impl FixPlanner for SecPlanner {
    fn domain(&self) -> PipelineKind {
        PipelineKind::Security
    }

    fn plan_type(&self) -> PlanType {
        PlanType::Security
    }

    fn plan(
        &self,
        ctx: &PlanningContext,
        intent: &Intent,
        _session: &FixSession,
    ) -> Result<FixPlan> {
        let target_path = ctx.target_path.to_string_lossy().to_string();
        let plan_type = self.plan_type();
        let requirement = ctx.check_requirement(intent.check_id());
        Ok(FixPlan {
            id: None,
            session_id: String::new(),
            report_id: 0,
            criterion_id: String::new(),
            domain: ctx.domain().to_string(),
            plan_type,
            title: format!("Security fix for {}", ctx.domain()),
            summary: format!("Phasewise security remediation plan for {}", ctx.domain()),
            prerequisites: vec![
                "Security configuration files are writable".into(),
                format!("Target: {}", target_path),
            ],
            steps: vec![PlanStep {
                id: None,
                plan_id: None,
                step_order: 1,
                action: "add_security_check".into(),
                target: target_path,
                rationale: requirement.clone().unwrap_or_else(|| {
                    "Security audit finding requires mitigation".into()
                }),
                detail: match &requirement {
                    Some(req) => format!("Implement the security remediation to satisfy:\n\n{}", req),
                    None => "Implement the security remediation as described in the audit finding".into(),
                },
                verification: "Re-run security audit checks".into(),
                rollback: Some("git checkout before changes".into()),
                status: crate::fix::types::FixStepStatus::Pending,
                verified_at: None,
                score: None,
            }],
            rollback_instructions: Some("Use git checkout to revert changes".into()),
            expected_checks: Vec::new(),
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        })
    }
}

pub struct TestPlanner;

impl FixPlanner for TestPlanner {
    fn domain(&self) -> PipelineKind {
        PipelineKind::Doc
    }

    fn plan_type(&self) -> PlanType {
        PlanType::Test
    }

    fn plan(
        &self,
        ctx: &PlanningContext,
        intent: &Intent,
        session: &FixSession,
    ) -> Result<FixPlan> {
        let plan_type = self.plan_type();
        let requirement = ctx.check_requirement(intent.check_id());
        // The finding carries the real, specific failure (which test, what
        // message) — `session.finding_json` is always populated by the
        // caller (generate_fix_plan / FixOrchestrator::execute), so a real
        // CV6/I8 finding turns this into "fix failing test X" instead of
        // the generic "add test cases" fallback used when there's no
        // finding-level detail to work with (e.g. audit is a stub result).
        let finding: Option<AuditFinding> = serde_json::from_str(&session.finding_json).ok();

        let target_path = finding
            .as_ref()
            .and_then(|f| f.location.clone())
            .unwrap_or_else(|| ctx.target_path.to_string_lossy().to_string());

        let (rationale, detail) = match finding.as_ref().filter(|f| !f.message.is_empty()) {
            Some(f) => (
                f.message.clone(),
                format!(
                    "Fix the specific failure reported by the audit finding:\n\n{}",
                    f.message
                ),
            ),
            None => (
                requirement.clone().unwrap_or_else(|| {
                    "Coverage audit finding requires additional test coverage".into()
                }),
                match &requirement {
                    Some(req) => format!("Implement test cases to satisfy:\n\n{}", req),
                    None => "Implement test cases as described in the audit finding".into(),
                },
            ),
        };

        Ok(FixPlan {
            id: None,
            session_id: String::new(),
            report_id: 0,
            criterion_id: String::new(),
            domain: ctx.domain().to_string(),
            plan_type,
            title: format!("Test plan for {}", ctx.domain()),
            summary: format!("Phasewise test addition plan for {}", ctx.domain()),
            prerequisites: vec![
                "Test directory exists and is writable".into(),
                format!("Target: {}", target_path),
            ],
            steps: vec![PlanStep {
                id: None,
                plan_id: None,
                step_order: 1,
                action: "add_test_case".into(),
                target: target_path,
                rationale,
                detail,
                verification: "Re-run coverage audit checks".into(),
                rollback: Some("git checkout before changes".into()),
                status: crate::fix::types::FixStepStatus::Pending,
                verified_at: None,
                score: None,
            }],
            rollback_instructions: Some("Use git checkout to revert changes".into()),
            expected_checks: Vec::new(),
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        })
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}... (truncated)", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::planning_context::PlanningContextBuilder;
    use crate::fix::types::SessionStatus;

    fn test_ctx(domain: &str) -> PlanningContext {
        test_ctx_with_spec(domain, "# Test Audit Spec")
    }

    fn test_ctx_with_spec(domain: &str, spec_body: &str) -> PlanningContext {
        let tmp = std::env::temp_dir().join(format!("sg_planner_test_{}", domain));
        let audit_dir = tmp.join("docs/raw/audit");
        std::fs::create_dir_all(&audit_dir).unwrap();
        std::fs::write(audit_dir.join(format!("{}-audit.md", domain)), spec_body).unwrap();
        let f = tmp.join("target.md");
        std::fs::write(&f, "# Test").unwrap();
        PlanningContextBuilder::new(tmp.clone())
            .build(domain, &f)
            .unwrap()
    }

    fn assert_valid_plan(plan: &FixPlan, expected_type: PlanType) {
        assert_eq!(plan.plan_type, expected_type);
        assert!(!plan.title.is_empty());
        assert!(!plan.summary.is_empty());
        assert!(!plan.steps.is_empty());
        for step in &plan.steps {
            assert!(!step.action.is_empty());
            assert!(!step.target.is_empty());
            assert!(!step.verification.is_empty());
        }
    }

    #[test]
    fn doc_planner_creates_plan() {
        let ctx = test_ctx("doc");
        let intent = Intent::restore_compliance("doc", "D1");
        let session = FixSession {
            id: None, report_id: 1, report_type: "deterministic".into(),
            criterion_id: "D1".into(), finding_json: "{}".into(),
            domain: "doc".into(), plan_type: PlanType::Documentation,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = DocPlanner.plan(&ctx, &intent, &session).unwrap();
        assert_valid_plan(&plan, PlanType::Documentation);
        assert_eq!(plan.steps.len(), 2);
    }

    #[test]
    fn doc_planner_uses_real_check_requirement_when_present() {
        let ctx = test_ctx_with_spec(
            "extraction-domain",
            "# Spec\n\n## A1. Real Requirement Title\n\nDetailed body text for A1.\n\n## A2. Other\n\nOther body.\n",
        );
        let intent = Intent::restore_compliance("extraction-domain", "A1");
        let session = FixSession {
            id: None, report_id: 1, report_type: "deterministic".into(),
            criterion_id: "A1".into(), finding_json: "{}".into(),
            domain: "extraction-domain".into(), plan_type: PlanType::Documentation,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = DocPlanner.plan(&ctx, &intent, &session).unwrap();
        assert!(plan.steps[0].rationale.contains("Real Requirement Title"));
        assert!(plan.steps[0].detail.contains("Detailed body text for A1."));
        assert!(!plan.steps[0].detail.contains("Other body."));
        // Old generic sentence must not appear once a real requirement is found.
        assert!(!plan.steps[0].detail.contains("Review audit standard and doc standard"));
    }

    #[test]
    fn config_planner_creates_plan() {
        let ctx = test_ctx("dependency");
        let intent = Intent::restore_compliance("dependency", "C1");
        let session = FixSession {
            id: None, report_id: 1, report_type: "build".into(),
            criterion_id: "C1".into(), finding_json: "{}".into(),
            domain: "dependency".into(), plan_type: PlanType::Configuration,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = ConfigPlanner.plan(&ctx, &intent, &session).unwrap();
        assert_valid_plan(&plan, PlanType::Configuration);
    }

    #[test]
    fn impl_planner_creates_plan() {
        let ctx = test_ctx("implementation");
        let intent = Intent::restore_compliance("implementation", "I1");
        let session = FixSession {
            id: None, report_id: 1, report_type: "implementation".into(),
            criterion_id: "I1".into(), finding_json: "{}".into(),
            domain: "implementation".into(), plan_type: PlanType::Implementation,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = ImplPlanner.plan(&ctx, &intent, &session).unwrap();
        assert_valid_plan(&plan, PlanType::Implementation);
    }

    #[test]
    fn build_planner_creates_plan() {
        let ctx = test_ctx("build");
        let intent = Intent::restore_compliance("build", "B1");
        let session = FixSession {
            id: None, report_id: 1, report_type: "build".into(),
            criterion_id: "B1".into(), finding_json: "{}".into(),
            domain: "build".into(), plan_type: PlanType::Build,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = BuildPlanner.plan(&ctx, &intent, &session).unwrap();
        assert_valid_plan(&plan, PlanType::Build);
    }

    #[test]
    fn sec_planner_creates_plan() {
        let ctx = test_ctx("security");
        let intent = Intent::restore_compliance("security", "S1");
        let session = FixSession {
            id: None, report_id: 1, report_type: "security".into(),
            criterion_id: "S1".into(), finding_json: "{}".into(),
            domain: "security".into(), plan_type: PlanType::Security,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = SecPlanner.plan(&ctx, &intent, &session).unwrap();
        assert_valid_plan(&plan, PlanType::Security);
    }

    #[test]
    fn test_planner_creates_plan() {
        let ctx = test_ctx("test");
        let intent = Intent::restore_compliance("test", "T1");
        let session = FixSession {
            id: None, report_id: 1, report_type: "coverage".into(),
            criterion_id: "T1".into(), finding_json: "{}".into(),
            domain: "test".into(), plan_type: PlanType::Test,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = TestPlanner.plan(&ctx, &intent, &session).unwrap();
        assert_valid_plan(&plan, PlanType::Test);
    }

    #[test]
    fn test_planner_uses_real_finding_message_when_present() {
        // Regression: TestPlanner used to ignore the session entirely and
        // always produce the generic "add test cases" template, even when
        // the finding carried a specific failing-test message.
        let ctx = test_ctx("coverage");
        let intent = Intent::restore_compliance("coverage", "CV6");
        let finding = schemas::audit::AuditFinding {
            check_id: "CV6".into(),
            severity: schemas::audit::Severity::Warning,
            message: "Tests failing: unit 2/3 passed — unit:test_foo".into(),
            location: Some("unit:test_foo".into()),
            document_id: None,
            provider: "pipeline".into(),
            stage: None,
            section_id: None,
            confidence: None,
            evidence: None,
            status: None,
            strategy: None,
        };
        let session = FixSession {
            id: None, report_id: 1, report_type: "coverage".into(),
            criterion_id: "CV6".into(),
            finding_json: serde_json::to_string(&finding).unwrap(),
            domain: "coverage".into(), plan_type: PlanType::Test,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let plan = TestPlanner.plan(&ctx, &intent, &session).unwrap();
        assert!(plan.steps[0].rationale.contains("test_foo"));
        assert_eq!(plan.steps[0].target, "unit:test_foo");
    }

    #[test]
    fn intent_restore_compliance_defaults() {
        let i = Intent::restore_compliance("build", "B1");
        match i {
            Intent::RestoreCompliance { domain, check_id, target_score } => {
                assert_eq!(domain, "build");
                assert_eq!(check_id, "B1");
                assert_eq!(target_score, 9.0);
            }
        }
    }
}
