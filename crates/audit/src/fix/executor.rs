use crate::fix::types::{ExecutorKind, FixPlan, PlanType};
use schemas::fix::ExecutionResult;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub trait Executor {
    fn kind(&self) -> ExecutorKind;
    fn execute(&self, plan: &FixPlan) -> Result<ExecutionResult>;
}

pub struct DocExecutor;

impl Executor for DocExecutor {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::Doc
    }

    fn execute(&self, plan: &FixPlan) -> Result<ExecutionResult> {
        if plan.steps.is_empty() {
            bail!("Doc plan has no steps");
        }
        for step in &plan.steps {
            let path = Path::new(&step.target);
            let content = if path.exists() {
                fs::read_to_string(path)
                    .context(format!("Failed to read target: {}", step.target))?
            } else {
                String::new()
            };
            let (open, close) = fix_marker("<!--", "-->", &plan.domain, &plan.criterion_id, step.step_order);
            let updated = apply_doc_step(&content, step, &open, &close);
            fs::write(path, &updated)
                .context(format!("Failed to write target: {}", step.target))?;
        }
        Ok(ExecutionResult {
            success: true,
            message: format!("Applied {} doc steps", plan.steps.len()),
            modified_files: plan.steps.iter().map(|s| s.target.clone()).collect(),
        })
    }
}

pub struct ConfigExecutor;

impl Executor for ConfigExecutor {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::Config
    }

    fn execute(&self, plan: &FixPlan) -> Result<ExecutionResult> {
        if plan.steps.is_empty() {
            bail!("Config plan has no steps");
        }
        for step in &plan.steps {
            let path = Path::new(&step.target);
            // JSON has no comment syntax — the marker-comment strategy below
            // would write invalid JSON, so refuse rather than corrupt it.
            // Structured JSON editing is follow-up work (see proposal notes
            // on toml_edit/serde_yaml — this executor still doesn't do real
            // key edits for any format, only an annotated comment).
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                bail!(
                    "ConfigExecutor cannot safely auto-write JSON target '{}' \
                     (no comment syntax to mark the fix) — needs_human_review instead",
                    step.target
                );
            }
            let content = if path.exists() {
                fs::read_to_string(path)
                    .context(format!("Failed to read config target: {}", step.target))?
            } else {
                String::new()
            };
            let (open, close) = fix_marker("#", "", &plan.domain, &plan.criterion_id, step.step_order);
            let updated = apply_config_step(&content, step, &open, &close);
            fs::write(path, &updated)
                .context(format!("Failed to write config target: {}", step.target))?;
        }
        Ok(ExecutionResult {
            success: true,
            message: format!("Applied {} config steps", plan.steps.len()),
            modified_files: plan.steps.iter().map(|s| s.target.clone()).collect(),
        })
    }
}

pub struct PlanExecutor;

impl Executor for PlanExecutor {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::Plan
    }

    fn execute(&self, plan: &FixPlan) -> Result<ExecutionResult> {
        // PlanExecutor does not auto-write anything.
        // The plan is already stored in SQLite by the caller.
        // Rendering via template engine happens in Phase 3.
        Ok(ExecutionResult {
            success: true,
            message: format!(
                "Plan of type {:?} recorded — user must execute steps manually",
                plan.plan_type
            ),
            modified_files: Vec::new(),
        })
    }
}

/// Build a stable open/close marker pair for one plan step. Stable across
/// retry attempts (same domain + finding + step_order every time a session
/// re-plans), so re-applying replaces the previous attempt's block instead
/// of appending another copy on every failed verification loop iteration.
fn fix_marker(prefix: &str, suffix: &str, domain: &str, criterion_id: &str, step_order: usize) -> (String, String) {
    let tag = format!("samgraha-fix:{}.{}.{}", domain, criterion_id, step_order);
    (format!("{} {} {}", prefix, tag, suffix).trim_end().to_string(),
     format!("{} /{} {}", prefix, tag, suffix).trim_end().to_string())
}

/// Replace the block between `open`/`close` markers if present, otherwise
/// append a new marked block. Makes repeated writes for the same step
/// idempotent instead of accumulating duplicate content on every retry.
fn upsert_marked_block(content: &str, open: &str, close: &str, body: &str) -> String {
    let block = format!("{}\n{}\n{}", open, body, close);
    if let Some(start) = content.find(open) {
        if let Some(close_rel) = content[start..].find(close) {
            let end = start + close_rel + close.len();
            return format!("{}{}{}", &content[..start], block, &content[end..]);
        }
    }
    if content.trim().is_empty() {
        block
    } else {
        format!("{}\n\n{}", content.trim_end(), block)
    }
}

fn apply_doc_step(content: &str, step: &crate::fix::types::PlanStep, open: &str, close: &str) -> String {
    match step.action.as_str() {
        "add_section" | "rewrite_section" | "update_table" => {
            // step.detail quotes the real check requirement (see
            // PlanningContext::check_requirement) but isn't authored prose —
            // mark it clearly as a stub needing human completion rather than
            // presenting planner instructions as if they were finished
            // document content.
            let body = format!(
                "<!-- TODO(samgraha-fix): complete this section -->\n\n{}\n\n{}",
                step.rationale, step.detail
            );
            upsert_marked_block(content, open, close, &body)
        }
        _ => content.to_string(),
    }
}

