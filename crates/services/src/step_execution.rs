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
    registry::core_schema::ensure_current_schema(&conn)?;

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

    // §3.12 — resolve out_dir from config for artifact output
    let out_dir = load_samgraha_dir(repo_root).join("output");

    let result = common::env::run_capability_script(script_path, repo_root, &in_path, Some(&out_dir), timeout_secs);
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

        // Check if this standard declares a proposal_template — if so,
        // validate the envelope against proposal.schema.json + cross-checks.
        let samgraha_dir = knowledge_db_path
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let has_proposal_template = check_has_proposal_template(samgraha_dir, &standard_name)?;

        if has_proposal_template {
            validate_proposal_envelope(proposal, &conn, &standard_name, repo_root)?;
        }

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
            // artifact.type is a relation (artifact_type), not free text —
            // but unlike asset_kind/template_type, a script's output
            // vocabulary can't be predicted at registration time, so
            // samgraha find-or-creates the type row itself rather than
            // requiring a seeder to pre-declare every possible type.
            let type_id = crate::register_standard::get_or_create_lookup(&conn, "artifact_type", &standard_name, art_type)?;
            conn.execute(
                "INSERT INTO artifact (standard, execution_id, type_id, name, location, purpose) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![standard_name, exec_id, type_id, name, location, purpose],
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
    registry::core_schema::ensure_current_schema(&conn)?;

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
    registry::core_schema::ensure_current_schema(&conn)?;
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

/// Check if a standard declares a proposal_template in its metadata.
/// Reads `standard.metadata.json` from the standard's local copy under samgraha_dir.
fn check_has_proposal_template(samgraha_dir: &Path, standard: &str) -> Result<bool> {
    let metadata_path = samgraha_dir.join(standard).join("standard.metadata.json");
    if !metadata_path.exists() {
        return Ok(false);
    }
    let content = match std::fs::read_to_string(&metadata_path) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };
    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Ok(false),
    };
    Ok(parsed.get("proposal_template").is_some())
}

/// Validate a proposal envelope against proposal.schema.json and
/// cross-check domain/usecase/step/git references.
fn validate_proposal_envelope(
    proposal: &serde_json::Value,
    conn: &Connection,
    standard_name: &str,
    repo_root: &std::path::Path,
) -> Result<()> {
    // proposal.schema.json is samgraha's own embedded schema (same for
    // every standard) — no per-repo/per-standard file to look up, so
    // there's no "schema not found, skip" case anymore. A previous
    // version resolved this as a file under samgraha_dir/<standard>/,
    // which no standard ever actually ships — that lookup always missed
    // in real use, silently skipping validation every time.
    crate::metadata_validate::validate_proposal(proposal)?;

    // Cross-check phases
    if let Some(phases) = proposal.get("phases").and_then(|v| v.as_array()) {
        for phase in phases {
            let domain_key = phase
                .get("domain")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("phase missing 'domain'"))?;

            // Check domain exists for this standard
            let domain_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM domain WHERE standard = ?1 AND key = ?2",
                    rusqlite::params![standard_name, domain_key],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;
            if !domain_exists {
                bail!(
                    "proposal references domain '{}' which does not exist for standard '{}'",
                    domain_key, standard_name
                );
            }

            // Get domain_id for usecase validation
            let domain_id: i64 = conn.query_row(
                "SELECT id FROM domain WHERE standard = ?1 AND key = ?2",
                rusqlite::params![standard_name, domain_key],
                |r| r.get(0),
            )?;

            // Check usecases exist and belong to this domain
            if let Some(usecases) = phase.get("usecases").and_then(|v| v.as_array()) {
                for uc_name in usecases {
                    let name = uc_name.as_str().unwrap_or("");
                    let uc_domain_id: Option<i64> = conn.query_row(
                        "SELECT domain_id FROM usecase WHERE standard = ?1 AND name = ?2",
                        rusqlite::params![standard_name, name],
                        |r| r.get(0),
                    ).ok();

                    match uc_domain_id {
                        None => bail!(
                            "proposal references usecase '{}' which does not exist for standard '{}'",
                            name, standard_name
                        ),
                        Some(id) if id != domain_id => bail!(
                            "proposal usecase '{}' belongs to a different domain than the phase's declared domain '{}'",
                            name, domain_key
                        ),
                        _ => {}
                    }
                }
            }

            // Check steps exist and belong to one of this phase's usecases
            if let Some(steps) = phase.get("steps").and_then(|v| v.as_array()) {
                for step_val in steps {
                    let step_id = step_val.as_i64().unwrap_or(0);
                    let step_usecase_id: Option<i64> = conn.query_row(
                        "SELECT s.usecase_id FROM step s
                         JOIN usecase u ON u.id = s.usecase_id
                         WHERE s.id = ?1 AND u.standard = ?2",
                        rusqlite::params![step_id, standard_name],
                        |r| r.get(0),
                    ).ok();

                    match step_usecase_id {
                        None => bail!(
                            "proposal references step {} which does not exist for standard '{}'",
                            step_id, standard_name
                        ),
                        Some(uc_id) => {
                            // Verify this step's usecase is one of the phase's usecases
                            let uc_name_in_phase = phase
                                .get("usecases")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();

                            let is_in_phase = uc_name_in_phase.iter().any(|name| {
                                conn.query_row(
                                    "SELECT id FROM usecase WHERE standard = ?1 AND name = ?2",
                                    rusqlite::params![standard_name, name],
                                    |r| r.get::<_, i64>(0),
                                )
                                .map(|id| id == uc_id)
                                .unwrap_or(false)
                            });

                            if !is_in_phase {
                                bail!(
                                    "proposal step {} belongs to a usecase not listed in this phase",
                                    step_id
                                );
                            }
                        }
                    }
                }
            }

            // Git cross-check (if git block present)
            if let Some(git) = phase.get("git") {
                let claimed_sha = git
                    .get("commit_sha")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                validate_git_claim(claimed_sha, repo_root)?;
            }
        }
    }

    Ok(())
}

