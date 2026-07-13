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
/// `.ps1`/`.sh` aren't natively executable on every platform (a `.ps1` isn't
/// a Win32 executable even on Windows; a `.sh` has no shebang-exec on Windows
/// at all). Anything else (a compiled native binary a `check_overrides` entry
/// points at) runs directly. `args` are appended after the script path.
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
        _ => {
            let mut cmd = std::process::Command::new(script_path);
            cmd.args(args);
            cmd
        }
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

    let run_result: std::io::Result<std::process::Output> = if let Some(secs) = timeout_secs {
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
    } else {
        cmd.output()
    };

    let output = run_result.context("Failed to execute script")?;
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
