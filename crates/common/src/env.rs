use anyhow::Context;
use std::path::Path;

/// Load `KEY=VALUE` lines from a `.env` file, found by walking up from the
/// current directory, into the process environment. Keys already set in the
/// real environment are left untouched — the file only fills gaps, same
/// precedence as the `dotenv` convention. Mirrors the manual parser in
/// `crates/mcp/build.rs` (no `dotenv` crate dependency); this is the runtime
/// counterpart — that one only runs at compile time for build-only keys.
///
/// Called once at process startup (CLI and MCP `main()`), before any config
/// field that uses `resolve_configured_dir` is read — otherwise `${VAR}`
/// placeholders in samgraha.toml would never see values from `.env`.
pub fn load_dotenv() {
    let Ok(cwd) = std::env::current_dir() else { return };
    let Some(path) = find_dotenv(&cwd) else { return };
    let Ok(content) = std::fs::read_to_string(&path) else { return };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, val)) = trimmed.split_once('=') else { continue };
        let key = key.trim();
        if key.is_empty() || std::env::var(key).is_ok() {
            continue;
        }
        let val = val.trim().trim_matches('"').trim_matches('\'');
        std::env::set_var(key, val);
    }
}

/// Cross-platform home directory resolution.
/// Windows uses `USERPROFILE`; Unix uses `HOME`. Falls back to `.` if neither
/// is set — matches the previous behavior but actually works on Windows.
pub fn home_dir() -> std::path::PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Compute a stable fingerprint for a repo root path. Used as the
/// `repo_fingerprint` column in script_runs / system_plans — same
/// convention check_runner uses (`{name}-{repo_root.display()}`), but
/// here the name is omitted because the fingerprint is repo-scoped, not
/// check-scoped.
pub fn repo_fingerprint(repo_root: &std::path::Path) -> String {
    repo_root.display().to_string()
}

/// Current git HEAD commit sha for a repo, or `None` if the repo has no
/// commits yet, isn't a git repo, or `git` isn't on PATH. Used to evaluate
/// `head_commit`-type expiry rules (§8.5) — a rule is expired when this
/// differs from the sha recorded at the time a phase last ran.
pub fn current_head_sha(repo_root: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sha = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if sha.is_empty() { None } else { Some(sha) }
}

/// Directory the running binary (mcp.exe / cli.exe) lives in — where
/// `standards.db` (knowledge-hub schema, multi-system) and `help.db`
/// (registry schema, `domain = 'help'` documents) ship alongside the build.
/// `SAMGRAHA_MCP_DIR` overrides for tests/dev; falls back to `.` if the
/// binary's own path can't be resolved.
pub fn mcp_dir() -> std::path::PathBuf {
    std::env::var("SAMGRAHA_MCP_DIR")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(std::path::PathBuf::from))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Resolve the Python interpreter to shell out to for the knowledge-hub
/// loader. Prefers `python3` (Unix convention); falls back to `python`
/// (common on Windows, where `python3` often isn't on PATH even with
/// Python 3 installed — verified on this dev machine).
pub fn python_command() -> std::process::Command {
    for candidate in ["python3", "python"] {
        if std::process::Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            return std::process::Command::new(candidate);
        }
    }
    std::process::Command::new("python3")
}

/// Build the `Command` to run a check script, wrapped in its interpreter —
/// `.ps1`/`.sh`/`.py`/`.js` aren't natively executable on every platform (a
/// `.ps1` isn't a Win32 executable even on Windows; a `.sh` has no
/// shebang-exec on Windows at all; `.py`/`.js` need their interpreter
/// regardless of platform or shebang/execute-bit). Anything else (a compiled
/// native binary a `check_overrides` entry points at) runs directly. `args`
/// are appended after the script path.
///
/// `.rs` and other compile-first languages are deliberately not here —
/// "compile then run" is a different, heavier mechanism (needs a toolchain
/// check, is slow) than "invoke an interpreter that's either present or
/// isn't." Add one only when a standard actually ships a script needing it.
pub fn script_command(script_path: &Path, args: &[&str]) -> std::process::Command {
    match script_path.extension().and_then(|e| e.to_str()) {
        Some("ps1") => {
            let mut cmd = pwsh_command();
            cmd.arg("-NoProfile").arg("-NonInteractive").arg("-File").arg(script_path);
            cmd.args(args);
            cmd
        }
        Some("sh") => {
            let mut cmd = sh_command();
            cmd.arg(script_path);
            cmd.args(args);
            cmd
        }
        Some("py") => {
            let mut cmd = python_command();
            cmd.arg(script_path);
            cmd.args(args);
            cmd
        }
        Some("js") => {
            let mut cmd = node_command();
            cmd.arg(script_path);
            cmd.args(args);
            cmd
        }
        _ => {
            let mut cmd = std::process::Command::new(script_path);
            cmd.args(args);
            cmd
        }
    }
}

