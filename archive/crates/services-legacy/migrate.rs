use anyhow::{Context, Result};
use common::config::SamgrahaConfig;
use compiler::DiscoveryEngine;
use standards::StandardRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A markdown file the plan proposes moving into `{docs_root}/raw/{domain}/`.
#[derive(Debug, Clone)]
pub struct DocumentMove {
    pub from: PathBuf,
    pub to: PathBuf,
    pub domain: String,
}

/// Result of scanning a repo for documentation that lives outside the
/// canonical `{docs_root}/raw/{domain}/` layout. `moves` are files whose
/// domain could be determined (either by the compiler's own inference or by
/// a caller-supplied `overrides` entry) and are safe to relocate.
/// `unclassified` are files nothing could confidently place — these are
/// left alone and reported so a human can pick a domain (via `overrides`)
/// or leave them where they are.
#[derive(Debug, Clone, Default)]
pub struct MigrationPlan {
    pub moves: Vec<DocumentMove>,
    pub unclassified: Vec<PathBuf>,
}

/// Scan `root` for markdown files outside `{docs_root}/raw/` and classify
/// each using the same domain inference `samgraha compile` uses
/// (`DiscoveryEngine::infer_standard`, including numbered-prefix
/// normalization), so the plan matches what compilation would already tag
/// the file as. A file only becomes a `move` if its inferred (or
/// overridden) domain is a *registered* standard — anything else is
/// reported as `unclassified` rather than guessed at, since a wrong
/// automatic move is harder to notice than a file staying put.
///
/// `overrides` maps a source directory's *basename* (e.g. `"06-audits"`) to
/// a domain name, for repos whose existing layout doesn't match any
/// built-in domain by name (Gap 3/6 in docs/raw/proposal.md) — the caller
/// supplies the mapping since the compiler has no generic way to guess it.
///
/// Ignore patterns are resolved the same way `samgraha compile` resolves
/// them (`.samagraignore` + `[repository.ignore].patterns`), so a migration
/// scan never proposes moving something compilation would have skipped.
pub fn plan_migration(
    root: &Path,
    config: &SamgrahaConfig,
    registry: &StandardRegistry,
    overrides: &HashMap<String, String>,
) -> Result<MigrationPlan> {
    let ignore_patterns = crate::compilation::merge_ignore_patterns(root, config);
    let docs_root = common::config::resolve_configured_dir(
        &config.repository.documentation.root_dir,
        root,
        "docs",
    );
    let canonical_root = docs_root.join("raw");

    let discovered = DiscoveryEngine::discover(root, &[], &ignore_patterns)
        .context("Failed to walk repository for markdown files")?;

    let mut plan = MigrationPlan::default();

    for doc in discovered {
        if doc.path.starts_with(&canonical_root) {
            continue; // already in place
        }
        if is_repo_root_readme(&doc.path, root) {
            continue; // README.md stays at the repo root by convention
        }

        let parent_name = doc
            .path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let domain = overrides.get(&parent_name).cloned().unwrap_or(doc.standard);

        if registry.has_standard(&domain) {
            let file_name = doc.path.file_name().expect("markdown file has a name").to_owned();
            plan.moves.push(DocumentMove {
                from: doc.path,
                to: canonical_root.join(&domain).join(file_name),
                domain,
            });
        } else {
            plan.unclassified.push(doc.path);
        }
    }

    Ok(plan)
}

fn is_repo_root_readme(path: &Path, root: &Path) -> bool {
    path.parent() == Some(root)
        && path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("readme.md"))
            .unwrap_or(false)
}

/// Execute a previously computed plan, moving each file on disk.
/// Never overwrites: a destination that already exists is skipped (and
/// reported back, not silently dropped) so re-running a plan is always
/// safe. Creates destination domain directories as needed.
pub fn apply_migration(plan: &MigrationPlan) -> Result<AppliedMigration> {
    let mut moved = Vec::new();
    let mut skipped_existing = Vec::new();

    for mv in &plan.moves {
        if mv.to.exists() {
            skipped_existing.push(mv.clone());
            continue;
        }
        if let Some(parent) = mv.to.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        std::fs::rename(&mv.from, &mv.to)
            .with_context(|| format!("Failed to move {} to {}", mv.from.display(), mv.to.display()))?;
        moved.push(mv.clone());
    }

    Ok(AppliedMigration { moved, skipped_existing })
}

