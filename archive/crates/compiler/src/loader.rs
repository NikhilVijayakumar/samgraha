use anyhow::Result;
use schemas::KnowledgeSystemIdentity;
use std::path::{Path, PathBuf};

/// Expected subdirectories for a well-formed Knowledge System.
/// Missing directories produce warnings but do not fail discovery.
const EXPECTED_DIRS: &[&str] = &["standards", "audit", "templates"];

/// A validation warning from loading a Knowledge System.
#[derive(Debug, Clone)]
pub struct KnowledgeSystemWarning {
    pub system_id: String,
    pub message: String,
}

/// A Knowledge System discovered inside a Knowledge Repository.
#[derive(Debug, Clone)]
pub struct DiscoveredKnowledgeSystem {
    pub identity: KnowledgeSystemIdentity,
    pub path: PathBuf,
    /// Non-fatal validation warnings (e.g. missing recommended subdirectories).
    pub warnings: Vec<KnowledgeSystemWarning>,
}

pub struct KnowledgeSystemLoader;

impl KnowledgeSystemLoader {
    /// Discovers and loads all valid knowledge systems within a given root directory (e.g. `system/`).
    /// Results are sorted by system `id` for deterministic output regardless of filesystem ordering.
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

                    // Validate expected subdirectory structure — non-fatal warnings.
                    let mut warnings = Vec::new();
                    for dir in EXPECTED_DIRS {
                        if !path.join(dir).is_dir() {
                            warnings.push(KnowledgeSystemWarning {
                                system_id: identity.id.clone(),
                                message: format!(
                                    "Missing recommended directory '{}/' in system '{}'",
                                    dir, identity.id
                                ),
                            });
                        }
                    }

                    systems.push(DiscoveredKnowledgeSystem {
                        identity,
                        path,
                        warnings,
                    });
                }
            }
        }

        // Sort by system id for deterministic, stable output.
        systems.sort_by(|a, b| a.identity.id.cmp(&b.identity.id));

        Ok(systems)
    }
}
