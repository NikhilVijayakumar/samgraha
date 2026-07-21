use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::check_runner;
use common::config::SamgrahaConfig;

/// The seven capability types a system can provide scripts for, plus
/// a catch-all for future capabilities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Validate,
    Calculate,
    Report,
    Scaffold,
    PlanGeneration,
    Init,
    Assemble,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validate => write!(f, "validate"),
            Self::Calculate => write!(f, "calculate"),
            Self::Report => write!(f, "report"),
            Self::Scaffold => write!(f, "scaffold"),
            Self::PlanGeneration => write!(f, "plan-generation"),
            Self::Init => write!(f, "init"),
            Self::Assemble => write!(f, "assemble"),
        }
    }
}

impl Capability {
    /// Parse a capability name from a string, returning None for unknown names.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "validate" => Some(Self::Validate),
            "calculate" => Some(Self::Calculate),
            "report" => Some(Self::Report),
            "scaffold" => Some(Self::Scaffold),
            "plan-generation" => Some(Self::PlanGeneration),
            "init" => Some(Self::Init),
            "assemble" => Some(Self::Assemble),
            _ => None,
        }
    }

    /// The script name to look up during discovery (e.g. "calculate" resolves
    /// to `scripts/calculate.py` or `scripts/calculate.sh`).
    pub fn script_name(&self) -> &'static str {
        match self {
            Self::Validate => "validate",
            Self::Calculate => "calculate",
            Self::Report => "report",
            Self::Scaffold => "scaffold",
            Self::PlanGeneration => "plan-generation",
            Self::Init => "init",
            Self::Assemble => "assemble",
        }
    }
}

/// Result of running a capability script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResult {
    pub capability: String,
    pub status: CapabilityStatus,
    pub message: Option<String>,
    pub written: Vec<String>,
    pub duration_ms: u64,
    /// The raw JSON output from the script, if any. Deliberately
    /// unconstrained — samgraha stores it, never interprets it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Ok,
    Error,
}

/// Where the capability script was found — the same first 4 tiers as
/// `CheckSource`, just named for capabilities. Deliberately has no 5th
/// `RustNative` counterpart: capabilities have no built-in fallback at
/// all, unlike `check_runner`'s placeholder tier (which still exists there,
/// unused) — see `runtime.rs`'s "no Rust-native fallback" comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilitySource {
    /// `check_overrides` in samgraha.toml (reuses the same override map
    /// keyed by capability name).
    Override { script_path: PathBuf },
    /// Repo-level `scripts/<name>.<ext>`.
    RepoScript { script_path: PathBuf },
    /// Local synced copy `.samgraha/scripts/<name>.<ext>`.
    LocalScript { script_path: PathBuf },
    /// System-default scripts shipped next to the binary.
    GlobalScript { script_path: PathBuf },
}

impl CapabilitySource {
    pub fn script_path(&self) -> &Path {
        match self {
            Self::Override { script_path }
            | Self::RepoScript { script_path }
            | Self::LocalScript { script_path }
            | Self::GlobalScript { script_path } => script_path,
        }
    }
}

/// Resolve the implementation of a capability through the 4-tier discovery
/// chain. Reuses `check_runner`'s first 4 tiers, just scoped to a
/// capability name instead of a check name — no 5th tier, see
/// `CapabilitySource`'s doc comment.
///
/// Returns `None` if no script is found at any tier.
pub fn resolve_capability(
    capability: &Capability,
    repo_root: &Path,
    config: Option<&SamgrahaConfig>,
) -> Option<CapabilitySource> {
    let name = capability.script_name();

    // Tier 1: check_overrides in samgraha.toml (reuses same map, keyed by
    // capability name — e.g. "calculate" in check_overrides points to
    // scripts/calculate.py).
    if let Some(cfg) = config {
        if let Some(script_path) = cfg.repository.documentation.check_overrides.get(name) {
            let abs = repo_root.join(script_path);
            if abs.exists() {
                return Some(CapabilitySource::Override {
                    script_path: abs,
                });
            }
        }
    }

    // Tier 2: repo-level scripts/
    if let Some(p) = check_runner::probe_script(&repo_root.join("scripts"), name) {
        return Some(CapabilitySource::RepoScript { script_path: p });
    }

    // Tier 3: local synced copy .samgraha/scripts/
    if let Some(p) = check_runner::probe_script(
        &repo_root.join(".samgraha").join("scripts"),
        name,
    ) {
        return Some(CapabilitySource::LocalScript { script_path: p });
    }

    // Tier 4: system-default scripts shipped next to the binary
    if let Some(sys_name) = config.and_then(|c| c.repository.documentation.standard_system.as_deref()) {
        let namespaced = common::env::mcp_dir().join("systems").join(sys_name).join("scripts");
        if let Some(p) = check_runner::probe_script(&namespaced, name) {
            return Some(CapabilitySource::GlobalScript { script_path: p });
        }
    }
    if let Some(p) =
        check_runner::probe_script(&common::env::mcp_dir().join("scripts"), name)
    {
        return Some(CapabilitySource::GlobalScript { script_path: p });
    }

    None
}

