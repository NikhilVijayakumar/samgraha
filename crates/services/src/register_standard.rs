//! Registers a knowledge standard into `knowledge.db`'s core schema
//! (`usecase`/`script`/`prompt`/`step`/`step_script`/`step_prompt`/
//! `custom_data_tables`). Reads a standard's own `standard.yaml` manifest
//! — the only file format samgraha imposes; everything it declares
//! (script purpose, prompt content, usecase shape, custom table meaning)
//! is the standard's own business, never interpreted by samgraha.

use anyhow::{bail, Context, Result};
use registry::migration::RESERVED_TABLE_NAMES;
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct StandardManifest {
    pub name: String,
    #[serde(default)]
    pub scripts: Vec<ScriptDecl>,
    #[serde(default)]
    pub prompts: Vec<PromptDecl>,
    #[serde(default)]
    pub usecases: Vec<UsecaseDecl>,
    #[serde(default)]
    pub custom_tables: Vec<CustomTableDecl>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptDecl {
    pub name: String,
    pub location: String,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptDecl {
    pub name: String,
    pub location: String,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsecaseDecl {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub steps: Vec<StepDecl>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepDecl {
    pub order: i64,
    pub kind: String, // "deterministic" | "semantic"
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomTableDecl {
    pub table_name: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub owner_script: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegisterStandardResult {
    pub standard: String,
    pub scripts_registered: usize,
    pub prompts_registered: usize,
    pub usecases_registered: usize,
    pub steps_registered: usize,
    pub custom_tables_cataloged: usize,
}

/// Register a standard's `standard.yaml` manifest into `knowledge.db`.
/// `standard_path` is the standard's own source root (e.g. Kriti's
/// `samgraha/system/python_hackathon/`) — script/prompt `location` fields
/// resolve relative to it and are stored as absolute paths, so no
/// per-repo asset copy is needed: a deterministic step just runs whatever
/// is at that path on this machine.
pub fn register_standard(standard_path: &Path, knowledge_db_path: &Path) -> Result<RegisterStandardResult> {
    let manifest_path = standard_path.join("standard.yaml");
    if !manifest_path.is_file() {
        bail!("No standard.yaml at {}", manifest_path.display());
    }
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .context(format!("Failed to read {}", manifest_path.display()))?;
    let manifest: StandardManifest = serde_yaml::from_str(&manifest_content)
        .context("standard.yaml failed to parse")?;

    // Reject any custom table name colliding with samgraha's own reserved
    // names — before writing anything, same "fail fast, no partial state"
    // discipline as the rest of this codebase's registration checks.
    for ct in &manifest.custom_tables {
        if ct.table_name.trim().is_empty() {
            bail!("custom_tables entry has an empty table_name");
        }
        if RESERVED_TABLE_NAMES.contains(&ct.table_name.as_str()) {
            bail!(
                "custom table name '{}' collides with a samgraha-reserved table name — choose a different name",
                ct.table_name
            );
        }
    }

    if let Some(parent) = knowledge_db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(knowledge_db_path)
        .context(format!("Failed to open {}", knowledge_db_path.display()))?;
    for m in registry::core_schema::CORE_MIGRATIONS {
        conn.execute_batch(m)?;
    }

    // Re-registering a standard replaces its rows entirely — same
    // discipline the old store_system_plan used (delete then insert),
    // so a standard's declared shape never accumulates stale rows across
    // re-registrations.
    delete_existing(&conn, &manifest.name)?;

    let mut script_ids: HashMap<String, i64> = HashMap::new();
    for s in &manifest.scripts {
        let abs_location = resolve_location(standard_path, &s.location)?;
        conn.execute(
            "INSERT INTO script (standard, name, location, purpose) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![manifest.name, s.name, abs_location, s.purpose],
        )?;
        script_ids.insert(s.name.clone(), conn.last_insert_rowid());
    }

    let mut prompt_ids: HashMap<String, i64> = HashMap::new();
    for p in &manifest.prompts {
        let abs_location = resolve_location(standard_path, &p.location)?;
        let content = std::fs::read_to_string(&abs_location)
            .context(format!("Failed to read prompt file {}", abs_location))?;
        conn.execute(
            "INSERT INTO prompt (standard, name, purpose, content) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![manifest.name, p.name, p.purpose, content],
        )?;
        prompt_ids.insert(p.name.clone(), conn.last_insert_rowid());
    }

    let mut steps_registered = 0usize;
    for uc in &manifest.usecases {
        conn.execute(
            "INSERT INTO usecase (standard, name, description) VALUES (?1, ?2, ?3)",
            rusqlite::params![manifest.name, uc.name, uc.description],
        )?;
        let usecase_id = conn.last_insert_rowid();

        for step in &uc.steps {
            match step.kind.as_str() {
                "deterministic" => {}
                "semantic" => {}
                other => bail!(
                    "usecase '{}' step order {} has invalid kind '{}' (must be 'deterministic' or 'semantic')",
                    uc.name, step.order, other
                ),
            }
            conn.execute(
                "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![usecase_id, step.order, step.kind, step.description],
            )?;
            let step_id = conn.last_insert_rowid();
            steps_registered += 1;

            match step.kind.as_str() {
                "deterministic" => {
                    let script_name = step.script.as_ref().ok_or_else(|| {
                        anyhow::anyhow!(
                            "usecase '{}' step order {} is deterministic but has no 'script' field",
                            uc.name, step.order
                        )
                    })?;
                    let script_id = *script_ids.get(script_name).ok_or_else(|| {
                        anyhow::anyhow!(
                            "usecase '{}' step order {} references unknown script '{}'",
                            uc.name, step.order, script_name
                        )
                    })?;
                    conn.execute(
                        "INSERT INTO step_script (step_id, script_id) VALUES (?1, ?2)",
                        rusqlite::params![step_id, script_id],
                    )?;
                }
                "semantic" => {
                    let prompt_name = step.prompt.as_ref().ok_or_else(|| {
                        anyhow::anyhow!(
                            "usecase '{}' step order {} is semantic but has no 'prompt' field",
                            uc.name, step.order
                        )
                    })?;
                    let prompt_id = *prompt_ids.get(prompt_name).ok_or_else(|| {
                        anyhow::anyhow!(
                            "usecase '{}' step order {} references unknown prompt '{}'",
                            uc.name, step.order, prompt_name
                        )
                    })?;
                    conn.execute(
                        "INSERT INTO step_prompt (step_id, prompt_id) VALUES (?1, ?2)",
                        rusqlite::params![step_id, prompt_id],
                    )?;
                }
                _ => unreachable!(),
            }
        }
    }

    for ct in &manifest.custom_tables {
        let owner_script_id = ct.owner_script.as_ref().and_then(|n| script_ids.get(n).copied());
        conn.execute(
            "INSERT INTO custom_data_tables (standard, table_name, purpose, owner_script_id) \
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![manifest.name, ct.table_name, ct.purpose, owner_script_id],
        )?;
    }

    Ok(RegisterStandardResult {
        standard: manifest.name.clone(),
        scripts_registered: manifest.scripts.len(),
        prompts_registered: manifest.prompts.len(),
        usecases_registered: manifest.usecases.len(),
        steps_registered,
        custom_tables_cataloged: manifest.custom_tables.len(),
    })
}

fn delete_existing(conn: &Connection, standard: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM step_script WHERE step_id IN (
            SELECT step.id FROM step
            JOIN usecase ON step.usecase_id = usecase.id
            WHERE usecase.standard = ?1
        )",
        rusqlite::params![standard],
    )?;
    conn.execute(
        "DELETE FROM step_prompt WHERE step_id IN (
            SELECT step.id FROM step
            JOIN usecase ON step.usecase_id = usecase.id
            WHERE usecase.standard = ?1
        )",
        rusqlite::params![standard],
    )?;
    conn.execute(
        "DELETE FROM step WHERE usecase_id IN (SELECT id FROM usecase WHERE standard = ?1)",
        rusqlite::params![standard],
    )?;
    conn.execute("DELETE FROM usecase WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM custom_data_tables WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM prompt WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM script WHERE standard = ?1", rusqlite::params![standard])?;
    Ok(())
}

fn resolve_location(standard_path: &Path, location: &str) -> Result<String> {
    let candidate = Path::new(location);
    let resolved = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        standard_path.join(candidate)
    };
    if !resolved.exists() {
        bail!("declared location does not exist: {}", resolved.display());
    }
    Ok(resolved.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_manifest(dir: &Path, yaml: &str) {
        std::fs::write(dir.join("standard.yaml"), yaml).unwrap();
    }

    #[test]
    fn register_standard_full_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let standard_dir = tmp.path().join("standard");
        std::fs::create_dir_all(&standard_dir).unwrap();
        std::fs::write(standard_dir.join("audit_python.py"), "print('hi')").unwrap();
        std::fs::write(standard_dir.join("leaderboard.md"), "# Leaderboard prompt").unwrap();

        write_manifest(&standard_dir, r#"
name: test-standard
scripts:
  - name: audit-python
    location: audit_python.py
    purpose: "Static analysis"
prompts:
  - name: leaderboard-prompt
    location: leaderboard.md
    purpose: "Narrative"
usecases:
  - name: pipeline
    description: "Full pipeline"
    steps:
      - order: 1
        kind: deterministic
        description: "Run audit"
        script: audit-python
      - order: 2
        kind: semantic
        description: "Write narrative"
        prompt: leaderboard-prompt
custom_tables:
  - table_name: teststd_scores
    purpose: "Scores"
    owner_script: audit-python
"#);

        let db_path = tmp.path().join("knowledge.db");
        let result = register_standard(&standard_dir, &db_path).unwrap();
        assert_eq!(result.scripts_registered, 1);
        assert_eq!(result.prompts_registered, 1);
        assert_eq!(result.usecases_registered, 1);
        assert_eq!(result.steps_registered, 2);
        assert_eq!(result.custom_tables_cataloged, 1);

        let conn = Connection::open(&db_path).unwrap();
        let prompt_content: String = conn
            .query_row("SELECT content FROM prompt WHERE name = 'leaderboard-prompt'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(prompt_content, "# Leaderboard prompt");

        let owner: i64 = conn
            .query_row(
                "SELECT owner_script_id FROM custom_data_tables WHERE table_name = 'teststd_scores'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(owner > 0);
    }

    #[test]
    fn register_standard_rejects_reserved_table_name() {
        let tmp = tempfile::tempdir().unwrap();
        let standard_dir = tmp.path().join("standard");
        std::fs::create_dir_all(&standard_dir).unwrap();
        write_manifest(&standard_dir, r#"
name: bad-standard
custom_tables:
  - table_name: usecase
    purpose: "Collides with samgraha's own table"
"#);
        let db_path = tmp.path().join("knowledge.db");
        let err = register_standard(&standard_dir, &db_path).unwrap_err();
        assert!(err.to_string().contains("reserved"), "expected collision error, got: {err}");
    }

    #[test]
    fn register_standard_rejects_unknown_script_reference() {
        let tmp = tempfile::tempdir().unwrap();
        let standard_dir = tmp.path().join("standard");
        std::fs::create_dir_all(&standard_dir).unwrap();
        write_manifest(&standard_dir, r#"
name: bad-standard-2
usecases:
  - name: uc1
    steps:
      - order: 1
        kind: deterministic
        script: does-not-exist
"#);
        let db_path = tmp.path().join("knowledge.db");
        let err = register_standard(&standard_dir, &db_path).unwrap_err();
        assert!(err.to_string().contains("unknown script"), "expected unknown-script error, got: {err}");
    }

    #[test]
    fn re_registering_replaces_prior_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let standard_dir = tmp.path().join("standard");
        std::fs::create_dir_all(&standard_dir).unwrap();
        std::fs::write(standard_dir.join("a.py"), "").unwrap();

        write_manifest(&standard_dir, r#"
name: reg-test
scripts:
  - name: script-a
    location: a.py
"#);
        let db_path = tmp.path().join("knowledge.db");
        register_standard(&standard_dir, &db_path).unwrap();

        // Re-register with a different script name — old row should be gone.
        std::fs::write(standard_dir.join("b.py"), "").unwrap();
        write_manifest(&standard_dir, r#"
name: reg-test
scripts:
  - name: script-b
    location: b.py
"#);
        register_standard(&standard_dir, &db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM script WHERE standard = 'reg-test'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "expected old script row replaced, not accumulated");
        let name: String = conn
            .query_row("SELECT name FROM script WHERE standard = 'reg-test'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "script-b");
    }
}
