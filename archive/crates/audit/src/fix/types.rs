pub use schemas::fix::{
    ExecutionResult, FixAttempt, FixPlan, FixPlanStatus, FixSession, FixStepStatus, PlanStep,
    PlanType, SessionStatus,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutorKind {
    Doc,
    Config,
    Plan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Verdict {
    pub score: f64,
    pub check_scores: HashMap<String, f64>,
    pub details: Vec<VerificationDetail>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationDetail {
    pub check_id: String,
    pub domain: String,
    pub score: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    RestoreCompliance {
        domain: String,
        check_id: String,
        target_score: f32,
    },
}

impl Intent {
    pub fn restore_compliance(domain: &str, check_id: &str) -> Self {
        Self::RestoreCompliance {
            domain: domain.to_string(),
            check_id: check_id.to_string(),
            target_score: 9.0,
        }
    }

    pub fn domain(&self) -> &str {
        match self {
            Self::RestoreCompliance { domain, .. } => domain,
        }
    }

    pub fn check_id(&self) -> &str {
        match self {
            Self::RestoreCompliance { check_id, .. } => check_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlanningContext {
    pub target_path: PathBuf,
    pub target_content: String,
    pub audit_spec: ParsedAuditSpec,
    pub audit_standard: ParsedAuditStandard,
    pub doc_standard: ParsedDocStandard,
    pub feedback: Vec<VerificationDetail>,
    pub domain: String,
}

impl PlanningContext {
    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn add_feedback(&mut self, detail: VerificationDetail) {
        self.feedback.push(detail);
    }

    /// Extract the specific check's own section from the audit spec, e.g.
    /// the `## A5. Some Title\n\n<body>` block for check_id "A5". Audit
    /// specs consistently use `## {check_id}. <Title>` headings followed by
    /// free text until the next `##`/`---` — this holds across all domains
    /// (confirmed in architecture, coverage, external-context, etc.).
    ///
    /// Returns `None` if the heading isn't found, so callers can fall back
    /// to generic text rather than erroring — audit spec formatting isn't
    /// type-checked, so this stays defensive.
    pub fn check_requirement(&self, check_id: &str) -> Option<String> {
        let lines: Vec<&str> = self.audit_spec.raw.lines().collect();
        let prefix = format!("## {}", check_id);
        let start = lines.iter().position(|line| {
            line.strip_prefix(&prefix)
                // Reject prefix collisions: "A1" must not match "## A11. ...".
                .map(|rest| !rest.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false))
                .unwrap_or(false)
        })?;
        let end = lines[start + 1..]
            .iter()
            .position(|line| {
                let t = line.trim_start();
                t.starts_with("## ") || t == "---"
            })
            .map(|rel| start + 1 + rel)
            .unwrap_or(lines.len());
        let section = lines[start..end].join("\n").trim().to_string();
        if section.is_empty() { None } else { Some(section) }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedAuditSpec {
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct ParsedAuditStandard {
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct ParsedDocStandard {
    pub raw: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_spec(raw: &str) -> PlanningContext {
        PlanningContext {
            target_path: PathBuf::new(),
            target_content: String::new(),
            audit_spec: ParsedAuditSpec { raw: raw.to_string() },
            audit_standard: ParsedAuditStandard { raw: String::new() },
            doc_standard: ParsedDocStandard { raw: String::new() },
            feedback: Vec::new(),
            domain: "test".into(),
        }
    }

    #[test]
    fn check_requirement_finds_matching_section() {
        let ctx = ctx_with_spec(
            "# Spec\n\n## A1. First Check\n\nBody of A1.\nMore text.\n\n## A2. Second Check\n\nBody of A2.\n",
        );
        let section = ctx.check_requirement("A1").unwrap();
        assert!(section.contains("First Check"));
        assert!(section.contains("Body of A1."));
        assert!(!section.contains("Second Check"));
        assert!(!section.contains("Body of A2."));
    }

    #[test]
    fn check_requirement_returns_none_when_missing() {
        let ctx = ctx_with_spec("# Spec\n\n## A1. First Check\n\nBody.\n");
        assert!(ctx.check_requirement("Z9").is_none());
    }

    #[test]
    fn check_requirement_does_not_match_prefix_collision() {
        let ctx = ctx_with_spec(
            "## A1. First Check\n\nBody of A1.\n\n## A11. Eleventh Check\n\nBody of A11.\n",
        );
        let section = ctx.check_requirement("A1").unwrap();
        assert!(section.contains("Body of A1."));
        assert!(!section.contains("Eleventh Check"));
        assert!(!section.contains("Body of A11."));
    }

    #[test]
    fn check_requirement_stops_at_horizontal_rule() {
        let ctx = ctx_with_spec("## A1. First Check\n\nBody.\n\n---\n\nUnrelated section.\n");
        let section = ctx.check_requirement("A1").unwrap();
        assert!(section.contains("Body."));
        assert!(!section.contains("Unrelated section."));
    }

    #[test]
    fn intent_check_id_accessor() {
        let i = Intent::restore_compliance("build", "B1");
        assert_eq!(i.check_id(), "B1");
    }
}
