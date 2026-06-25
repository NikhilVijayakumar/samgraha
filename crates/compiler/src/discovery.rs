use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct DiscoveredDocument {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub standard: String,
}

pub struct DiscoveryEngine;

impl DiscoveryEngine {
    pub fn discover<P: AsRef<Path>>(
        root: P,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<Vec<DiscoveredDocument>> {
        let root = root.as_ref();
        let mut documents = Vec::new();

        if !root.exists() {
            return Ok(documents);
        }

        collect_markdown_files(root, root, include_patterns, exclude_patterns, &mut documents)?;

        Ok(documents)
    }

    pub fn infer_standard(path: &Path) -> String {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let domain_map = [
            ("readme", "readme"),
            ("vision", "vision"),
            ("philosophy", "philosophy"),
            ("feature-design", "feature-design"),
            ("feature-technical", "feature-technical"),
            ("engineering", "engineering"),
            ("external-context", "external-context"),
            ("prototype", "prototype"),
            ("system-overview", "architecture"),
            ("component-model", "architecture"),
            ("communication", "architecture"),
            ("knowledge-flow", "architecture"),
            ("persistence", "architecture"),
            ("runtime-boundary", "architecture"),
            ("security-architecture", "architecture"),
            ("deployment", "architecture"),
            ("extensibility", "architecture"),
            ("workspace", "architecture"),
        ];

        for (key, domain) in &domain_map {
            if name == *key || parent.contains(key) {
                return domain.to_string();
            }
        }

        if parent.contains("architecture") {
            return "architecture".to_string();
        }

        parent
    }
}

fn collect_markdown_files(
    root: &Path,
    dir: &Path,
    _include: &[String],
    exclude: &[String],
    documents: &mut Vec<DiscoveredDocument>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !exclude.iter().any(|p| name.contains(p.trim_matches('*'))) {
                collect_markdown_files(root, &path, _include, exclude, documents)?;
            }
        } else if path.extension().map_or(false, |e| e == "md") {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_path_buf();
            let standard = DiscoveryEngine::infer_standard(&relative);
            documents.push(DiscoveredDocument {
                path,
                relative_path: relative,
                standard,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_standard_readme() {
        let p = Path::new("README.md");
        assert_eq!(DiscoveryEngine::infer_standard(p), "readme");
    }

    #[test]
    fn test_infer_standard_architecture() {
        let p = Path::new("docs/architecture/system-overview.md");
        assert_eq!(DiscoveryEngine::infer_standard(p), "architecture");
    }

    #[test]
    fn test_infer_standard_feature() {
        let p = Path::new("docs/raw/feature/knowledge-compilation.md");
        assert_eq!(DiscoveryEngine::infer_standard(p), "feature");
    }
}
