//! §2.5 — Seeds a standard's usecases in a target repo. Walks `depends_on`
//! transitively, executes every `driver: samgraha` prerequisite in topological
//! order before the requested usecase.

use anyhow::{bail, Result};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SeedResult {
    pub standard: String,
    pub target_usecase: Option<String>,
    pub executed: Vec<SeedExecution>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SeedExecution {
    pub usecase: String,
    pub step_id: i64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Seed a standard's usecases in a target repo. If `usecase_filter` is Some,
/// only that usecase (and its transitive dependencies) are executed. Otherwise
/// all `driver: samgraha` usecases are executed in dependency order.
pub fn seed_standard(
    knowledge_db_path: &Path,
    standard: &str,
    repo_root: &Path,
    usecase_filter: Option<&str>,
) -> Result<SeedResult> {
    let conn = Connection::open(knowledge_db_path)?;
    registry::core_schema::ensure_current_schema(&conn)?;

    // Load all usecases for this standard
    let mut stmt = conn.prepare(
        "SELECT id, name, data FROM usecase WHERE standard = ?1 ORDER BY id",
    )?;
    let usecases: Vec<(i64, String, String)> = stmt
        .query_map(rusqlite::params![standard], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if usecases.is_empty() {
        bail!("No usecases found for standard '{}'", standard);
    }

    // Build name→(id, driver, depends_on) map
    let mut uc_map: HashMap<String, (i64, String, Vec<String>)> = HashMap::new();
    for (id, name, data_str) in &usecases {
        let data: serde_json::Value = serde_json::from_str(data_str).unwrap_or(serde_json::json!({}));
        let driver = data.get("driver").and_then(|v| v.as_str()).unwrap_or("samgraha").to_string();
        let depends_on: Vec<String> = data
            .get("depends_on")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        uc_map.insert(name.clone(), (*id, driver, depends_on));
    }

    // Determine which usecases to execute
    let to_execute: Vec<String> = if let Some(target) = usecase_filter {
        // Walk transitive dependencies of the target
        let mut needed = HashSet::new();
        let mut stack = vec![target.to_string()];
        while let Some(name) = stack.pop() {
            if needed.contains(&name) {
                continue;
            }
            needed.insert(name.clone());
            if let Some((_id, _driver, deps)) = uc_map.get(&name) {
                for dep in deps {
                    if !needed.contains(dep) {
                        stack.push(dep.clone());
                    }
                }
            }
        }
        // Topological sort of needed usecases
        topological_sort(&uc_map, &needed)?
    } else {
        // Execute all driver=samgraha usecases in dependency order
        let needed: HashSet<String> = uc_map.keys().cloned().collect();
        topological_sort(&uc_map, &needed)?
    };

    let mut executed = Vec::new();
    for uc_name in &to_execute {
        let (uc_id, driver, _deps) = match uc_map.get(uc_name) {
            Some(v) => v.clone(),
            None => continue,
        };
        if driver != "samgraha" {
            continue; // Skip external usecases — nothing to run
        }

        // Find all deterministic steps for this usecase
        let step_ids: Vec<i64> = {
            let mut step_stmt = conn.prepare(
                "SELECT s.id FROM step s
                 JOIN step_script ss ON ss.step_id = s.id
                 WHERE s.usecase_id = ?1
                 ORDER BY s.step_order",
            )?;
            let result = step_stmt
                .query_map(rusqlite::params![uc_id], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect::<Vec<i64>>();
            result
        };

        for step_id in step_ids {
            let input = serde_json::json!({});
            match crate::step_execution::run_script_step(knowledge_db_path, step_id, repo_root, &input, None) {
                Ok(_) => {
                    executed.push(SeedExecution {
                        usecase: uc_name.clone(),
                        step_id,
                        status: "ok".to_string(),
                        error: None,
                    });
                }
                Err(e) => {
                    executed.push(SeedExecution {
                        usecase: uc_name.clone(),
                        step_id,
                        status: "error".to_string(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    Ok(SeedResult {
        standard: standard.to_string(),
        target_usecase: usecase_filter.map(String::from),
        executed,
    })
}

/// Topological sort of usecases by depends_on. Returns names in execution
/// order (dependencies first). Skips usecases not in `needed`.
fn topological_sort(
    uc_map: &HashMap<String, (i64, String, Vec<String>)>,
    needed: &HashSet<String>,
) -> Result<Vec<String>> {
    let mut visited = HashSet::new();
    let mut order = Vec::new();

    fn visit(
        name: &str,
        uc_map: &HashMap<String, (i64, String, Vec<String>)>,
        needed: &HashSet<String>,
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if visited.contains(name) {
            return Ok(());
        }
        visited.insert(name.to_string());
        if let Some((_id, _driver, deps)) = uc_map.get(name) {
            for dep in deps {
                if needed.contains(dep) {
                    visit(dep, uc_map, needed, visited, order)?;
                }
            }
        }
        order.push(name.to_string());
        Ok(())
    }

    for name in needed {
        visit(name, uc_map, needed, &mut visited, &mut order)?;
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    #[cfg(unix)]
    use std::fs;

    fn setup_db() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("knowledge.db");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();
        (tmp, db)
    }

    #[cfg(unix)]
    fn insert_usecase(conn: &Connection, standard: &str, name: &str, driver: &str, depends_on: &[&str]) {
        let deps_json = serde_json::json!(depends_on);
        let data = serde_json::json!({"driver": driver, "depends_on": deps_json});
        conn.execute(
            "INSERT INTO usecase (standard, name, data) VALUES (?1, ?2, ?3)",
            rusqlite::params![standard, name, data.to_string()],
        ).unwrap();
    }

    // --- Topological sort unit tests (no scripts needed) ---

    #[test]
    fn linear_chain_abc_runs_in_order() {
        let mut uc_map: HashMap<String, (i64, String, Vec<String>)> = HashMap::new();
        uc_map.insert("a".into(), (1, "samgraha".into(), vec![]));
        uc_map.insert("b".into(), (2, "samgraha".into(), vec!["a".into()]));
        uc_map.insert("c".into(), (3, "samgraha".into(), vec!["b".into()]));
        let needed: HashSet<String> = uc_map.keys().cloned().collect();
        let order = topological_sort(&uc_map, &needed).unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn diamond_dependency() {
        //   top
        //   / \
        //  mid  mid2
        //   \ /
        //   bot
        let mut uc_map: HashMap<String, (i64, String, Vec<String>)> = HashMap::new();
        uc_map.insert("top".into(),  (1, "samgraha".into(), vec![]));
        uc_map.insert("mid".into(),  (2, "samgraha".into(), vec!["top".into()]));
        uc_map.insert("mid2".into(), (3, "samgraha".into(), vec!["top".into()]));
        uc_map.insert("bot".into(),  (4, "samgraha".into(), vec!["mid".into(), "mid2".into()]));
        let needed: HashSet<String> = uc_map.keys().cloned().collect();
        let order = topological_sort(&uc_map, &needed).unwrap();
        let top_idx = order.iter().position(|n| n == "top").unwrap();
        let mid_idx = order.iter().position(|n| n == "mid").unwrap();
        let mid2_idx = order.iter().position(|n| n == "mid2").unwrap();
        let bot_idx = order.iter().position(|n| n == "bot").unwrap();
        assert!(top_idx < mid_idx);
        assert!(top_idx < mid2_idx);
        assert!(mid_idx < bot_idx);
        assert!(mid2_idx < bot_idx);
    }

    #[test]
    fn external_driver_skip() {
        // External driver usecases should be present in the map but
        // topological_sort still includes them — seed_standard itself
        // filters them out at execution time.
        let mut uc_map: HashMap<String, (i64, String, Vec<String>)> = HashMap::new();
        uc_map.insert("ext".into(), (1, "external".into(), vec![]));
        uc_map.insert("inner".into(), (2, "samgraha".into(), vec!["ext".into()]));
        let needed: HashSet<String> = uc_map.keys().cloned().collect();
        let order = topological_sort(&uc_map, &needed).unwrap();
        // Both are present; ext comes before inner (dependency)
        assert_eq!(order, vec!["ext", "inner"]);
    }

    #[test]
    fn unknown_dependency_is_ignored_gracefully() {
        // A usecase depends on something not in uc_map —
        // topological_sort skips it (not in needed).
        let mut uc_map: HashMap<String, (i64, String, Vec<String>)> = HashMap::new();
        uc_map.insert("a".into(), (1, "samgraha".into(), vec!["missing".into()]));
        let needed: HashSet<String> = uc_map.keys().cloned().collect();
        let order = topological_sort(&uc_map, &needed).unwrap();
        // "missing" never appears; "a" still executes
        assert_eq!(order, vec!["a"]);
    }

    #[test]
    fn seed_standard_bails_on_empty_standard() {
        let (_tmp, db) = setup_db();
        let result = seed_standard(&db, "nonexistent", Path::new("/tmp"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No usecases found"));
    }

    #[cfg(unix)]
    #[test]
    fn seed_standard_executes_usecases_in_order() {
        let (_tmp, db) = setup_db();
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        insert_usecase(&conn, "test_std", "init", "samgraha", &[]);
        insert_usecase(&conn, "test_std", "deploy", "samgraha", &["init"]);

        // Insert a minimal script and step records for each usecase
        // so seed_standard actually has steps to iterate over.
        let script_path = _tmp.path().join("noop.sh");
        fs::write(&script_path, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&script_path, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
        let script_loc = script_path.to_str().unwrap();

        // Get usecase ids
        let init_id: i64 = conn.query_row(
            "SELECT id FROM usecase WHERE standard = 'test_std' AND name = 'init'",
            [], |r| r.get(0),
        ).unwrap();
        let deploy_id: i64 = conn.query_row(
            "SELECT id FROM usecase WHERE standard = 'test_std' AND name = 'deploy'",
            [], |r| r.get(0),
        ).unwrap();

        // Insert scripts (script table: standard, name, location, purpose)
        conn.execute(
            "INSERT INTO script (standard, name, location) VALUES ('test_std', 'init_script', ?1)",
            rusqlite::params![script_loc],
        ).unwrap();
        conn.execute(
            "INSERT INTO script (standard, name, location) VALUES ('test_std', 'deploy_script', ?1)",
            rusqlite::params![script_loc],
        ).unwrap();

        // Insert steps
        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 1, 'deterministic', 'init step')",
            rusqlite::params![init_id],
        ).unwrap();
        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 1, 'deterministic', 'deploy step')",
            rusqlite::params![deploy_id],
        ).unwrap();

        // Map steps to scripts
        let init_step_id: i64 = conn.query_row(
            "SELECT id FROM step WHERE usecase_id = ?1 ORDER BY step_order LIMIT 1",
            rusqlite::params![init_id], |r| r.get(0),
        ).unwrap();
        let deploy_step_id: i64 = conn.query_row(
            "SELECT id FROM step WHERE usecase_id = ?1 ORDER BY step_order LIMIT 1",
            rusqlite::params![deploy_id], |r| r.get(0),
        ).unwrap();

        let init_script_id: i64 = conn.query_row(
            "SELECT id FROM script WHERE name = 'init_script'", [], |r| r.get(0),
        ).unwrap();
        let deploy_script_id: i64 = conn.query_row(
            "SELECT id FROM script WHERE name = 'deploy_script'", [], |r| r.get(0),
        ).unwrap();

        conn.execute(
            "INSERT INTO step_script (step_id, script_id) VALUES (?1, ?2)",
            rusqlite::params![init_step_id, init_script_id],
        ).unwrap();
        conn.execute(
            "INSERT INTO step_script (step_id, script_id) VALUES (?1, ?2)",
            rusqlite::params![deploy_step_id, deploy_script_id],
        ).unwrap();

        // Now seed — init should execute before deploy
        let result = seed_standard(&db, "test_std", _tmp.path(), None).unwrap();
        assert_eq!(result.standard, "test_std");
        assert_eq!(result.executed.len(), 2);
        let init_pos = result.executed.iter().position(|e| e.usecase == "init").unwrap();
        let deploy_pos = result.executed.iter().position(|e| e.usecase == "deploy").unwrap();
        assert!(init_pos < deploy_pos, "init ({init_pos}) should execute before deploy ({deploy_pos})");
    }
}
