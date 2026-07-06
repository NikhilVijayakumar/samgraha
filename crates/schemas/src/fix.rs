use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanType {
    Documentation,
    Implementation,
    Configuration,
    Test,
    Build,
    Security,
}

impl PlanType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Documentation => "documentation",
            Self::Implementation => "implementation",
            Self::Configuration => "configuration",
            Self::Test => "test",
            Self::Build => "build",
            Self::Security => "security",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "documentation" => Some(Self::Documentation),
            "implementation" => Some(Self::Implementation),
            "configuration" => Some(Self::Configuration),
            "test" => Some(Self::Test),
            "build" => Some(Self::Build),
            "security" => Some(Self::Security),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FixPlanStatus {
    Draft,
    Review,
    Executing,
    Completed,
    RolledBack,
}

impl FixPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Review => "review",
            Self::Executing => "executing",
            Self::Completed => "completed",
            Self::RolledBack => "rolled_back",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "review" => Some(Self::Review),
            "executing" => Some(Self::Executing),
            "completed" => Some(Self::Completed),
            "rolled_back" => Some(Self::RolledBack),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FixStepStatus {
    Pending,
    Executing,
    Verified,
    Failed,
}

impl FixStepStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Executing => "executing",
            Self::Verified => "verified",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "executing" => Some(Self::Executing),
            "verified" => Some(Self::Verified),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    InProgress,
    Passed,
    Failed,
    NeedsHumanReview,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NeedsHumanReview => "needs_human_review",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "in_progress" => Some(Self::InProgress),
            "passed" => Some(Self::Passed),
            "failed" => Some(Self::Failed),
            "needs_human_review" => Some(Self::NeedsHumanReview),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixSession {
    pub id: Option<i64>,
    pub report_id: i64,
    pub report_type: String,
    pub criterion_id: String,
    pub finding_json: String,
    pub domain: String,
    pub plan_type: PlanType,
    pub target_file: Option<String>,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub status: SessionStatus,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixAttempt {
    pub id: Option<i64>,
    pub session_id: i64,
    pub attempt: i32,
    pub plan_id: Option<i64>,
    pub plan_type: PlanType,
    pub score: Option<f64>,
    pub check_scores: Option<HashMap<String, f64>>,
    pub passed: bool,
    pub error_message: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixPlan {
    pub id: Option<i64>,
    pub session_id: String,
    pub report_id: i64,
    pub criterion_id: String,
    pub domain: String,
    pub plan_type: PlanType,
    pub title: String,
    pub summary: String,
    pub prerequisites: Vec<String>,
    pub steps: Vec<PlanStep>,
    pub rollback_instructions: Option<String>,
    pub expected_checks: Vec<String>,
    pub status: FixPlanStatus,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanStep {
    pub id: Option<i64>,
    pub plan_id: Option<i64>,
    pub step_order: usize,
    pub action: String,
    pub target: String,
    pub rationale: String,
    pub detail: String,
    pub verification: String,
    pub rollback: Option<String>,
    pub status: FixStepStatus,
    pub verified_at: Option<String>,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionResult {
    pub success: bool,
    pub message: String,
    pub modified_files: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_type_round_trip() {
        for (s, expected) in &[
            ("documentation", PlanType::Documentation),
            ("implementation", PlanType::Implementation),
            ("configuration", PlanType::Configuration),
            ("test", PlanType::Test),
            ("build", PlanType::Build),
            ("security", PlanType::Security),
        ] {
            let parsed = PlanType::from_str(s).unwrap();
            assert_eq!(parsed, *expected);
            assert_eq!(parsed.as_str(), *s);
        }
        assert!(PlanType::from_str("unknown").is_none());
    }

    #[test]
    fn fix_plan_status_round_trip() {
        for (s, expected) in &[
            ("draft", FixPlanStatus::Draft),
            ("review", FixPlanStatus::Review),
            ("executing", FixPlanStatus::Executing),
            ("completed", FixPlanStatus::Completed),
            ("rolled_back", FixPlanStatus::RolledBack),
        ] {
            let parsed = FixPlanStatus::from_str(s).unwrap();
            assert_eq!(parsed, *expected);
            assert_eq!(parsed.as_str(), *s);
        }
        assert!(FixPlanStatus::from_str("unknown").is_none());
    }

    #[test]
    fn step_status_round_trip() {
        for (s, expected) in &[
            ("pending", FixStepStatus::Pending),
            ("executing", FixStepStatus::Executing),
            ("verified", FixStepStatus::Verified),
            ("failed", FixStepStatus::Failed),
        ] {
            let parsed = FixStepStatus::from_str(s).unwrap();
            assert_eq!(parsed, *expected);
            assert_eq!(parsed.as_str(), *s);
        }
        assert!(FixStepStatus::from_str("unknown").is_none());
    }

    #[test]
    fn session_status_round_trip() {
        for (s, expected) in &[
            ("in_progress", SessionStatus::InProgress),
            ("passed", SessionStatus::Passed),
            ("failed", SessionStatus::Failed),
            ("needs_human_review", SessionStatus::NeedsHumanReview),
        ] {
            let parsed = SessionStatus::from_str(s).unwrap();
            assert_eq!(parsed, *expected);
            assert_eq!(parsed.as_str(), *s);
        }
        assert!(SessionStatus::from_str("unknown").is_none());
    }

    #[test]
    fn fix_session_serialize_round_trip() {
        let s = FixSession {
            id: Some(1),
            report_id: 10,
            report_type: "build".into(),
            criterion_id: "B1".into(),
            finding_json: "{}".into(),
            domain: "build".into(),
            plan_type: PlanType::Build,
            target_file: Some("Cargo.toml".into()),
            attempt_count: 0,
            max_attempts: 3,
            status: SessionStatus::InProgress,
            created_at: None,
            updated_at: None,
        };
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: FixSession = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, s);
    }

    #[test]
    fn fix_plan_serialize_round_trip() {
        let p = FixPlan {
            id: Some(1),
            session_id: "sess-1".into(),
            report_id: 10,
            criterion_id: "B1".into(),
            domain: "build".into(),
            plan_type: PlanType::Build,
            title: "Fix build".into(),
            summary: "Update Cargo.toml".into(),
            prerequisites: vec!["File writable".into()],
            steps: vec![PlanStep {
                id: Some(1),
                plan_id: Some(1),
                step_order: 1,
                action: "modify".into(),
                target: "Cargo.toml".into(),
                rationale: "Missing dep".into(),
                detail: "Add dep x".into(),
                verification: "Check compile".into(),
                rollback: Some("git checkout".into()),
                status: FixStepStatus::Pending,
                verified_at: None,
                score: None,
            }],
            rollback_instructions: Some("git checkout".into()),
            expected_checks: vec!["B1".into()],
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        };
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: FixPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, p);
    }

    #[test]
    fn execution_result_default_fields() {
        let r = ExecutionResult {
            success: true,
            message: "OK".into(),
            modified_files: vec!["a.rs".into()],
        };
        assert!(r.success);
        assert_eq!(r.modified_files.len(), 1);
    }
}
