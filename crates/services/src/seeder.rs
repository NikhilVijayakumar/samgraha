//! Runs a standard's seeder script once, against a target repo. Unlike
//! `run_script_step`, this needs no pre-existing `step_id` — the seeder is
//! what creates `step`/`usecase`/`script`/`prompt`/`domain`/`template` rows
//! in the first place, so it can't be resolved through a step lookup the
//! way every other dispatch is.

use anyhow::{bail, Result};
use std::path::Path;

/// Runs a standard's seeder script once. Constructs the enriched `--in`
/// envelope (injecting `_samgraha_dir` and `_knowledge_db`) and validates
/// the returned status — matching every other execution entry point's own
/// discipline.
pub fn run_seeder(
    repo_root: &Path,
    seeder_script_path: &Path,
    samgraha_dir: &Path,
    knowledge_db: &Path,
    timeout_secs: Option<u64>,
) -> Result<serde_json::Value> {
    let payload = serde_json::json!({
        "_samgraha_dir": samgraha_dir.display().to_string(),
        "_knowledge_db": knowledge_db.display().to_string(),
    });
    let in_path = std::env::temp_dir().join(format!(
        "samgraha-seed-in-{}.json",
        uuid::Uuid::new_v4()
    ));
    std::fs::write(&in_path, serde_json::to_string(&payload)?)?;
    let result = common::env::run_capability_script(
        seeder_script_path,
        repo_root,
        &in_path,
        None,
        timeout_secs,
    );
    let _ = std::fs::remove_file(&in_path);
    let result = result?;
    let status = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("ok");
    if status != "ok" {
        bail!("seeder script reported non-ok status: {status}");
    }
    Ok(result)
}

/// Absolutize relative paths the seeder wrote into `script.location` and
/// `standard_asset.location`. Every relative path (those not starting with
/// `/`) is prefixed with `<samgraha_dir>/<standard>/`. Paths containing a
/// `..` segment are rejected as contract violations — the seeder must use
/// absolute paths for anything outside its own copied tree.
pub fn absolutize_paths(
    conn: &rusqlite::Connection,
    standard: &str,
    samgraha_dir: &Path,
) -> Result<()> {
    let prefix = samgraha_dir.join(standard).display().to_string();
    for table in ["script", "standard_asset"] {
        let mut stmt = conn.prepare(&format!(
            "SELECT id, location FROM {table} WHERE standard = ?1 AND location NOT LIKE '/%'"
        ))?;
        let rows: Vec<(i64, String)> = stmt
            .query_map(rusqlite::params![standard], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        for (id, location) in rows {
            if location.split('/').any(|seg| seg == "..") {
                bail!(
                    "seeder for '{standard}' wrote a '..'-containing relative location \
                     ('{location}') in {table} — not allowed; use an absolute path for \
                     anything outside the standard's own copied tree"
                );
            }
            conn.execute(
                &format!("UPDATE {table} SET location = ?1 || '/' || location WHERE id = ?2"),
                rusqlite::params![prefix, id],
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use rusqlite::Connection;

    #[test]
    fn absolutize_prepends_prefix() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE script (id INTEGER PRIMARY KEY, standard TEXT, location TEXT);
             CREATE TABLE standard_asset (id INTEGER PRIMARY KEY, standard TEXT, location TEXT);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO script (id, standard, location) VALUES (1, 'my-std', 'scripts/audit.py')",
            [],
        )
        .unwrap();
        let samgraha_dir = Path::new("/opt/repo/.samgraha");
        absolutize_paths(&conn, "my-std", samgraha_dir).unwrap();
        let loc: String = conn
            .query_row("SELECT location FROM script WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        let expected = samgraha_dir.join("my-std").join("scripts").join("audit.py");
        assert_eq!(PathBuf::from(loc), expected);
    }

    #[test]
    fn absolutize_skips_already_absolute() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE script (id INTEGER PRIMARY KEY, standard TEXT, location TEXT);
             CREATE TABLE standard_asset (id INTEGER PRIMARY KEY, standard TEXT, location TEXT);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO script (id, standard, location) VALUES (1, 'my-std', '/usr/bin/python')",
            [],
        )
        .unwrap();
        let samgraha_dir = Path::new("/opt/repo/.samgraha");
        absolutize_paths(&conn, "my-std", samgraha_dir).unwrap();
        let loc: String = conn
            .query_row("SELECT location FROM script WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(loc, "/usr/bin/python");
    }

    #[test]
    fn absolutize_rejects_dotdot() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE script (id INTEGER PRIMARY KEY, standard TEXT, location TEXT);
             CREATE TABLE standard_asset (id INTEGER PRIMARY KEY, standard TEXT, location TEXT);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO script (id, standard, location) VALUES (1, 'my-std', '../outside/script.py')",
            [],
        )
        .unwrap();
        let samgraha_dir = Path::new("/opt/repo/.samgraha");
        let err = absolutize_paths(&conn, "my-std", samgraha_dir).unwrap_err();
        assert!(err.to_string().contains(".."), "expected rejection of .. path");
    }
}
