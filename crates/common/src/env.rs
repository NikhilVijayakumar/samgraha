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