/// Resolve the Node interpreter — same probe-then-fallback shape as
/// `python_command`/`pwsh_command`/`sh_command`.
pub fn node_command() -> std::process::Command {
    if std::process::Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
    {
        return std::process::Command::new("node");
    }
    std::process::Command::new("node")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_command_dispatches_py_through_python_interpreter() {
        let cmd = script_command(Path::new("leaderboard.py"), &["--json-out", "out.json"]);
        let program = cmd.get_program().to_string_lossy().to_string();
        assert!(program == "python3" || program == "python", "expected a python interpreter, got '{}'", program);
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert_eq!(args, vec!["leaderboard.py", "--json-out", "out.json"]);
    }

    #[test]
    fn script_command_dispatches_js_through_node() {
        let cmd = script_command(Path::new("aggregate.js"), &["--team", "alpha"]);
        assert_eq!(cmd.get_program().to_string_lossy(), "node");
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert_eq!(args, vec!["aggregate.js", "--team", "alpha"]);
    }

    #[test]
    fn script_command_still_dispatches_sh_correctly() {
        let cmd = script_command(Path::new("check.sh"), &["--repo-root", "/tmp"]);
        let program = cmd.get_program().to_string_lossy().to_string();
        assert!(program == "sh" || program == "bash", "expected a shell interpreter, got '{}'", program);
    }

    #[test]
    fn script_command_runs_unrecognized_extension_directly() {
        // A check_overrides entry pointing at a compiled native binary.
        let cmd = script_command(Path::new("./checker"), &["--flag"]);
        assert_eq!(cmd.get_program().to_string_lossy(), "./checker");
    }

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("samgraha-test-env-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn run_workflow_script_captures_exit_code_stdout_args_and_env() {
        let dir = scratch_dir("workflow-script");
        let script = dir.join("echo_args.py");
        std::fs::write(
            &script,
            "import sys, os\n\
             print('args:' + ','.join(sys.argv[1:]))\n\
             print('team:' + os.environ.get('TEAM', 'missing'))\n\
             sys.exit(3)\n",
        )
        .unwrap();

        let result = run_workflow_script(
            &script,
            &dir,
            &["--json-out".to_string(), "out.json".to_string()],
            &[("TEAM".to_string(), "alpha".to_string())],
            Some(10),
        )
        .unwrap();

        assert_eq!(result.exit_code, Some(3));
        assert!(result.stdout.contains("args:--json-out,out.json"), "stdout: {}", result.stdout);
        assert!(result.stdout.contains("team:alpha"), "stdout: {}", result.stdout);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn run_workflow_script_kills_process_on_timeout() {
        let dir = scratch_dir("workflow-script-timeout");
        let script = dir.join("sleep_forever.py");
        std::fs::write(&script, "import time\ntime.sleep(30)\n").unwrap();

        let result = run_workflow_script(&script, &dir, &[], &[], Some(1));
        assert!(result.is_err(), "expected a timeout error");
        std::fs::remove_dir_all(&dir).ok();
    }
}

fn pwsh_command() -> std::process::Command {
    for candidate in ["pwsh", "powershell"] {
        if std::process::Command::new(candidate)
            .arg("-NoProfile")
            .arg("-Command")
            .arg("exit 0")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            return std::process::Command::new(candidate);
        }
    }
    std::process::Command::new("powershell")
}

