use anyhow::Result;
use schemas::KnowledgeSystemIdentity;
use std::path::{Path, PathBuf};

/// A Knowledge System discovered inside a Knowledge Repository.
#[derive(Debug, Clone)]
pub struct DiscoveredKnowledgeSystem {
    pub identity: KnowledgeSystemIdentity,
    pub path: PathBuf,
}

pub struct KnowledgeSystemLoader;

impl KnowledgeSystemLoader {
    /// Discovers and loads all valid knowledge systems within a given root directory (e.g. `system/`).
    pub fn load_systems<P: AsRef<Path>>(root: P) -> Result<Vec<DiscoveredKnowledgeSystem>> {
        let root = root.as_ref();
        let mut systems = Vec::new();

        if !root.exists() || !root.is_dir() {
            return Ok(systems);
        }

        for entry in std::fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let system_toml_path = path.join("system.toml");
                if system_toml_path.exists() {
                    let content = std::fs::read_to_string(&system_toml_path)?;
                    let identity: KnowledgeSystemIdentity = toml::from_str(&content)?;
                    systems.push(DiscoveredKnowledgeSystem {
                        identity,
                        path,
                    });
                }
            }
        }

        Ok(systems)
    }
}
