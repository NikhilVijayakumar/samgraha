use anyhow::{bail, Context, Result};
use common::config::{InitOptions, SamgrahaConfig};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────

/// Result of a Knowledge System sync from global store to local.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Whether `standards.db` was copied (file existed at source).
    pub standards_synced: bool,
    /// Number of help documents synced into `knowledge.db`.
    pub help_documents_synced: usize,
    /// Number of scripts copied from global `scripts/`.
    pub scripts_synced: usize,
}

/// Metadata written after a successful sync for staleness tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncMeta {
    pub system: String,
    pub version: String,
    pub synced_at: String,
}

/// Result of initializing (or backfilling) a repository's `samgraha.toml`.
pub struct InitResult {
    pub root: PathBuf,
    pub config: SamgrahaConfig,
    pub status: String,
    pub env_path: PathBuf,
    /// Populated only when `sync_knowledge_system = true`.
    pub sync_result: Option<SyncResult>,
}

/// Filesystem-probed directory candidates.
#[derive(Debug, Clone, Default)]
pub struct DetectedDirs {
    pub root_dir: Option<String>,
    pub implementation_dir: Option<String>,
    pub tests_dir: Option<String>,
    pub scripts_dir: Option<String>,
}

/// Staleness of the local Knowledge System vs the global store.
#[derive(Debug, Clone)]
pub enum StalenessStatus {
    /// No `sync-meta.json` — never synced.
    NeverSynced,
    /// No local `standards.db` — needs initial sync.
    MissingLocal,
    /// Local version matches global — up to date.
    UpToDate { version: String },
    /// Local version differs from global — stale.
    Stale { local_version: String, global_version: String },
    /// Global source DB doesn't exist.
    SourceMissing,
}

// ─────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────

/// Initialize `samgraha.toml` + `.samgraha/` at `root`, or backfill any keys
/// missing from an existing config (never overwrites a key that's already
/// there). Shared by the CLI `init` command and the MCP `init` tool so both
/// surfaces bootstrap a repo identically.
///
/// Phases:
/// 1. **Configuration** — build template, apply `InitOptions`
/// 2. **Discovery** — probe filesystem for directories (if enabled)
/// 3. **Synchronization** — write TOML, `.env.example`, optionally sync Knowledge System
/// 4. **Result** — return enriched `InitResult`
pub fn init_repository(root: &Path, options: &InitOptions) -> Result<InitResult> {
    let config_path = root.join("samgraha.toml");
    let samgraha_dir = root.join(".samgraha");

    std::fs::create_dir_all(&samgraha_dir)
        .context(format!("Failed to create {}", samgraha_dir.display()))?;

    // ── Phase 1: Configuration ──────────────────────────────
    let mut template = SamgrahaConfig::default();
    let dir_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repository")
        .to_string();
    template.repository.id = Some(dir_name.clone());
    template.repository.name = Some(dir_name);
    template.repository.uuid = Some(uuid::Uuid::new_v4());
    template.repository.documentation.domain = Vec::new();

    // Apply InitOptions into the template
    if let Some(ref sys) = options.standard_system {
        template.repository.documentation.standard_system = Some(sys.clone());
    }
    if !options.script_overrides.is_empty() {
        template.repository.documentation.script_overrides = options.script_overrides.clone();
    }
    if !options.check_overrides.is_empty() {
        template.repository.documentation.check_overrides = options.check_overrides.clone();
    }

    // ── Phase 2: Discovery ──────────────────────────────────
    if options.auto_detect_dirs {
        let detected = probe_directories(root);
        if let Some(ref dir) = detected.root_dir {
            template.repository.documentation.root_dir = dir.clone();
        }
        if let Some(ref dir) = detected.implementation_dir {
            template.repository.implementation.dir = dir.clone();
        }
        if let Some(ref dir) = detected.tests_dir {
            template.repository.tests = Some(common::config::TestsConfig { dir: dir.clone() });
        }
        if let Some(ref dir) = detected.scripts_dir {
            template.repository.scripts = Some(common::config::ScriptsConfig { dir: dir.clone() });
        }
    }

    // ── Phase 3: Synchronization ────────────────────────────
    let (config, status) = if config_path.exists() && !options.force {
        let existing = std::fs::read_to_string(&config_path)
            .context(format!("Failed to read {}", config_path.display()))?;
        let mut doc: toml_edit::DocumentMut = existing
            .parse()
            .context(format!("Failed to parse {}", config_path.display()))?;
        let template_content = toml::to_string_pretty(&template)?;
        let template_doc: toml_edit::DocumentMut = template_content
            .parse()
            .context("Failed to parse generated template config")?;

        let added = merge_missing_keys(doc.as_table_mut(), template_doc.as_table());
        if added > 0 {
            std::fs::write(&config_path, doc.to_string())
                .context(format!("Failed to write config to {}", config_path.display()))?;
        }

        let merged: SamgrahaConfig = toml::from_str(&doc.to_string())
            .context("Merged samgraha.toml failed to parse back as valid config")?;
        let status = if added > 0 {
            format!(
                "Updated {} — added {added} missing key(s), left the rest untouched",
                config_path.display()
            )
        } else {
            format!("{} already covers every known key — nothing to add", config_path.display())
        };
        (merged, status)
    } else {
        let content = toml::to_string_pretty(&template)?;
        std::fs::write(&config_path, content)
            .context(format!("Failed to write config to {}", config_path.display()))?;
        (template, format!("Initialized samgraha repository at {}", root.display()))
    };

    let env_path = write_env_example(root)?;

    let sync_result = if options.sync_knowledge_system {
        let sr = sync_knowledge_system(root)?;
        Some(sr)
    } else {
        None
    };

    // ── Phase 4: Result ─────────────────────────────────────
    Ok(InitResult {
        root: root.to_path_buf(),
        config,
        status,
        env_path,
        sync_result,
    })
}