#[derive(Debug, Clone, Default)]
pub struct AppliedMigration {
    pub moved: Vec<DocumentMove>,
    pub skipped_existing: Vec<DocumentMove>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "samgraha-migrate-test-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ))
    }

    #[test]
    fn plan_migration_classifies_flat_numbered_docs_and_leaves_unmatched_dirs_unclassified() {
        let root = temp_dir("flat");
        write(&root.join("docs/01-vision.md"), "# Vision");
        write(&root.join("docs/02-philosophy.md"), "# Philosophy");
        write(&root.join("docs/06-audits/repository-discovery.md"), "# Discovery");
        write(&root.join("README.md"), "# Repo");

        let config = SamgrahaConfig::default();
        let registry = StandardRegistry::with_builtins();
        let plan = plan_migration(&root, &config, &registry, &HashMap::new()).unwrap();

        assert_eq!(plan.moves.len(), 2, "expected vision + philosophy to classify: {:?}", plan.moves);
        assert!(plan.moves.iter().any(|m| m.domain == "vision" && m.to.ends_with("raw/vision/01-vision.md")));
        assert!(plan.moves.iter().any(|m| m.domain == "philosophy" && m.to.ends_with("raw/philosophy/02-philosophy.md")));

        assert_eq!(plan.unclassified.len(), 1);
        assert!(plan.unclassified[0].ends_with("06-audits/repository-discovery.md"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn plan_migration_respects_directory_overrides_for_unmatched_dirs() {
        let root = temp_dir("override");
        write(&root.join("docs/06-audits/repository-discovery.md"), "# Discovery");

        let config = SamgrahaConfig::default();
        let registry = StandardRegistry::with_builtins();
        let mut overrides = HashMap::new();
        overrides.insert("06-audits".to_string(), "feature".to_string());

        let plan = plan_migration(&root, &config, &registry, &overrides).unwrap();

        assert_eq!(plan.unclassified.len(), 0);
        assert_eq!(plan.moves.len(), 1);
        assert_eq!(plan.moves[0].domain, "feature");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn plan_migration_skips_files_already_in_canonical_layout() {
        let root = temp_dir("already-placed");
        write(&root.join("docs/raw/vision/vision.md"), "# Vision");

        let config = SamgrahaConfig::default();
        let registry = StandardRegistry::with_builtins();
        let plan = plan_migration(&root, &config, &registry, &HashMap::new()).unwrap();

        assert!(plan.moves.is_empty());
        assert!(plan.unclassified.is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn plan_migration_leaves_repo_root_readme_in_place() {
        let root = temp_dir("readme");
        write(&root.join("README.md"), "# Repo");

        let config = SamgrahaConfig::default();
        let registry = StandardRegistry::with_builtins();
        let plan = plan_migration(&root, &config, &registry, &HashMap::new()).unwrap();

        assert!(plan.moves.is_empty());
        assert!(plan.unclassified.is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn apply_migration_moves_files_and_skips_existing_destinations() {
        let root = temp_dir("apply");
        write(&root.join("docs/01-vision.md"), "# Vision content");
        write(&root.join("docs/raw/philosophy/02-philosophy.md"), "already here");
        write(&root.join("docs/02-philosophy.md"), "# would collide");

        let config = SamgrahaConfig::default();
        let registry = StandardRegistry::with_builtins();
        let plan = plan_migration(&root, &config, &registry, &HashMap::new()).unwrap();
        let applied = apply_migration(&plan).unwrap();

        assert_eq!(applied.moved.len(), 1);
        assert_eq!(applied.skipped_existing.len(), 1);
        assert!(root.join("docs/raw/vision/01-vision.md").exists());
        assert!(!root.join("docs/01-vision.md").exists());
        // Collision left untouched at its original location.
        assert!(root.join("docs/02-philosophy.md").exists());
        assert_eq!(
            std::fs::read_to_string(root.join("docs/raw/philosophy/02-philosophy.md")).unwrap(),
            "already here"
        );

        std::fs::remove_dir_all(&root).ok();
    }
}
