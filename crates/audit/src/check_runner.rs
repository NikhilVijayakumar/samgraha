use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use common::config::SamgrahaConfig;

/// Result of running a single check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_name: String,
    pub status: CheckStatus,
    pub message: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail,
    Error,
    Skip,
}

/// Where the check implementation was found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckSource {
    /// `check_overrides` in samgraha.toml — highest priority.
    CheckOverride { script_path: PathBuf },
    /// Repo-level `scripts/<name>.sh`.
    RepoScript { script_path: PathBuf },
    /// Local synced copy `.samgraha/scripts/<name>.sh`.
    LocalScript { script_path: PathBuf },
    /// System-default scripts shipped next to the binary (`common::env::mcp_dir()/scripts`,
    /// synced from whatever documentation system's own script source — samgraha's own
    /// `docs/knowledge-hub/script/` is one such source, wired via that repo's own
    /// `check_overrides`, not hardcoded here. A different registered system ships its
    /// own scripts the same way, no samgraha-specific path assumption.
    GlobalScript { script_path: PathBuf },
    /// Rust-native check compiled into the audit crate.
    RustNative,
}

/// Resolve the implementation of a named check through the resolution chain.
///
/// Returns `None` if no implementation is found at any tier.
pub fn resolve_check(
    name: &str,
    repo_root: &Path,
    config: Option<&SamgrahaConfig>,
) -> Option<CheckSource> {
    // Tier 1: check_overrides in samgraha.toml
    if let Some(cfg) = config {
        if let Some(script_path) = cfg.repository.documentation.check_overrides.get(name) {
            let abs = repo_root.join(script_path);
            if abs.exists() {
                return Some(CheckSource::CheckOverride {
                    script_path: abs,
                });
            }
        }
    }

    // Tier 2: repo-level scripts/ — probe .sh then .ps1
    if let Some(p) = probe_script(&repo_root.join("scripts"), name) {
        return Some(CheckSource::RepoScript { script_path: p });
    }

    // Tier 3: local synced copy .samgraha/scripts/
    if let Some(p) = probe_script(&repo_root.join(".samgraha").join("scripts"), name) {
        return Some(CheckSource::LocalScript { script_path: p });
    }

    // Tier 4: system-default scripts shipped next to the binary —
    // mcp_dir()/systems/<name>/scripts/ (namespaced, §3.1 of asset sync
    // proposal) for standards that registered with the new layout, falling
    // back to the legacy flat mcp_dir()/scripts/ for backward compat.
    if let Some(sys_name) = config.and_then(|c| c.repository.documentation.standard_system.as_deref()) {
        let namespaced = common::env::mcp_dir().join("systems").join(sys_name).join("scripts");
        if let Some(p) = probe_script(&namespaced, name) {
            return Some(CheckSource::GlobalScript { script_path: p });
        }
    }
    if let Some(p) = probe_script(&common::env::mcp_dir().join("scripts"), name) {
        return Some(CheckSource::GlobalScript { script_path: p });
    }

    // Tier 5: Rust-native check (would need a registry lookup — placeholder)
    // For now, skip. Will be wired when native checks are added.

    None
}

/// Probe a directory for a script named `name`, trying platform-native
/// extensions first (`.ps1` on Windows, `.sh` elsewhere — for a repo
/// shipping both, e.g. samgraha's own `scripts/`, matching
/// docs/knowledge-hub's `windows/`+`ubuntu/` split), then the two
/// cross-platform interpreter extensions (`.py`, `.js` — no OS-specific
/// variant, no platform subfolder).
///
/// `name` may already carry an extension and/or a relative path (e.g. a
/// rule's `script` param naming `script/audit_testing.py` directly, the real
/// shape `python_hackathon`'s deterministic rules use) — that's resolved as
/// `dir.join(name)` directly, skipping the bare-name+extension-append
/// convention entirely, since appending `.sh` to an already-`.py`-suffixed
/// path would just build a path that can't exist.
pub fn probe_script(dir: &Path, name: &str) -> Option<PathBuf> {
    if has_known_script_extension(name) {
        let direct = dir.join(name);
        return direct.exists().then_some(direct);
    }

    let native = if cfg!(windows) { ("windows", "ps1") } else { ("ubuntu", "sh") };
    let other = if cfg!(windows) { ("ubuntu", "sh") } else { ("windows", "ps1") };
    for (platform_dir, ext) in [native, other] {
        // Platform subfolder first (docs/knowledge-hub/script/{windows,ubuntu}/
        // convention, e.g. this repo's own scripts/windows/, scripts/ubuntu/),
        // then flat (back-compat / dirs with no per-platform split, like
        // .samgraha/scripts/ and the mcp-adjacent system-default scripts/).
        let nested = dir.join(platform_dir).join(format!("{}.{}", name, ext));
        if nested.exists() {
            return Some(nested);
        }
        let flat = dir.join(format!("{}.{}", name, ext));
        if flat.exists() {
            return Some(flat);
        }
    }

    for ext in ["py", "js"] {
        let flat = dir.join(format!("{}.{}", name, ext));
        if flat.exists() {
            return Some(flat);
        }
    }

    None
}

