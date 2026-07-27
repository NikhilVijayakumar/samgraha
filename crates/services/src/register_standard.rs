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
use tracing;

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
    /// Path to the seeder script (relative to manifest dir). When present,
    /// the seeder is invoked instead of parsing YAML workflow declarations.
    #[serde(default)]
    pub seeder_script: Option<String>,
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
    registry::core_schema::ensure_current_schema(&conn)?;

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

    // §2.9 — assets: insert into standard_asset table. `kind` is now a
    // relation (asset_kind), not free text — find-or-create the kind row
    // the manifest names, then reference it by id.
    for a in &manifest.assets {
        let abs_location = resolve_location(manifest_dir, &a.location)?;
        let kind_id = get_or_create_lookup(&conn, "asset_kind", &manifest.name, &a.kind)?;
        conn.execute(
            "INSERT INTO standard_asset (standard, kind_id, name, location, purpose) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![manifest.name, kind_id, a.name, abs_location, a.purpose],
        )?;
    }

    // §2.7 — templates: insert into template table (content read from
    // file). `type` is now a relation (template_type), not free text.
    for t in &manifest.templates {
        let abs_location = resolve_location(manifest_dir, &t.location)?;
        let content = std::fs::read_to_string(&abs_location)
            .context(format!("Failed to read template file {}", abs_location))?;
        let type_id = get_or_create_lookup(&conn, "template_type", &manifest.name, &t.template_type)?;
        conn.execute(
            "INSERT INTO template (standard, name, type_id, content, purpose) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![manifest.name, t.name, type_id, content, t.purpose],
        )?;
    }

    // Per-repo catalog metadata (category/subcategory/extends/version) is no
    // longer stored in knowledge.db — it lives in registry.db's
    // `active_standard` table instead (one standard per repo at a time),
    // written by the caller after this function returns. `metadata_json`
    // is accepted for call-site compatibility but unused here now.
    let _ = metadata_json;

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

pub fn delete_existing(conn: &Connection, standard: &str) -> Result<()> {
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
    // Children before parents — asset_kind/template_type are referenced
    // by standard_asset.kind_id/template.type_id.
    conn.execute("DELETE FROM standard_asset WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM asset_kind WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM template WHERE standard = ?1", rusqlite::params![standard])?;
    conn.execute("DELETE FROM template_type WHERE standard = ?1", rusqlite::params![standard])?;
    // artifact/artifact_type are NOT deleted here — historical output
    // record, survives a standard re-registering (core_schema.rs).
    Ok(())
}

