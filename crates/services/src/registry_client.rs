use anyhow::Result;
use common::config::{parse_ttl_duration, SamgrahaConfig};
use common::fs::validate_path;
use registry::registry_db::RegistryDb;
use schemas::manifest::{CachedRepoMetadata, RepositoryManifest};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

use crate::metadata_cache::MetadataCache;

/// Query filter for repository discovery.
#[derive(Debug, Clone, Default)]
pub struct RegistryQuery {
    pub uuid: Option<Uuid>,
    pub id: Option<String>,
    pub export: Option<String>,
    pub capability: Option<String>,
    pub workspace: Option<String>,
}

/// Repository Registry service interface.
pub trait RegistryClient: Send + Sync {
    /// Register current repository in local registry.
    fn register(&self, manifest: &RepositoryManifest) -> Result<()>;
    /// Unregister a repository by UUID.
    fn unregister(&self, uuid: &Uuid) -> Result<()>;
    /// Synchronize dependency metadata from their manifests into local cache.
    fn sync(&self, config: &SamgrahaConfig) -> Result<()>;
    /// Discover repositories matching query.
    fn discover(&self, query: &RegistryQuery) -> Result<Vec<CachedRepoMetadata>>;
    /// Get metadata for a single repository.
    fn get_metadata(&self, uuid: &Uuid) -> Result<Option<CachedRepoMetadata>>;
    /// List all registered repositories.
    fn list(&self) -> Result<Vec<CachedRepoMetadata>>;
}

/// SQLite-backed RegistryClient implementation.
///
/// Uses `.samgraha/registry.db` as the backing store (via `RegistryDb`).
/// Supersedes the Phase 1-5 JSON file approach.
pub struct FileRegistryClient {
    root: PathBuf,
    db: Arc<RegistryDb>,
    ttl_seconds: i64,
}

impl FileRegistryClient {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            db: Arc::new(RegistryDb::open(root).expect("Failed to open registry database")),
            ttl_seconds: parse_ttl_duration("24h").unwrap_or(86400),
        }
    }

    pub fn with_config(root: &Path, config: &common::config::ResolverConfig) -> Self {
        Self {
            root: root.to_path_buf(),
            db: Arc::new(RegistryDb::open(root).expect("Failed to open registry database")),
            ttl_seconds: parse_ttl_duration(&config.metadata_ttl).unwrap_or(86400),
        }
    }

    /// Build CachedRepoMetadata from a RepositoryManifest with TTL-expiry computed.
    fn meta_from_manifest(&self, manifest: &RepositoryManifest) -> CachedRepoMetadata {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::seconds(self.ttl_seconds);
        CachedRepoMetadata {
            repository: manifest.repository.clone(),
            revision: manifest.revision,
            repository_root: manifest.repository_root.clone(),
            knowledge: manifest.knowledge.clone(),
            exports: manifest.exports.clone(),
            audit: manifest.audit.status.clone(),
            last_sync: now.to_rfc3339(),
            expires: expires.to_rfc3339(),
            dependencies: manifest.dependencies.clone(),
        }
    }
}

impl RegistryClient for FileRegistryClient {
    fn register(&self, manifest: &RepositoryManifest) -> Result<()> {
        validate_path(Path::new(&manifest.repository_root), &self.root)?;
        // ENG-GAP-06: UUID spoofing prevention — reject if id exists with different UUID.
        if let Some(existing) = self.db.get_by_id(&manifest.repository.id)? {
            if existing.repository.uuid != manifest.repository.uuid {
                anyhow::bail!(
                    "UUID mismatch for '{}': stored {} != manifest {}",
                    manifest.repository.id,
                    existing.repository.uuid,
                    manifest.repository.uuid
                );
            }
        }
        let meta = self.meta_from_manifest(manifest);
        self.db.cache_write(&meta)?;
        Ok(())
    }

    fn unregister(&self, uuid: &Uuid) -> Result<()> {
        if !self.db.unregister(uuid)? {
            anyhow::bail!("Repository with UUID {} not found in local registry", uuid);
        }
        Ok(())
    }

    fn sync(&self, config: &SamgrahaConfig) -> Result<()> {
        for dep in &config.repository.dependencies {
            let dep_root = dep.path.as_ref().map(|p| {
                let p = Path::new(p);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    self.root.join(p)
                }
            });

            if let Some(ref root) = dep_root {
                if let Ok(Some(manifest)) = MetadataCache::read_dependency_manifest(root) {
                    // ENG-GAP-06: UUID spoofing prevention — reject if id exists with different UUID.
                    if let Some(existing) = self.db.get_by_id(&manifest.repository.id)? {
                        if existing.repository.uuid != manifest.repository.uuid {
                            anyhow::bail!(
                                "UUID mismatch for dependency '{}': stored {} != manifest {}",
                                manifest.repository.id,
                                existing.repository.uuid,
                                manifest.repository.uuid
                            );
                        }
                    }
                    let meta = self.meta_from_manifest(&manifest);
                    self.db.cache_write(&meta)?;
                }
            }
        }
        Ok(())
    }

    fn discover(&self, query: &RegistryQuery) -> Result<Vec<CachedRepoMetadata>> {
        let all = self.list()?;
        Ok(all
            .into_iter()
            .filter(|m| {
                if let Some(uuid) = &query.uuid {
                    if &m.repository.uuid != uuid {
                        return false;
                    }
                }
                if let Some(id) = &query.id {
                    if &m.repository.id != id {
                        return false;
                    }
                }
                if let Some(export) = &query.export {
                    if !m.exports.contains(export) {
                        return false;
                    }
                }
                // ponytail: capability/workspace filters not applied — CachedRepoMetadata
                // stores exports only; capabilities are not cached in repository_cache.
                // Add a capabilities column to REG_V2 if these filters are needed.
                true
            })
            .collect())
    }

    fn get_metadata(&self, uuid: &Uuid) -> Result<Option<CachedRepoMetadata>> {
        self.db.get_by_uuid(uuid)
    }

    fn list(&self) -> Result<Vec<CachedRepoMetadata>> {
        self.db.list()
    }
}