// ─────────────────────────────────────────────────────
// Knowledge System Sync
// ─────────────────────────────────────────────────────

/// Synchronize the declared Knowledge System from the global store
/// (`common::env::mcp_dir()`) into a repo's local `.samgraha/`. Single
/// implementation shared by CLI `knowledge pull`, MCP `sync_standards`,
/// and `init_repository()`.
pub fn sync_knowledge_system(root: &Path) -> Result<SyncResult> {
    let mcp_dir = common::env::mcp_dir();
    let local_db = root.join(".samgraha").join("standards.db");
    let source_db = mcp_dir.join("standards.db");

    // 1. Copy standards.db (with integrity + schema version check)
    let standards_synced = if source_db.exists() {
        let check_conn = rusqlite::Connection::open(&source_db)
            .context("Failed to open source standards.db")?;
        let ok: String = check_conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .context("Failed to run integrity check")?;
        if ok != "ok" {
            bail!("Standards DB integrity check failed: {}", ok);
        }
        standards::check_schema_version(&check_conn)?;
        if let Some(parent) = local_db.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&source_db, &local_db)
            .context("Failed to copy standards.db to local")?;
        true
    } else {
        false
    };

    // 2. Sync help docs into knowledge.db
    let help_documents_synced = crate::builtin::sync_help_into_local(root)?;

    // 3. Sync scripts/
    let source_scripts = mcp_dir.join("scripts");
    let mut scripts_synced = 0usize;
    if source_scripts.exists() {
        let local_scripts = root.join(".samgraha").join("scripts");
        std::fs::create_dir_all(&local_scripts)?;
        for entry in std::fs::read_dir(&source_scripts)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            std::fs::copy(entry.path(), local_scripts.join(entry.file_name()))?;
            scripts_synced += 1;
        }
    }

    // 4. Write sync metadata for staleness tracking
    let (system_name, system_version) = source_db
        .exists()
        .then(|| read_system_info_from_db(&source_db))
        .flatten()
        .unwrap_or_else(|| ("unknown".to_string(), "0.0.0".to_string()));

    let meta = SyncMeta {
        system: system_name,
        version: system_version,
        synced_at: chrono::Utc::now().to_rfc3339(),
    };
    let meta_path = root.join(".samgraha").join("sync-meta.json");
    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)
        .context("Failed to write sync-meta.json")?;

    Ok(SyncResult {
        standards_synced,
        help_documents_synced,
        scripts_synced,
    })
}

