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
