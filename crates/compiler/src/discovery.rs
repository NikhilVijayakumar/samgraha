use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DiscoveredDocument {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub standard: String,
}

pub struct DiscoveryEngine;

/// Maps a directory name to the domain it compiles to when that domain's
/// on-disk directory name differs from the domain string itself. Used by
/// both `discover()`'s root-dir check and `infer_standard()`'s parent-dir
/// check — a single table instead of two independently-maintained checks.
///
/// `docs/raw/product-guide/` compiles to domain `"help"` (not
/// `"product-guide"`) for backward compatibility with existing
/// `PipelineKind::Help`/`domain_exclusion` consumers; `("help", "help")`
/// keeps an unrenamed local checkout working during the transition.
const DOMAIN_OVERRIDE: &[(&str, &str)] = &[
    ("product-guide", "help"),
    ("help", "help"),
    ("standards", "standards"),
];

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

        collect_markdown_files(
            root,
            root,
            include_patterns,
            exclude_patterns,
            &mut documents,
        )?;

        // When the compile root itself is a built-in knowledge directory (e.g. compiling
        // `docs/raw/product-guide` or `docs/raw/standards` directly, as the release build does),
        // every file under it belongs to that one domain regardless of nesting — per-file
        // inference by immediate parent directory name (below) can't see this in this case
        // since the root itself is stripped from every relative path before inference runs.
        let root_name = root.file_name().and_then(|s| s.to_str()).map(|s| s.to_lowercase());
        if let Some(name) = root_name {
            if let Some((_, domain)) = DOMAIN_OVERRIDE.iter().find(|(dir, _)| *dir == name) {
                for doc in &mut documents {
                    doc.standard = domain.to_string();
                }
            }
        }

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

        if let Some((_, domain)) = DOMAIN_OVERRIDE.iter().find(|(dir, _)| *dir == parent) {
            return domain.to_string();
        }

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
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            if !exclude.iter().any(|p| common::glob::matches_glob(p, &relative_str)) {
                collect_markdown_files(root, &path, _include, exclude, documents)?;
            }
        } else if path.extension().map_or(false, |e| e == "md") {
            // `matches_glob` matches full relative paths, not bare filenames —
            // file-level patterns like "**/manual-audit.md" need the same
            // check directories get above. Without it, a file matching an
            // exclude pattern still gets discovered and compiled here, then
            // deleted later by compilation.rs's post-insert ignore-pattern
            // cleanup — but enrichment has already queued a row for it by
            // then, so that row's document_id FK breaks on insert.
            if exclude.iter().any(|p| common::glob::matches_glob(p, &relative_str)) {
                continue;
            }
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

    #[test]
    fn test_infer_standard_definition() {
        let p = Path::new("docs/raw/standards/feature-design.md");
        assert_eq!(DiscoveryEngine::infer_standard(p), "standards");
    }

    #[test]
    fn test_infer_standard_product_guide_maps_to_help_domain() {
        let p = Path::new("docs/raw/product-guide/index.md");
        assert_eq!(DiscoveryEngine::infer_standard(p), "help");
    }

    #[test]
    fn test_infer_standard_help_still_maps_to_help_domain() {
        // Backward compat: an unrenamed local checkout still works.
        let p = Path::new("docs/raw/help/index.md");
        assert_eq!(DiscoveryEngine::infer_standard(p), "help");
    }

    #[test]
    fn test_discover_root_product_guide_tags_every_file_help() {
        // The override matches on the discover() *root* directory's own
        // name, so the leaf component must literally be "product-guide".
        let dir = std::env::temp_dir()
            .join(format!(
                "samgraha-discover-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            ))
            .join("product-guide");
        std::fs::create_dir_all(dir.join("commands")).unwrap();
        std::fs::write(dir.join("index.md"), "# Guide").unwrap();
        std::fs::write(dir.join("commands/audit.md"), "# Audit").unwrap();

        let docs = DiscoveryEngine::discover(&dir, &[], &[]).unwrap();
        assert_eq!(docs.len(), 2);
        assert!(docs.iter().all(|d| d.standard == "help"), "expected every doc tagged 'help', got: {:?}", docs.iter().map(|d| &d.standard).collect::<Vec<_>>());

        std::fs::remove_dir_all(dir.parent().unwrap()).ok();
    }
}
