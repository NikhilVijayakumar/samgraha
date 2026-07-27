//! JSON Schema validation for standard metadata and proposal envelopes.
//! Two schemas live under `metadata/` at the samgraha repo root:
//! `standard.metadata.schema.json` (what a standard's own metadata.json
//! must look like) and `proposal.schema.json` (what a generated proposal's
//! phase-per-domain content must look like). These are the single source
//! of truth for shape validation — samgraha validates *shape* and
//! *referential completeness*, never content correctness.

use anyhow::{bail, Context, Result};
use std::path::Path;

// Both schemas are samgraha's own fixed assets — identical for every
// standard, never copied per-standard or per-repo — so they're embedded
// at compile time. A previous version resolved these by walking up from
// a runtime path (a target repo's `.samgraha/` tree, or a temp dir); that
// walk can never reach samgraha's own `metadata/` directory unless the
// repo being registered happens to be nested inside samgraha's own
// checkout — on Windows it can't even cross a drive letter. Embedding
// removes the lookup entirely: there is nothing to fail to find.
const STANDARD_METADATA_SCHEMA_JSON: &str =
    include_str!("../../../metadata/standard.metadata.schema.json");
const PROPOSAL_SCHEMA_JSON: &str = include_str!("../../../metadata/proposal.schema.json");

pub fn standard_metadata_schema() -> Result<serde_json::Value> {
    serde_json::from_str(STANDARD_METADATA_SCHEMA_JSON)
        .context("embedded standard.metadata.schema.json is not valid JSON")
}

pub fn proposal_schema() -> Result<serde_json::Value> {
    serde_json::from_str(PROPOSAL_SCHEMA_JSON).context("embedded proposal.schema.json is not valid JSON")
}

/// Validate a JSON value against a JSON Schema file on disk. Only
/// appropriate for schemas that are genuinely per-repo/per-standard files
/// (there are none today — both samgraha's schemas are embedded, above);
/// kept for direct testing against a real schema file on disk.
pub fn validate_against_schema(instance: &serde_json::Value, schema_path: &Path) -> Result<()> {
    let schema_str = std::fs::read_to_string(schema_path)
        .context(format!("Failed to read schema {}", schema_path.display()))?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_str)
        .context(format!("Failed to parse schema {}", schema_path.display()))?;
    validate_against_schema_value(instance, &schema_json)
        .map_err(|e| anyhow::anyhow!("validation failed against {}: {e}", schema_path.display()))
}

/// Validate a JSON value against an already-parsed JSON Schema — no disk
/// read, no path resolution. This is what every real call site in this
/// codebase should use for samgraha's own embedded schemas.
pub fn validate_against_schema_value(instance: &serde_json::Value, schema: &serde_json::Value) -> Result<()> {
    let validator = jsonschema::validator_for(schema)
        .map_err(|e| anyhow::anyhow!("invalid schema: {e}"))?;
    if let Err(error) = validator.validate(instance) {
        bail!("{error}");
    }
    Ok(())
}

