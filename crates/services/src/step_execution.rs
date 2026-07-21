//! Dispatches a usecase's steps. `kind: "deterministic"` runs a script
//! directly (samgraha's one fixed contract:
//! `common::env::run_capability_script`'s `--repo-root`/`--in`/`--out`).
//! `kind: "semantic"` splits into two calls, since MCP is request-response
//! and can't block mid-call while an external model reasons:
//! `prepare_semantic_step` stages the prompt content, the calling agent
//! reasons off-MCP, then whatever persists the result is just the *next*
//! deterministic step in the usecase's sequence — there is no separate
//! "post-script" concept here, a step is the atomic unit and a
//! pre/semantic/post triad is simply three consecutive steps.

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SemanticStepPrep {
    pub step_id: i64,
    pub description: String,
    pub prompt_name: String,
    pub prompt_content: String,
}

/// Run a `kind: "deterministic"` step's script and record the execution.
/// `input_json` is written to a temp file and handed to the script via
/// `--in`; samgraha never inspects its contents.
pub fn run_script_step(
    knowledge_db_path: &Path,
    step_id: i64,
    repo_root: &Path,
    input_json: &serde_json::Value,
    timeout_secs: Option<u64>,
) -> Result<serde_json::Value> {
    let conn = Connection::open(knowledge_db_path)?;

    let kind: String = conn
        .query_row("SELECT kind FROM step WHERE id = ?1", rusqlite::params![step_id], |r| r.get(0))
        .context(format!("No step with id {step_id}"))?;
    if kind != "deterministic" {
        bail!("step {step_id} is kind '{kind}', not 'deterministic' — use prepare_semantic_step instead");
    }

    let script_location: String = conn.query_row(
        "SELECT s.location FROM script s
         JOIN step_script ss ON ss.script_id = s.id
         WHERE ss.step_id = ?1",
        rusqlite::params![step_id],
        |r| r.get(0),
    ).context(format!("No script mapped to deterministic step {step_id}"))?;

    let script_path = Path::new(&script_location);
    if !script_path.exists() {
        bail!("script location no longer exists on disk: {script_location}");
    }

    let in_path = std::env::temp_dir().join(format!("samgraha-step-in-{}.json", uuid::Uuid::new_v4()));
    std::fs::write(&in_path, serde_json::to_string(input_json)?)?;

    let result = common::env::run_capability_script(script_path, repo_root, &in_path, timeout_secs);
    let _ = std::fs::remove_file(&in_path);
    let result = result?;

    let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("ok");
    record_execution(&conn, step_id, repo_root, status)?;

    Ok(result)
}

