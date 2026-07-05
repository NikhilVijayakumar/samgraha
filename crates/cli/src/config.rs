use anyhow::{Context, Result};
use common::config::SamgrahaConfig;
use std::path::{Path, PathBuf};

pub fn load_config(config_path: Option<&PathBuf>) -> Result<SamgrahaConfig> {
    if let Some(path) = config_path {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .context(format!("Failed to read config at {}", path.display()))?;
            let config: SamgrahaConfig = toml::from_str(&content)?;
            return Ok(config);
        }
        anyhow::bail!("Config file not found: {}", path.display());
    }

    let search_paths = [
        Path::new("samgraha.toml"),
        &Path::new(".").join("samgraha.toml"),
    ];

    for path in &search_paths {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: SamgrahaConfig = toml::from_str(&content)?;
            return Ok(config);
        }
    }

    Ok(SamgrahaConfig::default())
}

pub fn discover_repository_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let mut current = Some(cwd.as_path());

    while let Some(dir) = current {
        if dir.join(".samgraha").is_dir() || dir.join("samgraha.toml").exists() {
            return Ok(dir.to_path_buf());
        }
        current = dir.parent();
    }

    anyhow::bail!(
        "fatal: not a samgraha repository (or any of the parent directories). \
         Run 'samgraha init' first to initialize."
    );
}
