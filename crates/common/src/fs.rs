use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Validate that a path is within an allowed root directory.
///
/// Canonicalizes both paths and checks that the canonicalized target
/// path starts with the canonicalized root. Returns the canonicalized
/// path on success.
pub fn validate_path(path: &Path, root: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Cannot canonicalize path: {}", path.display()))?;
    let canonical_root = root
        .canonicalize()
        .with_context(|| format!("Cannot canonicalize root: {}", root.display()))?;
    if !canonical.starts_with(&canonical_root) {
        anyhow::bail!(
            "Path {} is outside allowed root {}",
            canonical.display(),
            canonical_root.display()
        );
    }
    Ok(canonical)
}
