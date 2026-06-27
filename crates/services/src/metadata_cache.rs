use anyhow::Result;
use schemas::manifest::{CachedRepoMetadata, RepositoryManifest};
use std::path::Path;

/// Static utilities for reading dependency manifest files.
///
/// Superseded by `RegistryDb` for cache storage.
/// These helpers parse `manifest.json` files from dependency repositories
/// (compiler outputs, not cache) and are used by the Knowledge Resolver
/// for transitive dependency graph discovery.
pub struct MetadataCache;

impl MetadataCache {
    /// Read a dependency's manifest.json directly. Used as cache miss fallback.
    pub fn read_dependency_manifest(
        dep_root: &Path,
    ) -> Result<Option<RepositoryManifest>> {
        let manifest_path = dep_root.join(".samgraha").join("manifest.json");
        if !manifest_path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: RepositoryManifest = serde_json::from_str(&content)?;
        Ok(Some(manifest))
    }

    /// Build a CachedRepoMetadata from a RepositoryManifest with a default 24h TTL.
    pub fn from_manifest(manifest: &RepositoryManifest) -> CachedRepoMetadata {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::hours(24);
        CachedRepoMetadata {
            repository: manifest.repository.clone(),
            revision: manifest.revision,
            repository_root: manifest.repository_root.clone(),
            knowledge: manifest.knowledge.clone(),
            exports: manifest.exports.clone(),
            audit: manifest.audit.status.clone(),
            last_sync: now.to_rfc3339(),
            expires: expires.to_rfc3339(),
        }
    }
}
