use anyhow::{Context, Result};
use common::config::{InitOptions, SamgrahaConfig};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────

/// Result of initializing (or backfilling) a repository's `samgraha.toml`.
pub struct InitResult {
    pub root: PathBuf,
    pub config: SamgrahaConfig,
    pub status: String,
    pub env_path: PathBuf,
}

/// Filesystem-probed directory candidates.
#[derive(Debug, Clone, Default)]
pub struct DetectedDirs {
    pub root_dir: Option<String>,
    pub implementation_dir: Option<String>,
    pub tests_dir: Option<String>,
    pub scripts_dir: Option<String>,
}

// ─────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────

/// Initialize `samgraha.toml` + `.samgraha/` at `root`, or backfill any keys
/// missing from an existing config (never overwrites a key that's already
/// there). Shared by the CLI `init` command and the MCP `init` tool so both
/// surfaces bootstrap a repo identically.
///
/// No knowledge-system sync step — `register_standard` records absolute
/// script paths and inline prompt content directly into `knowledge.db` at
/// registration time; there's nothing to copy into a fresh repo anymore.
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

    // ── Phase 4: Result ─────────────────────────────────────
    Ok(InitResult {
        root: root.to_path_buf(),
        config,
        status,
        env_path,
    })
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
            "# Absolute path for generated reports.\n\
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

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn init_repository_backfill_does_not_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("samgraha.toml");
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
        assert_eq!(result.config.repository.documentation.root_dir, "custom-docs");
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
        assert_ne!(result.config.repository.id.as_deref(), Some("old-repo"));
    }
}
