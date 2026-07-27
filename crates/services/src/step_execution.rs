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

/// §2.10 — capture git state for a repo: commit SHA, branch, dirty flag.
/// Best-effort: returns None if not a git repo or git is unavailable.
fn capture_git_state(repo_root: &Path) -> Option<GitState> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit_sha = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let branch_output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    let branch = if branch_output.status.success() {
        String::from_utf8_lossy(&branch_output.stdout).trim().to_string()
    } else {
        String::new()
    };

    let status_output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    let dirty = if status_output.status.success() {
        !status_output.stdout.is_empty()
    } else {
        false
    };

    Some(GitState { commit_sha, branch, dirty })
}

#[derive(Debug, Clone)]
struct GitState {
    commit_sha: String,
    branch: String,
    dirty: bool,
}

/// Run a `kind: "deterministic"` step's script and record the execution.
/// `input_json` is written to a temp file and handed to the script via
/// `--in`; samgraha never inspects its contents. §2.10: injects `_git`
/// into the input payload and captures git state for the execution row.
pub fn run_script_step(
    knowledge_db_path: &Path,
    step_id: i64,
    repo_root: &Path,
    input_json: &serde_json::Value,
    timeout_secs: Option<u64>,
) -> Result<serde_json::Value> {
    let conn = Connection::open(knowledge_db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

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

    // §2.10 — capture git state and inject _git into input payload
    let git_state = capture_git_state(repo_root);
    let mut enriched_input = input_json.clone();
    if let Some(ref git) = git_state {
        let git_obj = serde_json::json!({
            "commit_sha": git.commit_sha,
            "branch": git.branch,
            "dirty": git.dirty,
        });
        if let serde_json::Value::Object(ref mut map) = enriched_input {
            map.insert("_git".to_string(), git_obj);
        }
    }

    let in_path = std::env::temp_dir().join(format!("samgraha-step-in-{}.json", uuid::Uuid::new_v4()));
    std::fs::write(&in_path, serde_json::to_string(&enriched_input)?)?;

    let result = common::env::run_capability_script(script_path, repo_root, &in_path, timeout_secs);
    let _ = std::fs::remove_file(&in_path);
    let result = result?;

    let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("ok");
    let exec_id = record_execution(&conn, step_id, repo_root, status, git_state.as_ref())?;

    // §2.8, §2.9 — read proposal/artifact envelope from result.
    // Look up the real standard name and usecase_id from the step we just ran.
    let (standard_name, usecase_id): (String, i64) = conn.query_row(
        "SELECT u.standard, u.id FROM usecase u
         JOIN step s ON s.usecase_id = u.id
         WHERE s.id = ?1",
        rusqlite::params![step_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).context(format!("Could not resolve usecase for step {step_id}"))?;

    if let Some(proposal) = result.get("proposal") {
        let title = proposal.get("title").and_then(|v| v.as_str()).unwrap_or("untitled");
        let location = proposal.get("location").and_then(|v| v.as_str());
        conn.execute(
            "INSERT INTO proposal (standard, usecase_id, execution_id, title, location) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![standard_name, usecase_id, exec_id, title, location],
        )?;
    }
    if let Some(artifacts) = result.get("artifacts").and_then(|v| v.as_array()) {
        for art in artifacts {
            let name = art.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed");
            let art_type = art.get("type").and_then(|v| v.as_str()).unwrap_or("file");
            let location = art.get("location").and_then(|v| v.as_str()).unwrap_or("");
            let purpose = art.get("purpose").and_then(|v| v.as_str()).unwrap_or("");
            conn.execute(
                "INSERT INTO artifact (standard, execution_id, name, type, location, purpose) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![standard_name, exec_id, name, art_type, location, purpose],
            )?;
        }
    }

    Ok(result)
}

/// Stage a `kind: "semantic"` step's prompt content for the calling agent.
/// Does not run anything and does not record an execution row yet —
/// that happens once the agent's result is known
/// (`complete_semantic_step`). Samgraha reads the prompt for bytes only,
/// never for meaning.
pub fn prepare_semantic_step(knowledge_db_path: &Path, step_id: i64) -> Result<SemanticStepPrep> {
    let conn = Connection::open(knowledge_db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

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
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let kind: String = conn
        .query_row("SELECT kind FROM step WHERE id = ?1", rusqlite::params![step_id], |r| r.get(0))
        .context(format!("No step with id {step_id}"))?;
    if kind != "semantic" {
        bail!("step {step_id} is kind '{kind}', not 'semantic'");
    }
    let git_state = capture_git_state(repo_root);
    let _exec_id = record_execution(&conn, step_id, repo_root, status, git_state.as_ref())?;
    Ok(())
}

/// §2.10 — upsert git_detail, then insert execution with git_detail_id.
/// Returns the execution row id for callers that need to link proposals/artifacts.
fn record_execution(conn: &Connection, step_id: i64, repo_root: &Path, status: &str, git: Option<&GitState>) -> Result<i64> {
    let git_detail_id = if let Some(git) = git {
        // Upsert git_detail (unique on repo_root+commit_sha+dirty)
        conn.execute(
            "INSERT OR IGNORE INTO git_detail (repo_root, commit_sha, branch, dirty) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![repo_root.display().to_string(), git.commit_sha, git.branch, git.dirty as i64],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM git_detail WHERE repo_root = ?1 AND commit_sha = ?2 AND dirty = ?3",
            rusqlite::params![repo_root.display().to_string(), git.commit_sha, git.dirty as i64],
            |r| r.get(0),
        )?;
        Some(id)
    } else {
        None
    };
    conn.execute(
        "INSERT INTO execution (step_id, repo_root, status, git_detail_id) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![step_id, repo_root.display().to_string(), status, git_detail_id],
    )?;
    Ok(conn.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db(tmp: &Path) -> (i64, i64) {
        let db_path = tmp.join("knowledge.db");
        let conn = Connection::open(&db_path).unwrap();
        registry::core_schema::run_core_migrations(&conn).unwrap();
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