/// Check staleness and sync only if needed. Shared by CLI `knowledge pull`
/// and MCP `sync_standards` so both surfaces have identical skip/warn behavior.
///
/// - `force = true` → always sync
/// - `force = false` → skip if `UpToDate`, warn-and-continue on any error
///
/// Returns `Ok(Some(result))` if sync ran, `Ok(None)` if skipped.
pub fn sync_if_stale(root: &Path, force: bool) -> Result<Option<SyncResult>> {
    if !force {
        match check_knowledge_staleness(root) {
            Ok(StalenessStatus::UpToDate { version }) => {
                println!("Knowledge System v{version} is up to date. Use --force to re-sync.");
                return Ok(None);
            }
            Ok(status) => tracing::info!("Staleness check: {:?}", status),
            Err(e) => tracing::warn!("Staleness check failed (treating as stale): {}", e),
        }
    }
    let result = sync_knowledge_system(root)?;
    Ok(Some(result))
}

/// Check whether the local Knowledge System is up-to-date vs the global store.
pub fn check_knowledge_staleness(root: &Path) -> Result<StalenessStatus> {
    let meta_path = root.join(".samgraha").join("sync-meta.json");
    if !meta_path.exists() {
        return Ok(StalenessStatus::NeverSynced);
    }
    let meta: SyncMeta = serde_json::from_str(&std::fs::read_to_string(&meta_path)?)
        .context("Failed to parse sync-meta.json")?;

    let source_db = common::env::mcp_dir().join("standards.db");
    if !source_db.exists() {
        return Ok(StalenessStatus::SourceMissing);
    }

    let local_db = root.join(".samgraha").join("standards.db");
    if !local_db.exists() {
        return Ok(StalenessStatus::MissingLocal);
    }

    let global_version = read_system_info_from_db(&source_db)
        .map(|(_, v)| v)
        .unwrap_or_default();

    if global_version == meta.version {
        Ok(StalenessStatus::UpToDate { version: meta.version })
    } else {
        Ok(StalenessStatus::Stale {
            local_version: meta.version,
            global_version,
        })
    }
}

// ─────────────────────────────────────────────────────
// Directory Probing
// ─────────────────────────────────────────────────────

/// Probe `root` for common directory layouts and return detected paths.
/// Never creates directories — only checks for existence.
pub fn probe_directories(root: &Path) -> DetectedDirs {
    let mut detected = DetectedDirs::default();

    // Docs
    if root.join("docs").is_dir() {
        detected.root_dir = Some("docs".to_string());
    } else if root.join("documentation").is_dir() {
        detected.root_dir = Some("documentation".to_string());
    }

    // Implementation — prefer `crates/` over `src/` for Rust workspaces
    if root.join("crates").is_dir() {
        detected.implementation_dir = Some("crates".to_string());
    } else if root.join("src").is_dir() {
        detected.implementation_dir = Some("src".to_string());
    } else if root.join("lib").is_dir() {
        detected.implementation_dir = Some("lib".to_string());
    }

    // Tests
    if root.join("tests").is_dir() {
        detected.tests_dir = Some("tests".to_string());
    } else if root.join("test").is_dir() {
        detected.tests_dir = Some("test".to_string());
    }

    // Scripts
    if root.join("scripts").is_dir() {
        detected.scripts_dir = Some("scripts".to_string());
    }

    detected
}

// ─────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────

fn merge_missing_keys(existing: &mut toml_edit::Table, defaults: &toml_edit::Table) -> usize {
    let mut added = 0;
    for (key, default_item) in defaults.iter() {
        if !existing.contains_key(key) {
            existing.insert(key, default_item.clone());
            added += 1;
        } else if let Some(default_tbl) = default_item.as_table() {
            if let Some(existing_tbl) = existing.get_mut(key).and_then(|i| i.as_table_mut()) {
                added += merge_missing_keys(existing_tbl, default_tbl);
            }
        }
    }
    added
}