/// Find-or-create a row in a per-standard lookup table (`asset_kind`,
/// `template_type`, `artifact_type` — all three share the shape
/// `(id, standard, name, description)` with `UNIQUE(standard, name)`) and
/// return its id. `table` is always one of those three fixed literals
/// from call sites in this crate, never external input.
pub fn get_or_create_lookup(conn: &Connection, table: &str, standard: &str, name: &str) -> Result<i64> {
    conn.execute(
        &format!("INSERT INTO {table} (standard, name) VALUES (?1, ?2) ON CONFLICT(standard, name) DO NOTHING"),
        rusqlite::params![standard, name],
    )?;
    let id: i64 = conn.query_row(
        &format!("SELECT id FROM {table} WHERE standard = ?1 AND name = ?2"),
        rusqlite::params![standard, name],
        |r| r.get(0),
    )?;
    Ok(id)
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

/// §3.9 — Activate an already-globally-registered standard in a specific
/// repository. Copies the standard from the global mcp-registry into
/// `<samgraha_dir>/<standard_name>/`, runs the seeder, and absolutizes
/// paths. This replaces the old per-repo `register_standard` which parsed
/// manifests directly. Catalog metadata (category/subcategory/extends/
/// version) is the caller's job to persist — one standard is active per
/// repo at a time, so that fact lives in registry.db's `active_standard`
/// table (`RegistryDb::set_active_standard`), not in this function or in
/// `knowledge.db`.
pub fn activate_standard(
    standard_name: &str,
    source_path: &str,
    knowledge_db_path: &Path,
    repo_root: &Path,
    samgraha_dir: &Path,
    timeout_secs: Option<u64>,
) -> Result<()> {
    use rusqlite::Connection;

    // Ensure .samgraha/ exists — knowledge.db lives here and
    // RegistryDb::open() (called after us) also expects it.
    std::fs::create_dir_all(samgraha_dir)
        .context(format!("Failed to create {}", samgraha_dir.display()))?;

    let local_copy = samgraha_dir.join(standard_name);

    // §3.9 step 2 — atomic copy from global registry into local tree
    let source = std::path::Path::new(source_path);
    if !source.exists() {
        bail!("global registry source_path does not exist: {source_path}");
    }
    common::fs_sync::copy_dir_atomic(source, &local_copy, &common::fs_sync::DEFAULT_EXCLUDES)?;

    let conn = Connection::open(knowledge_db_path)
        .context(format!("Failed to open {}", knowledge_db_path.display()))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    registry::core_schema::ensure_current_schema(&conn)?;

    // §3.9 step 3 — delete this standard's existing rows first
    delete_existing(&conn, standard_name)?;

    // §3.9 step 4 — find and run the seeder
    let manifest_path = resolve_manifest_path(&local_copy)?;
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let raw: serde_yaml::Mapping = serde_yaml::from_str(&manifest_content)?;
    let get_str = |key: &str| -> Option<String> {
        raw.get(&serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str().map(String::from))
    };
    let seeder_script = get_str("seeder_script");

    if let Some(ref script_name) = seeder_script {
        // resolve_location itself canonicalizes and requires the target
        // to exist — it errors via `?` before any later check runs, so
        // that error needs the same cleanup every other fallible step
        // in this function already gets, not just the ones after it.
        let script_location = resolve_location(&local_copy, script_name)
            .inspect_err(|_| cleanup_on_failure(&conn, standard_name, &local_copy))?;
        let script_path = std::path::Path::new(&script_location);
        crate::seeder::run_seeder(
            repo_root,
            script_path,
            samgraha_dir,
            knowledge_db_path,
            timeout_secs,
        ).inspect_err(|_| {
            cleanup_on_failure(&conn, standard_name, &local_copy);
        })?;
    }

    // §3.9 step 5 — absolutize pass
    crate::seeder::absolutize_paths(&conn, standard_name, samgraha_dir)
        .inspect_err(|_| {
            cleanup_on_failure(&conn, standard_name, &local_copy);
        })?;

    // Standard metadata contract — custom-table handling (closes §1.1 gap).
    // Read standard.metadata.json from the local copy, run RESERVED_TABLE_NAMES
    // collision check, and insert custom_data_tables rows (shape_json populated
    // via sqlite_master introspection).
    let metadata_path = local_copy.join("standard.metadata.json");
    if metadata_path.exists() {
        let metadata = crate::metadata_validate::load_and_validate_metadata(&metadata_path)
            .inspect_err(|_| {
                cleanup_on_failure(&conn, standard_name, &local_copy);
            })?;

        if let Some(tables) = metadata.get("custom_tables").and_then(|v| v.as_array()) {
            for ct in tables {
                let table_name = ct.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if table_name.trim().is_empty() {
                    cleanup_on_failure(&conn, standard_name, &local_copy);
                    bail!("custom_tables entry has an empty name in standard.metadata.json");
                }
                if RESERVED_TABLE_NAMES.contains(&table_name) {
                    cleanup_on_failure(&conn, standard_name, &local_copy);
                    bail!(
                        "custom table name '{}' collides with a samgraha-reserved table name",
                        table_name
                    );
                }
                let purpose = ct.get("purpose").and_then(|v| v.as_str()).unwrap_or("");

                // Introspect shape_json via PRAGMA table_info
                let shape_json = introspect_table_shape(&conn, table_name);

                conn.execute(
                    "INSERT OR IGNORE INTO custom_data_tables (standard, table_name, purpose, shape_json) \
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![standard_name, table_name, purpose, shape_json],
                )?;

                // Enforce required_columns if declared
                if let Some(required_cols) = ct.get("required_columns").and_then(|v| v.as_array()) {
                    for col in required_cols {
                        let col_name = col.as_str().unwrap_or("");
                        if !column_exists_in_table(&conn, table_name, col_name) {
                            cleanup_on_failure(&conn, standard_name, &local_copy);
                            bail!(
                                "required column '{}' does not exist in table '{}'",
                                col_name, table_name
                            );
                        }
                    }
                }
            }
        }
    }

    // Layer A structural completeness audit — mandatory final step.
    crate::layer_a_audit::run_layer_a_audit(&conn, standard_name)
        .inspect_err(|_| {
            cleanup_on_failure(&conn, standard_name, &local_copy);
        })?;

    Ok(())
}

fn cleanup_on_failure(conn: &Connection, standard_name: &str, local_copy: &Path) {
    if let Err(e) = delete_existing(conn, standard_name) {
        tracing::warn!("cleanup: failed to delete DB rows for '{standard_name}': {e}");
    }
    if let Err(e) = std::fs::remove_dir_all(local_copy) {
        tracing::warn!(
            "cleanup: failed to remove local copy at {}: {e}",
            local_copy.display()
        );
    }
}

/// Introspect a table's columns via `PRAGMA table_info` and return the
/// result as a JSON string. Returns None if the table doesn't exist.
fn introspect_table_shape(conn: &Connection, table_name: &str) -> Option<String> {
    let query = format!("PRAGMA table_info(\"{table_name}\")");
    let mut stmt = conn.prepare(&query).ok()?;
    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "cid": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "type": row.get::<_, String>(2)?,
                "notnull": row.get::<_, bool>(3)?,
                "dflt_value": row.get::<_, Option<String>>(4)?,
                "pk": row.get::<_, bool>(5)?,
            }))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();
    if rows.is_empty() {
        None
    } else {
        serde_json::to_string(&rows).ok()
    }
}

