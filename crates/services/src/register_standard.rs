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
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct StandardManifest {
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub smoke_test: Option<String>,
    #[serde(default)]
    pub scripts: Vec<ScriptDecl>,
    #[serde(default)]
    pub prompts: Vec<PromptDecl>,
    #[serde(default)]
    pub usecases: Vec<UsecaseDecl>,
    #[serde(default)]
    pub custom_tables: Vec<CustomTableDecl>,
    #[serde(default)]
    pub domains: Vec<DomainDecl>,
    #[serde(default)]
    pub assets: Vec<AssetDecl>,
    #[serde(default)]
    pub templates: Vec<TemplateDecl>,
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
    #[serde(default)]
    pub driver: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub verify_script: Option<String>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct DomainDecl {
    pub key: String,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetDecl {
    pub name: String,
    pub kind: String,
    pub location: String,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub template_type: String,
    pub location: String,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegisterStandardResult {
    pub standard: String,
    pub scripts_registered: usize,
    pub prompts_registered: usize,
    pub usecases_registered: usize,
    pub steps_registered: usize,
    pub custom_tables_cataloged: usize,
    pub domains_registered: usize,
    pub assets_registered: usize,
    pub templates_registered: usize,
}

/// Register a standard's `standard.yaml` manifest into `knowledge.db`.
/// `standard_path` is the standard's own source root (e.g. Kriti's
/// `samgraha/system/python_hackathon/`) — script/prompt `location` fields
/// resolve relative to it and are stored as absolute paths, so no
/// per-repo asset copy is needed: a deterministic step just runs whatever
/// is at that path on this machine.
pub fn register_standard(standard_path: &Path, knowledge_db_path: &Path, metadata_json: Option<&str>) -> Result<RegisterStandardResult> {
    let manifest_path = resolve_manifest_path(standard_path)?;
    let manifest_dir = manifest_path.parent().unwrap_or(standard_path);
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
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    registry::core_schema::run_core_migrations(&conn)?;

    // Re-registering a standard replaces its rows entirely — same
    // discipline the old store_system_plan used (delete then insert),
    // so a standard's declared shape never accumulates stale rows across
    // re-registrations.
    delete_existing(&conn, &manifest.name)?;

    let mut script_ids: HashMap<String, i64> = HashMap::new();
    for s in &manifest.scripts {
        let abs_location = resolve_location(manifest_dir, &s.location)?;
        conn.execute(
            "INSERT INTO script (standard, name, location, purpose) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![manifest.name, s.name, abs_location, s.purpose],
        )?;
        script_ids.insert(s.name.clone(), conn.last_insert_rowid());
    }

    let mut prompt_ids: HashMap<String, i64> = HashMap::new();
    for p in &manifest.prompts {
        let abs_location = resolve_location(manifest_dir, &p.location)?;
        let content = std::fs::read_to_string(&abs_location)
            .context(format!("Failed to read prompt file {}", abs_location))?;
        conn.execute(
            "INSERT INTO prompt (standard, name, purpose, content) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![manifest.name, p.name, p.purpose, content],
        )?;
        prompt_ids.insert(p.name.clone(), conn.last_insert_rowid());
    }

    let mut steps_registered = 0usize;

    // §2.11 — domains: insert into domain table, build key→id map
    let mut domain_ids: HashMap<String, i64> = HashMap::new();
    for d in &manifest.domains {
        conn.execute(
            "INSERT INTO domain (standard, key, sort_order, description) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![manifest.name, d.key, d.sort_order, d.description],
        )?;
        domain_ids.insert(d.key.clone(), conn.last_insert_rowid());
    }

    // §2.9 — assets: insert into standard_asset table
    for a in &manifest.assets {
        let abs_location = resolve_location(manifest_dir, &a.location)?;
        conn.execute(
            "INSERT INTO standard_asset (standard, kind, name, location, purpose) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![manifest.name, a.kind, a.name, abs_location, a.purpose],
        )?;
    }

    // §2.7 — templates: insert into template table (content read from file)
    for t in &manifest.templates {
        let abs_location = resolve_location(manifest_dir, &t.location)?;
        let content = std::fs::read_to_string(&abs_location)
            .context(format!("Failed to read template file {}", abs_location))?;
        conn.execute(
            "INSERT INTO template (standard, name, type, content, purpose) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![manifest.name, t.name, t.template_type, content, t.purpose],
        )?;
    }

    // §2.13 — local standard row
    let category = manifest.category.as_deref().unwrap_or("");
    let metadata_json = metadata_json.unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO standard (name, category, subcategory, extends, version, metadata_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            manifest.name,
            category,
            manifest.subcategory,
            manifest.extends,
            manifest.version.as_deref().unwrap_or("0.0.0"),
            metadata_json,
        ],
    )?;

    // §8.5 — validate depends_on references (all usecase names must be known)
    let all_uc_names: Vec<String> = manifest.usecases.iter().map(|uc| uc.name.clone()).collect();
    for uc in &manifest.usecases {
        for dep in &uc.depends_on {
            if !all_uc_names.contains(dep) {
                bail!(
                    "usecase '{}' depends_on unknown usecase '{}'",
                    uc.name, dep
                );
            }
        }
    }

    for uc in &manifest.usecases {
        // §2.11 — resolve domain_id from domain key
        let domain_id = uc.domain.as_ref().and_then(|key| domain_ids.get(key).copied());
        // §2.5 — store driver and depends_on as JSON in usecase.data
        let data = serde_json::json!({
            "driver": uc.driver.as_deref().unwrap_or("samgraha"),
            "depends_on": uc.depends_on,
            "verify_script": uc.verify_script,
        });
        conn.execute(
            "INSERT INTO usecase (standard, name, description, domain_id, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![manifest.name, uc.name, uc.description, domain_id, data.to_string()],
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
        domains_registered: manifest.domains.len(),
        assets_registered: manifest.assets.len(),
        templates_registered: manifest.templates.len(),
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
    conn.execute("DELETE FROM domain WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM standard_asset WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM template WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM standard WHERE name = ?1", rusqlite::params![standard])?;
    Ok(())
}

/// §2.14 — two-location manifest check: tries `<path>/standard.yaml` first,
/// then `<path>/script/schema/standard.yaml` (for standards like pcems_2026
/// whose manifest lives one level below root).
pub fn resolve_manifest_path(standard_path: &Path) -> Result<PathBuf> {
    let primary = standard_path.join("standard.yaml");
    if primary.is_file() {
        return Ok(primary);
    }
    let alt = standard_path.join("script/schema/standard.yaml");
    if alt.is_file() {
        return Ok(alt);
    }
    bail!(
        "No standard.yaml at {} or {}",
        primary.display(),
        alt.display()
    );
}

pub fn resolve_location(manifest_dir: &Path, location: &str) -> Result<String> {
    let candidate = Path::new(location);
    let resolved = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        manifest_dir.join(candidate)
    };
    // Canonicalize to resolve `..` segments (e.g. `script/schema/../my_script.py` → `script/my_script.py`)
    let resolved = std::fs::canonicalize(&resolved)
        .with_context(|| format!("declared location does not exist: {}", resolved.display()))?;
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
        let result = register_standard(&standard_dir, &db_path, None).unwrap();
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
        let err = register_standard(&standard_dir, &db_path, None).unwrap_err();
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
        let err = register_standard(&standard_dir, &db_path, None).unwrap_err();
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
        register_standard(&standard_dir, &db_path, None).unwrap();

        // Re-register with a different script name — old row should be gone.
        std::fs::write(standard_dir.join("b.py"), "").unwrap();
        write_manifest(&standard_dir, r#"
name: reg-test
scripts:
  - name: script-b
    location: b.py
"#);
        register_standard(&standard_dir, &db_path, None).unwrap();

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

    #[test]
    fn resolve_manifest_path_finds_root_standard_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("my-standard");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("standard.yaml"), "name: root-manifest\n").unwrap();
        let found = resolve_manifest_path(&dir).unwrap();
        assert_eq!(found, dir.join("standard.yaml"));
    }

    #[test]
    fn resolve_manifest_path_finds_nested_standard_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        // pcems_2026 layout: root/script/schema/standard.yaml
        let dir = tmp.path().join("pcems-style");
        let nested = dir.join("script/schema");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("standard.yaml"), "name: nested-manifest\n").unwrap();
        let found = resolve_manifest_path(&dir).unwrap();
        assert_eq!(found, nested.join("standard.yaml"));
    }

    #[test]
    fn resolve_manifest_path_fails_when_neither_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("empty-standard");
        std::fs::create_dir_all(&dir).unwrap();
        let err = resolve_manifest_path(&dir).unwrap_err();
        assert!(err.to_string().contains("No standard.yaml"), "expected manifest-not-found error, got: {err}");
    }

    #[test]
    fn register_standard_resolves_nested_manifest_paths() {
        let tmp = tempfile::tempdir().unwrap();
        // Simulate pcems_2026 layout: manifest at script/schema/standard.yaml,
        // scripts relative to that dir.
        let root = tmp.path().join("nested-std");
        let schema_dir = root.join("script/schema");
        std::fs::create_dir_all(&schema_dir).unwrap();
        std::fs::write(root.join("script/my_script.py"), "print('ok')").unwrap();

        // Manifest at script/schema/standard.yaml, script location relative to it
        std::fs::write(
            schema_dir.join("standard.yaml"),
            r#"
name: nested-test
scripts:
  - name: my-script
    location: ../my_script.py
    purpose: "Test"
usecases:
  - name: uc1
    steps:
      - order: 1
        kind: deterministic
        script: my-script
"#,
        ).unwrap();

        let db_path = tmp.path().join("knowledge.db");
        let result = register_standard(&root, &db_path, None).unwrap();
        assert_eq!(result.scripts_registered, 1);

        // Verify the script location resolved correctly (relative to script/schema/, not root)
        let conn = Connection::open(&db_path).unwrap();
        let location: String = conn
            .query_row("SELECT location FROM script WHERE name = 'my-script'", [], |r| r.get(0))
            .unwrap();
        let expected = root.join("script/my_script.py");
        assert_eq!(location, expected.display().to_string());
    }
}
