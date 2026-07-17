use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_case_roundtrip() {
        let cases = vec![
            ProjectCase::NewProject,
            ProjectCase::DocAudit,
            ProjectCase::ImplTestAudit,
            ProjectCase::BuildAudit,
        ];
        for c in cases {
            let s = c.to_string();
            let back = ProjectCase::from_str(&s).unwrap();
            assert_eq!(c, back);
        }
    }

    #[test]
    fn test_phase_type_roundtrip() {
        let types = vec![
            PhaseType::Generate,
            PhaseType::Audit,
            PhaseType::Fix,
            PhaseType::Verify,
        ];
        for t in types {
            let s = t.to_string();
            let back = PhaseType::from_str(&s).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_plan_status_roundtrip() {
        let statuses = vec![
            PlanStatus::Active,
            PlanStatus::Completed,
            PlanStatus::Failed,
        ];
        for s in statuses {
            let str = s.to_string();
            let back = PlanStatus::from_str(&str).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn test_phase_status_roundtrip() {
        let statuses = vec![
            PhaseStatus::Pending,
            PhaseStatus::InProgress,
            PhaseStatus::Completed,
            PhaseStatus::Failed,
            PhaseStatus::Blocked,
        ];
        for s in statuses {
            let str = s.to_string();
            let back = PhaseStatus::from_str(&str).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn test_plan_progress_serialization() {
        let progress = PlanProgress {
            plan_id: "p1".into(),
            title: "Test".into(),
            case_type: ProjectCase::DocAudit,
            status: PlanStatus::Active,
            total_phases: 5,
            completed_phases: 2,
            current_phase: Some("phase-3".into()),
            phases_by_status: [("completed".into(), 2usize), ("pending".into(), 3)].into(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("plan_id"));
        assert!(json.contains("total_phases"));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCase {
    NewProject,
    DocAudit,
    ImplTestAudit,
    BuildAudit,
    /// Generate/audit/fix phases sourced from a registered standard's own
    /// `plan_scenarios` (tier x repo_state x doc_state x step content),
    /// instead of the fixed `DOC_DOMAINS`/`IMPL_DOMAINS` const arrays the
    /// other 4 cases use — see `StandardWorkflowPlanner`.
    Standard,
}

impl std::fmt::Display for ProjectCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewProject => write!(f, "new_project"),
            Self::DocAudit => write!(f, "docs_audit"),
            Self::ImplTestAudit => write!(f, "impl_test_audit"),
            Self::BuildAudit => write!(f, "build_audit"),
            Self::Standard => write!(f, "standard"),
        }
    }
}

impl ProjectCase {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "new_project" => Some(Self::NewProject),
            "docs_audit" => Some(Self::DocAudit),
            "impl_test_audit" => Some(Self::ImplTestAudit),
            "build_audit" => Some(Self::BuildAudit),
            "standard" => Some(Self::Standard),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhaseType {
    Generate,
    Audit,
    Fix,
    Verify,
}

impl std::fmt::Display for PhaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generate => write!(f, "generate"),
            Self::Audit => write!(f, "audit"),
            Self::Fix => write!(f, "fix"),
            Self::Verify => write!(f, "verify"),
        }
    }
}

impl PhaseType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "generate" => Some(Self::Generate),
            "audit" => Some(Self::Audit),
            "fix" => Some(Self::Fix),
            "verify" => Some(Self::Verify),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Active,
    Completed,
    Failed,
}

impl std::fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl PlanStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    /// Not yet run. Dependencies may still be incomplete but could still
    /// succeed — this phase remains eligible for `execute_phase` once they do.
    Pending,
    InProgress,
    Completed,
    Failed,
    /// A direct dependency `Failed`, so this phase can never run. Terminal —
    /// nothing transitions a phase back out of `Blocked`.
    Blocked,
}

impl std::fmt::Display for PhaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

impl PhaseStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPlan {
    pub id: String,
    pub title: String,
    pub case_type: ProjectCase,
    pub status: PlanStatus,
    pub current_phase: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPhase {
    pub id: String,
    pub plan_id: String,
    pub phase_number: u32,
    pub name: String,
    pub phase_type: PhaseType,
    pub domains: Vec<String>,
    pub pipeline_ids: Vec<String>,
    pub dependencies: Vec<String>,
    pub status: PhaseStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub result_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPlanWithPhases {
    pub plan: ProjectPlan,
    pub phases: Vec<ProjectPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanProgress {
    pub plan_id: String,
    pub title: String,
    pub case_type: ProjectCase,
    pub status: PlanStatus,
    pub total_phases: usize,
    pub completed_phases: usize,
    pub current_phase: Option<String>,
    pub phases_by_status: std::collections::HashMap<String, usize>,
}