/// Execute a capability script and return the result.
///
/// The script receives:
///   --repo-root <path>  --in <input_json_path>  --out <output_json_path>
///
/// It must write a JSON envelope to `--out`:
///   {"status": "ok"|"error", "message": "...", "written": [...]}
///
/// For value-returning capabilities (calculate), `written` may be empty
/// and the script puts its result in the top-level JSON fields.
pub fn execute_capability(
    source: &CapabilitySource,
    capability: &Capability,
    repo_root: &Path,
    input_json_path: &Path,
    timeout_secs: Option<u64>,
) -> CapabilityResult {
    let start = std::time::Instant::now();
    let duration_ms_fn = || start.elapsed().as_millis() as u64;
    let cap_name = capability.to_string();

    match common::env::run_capability_script(
        source.script_path(),
        repo_root,
        input_json_path,
        timeout_secs,
    ) {
        Ok(json) => {
            let status = match json.get("status").and_then(|v| v.as_str()) {
                Some("ok") => CapabilityStatus::Ok,
                _ => CapabilityStatus::Error,
            };
            let message = json
                .get("message")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let written = json
                .get("written")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            CapabilityResult {
                capability: cap_name,
                status,
                message,
                written,
                duration_ms: duration_ms_fn(),
                output_json: Some(json),
            }
        }
        Err(e) => CapabilityResult {
            capability: cap_name,
            status: CapabilityStatus::Error,
            message: Some(e.to_string()),
            written: vec![],
            duration_ms: duration_ms_fn(),
            output_json: None,
        },
    }
}

// ── Init plan types (§8.4) ───────────────────────────────────────────────

/// Expiry rule for a phase — comes from the system's init plan (§8.4's
/// `expiry` field). Samgraha only stores and evaluates it; it doesn't
/// decide what "valid" means for any particular script.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpiryRule {
    /// Valid for `seconds` from the time it ran.
    Ttl { seconds: u64 },
    /// Valid until the repo's HEAD commit changes.
    HeadCommit,
}

/// A single phase in an init plan (§8.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlanPhase {
    pub id: String,
    /// "semantic" (LLM step, no script) or "script" (samgraha dispatches).
    pub kind: String,
    /// Human-readable description of what this phase does.
    #[serde(default)]
    pub description: String,
    /// Phase IDs this phase depends on — samgraha checks script_runs
    /// before allowing execution (§8.6).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// For `kind: "script"` — the script name resolved via Tier 4 discovery.
    #[serde(default)]
    pub script: Option<String>,
    /// Optional pre-phase hook script.
    #[serde(default)]
    pub pre_script: Option<String>,
    /// Optional post-phase hook script.
    #[serde(default)]
    pub post_script: Option<String>,
    /// For `kind: "semantic"` — instruction text passed to the LLM agent
    /// during prepare_semantic_phase (§3.1 of the MCP execution substrate).
    #[serde(default)]
    pub instruction: Option<String>,
    /// Staleness rule — NULL means never expires. Consumed by script_runs
    /// (§8.5).
    #[serde(default)]
    pub expiry: Option<ExpiryRule>,
}

/// A use-case in an init plan (§8.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlanUseCase {
    pub id: String,
    pub label: String,
    pub phases: Vec<PlanPhase>,
}