fn sh_command() -> std::process::Command {
    for candidate in ["sh", "bash"] {
        if std::process::Command::new(candidate)
            .arg("-c")
            .arg("exit 0")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            return std::process::Command::new(candidate);
        }
    }
    std::process::Command::new("sh")
}

/// Run a check script (`.sh` or `.ps1`) using its *actual* interface
/// contract — verified directly against the real scripts in
/// `docs/knowledge-hub/script/{windows,ubuntu}/`, not assumed:
///
/// - `.ps1` scripts declare PowerShell-native mandatory params
///   (`-RepoRoot`, `-RepoFingerprint`, `-Out`) — GNU-style `--repo-root`
///   doesn't bind to them at all.
/// - `.sh` scripts parse GNU-style flags (`--repo-root`, `--repo-fingerprint`,
///   `--out`) via a manual `case` loop.
/// - **Neither** platform supports `-Out -` / `--out -` as "write to
///   stdout" — both always `Set-Content`/`cat >` a real file path. So this
///   always gives them a real temp file, reads it back after the process
///   exits, and never touches stdout for the result.
///
/// Returns the parsed JSON the script wrote — regardless of the check's own
/// pass/fail verdict, which lives in the JSON's `status` field. `Err` means
/// the script itself didn't run or didn't produce valid JSON (timeout,
/// missing script, crashed with no output, malformed JSON).
pub fn run_check_script(
    script_path: &Path,
    repo_root: &Path,
    repo_fingerprint: &str,
    timeout_secs: Option<u64>,
) -> anyhow::Result<serde_json::Value> {
    let out_file = std::env::temp_dir().join(format!("samgraha-check-{}.json", uuid::Uuid::new_v4()));
    let repo_root_str = repo_root.display().to_string();
    let out_str = out_file.display().to_string();

    let is_ps1 = script_path.extension().and_then(|e| e.to_str()) == Some("ps1");
    let args: Vec<String> = if is_ps1 {
        vec![
            "-RepoRoot".into(), repo_root_str,
            "-RepoFingerprint".into(), repo_fingerprint.into(),
            "-Out".into(), out_str.clone(),
        ]
    } else {
        vec![
            "--repo-root".into(), repo_root_str,
            "--repo-fingerprint".into(), repo_fingerprint.into(),
            "--out".into(), out_str.clone(),
        ]
    };
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut cmd = script_command(script_path, &arg_refs);
    cmd.current_dir(repo_root);

    let output = run_with_optional_timeout(cmd, timeout_secs).context("Failed to execute script")?;
    let content = std::fs::read_to_string(&out_file);
    let _ = std::fs::remove_file(&out_file);

    match content {
        // PowerShell's `Set-Content -Encoding UTF8` writes a UTF-8 BOM —
        // serde_json chokes on it before the opening `{`. Strip it if present;
        // .sh's `cat >` never writes one, so this is a no-op there.
        Ok(text) => serde_json::from_str(text.trim_start_matches('\u{FEFF}'))
            .with_context(|| format!("Script wrote invalid JSON to {}", out_file.display())),
        Err(_) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "Script did not write an output file (exit {:?}); stderr: {} stdout: {}",
                output.status.code(),
                stderr.trim(),
                stdout.trim()
            )
        }
    }
}

/// Run `cmd` to completion, killing it and returning a `TimedOut` error if
/// `timeout_secs` elapses first. Shared by `run_check_script` and
/// `run_workflow_script` — the only difference between the two contracts is
/// which args/env go into `cmd` before it gets here, not how it's run.
fn run_with_optional_timeout(
    mut cmd: std::process::Command,
    timeout_secs: Option<u64>,
) -> std::io::Result<std::process::Output> {
    let Some(secs) = timeout_secs else {
        return cmd.output();
    };
    // spawn() inherits the parent's stdio by default (unlike output(), which
    // pipes automatically) — without this, wait_with_output() below always
    // returns empty stdout/stderr regardless of what the child actually
    // printed. Pre-existing bug in this exact spawn/poll/kill shape before
    // it was extracted into this shared helper — surfaced by a test that
    // actually asserts on captured output instead of just checking exit
    // status.
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.spawn().and_then(|mut child| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
        loop {
            match child.try_wait() {
                Ok(Some(_status)) => return child.wait_with_output(),
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        let _ = child.kill();
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            format!("Script timed out after {}s", secs),
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => return Err(e),
            }
        }
    })
}