/// Load and validate a standard's `standard.metadata.json` against the
/// embedded schema. Returns the parsed JSON value on success.
pub fn load_and_validate_metadata(metadata_path: &Path) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(metadata_path)
        .context(format!("Failed to read {}", metadata_path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", metadata_path.display()))?;
    validate_against_schema_value(&value, &standard_metadata_schema()?)?;
    Ok(value)
}

/// Validate a proposal envelope against the embedded proposal schema.
pub fn validate_proposal(proposal: &serde_json::Value) -> Result<()> {
    validate_against_schema_value(proposal, &proposal_schema()?)
}

/// Extract the `proposal_template` field from a metadata value, if present.
pub fn get_proposal_template(metadata: &serde_json::Value) -> Option<String> {
    metadata
        .get("proposal_template")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Check that exactly one template has role='proposal' and that
/// proposal_template names it. Returns Ok(()) if valid, or an error
/// describing what's wrong.
pub fn validate_proposal_template_consistency(metadata: &serde_json::Value) -> Result<()> {
    let templates = metadata
        .get("templates")
        .and_then(|v| v.as_array());

    let proposal_templates: Vec<&str> = templates
        .map(|t| {
            t.iter()
                .filter_map(|item| {
                    if item.get("role").and_then(|v| v.as_str()) == Some("proposal") {
                        item.get("name").and_then(|v| v.as_str())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let proposal_template_name = metadata
        .get("proposal_template")
        .and_then(|v| v.as_str());

    match (proposal_template_name, proposal_templates.len()) {
        (Some(name), 1) => {
            if proposal_templates[0] != name {
                bail!(
                    "proposal_template '{}' does not match the template with role='proposal' ('{}')",
                    name,
                    proposal_templates[0]
                );
            }
            Ok(())
        }
        (Some(name), 0) => {
            bail!(
                "proposal_template '{}' references a template that doesn't have role='proposal'",
                name
            );
        }
        (Some(_), n) => {
            bail!(
                "exactly one template may have role='proposal', found {}",
                n
            );
        }
        (None, 1) => {
            bail!(
                "a template has role='proposal' but proposal_template is not set"
            );
        }
        (None, _) => Ok(()), // No proposal template — standards that don't emit proposals are fine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn schema_dir() -> PathBuf {
        // Walk up from the crate root to find the repo root, then add metadata/
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
        repo_root.join("metadata")
    }

    #[test]
    fn validate_valid_metadata() {
        let metadata = serde_json::json!({
            "custom_tables": [
                {
                    "name": "hackathon_scores",
                    "purpose": "Per-team leaderboard rows",
                    "required_columns": ["team_id", "score", "recorded_at"]
                }
            ],
            "templates": [
                {
                    "name": "phase-plan",
                    "purpose": "Renders the phase-wise proposal",
                    "role": "proposal"
                }
            ],
            "proposal_template": "phase-plan"
        });
        let schema_path = schema_dir().join("standard.metadata.schema.json");
        validate_against_schema(&metadata, &schema_path).unwrap();
    }

    #[test]
    fn validate_minimal_metadata() {
        let metadata = serde_json::json!({});
        let schema_path = schema_dir().join("standard.metadata.schema.json");
        validate_against_schema(&metadata, &schema_path).unwrap();
    }

    #[test]
    fn reject_metadata_with_unknown_field() {
        let metadata = serde_json::json!({
            "unknown_field": "should fail"
        });
        let schema_path = schema_dir().join("standard.metadata.schema.json");
        let err = validate_against_schema(&metadata, &schema_path).unwrap_err();
        assert!(err.to_string().contains("Additional properties"), "expected additionalProperties error, got: {err}");
    }

    #[test]
    fn reject_custom_table_missing_name() {
        let metadata = serde_json::json!({
            "custom_tables": [{ "purpose": "no name" }]
        });
        let schema_path = schema_dir().join("standard.metadata.schema.json");
        let err = validate_against_schema(&metadata, &schema_path).unwrap_err();
        assert!(err.to_string().contains("name"), "expected name-required error, got: {err}");
    }

    #[test]
    fn validate_valid_proposal() {
        let proposal = serde_json::json!({
            "title": "Phase Plan",
            "phases": [{
                "domain": "grading",
                "phase_number": 1,
                "usecases": ["ingest-submissions"],
                "steps": [12, 13],
                "rationale": "Single phase"
            }]
        });
        validate_proposal(&proposal).unwrap();
    }

    #[test]
    fn reject_proposal_empty_phases() {
        let proposal = serde_json::json!({
            "title": "Empty",
            "phases": []
        });
        let err = validate_proposal(&proposal).unwrap_err();
        assert!(err.to_string().contains("less than 1 item"), "expected minItems error, got: {err}");
    }

    #[test]
    fn reject_proposal_missing_title() {
        let proposal = serde_json::json!({
            "phases": [{
                "domain": "x",
                "phase_number": 1,
                "usecases": ["a"],
                "steps": [1],
                "rationale": "because"
            }]
        });
        let err = validate_proposal(&proposal).unwrap_err();
        assert!(err.to_string().contains("title"), "expected title-required error, got: {err}");
    }

    #[test]
    fn proposal_template_consistency_valid() {
        let metadata = serde_json::json!({
            "templates": [
                { "name": "report", "role": "report" },
                { "name": "phase-plan", "role": "proposal" }
            ],
            "proposal_template": "phase-plan"
        });
        validate_proposal_template_consistency(&metadata).unwrap();
    }

    #[test]
    fn proposal_template_consistency_mismatch() {
        let metadata = serde_json::json!({
            "templates": [
                { "name": "phase-plan", "role": "proposal" }
            ],
            "proposal_template": "wrong-name"
        });
        let err = validate_proposal_template_consistency(&metadata).unwrap_err();
        assert!(err.to_string().contains("does not match"), "expected mismatch error, got: {err}");
    }

    #[test]
    fn proposal_template_consistency_missing_template() {
        let metadata = serde_json::json!({
            "proposal_template": "nonexistent"
        });
        let err = validate_proposal_template_consistency(&metadata).unwrap_err();
        assert!(err.to_string().contains("doesn't have role"), "expected missing-role error, got: {err}");
    }

    #[test]
    fn proposal_template_consistency_role_without_declaration() {
        let metadata = serde_json::json!({
            "templates": [
                { "name": "phase-plan", "role": "proposal" }
            ]
        });
        let err = validate_proposal_template_consistency(&metadata).unwrap_err();
        assert!(err.to_string().contains("proposal_template is not set"), "expected missing-declaration error, got: {err}");
    }

    #[test]
    fn no_proposal_template_is_fine() {
        let metadata = serde_json::json!({
            "templates": [
                { "name": "report", "role": "report" }
            ]
        });
        validate_proposal_template_consistency(&metadata).unwrap();
    }
}