/// Ensure `.env.example` documents every env key samgraha reads for `${VAR}`
/// placeholders in samgraha.toml (see `resolve_configured_dir`), values left
/// blank/commented for the user to fill in per machine.
///
/// Additive, not overwriting: a repo may already have an `.env.example` with
/// unrelated keys. Only keys not already present get appended, so
/// regenerating never clobbers existing content.
pub fn write_env_example(root: &Path) -> Result<PathBuf> {
    const KEYS: &[(&str, &str)] = &[
        (
            "SAMGRAHA_REPORT_DIR",
            "# Absolute path for generated reports (e.g. `samgraha audit --report`).\n\
             # Unset falls back to <repo>/docs/raw/reports.\n\
             # SAMGRAHA_REPORT_DIR=\n",
        ),
        (
            "SAMGRAHA_DOCS_DIR",
            "# Absolute path to this repository's documentation root.\n\
             # Unset falls back to <repo>/docs.\n\
             # SAMGRAHA_DOCS_DIR=\n",
        ),
        (
            "SAMGRAHA_IMPLEMENTATION_DIR",
            "# Absolute path to this repository's implementation/source directory.\n\
             # Reserved for future traceability checks; unset falls back to <repo>/src.\n\
             # SAMGRAHA_IMPLEMENTATION_DIR=\n",
        ),
        (
            "SAMGRAHA_SCRIPTS_DIR",
            "# Absolute path to this repository's external scripts directory.\n\
             # Only relevant if [repository.scripts] is set in samgraha.toml.\n\
             # SAMGRAHA_SCRIPTS_DIR=\n",
        ),
        (
            "SAMGRAHA_TESTS_DIR",
            "# Absolute path to this repository's test directory, if kept outside\n\
             # implementation.dir. Only relevant if [repository.tests] is set.\n\
             # SAMGRAHA_TESTS_DIR=\n",
        ),
    ];

    let path = root.join(".env.example");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    let mut appended = String::new();
    for (key, block) in KEYS {
        if !existing.contains(key) {
            if !appended.is_empty() || !existing.is_empty() {
                appended.push('\n');
            }
            appended.push_str(block);
        }
    }

    if appended.is_empty() {
        return Ok(path);
    }

    let mut content = existing;
    content.push_str(&appended);
    if !content.contains("cp .env.example .env") {
        content.push_str("\n# Copy this file to .env and uncomment the values to configure:\n#   cp .env.example .env\n");
    }
    std::fs::write(&path, content).context(format!("Failed to write {}", path.display()))?;

    Ok(path)
}