/// Run a script with the capability contract: `--repo-root`, `--in`,
/// `--out` — same structure as check scripts but without the fingerprint
/// argument (capabilities don't cache by fingerprint). The script must
/// write a JSON envelope to `--out`:
///
/// ```json
/// { "status": "ok"|"error", "message": "...", "written": [...] }
/// ```
///
/// Returns the parsed JSON envelope on success. If the script fails to
/// write the output file (e.g. non-zero exit), returns a descriptive
/// error with stderr content.
pub fn run_capability_script(
    script_path: &Path,
    repo_root: &Path,
    input_json_path: &Path,
    timeout_secs: Option<u64>,
) -> anyhow::Result<serde_json::Value> {
    let out_file = std::env::temp_dir().join(format!("samgraha-cap-{}.json", uuid::Uuid::new_v4()));
    let repo_root_str = repo_root.display().to_string();
    let in_str = input_json_path.display().to_string();
    let out_str = out_file.display().to_string();

    let is_ps1 = script_path.extension().and_then(|e| e.to_str()) == Some("ps1");
    let args: Vec<String> = if is_ps1 {
        vec![
            "-RepoRoot".into(), repo_root_str,
            "-In".into(), in_str,
            "-Out".into(), out_str.clone(),
        ]
    } else {
        vec![
            "--repo-root".into(), repo_root_str,
            "--in".into(), in_str,
            "--out".into(), out_str.clone(),
        ]
    };
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut cmd = script_command(script_path, &arg_refs);
    cmd.current_dir(repo_root);

    let output = run_with_optional_timeout(cmd, timeout_secs)
        .context("Failed to execute capability script")?;
    let content = std::fs::read_to_string(&out_file);
    let _ = std::fs::remove_file(&out_file);

    match content {
        Ok(text) => serde_json::from_str(text.trim_start_matches('\u{FEFF}'))
            .with_context(|| format!("Script wrote invalid JSON to {}", out_file.display())),
        Err(_) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "Capability script did not write an output file (exit {:?}); stderr: {} stdout: {}",
                output.status.code(),
                stderr.trim(),
                stdout.trim()
            )
        }
    }
}

/// Result of a workflow-script run — raw process output, no interpretation.
/// Unlike `run_check_script`'s fixed `{status, evidence, metrics}` JSON
/// contract, a workflow script (e.g. `leaderboard.py`) defines its own
/// output shape (writes whatever files it wants); the caller inspects
/// `exit_code`/`stdout`/`stderr` and reads back any output file it knows to
/// expect, this function doesn't guess at one.
pub struct WorkflowScriptOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Run a script with the workflow-script contract (see
/// `docs/crates-refactor-proposal.md` Phase 4): caller-supplied CLI `args`
/// and `env`, not the check-script contract's fixed
/// `--repo-root/--repo-fingerprint/--out` flags. Right shape for a script
/// like `leaderboard.py --adjusted ... --weights ... --json-out ...` that
/// operates across multiple repos/teams and has its own CLI, not a
/// per-repo pass/fail check. Reuses `script_command`'s interpreter dispatch
/// (`.sh`/`.ps1`/`.py`/`.js`) — the only difference from the check-script
/// contract is which arguments get built, not how the process runs.
pub fn run_workflow_script(
    script_path: &Path,
    cwd: &Path,
    args: &[String],
    env: &[(String, String)],
    timeout_secs: Option<u64>,
) -> anyhow::Result<WorkflowScriptOutput> {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut cmd = script_command(script_path, &arg_refs);
    cmd.current_dir(cwd);
    for (key, value) in env {
        cmd.env(key, value);
    }

    let output = run_with_optional_timeout(cmd, timeout_secs)
        .with_context(|| format!("Failed to run script: {}", script_path.display()))?;

    Ok(WorkflowScriptOutput {
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn find_dotenv(start: &Path) -> Option<std::path::PathBuf> {
    let mut dir = start;
    loop {
        let candidate = dir.join(".env");
        if candidate.exists() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
}