fn has_known_script_extension(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| matches!(ext, "sh" | "ps1" | "py" | "js"))
}

/// Execute a resolved check and return the result.
pub fn execute_check(source: &CheckSource, repo_root: &Path, check_name: &str) -> CheckResult {
    let start = std::time::Instant::now();

    match source {
        CheckSource::RustNative => CheckResult {
            check_name: check_name.to_string(),
            status: CheckStatus::Skip,
            message: "Rust-native checks not yet implemented".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            metadata: None,
        },
        CheckSource::CheckOverride { script_path }
        | CheckSource::RepoScript { script_path }
        | CheckSource::LocalScript { script_path }
        | CheckSource::GlobalScript { script_path } => {
            execute_script(script_path, repo_root, check_name, start)
        }
    }
}

fn execute_script(
    script_path: &Path,
    repo_root: &Path,
    check_name: &str,
    start: std::time::Instant,
) -> CheckResult {
    let fingerprint = format!("{}-{}", check_name, repo_root.display());
    let duration_ms_fn = || start.elapsed().as_millis() as u64;

    match common::env::run_check_script(script_path, repo_root, &fingerprint, None) {
        Ok(json) => {
            let status = match json.get("status").and_then(|v| v.as_str()) {
                Some("pass") => CheckStatus::Pass,
                Some("fail") => CheckStatus::Fail,
                Some("not_applicable") => CheckStatus::Skip,
                _ => CheckStatus::Error,
            };
            let message = json
                .get("evidence")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Check {:?}", status));
            let metadata = json.get("metrics").and_then(|v| v.as_object()).map(|map| {
                map.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<HashMap<_, _>>()
            });
            CheckResult {
                check_name: check_name.to_string(),
                status,
                message,
                duration_ms: duration_ms_fn(),
                metadata,
            }
        }
        Err(e) => CheckResult {
            check_name: check_name.to_string(),
            status: CheckStatus::Error,
            message: e.to_string(),
            duration_ms: duration_ms_fn(),
            metadata: None,
        },
    }
}

/// Run all resolved checks for a repo root. Returns results in resolution order.
pub fn run_all_checks(
    repo_root: &Path,
    config: Option<&SamgrahaConfig>,
    check_names: &[String],
) -> Vec<CheckResult> {
    check_names
        .iter()
        .map(|name| {
            if let Some(source) = resolve_check(name, repo_root, config) {
                execute_check(&source, repo_root, name)
            } else {
                CheckResult {
                    check_name: name.clone(),
                    status: CheckStatus::Skip,
                    message: format!("No implementation found for check '{}'", name),
                    duration_ms: 0,
                    metadata: None,
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("samgraha-test-check-runner-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn probe_script_finds_py_extension() {
        let dir = scratch_dir("probe-py");
        std::fs::write(dir.join("leaderboard.py"), "").unwrap();
        assert_eq!(probe_script(&dir, "leaderboard"), Some(dir.join("leaderboard.py")));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn probe_script_finds_js_extension() {
        let dir = scratch_dir("probe-js");
        std::fs::write(dir.join("aggregate.js"), "").unwrap();
        assert_eq!(probe_script(&dir, "aggregate"), Some(dir.join("aggregate.js")));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn probe_script_resolves_name_that_already_has_a_path_and_extension() {
        // Real shape: python_hackathon's rule_evidence_params stores
        // "script/audit_testing.py" directly, not a bare check name.
        let dir = scratch_dir("probe-direct-path");
        std::fs::create_dir_all(dir.join("script")).unwrap();
        std::fs::write(dir.join("script").join("audit_testing.py"), "").unwrap();
        assert_eq!(
            probe_script(&dir, "script/audit_testing.py"),
            Some(dir.join("script").join("audit_testing.py"))
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn probe_script_returns_none_when_nothing_matches() {
        let dir = scratch_dir("probe-none");
        assert_eq!(probe_script(&dir, "nonexistent"), None);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn probe_script_prefers_platform_native_over_py() {
        // .sh/.ps1 (platform-native) still win over .py/.js when both exist
        // for a bare check name — only the interpreter set grew, not the
        // existing platform-native precedence.
        let dir = scratch_dir("probe-precedence");
        let native_ext = if cfg!(windows) { "ps1" } else { "sh" };
        std::fs::write(dir.join(format!("check.{}", native_ext)), "").unwrap();
        std::fs::write(dir.join("check.py"), "").unwrap();
        assert_eq!(probe_script(&dir, "check"), Some(dir.join(format!("check.{}", native_ext))));
        std::fs::remove_dir_all(&dir).ok();
    }
}