/// Check if a column exists in a table via `PRAGMA table_info`.
fn column_exists_in_table(conn: &Connection, table_name: &str, column_name: &str) -> bool {
    let query = format!("PRAGMA table_info(\"{table_name}\")");
    let mut stmt = match conn.prepare(&query) {
        Ok(s) => s,
        Err(_) => return false,
    };
    stmt.query_map([], |row| {
        row.get::<_, String>(1) // column name is at index 1
    })
    .map(|rows| {
        rows.filter_map(|r| r.ok())
            .any(|name| name == column_name)
    })
    .unwrap_or(false)
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
    fn register_standard_assets_and_templates_use_relational_kind_type() {
        let tmp = tempfile::tempdir().unwrap();
        let standard_dir = tmp.path().join("standard");
        std::fs::create_dir_all(&standard_dir).unwrap();
        std::fs::write(standard_dir.join("guide.md"), "# Guide").unwrap();
        std::fs::write(standard_dir.join("report.md"), "# Report template").unwrap();

        write_manifest(&standard_dir, r#"
name: asset-template-standard
assets:
  - name: setup-guide
    kind: guide
    location: guide.md
    purpose: "Onboarding"
templates:
  - name: final-report
    type: markdown
    location: report.md
    purpose: "Output shape"
"#);

        let db_path = tmp.path().join("knowledge.db");
        let result = register_standard(&standard_dir, &db_path, None).unwrap();
        assert_eq!(result.assets_registered, 1);
        assert_eq!(result.templates_registered, 1);

        let conn = Connection::open(&db_path).unwrap();
        let asset_kind_name: String = conn
            .query_row(
                "SELECT ak.name FROM standard_asset sa JOIN asset_kind ak ON ak.id = sa.kind_id WHERE sa.name = 'setup-guide'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(asset_kind_name, "guide");

        let template_type_name: String = conn
            .query_row(
                "SELECT tt.name FROM template t JOIN template_type tt ON tt.id = t.type_id WHERE t.name = 'final-report'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(template_type_name, "markdown");
    }

    #[test]
    fn get_or_create_lookup_dedupes_by_standard_and_name() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("knowledge.db");
        let conn = Connection::open(&db_path).unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();

        let id1 = get_or_create_lookup(&conn, "artifact_type", "std-a", "image").unwrap();
        let id2 = get_or_create_lookup(&conn, "artifact_type", "std-a", "image").unwrap();
        assert_eq!(id1, id2, "same standard+name must resolve to the same row, not duplicate");

        let id3 = get_or_create_lookup(&conn, "artifact_type", "std-b", "image").unwrap();
        assert_ne!(id1, id3, "same name under a different standard is a distinct row");

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM artifact_type", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
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
    fn delete_existing_preserves_artifact_history() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("knowledge.db");
        let conn = Connection::open(&db_path).unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();

        let type_id = get_or_create_lookup(&conn, "artifact_type", "hist-std", "image").unwrap();
        conn.execute(
            "INSERT INTO artifact (standard, type_id, name, location) VALUES (?1, ?2, 'chart', '/out/chart.png')",
            rusqlite::params!["hist-std", type_id],
        ).unwrap();

        delete_existing(&conn, "hist-std").unwrap();

        let artifact_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM artifact WHERE standard = 'hist-std'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(artifact_count, 1, "artifact rows must survive delete_existing — historical output record");
        let type_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM artifact_type WHERE standard = 'hist-std'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(type_count, 1, "artifact_type rows must survive too, or the surviving artifact row's FK would dangle");
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
        let expected = std::fs::canonicalize(root.join("script").join("my_script.py")).unwrap();
        assert_eq!(std::path::PathBuf::from(location), expected);
    }

    #[test]
    fn introspect_table_shape_returns_columns() {
        let conn = Connection::open_in_memory().unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();
        let shape = introspect_table_shape(&conn, "usecase").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&shape).unwrap();
        let cols = parsed.as_array().unwrap();
        assert!(cols.iter().any(|c| c["name"] == "name"), "expected 'name' column");
        assert!(cols.iter().any(|c| c["name"] == "standard"), "expected 'standard' column");
    }

    #[test]
    fn introspect_table_shape_returns_none_for_missing_table() {
        let conn = Connection::open_in_memory().unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();
        assert!(introspect_table_shape(&conn, "nonexistent_table").is_none());
    }

    #[test]
    fn column_exists_in_table_true() {
        let conn = Connection::open_in_memory().unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();
        assert!(column_exists_in_table(&conn, "usecase", "name"));
        assert!(column_exists_in_table(&conn, "usecase", "standard"));
    }

    #[test]
    fn column_exists_in_table_false() {
        let conn = Connection::open_in_memory().unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();
        assert!(!column_exists_in_table(&conn, "usecase", "nonexistent_column"));
        assert!(!column_exists_in_table(&conn, "nonexistent_table", "name"));
    }

    /// A minimal real seeder: reads `_knowledge_db` from the `--in`
    /// envelope, inserts one domain/usecase/step/script/step_script row
    /// directly via `sqlite3` (stdlib, no dependency), creates the custom
    /// table `standard.metadata.json` declares, and reports `{"status":"ok"}`.
    /// Requires a `python`/`python3` interpreter on PATH — same baseline
    /// assumption `run_capability_script`'s whole dispatch model already
    /// makes for every deterministic step, not a new one introduced here.
    const FIXTURE_SEEDER_PY: &str = r#"
import argparse, json, sqlite3

p = argparse.ArgumentParser()
p.add_argument("--repo-root")
p.add_argument("--in", dest="in_path")
p.add_argument("--out")
args = p.parse_args()

with open(args.in_path) as f:
    envelope = json.load(f)

conn = sqlite3.connect(envelope["_knowledge_db"])
conn.execute("PRAGMA foreign_keys = ON")
standard = "fixture-std"

conn.execute("INSERT INTO domain (standard, key, description) VALUES (?, 'main', 'Main domain')", (standard,))
domain_id = conn.execute("SELECT id FROM domain WHERE standard=? AND key='main'", (standard,)).fetchone()[0]

conn.execute("INSERT INTO usecase (standard, name, domain_id) VALUES (?, 'uc1', ?)", (standard, domain_id))
uc_id = conn.execute("SELECT id FROM usecase WHERE standard=? AND name='uc1'", (standard,)).fetchone()[0]

conn.execute("INSERT INTO script (standard, name, location, purpose) VALUES (?, 's1', 'seed.py', '')", (standard,))
script_id = conn.execute("SELECT id FROM script WHERE standard=? AND name='s1'", (standard,)).fetchone()[0]

conn.execute("INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?, 1, 'deterministic', 'run')", (uc_id,))
step_id = conn.execute("SELECT id FROM step WHERE usecase_id=? AND step_order=1", (uc_id,)).fetchone()[0]

conn.execute("INSERT INTO step_script (step_id, script_id) VALUES (?, ?)", (step_id, script_id))
conn.execute("CREATE TABLE IF NOT EXISTS fixture_scores (team_id TEXT, score INTEGER)")
conn.commit()
conn.close()

with open(args.out, "w") as f:
    json.dump({"status": "ok"}, f)
"#;

    fn write_fixture_source(source_dir: &Path) {
        std::fs::create_dir_all(source_dir).unwrap();
        std::fs::write(source_dir.join("standard.yaml"), "name: fixture-std\nseeder_script: seed.py\n").unwrap();
        std::fs::write(source_dir.join("seed.py"), FIXTURE_SEEDER_PY).unwrap();
        std::fs::write(
            source_dir.join("standard.metadata.json"),
            r#"{"custom_tables":[{"name":"fixture_scores","purpose":"test scores","required_columns":["team_id","score"]}]}"#,
        ).unwrap();
    }

    #[test]
    fn activate_standard_end_to_end_seeder_metadata_and_audit() {
        let tmp = tempfile::tempdir().unwrap();
        let source_dir = tmp.path().join("source-std");
        write_fixture_source(&source_dir);

        let repo_root = tmp.path().join("repo");
        let samgraha_dir = repo_root.join(".samgraha");
        let knowledge_db_path = samgraha_dir.join("knowledge.db");

        activate_standard(
            "fixture-std",
            &source_dir.display().to_string(),
            &knowledge_db_path,
            &repo_root,
            &samgraha_dir,
            Some(30),
        ).unwrap();

        // Local copy landed under samgraha_dir/<standard>/, not just the
        // global source — proves the copy step ran, not just the seeder.
        assert!(samgraha_dir.join("fixture-std").join("seed.py").exists());

        let conn = Connection::open(&knowledge_db_path).unwrap();
        let uc_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM usecase WHERE standard = 'fixture-std'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(uc_count, 1, "seeder should have inserted exactly one usecase");

        // Custom-table handling (§1.1's restored gap): declared, created,
        // and catalogued with its actual columns.
        let (table_name, shape_json): (String, Option<String>) = conn
            .query_row(
                "SELECT table_name, shape_json FROM custom_data_tables WHERE standard = 'fixture-std'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(table_name, "fixture_scores");
        assert!(shape_json.unwrap().contains("team_id"), "shape_json should reflect the real table's columns");

        // Layer A audit ran and passed (activate_standard would have
        // errored otherwise) — script is referenced, usecase has a step,
        // deterministic step has a script mapping.
        let script_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM script sc JOIN step_script ss ON ss.script_id = sc.id WHERE sc.standard = 'fixture-std'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(script_count, 1);
    }

    #[test]
    fn activate_standard_rejects_undeclared_custom_table() {
        // Seeder creates fixture_scores via CREATE TABLE, but metadata
        // never declares it — bidirectional Layer A check must reject,
        // and cleanup must remove the local copy it made.
        let tmp = tempfile::tempdir().unwrap();
        let source_dir = tmp.path().join("source-std");
        write_fixture_source(&source_dir);
        std::fs::write(source_dir.join("standard.metadata.json"), "{}").unwrap();

        let repo_root = tmp.path().join("repo");
        let samgraha_dir = repo_root.join(".samgraha");
        let knowledge_db_path = samgraha_dir.join("knowledge.db");

        let err = activate_standard(
            "fixture-std",
            &source_dir.display().to_string(),
            &knowledge_db_path,
            &repo_root,
            &samgraha_dir,
            Some(30),
        ).unwrap_err();
        assert!(err.to_string().contains("fixture_scores"), "expected undeclared-table error, got: {err}");
        assert!(!samgraha_dir.join("fixture-std").exists(), "cleanup-on-failure should remove the local copy");
    }
}