/// The full init plan output (§8.4) — the wire format `init` scripts return
/// and `store_system_plan` accepts. No longer stored as-is: `store_system_plan`
/// splits it into `workflow_use_cases`/`workflow_phases`/
/// `workflow_phase_dependencies` (real rows, not a JSON blob column —
/// schema-redesign-proposal.md §2.1); `get_system_plan` reconstructs this
/// same shape by querying those tables back.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InitPlan {
    pub system: String,
    pub use_cases: Vec<PlanUseCase>,
}

/// Result of checking whether a phase's prerequisites are met (§8.6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteCheck {
    pub blocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ── Prerequisite checking (§8.6) ─────────────────────────────────────────

/// Check whether a phase's dependencies are all satisfied.
///
/// Queries `workflow_phase_dependencies` to find the phase's dependency
/// phase IDs, then for each dependency looks up `script_runs` for a
/// matching row and evaluates its validity:
/// - No row → `reason: "missing_precondition"`
/// - Row exists but expired → `reason: "expired_precondition"`
/// - Row exists and valid → continue to next dep
///
/// Returns the first blocking reason found, or an unblocked check.
pub fn check_phase_prerequisites(
    conn: &rusqlite::Connection,
    standard_id: i64,
    repo_fingerprint: &str,
    phase_id: &str,
    current_head: Option<&str>,
) -> PrerequisiteCheck {
    // Resolve the phase's workflow_phases.id from its string phase_id.
    let phase_row_id: Option<i64> = conn.query_row(
        "SELECT wp.id FROM workflow_phases wp
         JOIN workflow_use_cases wuc ON wp.use_case_id = wuc.id
         WHERE wuc.standard_id = ?1 AND wp.phase_id = ?2",
        rusqlite::params![standard_id, phase_id],
        |row| row.get(0),
    ).ok();

    let phase_row_id = match phase_row_id {
        Some(id) => id,
        None => {
            // Phase not in the plan at all — nothing to gate on.
            return PrerequisiteCheck {
                blocked: false,
                reason: None,
                phase_id: None,
                message: None,
            };
        }
    };

    // Get dependency phase IDs (string IDs) from the normalized edges.
    let mut dep_stmt = conn.prepare(
        "SELECT wp2.phase_id FROM workflow_phase_dependencies wpd
         JOIN workflow_phases wp2 ON wpd.depends_on_phase_id = wp2.id
         WHERE wpd.phase_id = ?1"
    ).expect("failed to prepare dependency query");
    let dep_ids: Vec<String> = dep_stmt
        .query_map(rusqlite::params![phase_row_id], |row| row.get(0))
        .expect("failed to query dependencies")
        .filter_map(|r| r.ok())
        .collect();

    for dep_id in &dep_ids {
        // Look up the dependency's last run.
        let row: Option<(String, Option<String>, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT ran_at, expiry_rule_json, expires_at, head_commit_at_run
                 FROM script_runs
                 WHERE standard_id = ?1
                   AND repo_fingerprint = ?2
                   AND phase_or_check_key = ?3
                 ORDER BY id DESC LIMIT 1",
                rusqlite::params![standard_id, repo_fingerprint, dep_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                    ))
                },
            )
            .ok();

        match row {
            None => {
                return PrerequisiteCheck {
                    blocked: true,
                    reason: Some("missing_precondition".to_string()),
                    phase_id: Some(dep_id.clone()),
                    message: Some(format!("Phase '{}' has not been run yet for this repo.", dep_id)),
                };
            }
            Some((_, expiry_json, expires_at, head_at_run)) => {
                if let Some(reason) = evaluate_expiry(
                    expiry_json.as_deref(),
                    expires_at.as_deref(),
                    head_at_run.as_deref(),
                    current_head,
                ) {
                    return PrerequisiteCheck {
                        blocked: true,
                        reason: Some("expired_precondition".to_string()),
                        phase_id: Some(dep_id.clone()),
                        message: Some(format!(
                            "Phase '{}' ran but its output has expired: {}",
                            dep_id, reason
                        )),
                    };
                }
            }
        }
    }

    PrerequisiteCheck {
        blocked: false,
        reason: None,
        phase_id: None,
        message: None,
    }
}

