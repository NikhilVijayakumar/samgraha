use anyhow::Result;
use schemas::ProjectCase;
use std::collections::HashMap;
use std::path::Path;

/// Describes what the project currently has — drives phase generation.
pub struct ProjectContext {
    /// Which goal the user selected.
    pub case: ProjectCase,
    /// True when docs/raw/ has at least some markdown files.
    pub has_docs: bool,
    /// Known domains that have been compiled.
    pub compiled_domains: Vec<String>,
    /// Audit pipelines that have been run, mapped to their latest score.
    pub existing_scores: HashMap<String, f64>,
}

impl ProjectContext {
    pub fn detect(root: &Path, _case: &ProjectCase) -> Result<Self> {
        let raw_dir = root.join("docs").join("raw");
        let has_docs = raw_dir.is_dir() && Self::has_md_files(&raw_dir);
        Ok(Self {
            case: _case.clone(),
            has_docs,
            compiled_domains: Vec::new(),
            existing_scores: HashMap::new(),
        })
    }

    fn has_md_files(dir: &Path) -> bool {
        std::fs::read_dir(dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
            })
            .unwrap_or(false)
    }
}
