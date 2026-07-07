use anyhow::{Context, Result};
use common::config::SamgrahaConfig;
use std::path::{Path, PathBuf};

/// Result of initializing (or backfilling) a repository's `samgraha.toml`.
pub struct InitResult {
    pub root: PathBuf,
    pub config: SamgrahaConfig,
    pub status: String,
    pub env_path: PathBuf,
}

/// Initialize `samgraha.toml` + `.samgraha/` at `root`, or backfill any keys
/// missing from an existing config (never overwrites a key that's already
/// there). Shared by the CLI `init` command and the MCP `init` tool so both
/// surfaces bootstrap a repo identically.
pub fn init_repository(root: &Path, force: bool) -> Result<InitResult> {
    let config_path = root.join("samgraha.toml");
    let samgraha_dir = root.join(".samgraha");

    std::fs::create_dir_all(&samgraha_dir)
        .context(format!("Failed to create {}", samgraha_dir.display()))?;

    let mut template = SamgrahaConfig::default();
    let dir_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repository")
        .to_string();
    template.repository.id = Some(dir_name.clone());
    template.repository.name = Some(dir_name);
    template.repository.uuid = Some(uuid::Uuid::new_v4());
    // Declare every builtin standard by default; repos that don't use one
    // (e.g. no `prototype` docs) add it to `domain_exclusion` instead of
    // deleting it here, so the full catalog stays visible in the toml.
    template.repository.documentation.domain = standards::all_builtin_standards()
        .into_iter()
        .map(|s| s.domain)
        .collect();

    let (config, status) = if config_path.exists() && !force {
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

    Ok(InitResult { root: root.to_path_buf(), config, status, env_path })
}

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
