//! Layer A — structural completeness audit. Pure SQL against
//! `knowledge.db`, no new declared file needed. Checks that every
//! declared entity has the references it should, and every existing
//! table is accounted for. Runs as the final mandatory step of
//! `activate_standard`.

use anyhow::{bail, Result};
use rusqlite::Connection;

/// Run the full Layer A structural completeness audit for a standard.
/// Returns Ok(()) if every check passes, or Err with a descriptive
/// message identifying the first failure.
pub fn run_layer_a_audit(conn: &Connection, standard: &str) -> Result<()> {
    audit_domains_have_usecases(conn, standard)?;
    audit_usecases_have_steps(conn, standard)?;
    audit_deterministic_steps_have_scripts(conn, standard)?;
    audit_semantic_steps_have_prompts(conn, standard)?;
    audit_scripts_are_referenced(conn, standard)?;
    audit_prompts_are_referenced(conn, standard)?;
    audit_custom_tables_exist(conn, standard)?;
    audit_bidirectional_tables(conn, standard)?;
    Ok(())
}

/// Every `domain` row for this standard is referenced by ≥1 `usecase`.
fn audit_domains_have_usecases(conn: &Connection, standard: &str) -> Result<()> {
    let orphans: Vec<String> = conn
        .prepare(
            "SELECT d.key FROM domain d
             LEFT JOIN usecase u ON u.domain_id = d.id
             WHERE d.standard = ?1
             GROUP BY d.id
             HAVING COUNT(u.id) = 0",
        )?
        .query_map(rusqlite::params![standard], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !orphans.is_empty() {
        bail!(
            "Layer A audit failed: domain(s) {} have no usecases referencing them",
            orphans.join(", ")
        );
    }
    Ok(())
}

/// Every `usecase` for this standard has ≥1 `step`.
fn audit_usecases_have_steps(conn: &Connection, standard: &str) -> Result<()> {
    let orphans: Vec<String> = conn
        .prepare(
            "SELECT u.name FROM usecase u
             LEFT JOIN step s ON s.usecase_id = u.id
             WHERE u.standard = ?1
             GROUP BY u.id
             HAVING COUNT(s.id) = 0",
        )?
        .query_map(rusqlite::params![standard], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !orphans.is_empty() {
        bail!(
            "Layer A audit failed: usecase(s) {} have no steps",
            orphans.join(", ")
        );
    }
    Ok(())
}

/// Every `step` with `kind = 'deterministic'` has exactly one `step_script` row.
fn audit_deterministic_steps_have_scripts(conn: &Connection, standard: &str) -> Result<()> {
    let orphans: Vec<(i64, String)> = conn
        .prepare(
            "SELECT s.id, u.name || ' step ' || s.step_order
             FROM step s
             JOIN usecase u ON u.id = s.usecase_id
             LEFT JOIN step_script ss ON ss.step_id = s.id
             WHERE u.standard = ?1 AND s.kind = 'deterministic' AND ss.step_id IS NULL",
        )?
        .query_map(rusqlite::params![standard], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if !orphans.is_empty() {
        let labels: Vec<String> = orphans.iter().map(|(_, label)| label.clone()).collect();
        bail!(
            "Layer A audit failed: deterministic step(s) {} have no step_script mapping",
            labels.join(", ")
        );
    }
    Ok(())
}

/// Every `step` with `kind = 'semantic'` has exactly one `step_prompt` row.
fn audit_semantic_steps_have_prompts(conn: &Connection, standard: &str) -> Result<()> {
    let orphans: Vec<(i64, String)> = conn
        .prepare(
            "SELECT s.id, u.name || ' step ' || s.step_order
             FROM step s
             JOIN usecase u ON u.id = s.usecase_id
             LEFT JOIN step_prompt sp ON sp.step_id = s.id
             WHERE u.standard = ?1 AND s.kind = 'semantic' AND sp.step_id IS NULL",
        )?
        .query_map(rusqlite::params![standard], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if !orphans.is_empty() {
        let labels: Vec<String> = orphans.iter().map(|(_, label)| label.clone()).collect();
        bail!(
            "Layer A audit failed: semantic step(s) {} have no step_prompt mapping",
            labels.join(", ")
        );
    }
    Ok(())
}

/// Every `script` row is referenced by ≥1 `step_script` (catches declared-but-unused).
fn audit_scripts_are_referenced(conn: &Connection, standard: &str) -> Result<()> {
    let unused: Vec<String> = conn
        .prepare(
            "SELECT sc.name FROM script sc
             LEFT JOIN step_script ss ON ss.script_id = sc.id
             WHERE sc.standard = ?1
             GROUP BY sc.id
             HAVING COUNT(ss.step_id) = 0",
        )?
        .query_map(rusqlite::params![standard], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !unused.is_empty() {
        bail!(
            "Layer A audit failed: script(s) {} are registered but not referenced by any step",
            unused.join(", ")
        );
    }
    Ok(())
}

/// Every `prompt` row is referenced by ≥1 `step_prompt` (catches declared-but-unused).
fn audit_prompts_are_referenced(conn: &Connection, standard: &str) -> Result<()> {
    let unused: Vec<String> = conn
        .prepare(
            "SELECT p.name FROM prompt p
             LEFT JOIN step_prompt sp ON sp.prompt_id = p.id
             WHERE p.standard = ?1
             GROUP BY p.id
             HAVING COUNT(sp.step_id) = 0",
        )?
        .query_map(rusqlite::params![standard], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !unused.is_empty() {
        bail!(
            "Layer A audit failed: prompt(s) {} are registered but not referenced by any step",
            unused.join(", ")
        );
    }
    Ok(())
}

/// Every `custom_data_tables.table_name` for this standard exists in
/// `sqlite_master` (catches declared-but-never-created).
fn audit_custom_tables_exist(conn: &Connection, standard: &str) -> Result<()> {
    let missing: Vec<String> = conn
        .prepare(
            "SELECT cdt.table_name FROM custom_data_tables cdt
             LEFT JOIN sqlite_master sm ON sm.type = 'table' AND sm.name = cdt.table_name
             WHERE cdt.standard = ?1 AND sm.name IS NULL",
        )?
        .query_map(rusqlite::params![standard], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !missing.is_empty() {
        bail!(
            "Layer A audit failed: custom_data_tables catalogs table(s) {} that don't exist in knowledge.db",
            missing.join(", ")
        );
    }
    Ok(())
}

/// Bidirectional: every non-reserved table that exists in `sqlite_master`
/// must have a corresponding `custom_data_tables` row for this standard.
/// Catches seeder-created tables that weren't declared in metadata.
fn audit_bidirectional_tables(conn: &Connection, standard: &str) -> Result<()> {
    use registry::migration::RESERVED_TABLE_NAMES;

    // Collect all user-created tables (not reserved, not custom_data_tables for this standard)
    let mut stmt = conn.prepare(
        "SELECT sm.name FROM sqlite_master sm
         WHERE sm.type = 'table'
         AND sm.name NOT LIKE '\\_%' ESCAPE '\\'
         AND sm.name NOT IN (SELECT cdt.table_name FROM custom_data_tables cdt WHERE cdt.standard = ?1)",
    )?;
    let all_tables: Vec<String> = stmt
        .query_map(rusqlite::params![standard], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    let undeclared: Vec<String> = all_tables
        .iter()
        .filter(|t| !RESERVED_TABLE_NAMES.contains(&t.as_str()))
        .filter(|t| {
            if t.starts_with("sqlite_") || t.starts_with("_") {
                return false;
            }
            // Table exists in sqlite_master but has no custom_data_tables row — undeclared.
            true
        })
        .cloned()
        .collect();

    if !undeclared.is_empty() {
        bail!(
            "Layer A audit failed: table(s) {} exist in knowledge.db but have no custom_data_tables row for standard '{}'",
            undeclared.join(", "),
            standard
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use registry::core_schema;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        core_schema::run_core_migrations(&conn).unwrap();
        conn
    }

    fn seed_standard(conn: &Connection, standard: &str) {
        // Create a minimal valid standard: 1 domain, 1 usecase, 1 step, 1 script
        conn.execute(
            "INSERT INTO domain (standard, key, description) VALUES (?1, 'main', 'Main domain')",
            rusqlite::params![standard],
        )
        .unwrap();
        let domain_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO usecase (standard, name, domain_id) VALUES (?1, 'uc1', ?2)",
            rusqlite::params![standard, domain_id],
        )
        .unwrap();
        let uc_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 1, 'deterministic', 'run')",
            rusqlite::params![uc_id],
        )
        .unwrap();
        let step_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO script (standard, name, location, purpose) VALUES (?1, 's1', '/tmp/s.py', '')",
            rusqlite::params![standard],
        )
        .unwrap();
        let script_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO step_script (step_id, script_id) VALUES (?1, ?2)",
            rusqlite::params![step_id, script_id],
        )
        .unwrap();
    }

    #[test]
    fn audit_passes_for_valid_standard() {
        let conn = setup_db();
        seed_standard(&conn, "test-std");
        run_layer_a_audit(&conn, "test-std").unwrap();
    }

    #[test]
    fn audit_fails_for_orphan_domain() {
        let conn = setup_db();
        // Add a domain with no usecases
        conn.execute(
            "INSERT INTO domain (standard, key, description) VALUES ('test-std', 'orphan', 'Empty domain')",
            [],
        )
        .unwrap();
        seed_standard(&conn, "test-std");
        let err = run_layer_a_audit(&conn, "test-std").unwrap_err();
        assert!(err.to_string().contains("orphan"), "expected orphan domain error, got: {err}");
    }

    #[test]
    fn audit_fails_for_unused_script() {
        let conn = setup_db();
        seed_standard(&conn, "test-std");
        // Add an extra script not referenced by any step
        conn.execute(
            "INSERT INTO script (standard, name, location, purpose) VALUES ('test-std', 'unused', '/tmp/u.py', '')",
            [],
        )
        .unwrap();
        let err = run_layer_a_audit(&conn, "test-std").unwrap_err();
        assert!(err.to_string().contains("unused"), "expected unused script error, got: {err}");
    }

    #[test]
    fn audit_fails_for_missing_custom_table() {
        let conn = setup_db();
        seed_standard(&conn, "test-std");
        // Declare a custom table that doesn't exist
        conn.execute(
            "INSERT INTO custom_data_tables (standard, table_name, purpose) VALUES ('test-std', 'ghost_table', 'does not exist')",
            [],
        )
        .unwrap();
        let err = run_layer_a_audit(&conn, "test-std").unwrap_err();
        assert!(err.to_string().contains("ghost_table"), "expected missing table error, got: {err}");
    }

    #[test]
    fn audit_fails_for_empty_undeclared_table() {
        let conn = setup_db();
        seed_standard(&conn, "test-std");
        // Create a table with zero rows — should still be caught as undeclared
        conn.execute_batch("CREATE TABLE seeder_data (id INTEGER PRIMARY KEY, val TEXT);")
            .unwrap();
        let err = run_layer_a_audit(&conn, "test-std").unwrap_err();
        assert!(
            err.to_string().contains("seeder_data"),
            "expected empty undeclared table error, got: {err}"
        );
    }
}
