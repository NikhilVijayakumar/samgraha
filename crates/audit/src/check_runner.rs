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

    // Tier 4: system-default scripts shipped next to the binary — same
    // mcp_dir() source `standards.db`/`help.db` sync uses, kept binary-adjacent
    // instead of home_dir()-based so one sync step covers standards + help +
    // scripts together (see crate::builtin, standards `sync`/`sync_standards`).
    if let Some(p) = probe_script(&common::env::mcp_dir().join("scripts"), name) {
        return Some(CheckSource::GlobalScript { script_path: p });
    }

    // Tier 5: Rust-native check (would need a registry lookup — placeholder)
    // For now, skip. Will be wired when native checks are added.

    None
}

/// Probe a directory for `{name}.sh`/`{name}.ps1` — platform-native extension
/// first (`.ps1` on Windows, `.sh` elsewhere), so a repo shipping both (e.g.
/// samgraha's own `scripts/`, matching docs/knowledge-hub's `windows/`+`ubuntu/`
/// split) runs the variant meant for the OS actually running it, not
/// whichever happens to sort first.
pub fn probe_script(dir: &Path, name: &str) -> Option<PathBuf> {
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
    None
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