/// Stage a `kind: "semantic"` step's prompt content for the calling agent.
/// Does not run anything and does not record an execution row yet —
/// that happens once the agent's result is known
/// (`complete_semantic_step`). Samgraha reads the prompt for bytes only,
/// never for meaning.
pub fn prepare_semantic_step(knowledge_db_path: &Path, step_id: i64) -> Result<SemanticStepPrep> {
    let conn = Connection::open(knowledge_db_path)?;

    let (kind, description): (String, String) = conn
        .query_row(
            "SELECT kind, description FROM step WHERE id = ?1",
            rusqlite::params![step_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .context(format!("No step with id {step_id}"))?;
    if kind != "semantic" {
        bail!("step {step_id} is kind '{kind}', not 'semantic' — use run_script_step instead");
    }

    let (prompt_name, prompt_content): (String, String) = conn.query_row(
        "SELECT p.name, p.content FROM prompt p
         JOIN step_prompt sp ON sp.prompt_id = p.id
         WHERE sp.step_id = ?1",
        rusqlite::params![step_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).context(format!("No prompt mapped to semantic step {step_id}"))?;

    Ok(SemanticStepPrep {
        step_id,
        description,
        prompt_name,
        prompt_content,
    })
}

/// Record that a `kind: "semantic"` step's agent-side reasoning finished.
/// Persisting the agent's actual result (into a custom table, a file,
/// anything) is the job of the *next* deterministic step in the usecase
/// sequence, run via `run_script_step` with the agent's result as its
/// input — samgraha never persists a semantic result itself.
pub fn complete_semantic_step(knowledge_db_path: &Path, step_id: i64, repo_root: &Path, status: &str) -> Result<()> {
    let conn = Connection::open(knowledge_db_path)?;
    let kind: String = conn
        .query_row("SELECT kind FROM step WHERE id = ?1", rusqlite::params![step_id], |r| r.get(0))
        .context(format!("No step with id {step_id}"))?;
    if kind != "semantic" {
        bail!("step {step_id} is kind '{kind}', not 'semantic'");
    }
    record_execution(&conn, step_id, repo_root, status)
}

fn record_execution(conn: &Connection, step_id: i64, repo_root: &Path, status: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO execution (step_id, repo_root, status) VALUES (?1, ?2, ?3)",
        rusqlite::params![step_id, repo_root.display().to_string(), status],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db(tmp: &Path) -> (i64, i64) {
        let db_path = tmp.join("knowledge.db");
        let conn = Connection::open(&db_path).unwrap();
        for m in registry::core_schema::CORE_MIGRATIONS {
            conn.execute_batch(m).unwrap();
        }
        conn.execute("INSERT INTO usecase (standard, name) VALUES ('t', 'uc')", []).unwrap();
        let uc_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 1, 'deterministic', 'run script')",
            rusqlite::params![uc_id],
        ).unwrap();
        let det_step = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 2, 'semantic', 'write narrative')",
            rusqlite::params![uc_id],
        ).unwrap();
        let sem_step = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO script (standard, name, location, purpose) VALUES ('t', 'echo', ?1, '')",
            rusqlite::params![tmp.join("echo.py").display().to_string()],
        ).unwrap();
        let script_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO step_script (step_id, script_id) VALUES (?1, ?2)", rusqlite::params![det_step, script_id]).unwrap();

        conn.execute(
            "INSERT INTO prompt (standard, name, purpose, content) VALUES ('t', 'narrative-prompt', '', 'Write a summary.')",
            [],
        ).unwrap();
        let prompt_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO step_prompt (step_id, prompt_id) VALUES (?1, ?2)", rusqlite::params![sem_step, prompt_id]).unwrap();

        (det_step, sem_step)
    }

    #[test]
    fn run_script_step_executes_and_records_execution() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("echo.py"),
            "import sys, json\n\
             i = sys.argv[sys.argv.index('--out') + 1]\n\
             json.dump({'status': 'ok'}, open(i, 'w'))\n",
        ).unwrap();
        let (det_step, _) = setup_db(tmp.path());
        let db_path = tmp.path().join("knowledge.db");

        let result = run_script_step(&db_path, det_step, tmp.path(), &serde_json::json!({}), Some(10)).unwrap();
        assert_eq!(result["status"], "ok");

        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM execution WHERE step_id = ?1", rusqlite::params![det_step], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn run_script_step_rejects_semantic_step() {
        let tmp = tempfile::tempdir().unwrap();
        let (_, sem_step) = setup_db(tmp.path());
        let db_path = tmp.path().join("knowledge.db");
        let err = run_script_step(&db_path, sem_step, tmp.path(), &serde_json::json!({}), Some(10)).unwrap_err();
        assert!(err.to_string().contains("not 'deterministic'"));
    }

    #[test]
    fn prepare_semantic_step_returns_prompt_content() {
        let tmp = tempfile::tempdir().unwrap();
        let (_, sem_step) = setup_db(tmp.path());
        let db_path = tmp.path().join("knowledge.db");
        let prep = prepare_semantic_step(&db_path, sem_step).unwrap();
        assert_eq!(prep.prompt_content, "Write a summary.");
        assert_eq!(prep.prompt_name, "narrative-prompt");
    }

    #[test]
    fn prepare_semantic_step_rejects_deterministic_step() {
        let tmp = tempfile::tempdir().unwrap();
        let (det_step, _) = setup_db(tmp.path());
        let db_path = tmp.path().join("knowledge.db");
        let err = prepare_semantic_step(&db_path, det_step).unwrap_err();
        assert!(err.to_string().contains("not 'semantic'"));
    }

    #[test]
    fn complete_semantic_step_records_execution() {
        let tmp = tempfile::tempdir().unwrap();
        let (_, sem_step) = setup_db(tmp.path());
        let db_path = tmp.path().join("knowledge.db");
        complete_semantic_step(&db_path, sem_step, tmp.path(), "ok").unwrap();

        let conn = Connection::open(&db_path).unwrap();
        let status: String = conn.query_row("SELECT status FROM execution WHERE step_id = ?1", rusqlite::params![sem_step], |r| r.get(0)).unwrap();
        assert_eq!(status, "ok");
    }
}