fn apply_config_step(content: &str, step: &crate::fix::types::PlanStep, open: &str, close: &str) -> String {
    match step.action.as_str() {
        "add_key" | "modify_value" | "add_table_entry" => {
            // Every body line must itself be a `#` comment — TOML/YAML have
            // no block-comment syntax, only the marker lines are guaranteed
            // comments.
            let body = format!("Fix applied: {}", step.detail)
                .lines()
                .map(|l| format!("# {}", l))
                .collect::<Vec<_>>()
                .join("\n");
            upsert_marked_block(content, open, close, &body)
        }
        _ => content.to_string(),
    }
}

/// Select the appropriate executor based on plan type.
pub fn executor_for_plan(plan_type: &PlanType) -> Box<dyn Executor> {
    match plan_type {
        PlanType::Documentation => Box::new(DocExecutor),
        PlanType::Configuration => Box::new(ConfigExecutor),
        PlanType::Help => Box::new(PlanExecutor),
        PlanType::Implementation
        | PlanType::Test
        | PlanType::Build
        | PlanType::Security => Box::new(PlanExecutor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::types::{FixPlanStatus, FixStepStatus, PlanStep};

    fn plan_with_step(target: &str, action: &str, detail: &str) -> FixPlan {
        FixPlan {
            id: None,
            session_id: "1".into(),
            report_id: 1,
            criterion_id: "A1".into(),
            domain: "architecture".into(),
            plan_type: PlanType::Documentation,
            title: "t".into(),
            summary: "s".into(),
            prerequisites: vec![],
            steps: vec![PlanStep {
                id: None,
                plan_id: None,
                step_order: 1,
                action: action.into(),
                target: target.into(),
                rationale: "r".into(),
                detail: detail.into(),
                verification: "v".into(),
                rollback: None,
                status: FixStepStatus::Pending,
                verified_at: None,
                score: None,
            }],
            rollback_instructions: None,
            expected_checks: vec![],
            status: FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn doc_executor_retry_is_idempotent_not_appended() {
        let tmp = std::env::temp_dir().join("sg_executor_test_doc_idempotent");
        std::fs::create_dir_all(&tmp).unwrap();
        let target = tmp.join("doc.md");
        std::fs::write(&target, "# Real Doc\n\nOriginal content.").unwrap();

        let plan = plan_with_step(target.to_str().unwrap(), "add_section", "first attempt detail");
        DocExecutor.execute(&plan).unwrap();
        let after_first = std::fs::read_to_string(&target).unwrap();

        // Simulate a second failed-verification retry for the SAME finding —
        // must replace the previous block, not append a second copy.
        let plan2 = plan_with_step(target.to_str().unwrap(), "add_section", "second attempt detail");
        DocExecutor.execute(&plan2).unwrap();
        let after_second = std::fs::read_to_string(&target).unwrap();

        assert!(after_first.contains("Original content."));
        assert!(after_second.contains("Original content."));
        assert!(after_second.contains("second attempt detail"));
        assert!(!after_second.contains("first attempt detail"), "stale attempt content must be replaced, not accumulated");
        // Only one marker pair present — no duplicate blocks.
        assert_eq!(after_second.matches("samgraha-fix:architecture.A1.1").count(), 2);
    }

    #[test]
    fn config_executor_refuses_json_target() {
        let tmp = std::env::temp_dir().join("sg_executor_test_config_json");
        std::fs::create_dir_all(&tmp).unwrap();
        let target = tmp.join("config.json");
        std::fs::write(&target, "{}").unwrap();

        let plan = plan_with_step(target.to_str().unwrap(), "add_key", "some detail");
        let result = ConfigExecutor.execute(&plan);
        assert!(result.is_err(), "JSON targets must be refused, not corrupted with comment syntax");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "{}", "JSON file must be untouched");
    }

    #[test]
    fn config_executor_retry_is_idempotent_and_valid_comments() {
        let tmp = std::env::temp_dir().join("sg_executor_test_config_idempotent");
        std::fs::create_dir_all(&tmp).unwrap();
        let target = tmp.join("config.toml");
        std::fs::write(&target, "[section]\nkey = \"value\"").unwrap();

        let mut plan = plan_with_step(target.to_str().unwrap(), "add_key", "attempt one");
        plan.domain = "dependency".into();
        ConfigExecutor.execute(&plan).unwrap();

        let mut plan2 = plan_with_step(target.to_str().unwrap(), "add_key", "attempt two");
        plan2.domain = "dependency".into();
        ConfigExecutor.execute(&plan2).unwrap();

        let after = std::fs::read_to_string(&target).unwrap();
        assert!(after.contains("key = \"value\""));
        assert!(after.contains("attempt two"));
        assert!(!after.contains("attempt one"));
        // Every non-marker, non-original line must be a `#` comment.
        for line in after.lines() {
            let l = line.trim();
            if l.is_empty() || l == "[section]" || l == "key = \"value\"" {
                continue;
            }
            assert!(l.starts_with('#'), "line '{}' is not a valid TOML comment", l);
        }
    }
}