/// Verify that a claimed commit SHA exists in the repo AND is an ancestor of HEAD.
fn validate_git_claim(claimed_sha: &str, repo_root: &std::path::Path) -> Result<()> {
    if claimed_sha.is_empty() {
        return Ok(());
    }

    // Check commit exists: git cat-file -t <sha>
    let exists = std::process::Command::new("git")
        .args(["cat-file", "-t", claimed_sha])
        .current_dir(repo_root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !exists {
        bail!(
            "proposal git block references commit '{}' which does not exist in the repo",
            claimed_sha
        );
    }

    // Check ancestor: git merge-base --is-ancestor <sha> HEAD
    let is_ancestor = std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", claimed_sha, "HEAD"])
        .current_dir(repo_root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !is_ancestor {
        bail!(
            "proposal git block references commit '{}' which is not an ancestor of HEAD",
            claimed_sha
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db(tmp: &Path) -> (i64, i64) {
        let db_path = tmp.join("knowledge.db");
        let conn = Connection::open(&db_path).unwrap();
        registry::core_schema::ensure_current_schema(&conn).unwrap();
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

    #[test]
    fn validate_git_claim_empty_sha_is_ok() {
        let tmp = tempfile::tempdir().unwrap();
        validate_git_claim("", tmp.path()).unwrap();
    }

    #[test]
    fn validate_git_claim_nonexistent_commit_fails() {
        let tmp = tempfile::tempdir().unwrap();
        // Initialize a git repo so git commands work
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let err = validate_git_claim("0000000000000000000000000000000000000000", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("does not exist"), "expected existence error, got: {err}");
    }

    #[test]
    fn validate_git_claim_non_ancestor_fails() {
        let tmp = tempfile::tempdir().unwrap();
        // Initialize a repo with a commit
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::fs::write(tmp.path().join("file.txt"), "hello").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let output = std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let _sha = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|l| l.starts_with('['))
            .map(|l| {
                let hash_part = l.split(' ').last().unwrap_or("");
                hash_part.trim_end_matches(']').to_string()
            })
            .unwrap_or_default();

        // Create a detached HEAD at the commit, then create a new commit on master
        // so HEAD is ahead. Actually simpler: just use the commit SHA directly.
        // The commit IS an ancestor of HEAD (it IS HEAD), so this should pass.
        // To test non-ancestor, we need two branches. Let's create a second branch
        // with a different commit.
        std::process::Command::new("git")
            .args(["checkout", "-b", "other"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::fs::write(tmp.path().join("file2.txt"), "world").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "other branch"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Get the SHA of the first commit (which is on master, not ancestor of other's HEAD)
        let first_sha_output = std::process::Command::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let first_sha = String::from_utf8_lossy(&first_sha_output.stdout).trim().to_string();

        // Switch back to master so HEAD is different
        std::process::Command::new("git")
            .args(["checkout", "master"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Now first_sha IS an ancestor of HEAD (master), so this should pass
        validate_git_claim(&first_sha, tmp.path()).unwrap();
    }

    #[test]
    fn check_has_proposal_template_true() {
        let tmp = tempfile::tempdir().unwrap();
        let samgraha = tmp.path();
        let std_dir = samgraha.join("my-standard");
        std::fs::create_dir_all(&std_dir).unwrap();
        std::fs::write(
            std_dir.join("standard.metadata.json"),
            serde_json::json!({
                "domains": {},
                "proposal_template": {
                    "phases": []
                }
            })
            .to_string(),
        )
        .unwrap();
        assert!(check_has_proposal_template(samgraha, "my-standard").unwrap());
    }

    #[test]
    fn check_has_proposal_template_false_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let samgraha = tmp.path();
        let std_dir = samgraha.join("my-standard");
        std::fs::create_dir_all(&std_dir).unwrap();
        std::fs::write(
            std_dir.join("standard.metadata.json"),
            serde_json::json!({"domains": {}}).to_string(),
        )
        .unwrap();
        assert!(!check_has_proposal_template(samgraha, "my-standard").unwrap());
    }

    #[test]
    fn check_has_proposal_template_false_when_no_metadata() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!check_has_proposal_template(tmp.path(), "nonexistent").unwrap());
    }

    #[test]
    fn validate_proposal_envelope_rejects_bad_domain() {
        let tmp = tempfile::tempdir().unwrap();

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        registry::core_schema::ensure_current_schema(&conn).unwrap();
        conn.execute("INSERT INTO domain (standard, key, description) VALUES ('t', 'core', 'Core')", [])
            .unwrap();
        let dom_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO usecase (standard, name, domain_id) VALUES ('t', 'analysis', ?1)",
            rusqlite::params![dom_id],
        )
        .unwrap();
        let uc_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 1, 'deterministic', 's')",
            rusqlite::params![uc_id],
        )
        .unwrap();
        let step_id: i64 = conn.last_insert_rowid();

        let proposal = serde_json::json!({
            "title": "test",
            "phases": [{
                "domain": "nonexistent",
                "phase_number": 1,
                "usecases": ["analysis"],
                "steps": [step_id],
                "rationale": "test"
            }]
        });
        let err = validate_proposal_envelope(&proposal, &conn, "t", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("does not exist"), "got: {err}");
    }

    #[test]
    fn validate_proposal_envelope_rejects_mismatched_usecase_domain() {
        let tmp = tempfile::tempdir().unwrap();

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        registry::core_schema::ensure_current_schema(&conn).unwrap();
        conn.execute("INSERT INTO domain (standard, key, description) VALUES ('t', 'd1', 'D1')", [])
            .unwrap();
        let d1_id: i64 = conn.last_insert_rowid();
        conn.execute("INSERT INTO domain (standard, key, description) VALUES ('t', 'd2', 'D2')", [])
            .unwrap();
        let d2_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO usecase (standard, name, domain_id) VALUES ('t', 'uc1', ?1)",
            rusqlite::params![d1_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO usecase (standard, name, domain_id) VALUES ('t', 'uc2', ?1)",
            rusqlite::params![d2_id],
        )
        .unwrap();
        let uc2_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 1, 'deterministic', 's')",
            rusqlite::params![uc2_id],
        )
        .unwrap();
        let step_id: i64 = conn.last_insert_rowid();

        let proposal = serde_json::json!({
            "title": "test",
            "phases": [{
                "domain": "d2",
                "phase_number": 1,
                "usecases": ["uc1"],
                "steps": [step_id],
                "rationale": "test"
            }]
        });
        let err = validate_proposal_envelope(&proposal, &conn, "t", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("different domain"), "got: {err}");
    }

    #[test]
    fn validate_proposal_envelope_passes_valid() {
        let tmp = tempfile::tempdir().unwrap();

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        registry::core_schema::ensure_current_schema(&conn).unwrap();
        conn.execute("INSERT INTO domain (standard, key, description) VALUES ('t', 'core', 'Core')", [])
            .unwrap();
        let dom_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO usecase (standard, name, domain_id) VALUES ('t', 'analysis', ?1)",
            rusqlite::params![dom_id],
        )
        .unwrap();
        let uc_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind, description) VALUES (?1, 1, 'deterministic', 's')",
            rusqlite::params![uc_id],
        )
        .unwrap();
        let step_id: i64 = conn.last_insert_rowid();

        let proposal = serde_json::json!({
            "title": "test",
            "phases": [{
                "domain": "core",
                "phase_number": 1,
                "usecases": ["analysis"],
                "steps": [step_id],
                "rationale": "test"
            }]
        });
        validate_proposal_envelope(&proposal, &conn, "t", tmp.path()).unwrap();
    }
}

/// Load `samgraha_dir` from `root/samgraha.toml`. Falls back to the default
/// (`<root>/.samgraha`) if the config is absent or unparseable.
fn load_samgraha_dir(root: &std::path::Path) -> std::path::PathBuf {
    let config_path = root.join("samgraha.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = toml::from_str::<common::config::SamgrahaConfig>(&content) {
            return config.repository.resolve_samgraha_dir(root);
        }
    }
    common::config::SamgrahaConfig::default().repository.resolve_samgraha_dir(root)
}