/// Read the default system's name and version from `standards.db` in a
/// single query, joining through `system_id` so the version always
/// corresponds to the system identified by name.
fn read_system_info_from_db(db_path: &Path) -> Option<(String, String)> {
    let conn = rusqlite::Connection::open(db_path).ok()?;
    conn.query_row(
        "SELECT sys.name, s.version \
         FROM standards s \
         JOIN systems sys ON sys.id = s.system_id \
         WHERE sys.is_default = 1 \
         LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .ok()
    .or_else(|| {
        conn.query_row(
            "SELECT sys.name, s.version \
             FROM standards s \
             JOIN systems sys ON sys.id = s.system_id \
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RAII guard that sets an env var for the duration of a test and
    /// restores the previous value (or unsets it) on drop.
    struct TempEnvGuard {
        key: String,
        old_val: Option<String>,
    }
    impl TempEnvGuard {
        fn new(key: &str, val: &std::path::Path) -> Self {
            let old_val = std::env::var(key).ok();
            std::env::set_var(key, val);
            Self { key: key.to_string(), old_val }
        }
    }
    impl Drop for TempEnvGuard {
        fn drop(&mut self) {
            match &self.old_val {
                Some(v) => std::env::set_var(&self.key, v),
                None => std::env::remove_var(&self.key),
            }
        }
    }

    #[test]
    fn init_options_default_matches_old_behavior() {
        let opts = InitOptions::default();
        assert!(!opts.force);
        assert!(opts.standard_system.is_none());
        assert!(opts.script_overrides.is_empty());
        assert!(opts.check_overrides.is_empty());
        assert!(!opts.auto_detect_dirs);
        assert!(!opts.sync_knowledge_system);
    }

    #[test]
    fn probe_directories_detects_docs() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
        let detected = probe_directories(tmp.path());
        assert_eq!(detected.root_dir.as_deref(), Some("docs"));
    }

    #[test]
    fn probe_directories_prefers_crates_over_src() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::create_dir_all(tmp.path().join("crates")).unwrap();
        let detected = probe_directories(tmp.path());
        assert_eq!(detected.implementation_dir.as_deref(), Some("crates"));
    }

    #[test]
    fn probe_directories_falls_back_to_src() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        let detected = probe_directories(tmp.path());
        assert_eq!(detected.implementation_dir.as_deref(), Some("src"));
    }

    #[test]
    fn probe_directories_skips_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let detected = probe_directories(tmp.path());
        assert!(detected.root_dir.is_none());
        assert!(detected.implementation_dir.is_none());
        assert!(detected.tests_dir.is_none());
        assert!(detected.scripts_dir.is_none());
    }

    #[test]
    fn probe_directories_detects_tests_and_scripts() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("tests")).unwrap();
        std::fs::create_dir_all(tmp.path().join("scripts")).unwrap();
        let detected = probe_directories(tmp.path());
        assert_eq!(detected.tests_dir.as_deref(), Some("tests"));
        assert_eq!(detected.scripts_dir.as_deref(), Some("scripts"));
    }

    #[test]
    fn probe_directories_prefers_tests_over_test() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("tests")).unwrap();
        std::fs::create_dir_all(tmp.path().join("test")).unwrap();
        let detected = probe_directories(tmp.path());
        assert_eq!(detected.tests_dir.as_deref(), Some("tests"));
    }

    #[test]
    fn probe_directories_detects_documentation_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("documentation")).unwrap();
        let detected = probe_directories(tmp.path());
        assert_eq!(detected.root_dir.as_deref(), Some("documentation"));
    }

    #[test]
    fn probe_directories_detects_lib_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("lib")).unwrap();
        let detected = probe_directories(tmp.path());
        assert_eq!(detected.implementation_dir.as_deref(), Some("lib"));
    }

    #[test]
    fn init_repository_fresh_with_auto_detect() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::create_dir_all(tmp.path().join("scripts")).unwrap();

        let options = InitOptions {
            auto_detect_dirs: true,
            standard_system: Some("test-std".to_string()),
            ..Default::default()
        };
        let result = init_repository(tmp.path(), &options).unwrap();

        assert!(tmp.path().join("samgraha.toml").exists());
        assert_eq!(
            result.config.repository.documentation.standard_system.as_deref(),
            Some("test-std")
        );
        assert_eq!(result.config.repository.documentation.root_dir, "docs");
        assert_eq!(result.config.repository.implementation.dir, "src");
        assert!(result.config.repository.scripts.is_some());
        assert!(result.sync_result.is_none());
    }

    #[test]
    fn init_repository_backfill_does_not_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("samgraha.toml");
        // Write a config with a custom root_dir
        std::fs::write(
            &config_path,
            "[repository.documentation]\nroot_dir = \"custom-docs\"\n",
        )
        .unwrap();

        let options = InitOptions {
            auto_detect_dirs: true,
            ..Default::default()
        };
        std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
        let result = init_repository(tmp.path(), &options).unwrap();
        // Backfill should NOT overwrite the existing root_dir
        assert_eq!(result.config.repository.documentation.root_dir, "custom-docs");
    }

    #[test]
    fn sync_meta_roundtrip() {
        let meta = SyncMeta {
            system: "test-sys".to_string(),
            version: "1.2.3".to_string(),
            synced_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: SyncMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.system, "test-sys");
        assert_eq!(parsed.version, "1.2.3");
    }

    #[test]
    fn check_overrides_propagation() {
        let tmp = tempfile::tempdir().unwrap();
        let mut script_overrides = std::collections::HashMap::new();
        script_overrides.insert("compile".to_string(), "my-custom-compile".to_string());
        let mut check_overrides = std::collections::HashMap::new();
        check_overrides.insert("architecture".to_string(), "my-check-arch".to_string());

        let options = InitOptions {
            standard_system: Some("my-system".to_string()),
            script_overrides,
            check_overrides,
            ..Default::default()
        };
        let result = init_repository(tmp.path(), &options).unwrap();

        assert_eq!(
            result.config.repository.documentation.standard_system.as_deref(),
            Some("my-system")
        );
        assert_eq!(
            result.config.repository.documentation.script_overrides.get("compile").map(|s| s.as_str()),
            Some("my-custom-compile")
        );
        assert_eq!(
            result.config.repository.documentation.check_overrides.get("architecture").map(|s| s.as_str()),
            Some("my-check-arch")
        );
    }

    #[test]
    fn init_repository_with_all_options() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
        std::fs::create_dir_all(tmp.path().join("crates")).unwrap();
        std::fs::create_dir_all(tmp.path().join("tests")).unwrap();
        std::fs::create_dir_all(tmp.path().join("scripts")).unwrap();

        let mut script_overrides = std::collections::HashMap::new();
        script_overrides.insert("lint".to_string(), "cargo clippy".to_string());
        let mut check_overrides = std::collections::HashMap::new();
        check_overrides.insert("security".to_string(), "cargo-audit".to_string());

        let options = InitOptions {
            force: false,
            standard_system: Some("full-system".to_string()),
            script_overrides,
            check_overrides,
            auto_detect_dirs: true,
            sync_knowledge_system: false,
        };
        let result = init_repository(tmp.path(), &options).unwrap();

        assert!(tmp.path().join("samgraha.toml").exists());
        assert!(tmp.path().join(".env.example").exists());
        assert_eq!(result.config.repository.documentation.root_dir, "docs");
        assert_eq!(result.config.repository.implementation.dir, "crates");
        assert!(result.config.repository.tests.is_some());
        assert!(result.config.repository.scripts.is_some());
        assert_eq!(
            result.config.repository.documentation.standard_system.as_deref(),
            Some("full-system")
        );
        assert_eq!(
            result.config.repository.documentation.script_overrides.get("lint").map(|s| s.as_str()),
            Some("cargo clippy")
        );
        assert_eq!(
            result.config.repository.documentation.check_overrides.get("security").map(|s| s.as_str()),
            Some("cargo-audit")
        );
    }

    #[test]
    fn init_repository_force_overwrites_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("samgraha.toml");
        std::fs::write(&config_path, "[repository]\nid = \"old-repo\"\n").unwrap();

        let options = InitOptions {
            force: true,
            ..Default::default()
        };
        let result = init_repository(tmp.path(), &options).unwrap();
        // force=true → fresh write, old id is gone
        assert_ne!(result.config.repository.id.as_deref(), Some("old-repo"));
    }

    #[test]
    fn staleness_detection_never_synced() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".samgraha")).unwrap();
        // No sync-meta.json → NeverSynced
        let status = check_knowledge_staleness(tmp.path()).unwrap();
        assert!(matches!(status, StalenessStatus::NeverSynced));
    }

    #[test]
    fn staleness_detection_missing_local_db() {
        let tmp = tempfile::tempdir().unwrap();
        let sg_dir = tmp.path().join(".samgraha");
        std::fs::create_dir_all(&sg_dir).unwrap();
        // Write a sync-meta.json that claims a version
        let meta = SyncMeta {
            system: "test-sys".to_string(),
            version: "1.0.0".to_string(),
            synced_at: "2026-01-01T00:00:00Z".to_string(),
        };
        std::fs::write(sg_dir.join("sync-meta.json"), serde_json::to_string(&meta).unwrap()).unwrap();
        // No local standards.db — result depends on whether global store exists:
        //   global missing → SourceMissing (checked first)
        //   global present  → MissingLocal
        let status = check_knowledge_staleness(tmp.path()).unwrap();
        let matches = matches!(
            status,
            StalenessStatus::MissingLocal | StalenessStatus::SourceMissing
        );
        assert!(matches, "expected MissingLocal or SourceMissing, got {:?}", status);
    }

    #[test]
    fn read_system_info_from_db_returns_joined_data() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE systems (
                id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                description TEXT, is_default INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE standards (
                id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                UNIQUE(system_id, name, version)
            );
            INSERT INTO systems (id, name, is_default) VALUES (1, 'my-system', 1);
            INSERT INTO standards (id, system_id, name, version) VALUES (1, 1, 'my-standard', '2.3.4');",
        )
        .unwrap();
        drop(conn);

        let (name, version) = read_system_info_from_db(&db_path).unwrap();
        assert_eq!(name, "my-system");
        assert_eq!(version, "2.3.4");
    }

    #[test]
    fn read_system_info_from_db_falls_back_to_any_system() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE systems (
                id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                description TEXT, is_default INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE standards (
                id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                UNIQUE(system_id, name, version)
            );
            INSERT INTO systems (id, name, is_default) VALUES (1, 'only-system', 0);
            INSERT INTO standards (id, system_id, name, version) VALUES (1, 1, 'std', '0.1.0');",
        )
        .unwrap();
        drop(conn);

        let (name, version) = read_system_info_from_db(&db_path).unwrap();
        assert_eq!(name, "only-system");
        assert_eq!(version, "0.1.0");
    }

    #[test]
    fn read_system_info_from_db_empty_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("empty.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE systems (
                id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                description TEXT, is_default INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE standards (
                id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                UNIQUE(system_id, name, version)
            );",
        )
        .unwrap();
        drop(conn);

        assert!(read_system_info_from_db(&db_path).is_none());
    }

    // ── Integration tests (use SAMGRAHA_MCP_DIR to mock global store) ────

    /// Create a mock global store at `dir` with `standards.db` containing a
    /// system and standard at the given version. help.db is not set up here
    /// because `RegistryStore` requires its own migration schema; the
    /// `sync_knowledge_system` function handles missing help.db gracefully.
    fn setup_mock_global_store(dir: &Path, system_name: &str, version: &str) {
        let conn = rusqlite::Connection::open(dir.join("standards.db")).unwrap();
        conn.execute_batch(&format!(
            "PRAGMA user_version = 3;
             CREATE TABLE IF NOT EXISTS systems (
                 id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                 description TEXT, is_default INTEGER NOT NULL DEFAULT 1
             );
             CREATE TABLE IF NOT EXISTS standards (
                 id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                 name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                 generation_granularity TEXT NOT NULL DEFAULT 'section',
                 UNIQUE(system_id, name, version)
             );
             DELETE FROM systems;
             DELETE FROM standards;
             INSERT INTO systems (id, name, is_default) VALUES (1, '{system_name}', 1);
             INSERT INTO standards (id, system_id, name, version)
                 VALUES (1, 1, '{system_name}-std', '{version}');",
        ))
        .unwrap();
    }

    /// Single lifecycle test covering: initial sync → staleness check →
    /// force re-sync after version bump → sync_if_stale skip → sync_if_stale
    /// force. Uses one env-var scope to avoid parallel-test races.
    #[test]
    fn sync_lifecycle_full() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        std::fs::create_dir_all(root.join(".samgraha")).unwrap();
        std::fs::write(
            root.join("samgraha.toml"),
            "[repository]\nid = \"test\"\n",
        )
        .unwrap();

        let global = tempfile::tempdir().unwrap();
        setup_mock_global_store(global.path(), "my-sys", "1.0.0");

        let _guard = TempEnvGuard::new("SAMGRAHA_MCP_DIR", global.path());

        // 1. Initial sync
        let result = sync_knowledge_system(&root).unwrap();
        assert!(result.standards_synced);
        // help_documents_synced may be 0 if help.db doesn't ship with
        // matching schema — the important thing is no error was thrown.

        // 2. sync-meta.json written correctly
        let meta_path = root.join(".samgraha").join("sync-meta.json");
        assert!(meta_path.exists());
        let meta: SyncMeta =
            serde_json::from_str(&std::fs::read_to_string(&meta_path).unwrap()).unwrap();
        assert_eq!(meta.system, "my-sys");
        assert_eq!(meta.version, "1.0.0");
        assert!(!meta.synced_at.is_empty());

        // 3. Staleness check — UpToDate
        let status = check_knowledge_staleness(&root).unwrap();
        assert!(matches!(status, StalenessStatus::UpToDate { ref version } if version == "1.0.0"));

        // 4. sync_if_stale without force — should skip
        let result = sync_if_stale(&root, false).unwrap();
        assert!(result.is_none(), "should skip when up to date");

        // 5. "Upgrade" global to 2.0.0
        setup_mock_global_store(global.path(), "my-sys", "2.0.0");

        // 6. Staleness check — Stale
        let status = check_knowledge_staleness(&root).unwrap();
        assert!(matches!(status, StalenessStatus::Stale { .. }));

        // 7. sync_if_stale without force — should still sync (stale, not UpToDate)
        let result = sync_if_stale(&root, false).unwrap();
        assert!(result.is_some(), "should sync when stale");
        let sr = result.unwrap();
        assert!(sr.standards_synced);

        // 8. Now up to date with new version
        let status = check_knowledge_staleness(&root).unwrap();
        assert!(matches!(status, StalenessStatus::UpToDate { ref version } if version == "2.0.0"));

        // 9. sync_if_stale with force — always syncs
        let result = sync_if_stale(&root, true).unwrap();
        assert!(result.is_some(), "should sync when forced");
    }
}
