use std::path::PathBuf;
use schemas::config::SamgrahaConfig;

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub repository_root: PathBuf,
    pub registry_path: PathBuf,
    pub config: SamgrahaConfig,
    pub workspace_id: Option<String>,
}

impl RuntimeContext {
    pub fn new(
        repository_root: PathBuf,
        registry_path: PathBuf,
        config: SamgrahaConfig,
    ) -> Self {
        Self {
            repository_root,
            registry_path,
            config,
            workspace_id: None,
        }
    }

    pub fn repository_name(&self) -> String {
        self.repository_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}