/// Record a successful capability/phase run into `script_runs` (§8.5) so
/// later `check_phase_prerequisites` calls can find it. Upserts on
/// `(standard_id, repo_fingerprint, capability, phase_or_check_key)` —
/// re-running the same phase replaces the prior run, matching §7.4's
/// idempotent/backfill principle. Only call this on a successful run;
/// a failed run must not satisfy a downstream dependency.
pub fn record_script_run(
    conn: &rusqlite::Connection,
    standard_id: i64,
    repo_fingerprint: &str,
    capability: &Capability,
    phase_or_check_key: &str,
    expiry: Option<&ExpiryRule>,
    current_head: Option<&str>,
) -> rusqlite::Result<()> {
    let (expiry_rule_json, expires_at, head_commit_at_run): (
        Option<String>,
        Option<String>,
        Option<String>,
    ) = match expiry {
        None => (None, None, None),
        Some(rule @ ExpiryRule::Ttl { seconds }) => {
            let expires = chrono::Utc::now() + chrono::Duration::seconds(*seconds as i64);
            (
                Some(serde_json::to_string(rule).unwrap_or_default()),
                Some(expires.to_rfc3339()),
                None,
            )
        }
        Some(rule @ ExpiryRule::HeadCommit) => (
            Some(serde_json::to_string(rule).unwrap_or_default()),
            None,
            current_head.map(|s| s.to_string()),
        ),
    };

    conn.execute(
        "INSERT INTO script_runs
            (standard_id, repo_fingerprint, capability, phase_or_check_key,
             expiry_rule_json, expires_at, head_commit_at_run, ran_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
         ON CONFLICT (standard_id, repo_fingerprint, capability, phase_or_check_key)
         DO UPDATE SET
            expiry_rule_json = excluded.expiry_rule_json,
            expires_at = excluded.expires_at,
            head_commit_at_run = excluded.head_commit_at_run,
            ran_at = excluded.ran_at",
        rusqlite::params![
            standard_id,
            repo_fingerprint,
            capability.to_string(),
            phase_or_check_key,
            expiry_rule_json,
            expires_at,
            head_commit_at_run,
        ],
    )?;
    Ok(())
}

/// Evaluate whether a script run is still valid based on its expiry rule.
///
/// Returns `None` if valid, `Some(reason)` if expired.
fn evaluate_expiry(
    expiry_rule_json: Option<&str>,
    expires_at: Option<&str>,
    head_commit_at_run: Option<&str>,
    current_head: Option<&str>,
) -> Option<String> {
    let Some(rule_json) = expiry_rule_json else {
        return None; // NULL expiry → always valid
    };

    let rule: ExpiryRule = serde_json::from_str(rule_json).ok()?;

    match rule {
        ExpiryRule::Ttl { .. } => {
            // Check against precomputed expires_at.
            let Some(expires_str) = expires_at else {
                return None;
            };
            let expires = chrono::DateTime::parse_from_rfc3339(expires_str).ok()?;
            let now = chrono::Utc::now();
            if now >= expires {
                Some(format!("TTL expired at {}", expires_str))
            } else {
                None
            }
        }
        ExpiryRule::HeadCommit => {
            let Some(ran_head) = head_commit_at_run else {
                return None;
            };
            let Some(cur_head) = current_head else {
                return None;
            };
            if ran_head != cur_head {
                Some(format!(
                    "HEAD changed: was {}, now {}",
                    ran_head, cur_head
                ))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_from_name_roundtrip() {
        assert_eq!(Capability::from_name("calculate"), Some(Capability::Calculate));
        assert_eq!(Capability::from_name("plan-generation"), Some(Capability::PlanGeneration));
        assert_eq!(Capability::from_name("bogus"), None);
    }

    /// In-memory DB with just enough schema to exercise `record_script_run`
    /// + `check_phase_prerequisites` — the write-then-read round trip that
    /// was previously missing entirely (§8.5/§8.6).
    fn test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE standards (id INTEGER PRIMARY KEY);
             CREATE TABLE workflow_use_cases (
                 id INTEGER PRIMARY KEY,
                 standard_id INTEGER NOT NULL,
                 use_case_id TEXT NOT NULL,
                 label TEXT NOT NULL,
                 UNIQUE(standard_id, use_case_id)
             );
             CREATE TABLE workflow_phases (
                 id INTEGER PRIMARY KEY,
                 use_case_id INTEGER NOT NULL,
                 phase_id TEXT NOT NULL,
                 sort_order INTEGER NOT NULL DEFAULT 0,
                 kind TEXT NOT NULL,
                 description TEXT,
                 script_name TEXT,
                 pre_script TEXT,
                 post_script TEXT,
                 instruction TEXT,
                 expiry_rule_json TEXT,
                 UNIQUE(use_case_id, phase_id)
             );
             CREATE TABLE workflow_phase_dependencies (
                 id INTEGER PRIMARY KEY,
                 phase_id INTEGER NOT NULL,
                 depends_on_phase_id INTEGER NOT NULL,
                 UNIQUE(phase_id, depends_on_phase_id)
             );
             CREATE TABLE script_runs (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 standard_id INTEGER NOT NULL,
                 repo_fingerprint TEXT NOT NULL,
                 capability TEXT NOT NULL,
                 phase_or_check_key TEXT NOT NULL,
                 ran_at TEXT NOT NULL DEFAULT (datetime('now')),
                 expiry_rule_json TEXT,
                 expires_at TEXT,
                 head_commit_at_run TEXT,
                 UNIQUE(standard_id, repo_fingerprint, capability, phase_or_check_key)
             );
             INSERT INTO standards (id) VALUES (1);
             INSERT INTO workflow_use_cases (id, standard_id, use_case_id, label)
                 VALUES (1, 1, 'test-uc', 'Test UC');
             INSERT INTO workflow_phases (id, use_case_id, phase_id, sort_order, kind)
                 VALUES (1, 1, 'consumer', 0, 'script');
             INSERT INTO workflow_phases (id, use_case_id, phase_id, sort_order, kind)
                 VALUES (2, 1, 'producer', 1, 'script');
             INSERT INTO workflow_phase_dependencies (phase_id, depends_on_phase_id)
                 VALUES (1, 2);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn no_run_recorded_blocks_with_missing_precondition() {
        let conn = test_db();
        let check = check_phase_prerequisites(&conn, 1, "repo-a", "consumer", None);
        assert!(check.blocked);
        assert_eq!(check.reason.as_deref(), Some("missing_precondition"));
        assert_eq!(check.phase_id.as_deref(), Some("producer"));
    }

    #[test]
    fn recording_a_run_unblocks_a_dependent_phase() {
        let conn = test_db();
        record_script_run(
            &conn,
            1,
            "repo-a",
            &Capability::Calculate,
            "producer",
            None, // no expiry — never expires
            None,
        )
        .unwrap();

        let check = check_phase_prerequisites(&conn, 1, "repo-a", "consumer", None);
        assert!(!check.blocked, "expected unblocked, got {:?}", check);
    }

    #[test]
    fn expired_ttl_run_blocks_with_expired_precondition() {
        let conn = test_db();
        // Record with a TTL already in the past — 0-second TTL expires
        // immediately, so this simulates "ran a while ago, now stale"
        // without needing to sleep in a test.
        record_script_run(
            &conn,
            1,
            "repo-a",
            &Capability::Calculate,
            "producer",
            Some(&ExpiryRule::Ttl { seconds: 0 }),
            None,
        )
        .unwrap();
        // record_script_run computes expires_at as now+0s; sleep a moment
        // so "now" at check-time is strictly after it.
        std::thread::sleep(std::time::Duration::from_millis(10));

        let check = check_phase_prerequisites(&conn, 1, "repo-a", "consumer", None);
        assert!(check.blocked);
        assert_eq!(check.reason.as_deref(), Some("expired_precondition"));
    }

    #[test]
    fn rerunning_the_same_key_upserts_not_duplicates() {
        let conn = test_db();
        for _ in 0..3 {
            record_script_run(&conn, 1, "repo-a", &Capability::Validate, "producer", None, None)
                .unwrap();
        }
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM script_runs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "expected upsert, not one row per run");
    }

    #[test]
    fn capability_script_name_matches_discovery_convention() {
        // Each capability's script_name must be a bare name without extension,
        // matching what probe_script expects.
        assert_eq!(Capability::Validate.script_name(), "validate");
        assert_eq!(Capability::Calculate.script_name(), "calculate");
        assert_eq!(Capability::Report.script_name(), "report");
        assert_eq!(Capability::Scaffold.script_name(), "scaffold");
        assert_eq!(Capability::PlanGeneration.script_name(), "plan-generation");
        assert_eq!(Capability::Init.script_name(), "init");
        assert_eq!(Capability::Assemble.script_name(), "assemble");
    }
}
