use crate::migration::REGISTRY_MIGRATIONS;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use schemas::manifest::{CachedRepoMetadata, RepoIdentity};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::info;
use uuid::Uuid;

/// SQLite-backed repository registry database at `.samgraha/registry.db`.
///
/// Replaces the Phase 1-5 JSON file approach (`.samgraha/dependencies/*.meta.json`).
/// Uses `REGISTRY_MIGRATIONS` (REG_V1) to create the `repository_cache` table.
/// The cache is disposable — fully rebuildable from dependency manifests.
pub struct RegistryDb {
    conn: Mutex<Connection>,
    _path: PathBuf,
}

impl RegistryDb {
    /// Open or create the registry database at `.samgraha/registry.db`.
    pub fn open(root: &Path) -> Result<Self> {
        let db_dir = root.join(".samgraha");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("registry.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open registry db at {}", db_path.display()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let store = Self {
            conn: Mutex::new(conn),
            _path: db_path,
        };
        store.run_registry_migrations()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let store = Self {
            conn: Mutex::new(conn),
            _path: PathBuf::from(":memory:"),
        };
        store.run_registry_migrations()?;
        Ok(store)
    }

    fn run_registry_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        for (i, migration) in REGISTRY_MIGRATIONS.iter().enumerate() {
            let version = i as i64 + 1;
            if version > current_version {
                info!("Applying registry migration REG_V{}", version);
                conn.execute_batch(migration)?;
                conn.execute(
                    "INSERT INTO _schema_version (version, applied_at) VALUES (?1, datetime('now'))",
                    params![version],
                )?;
            }
        }
        Ok(())
    }

    /// Register a repository from its manifest.
    ///
    /// Sets last_sync to generated_at and expires to 24h after generated_at.
    /// Prefer `FileRegistryClient::register()` for TTL-aware registration.
    pub fn register(&self, manifest: &schemas::manifest::RepositoryManifest) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let uuid_str = manifest.repository.uuid.to_string();
        let expires = chrono::DateTime::parse_from_rfc3339(&manifest.generated_at)
            .map(|dt| (dt + chrono::Duration::hours(24)).to_rfc3339())
            .unwrap_or_else(|_| (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339());
        let deps_str = serde_json::to_string(&manifest.dependencies).unwrap_or_default();
        conn.execute(
            "INSERT OR REPLACE INTO repository_cache (id, uuid, name, repository_root, knowledge_db, revision, exports, audit, last_sync, expires, dependencies)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                manifest.repository.id,
                uuid_str,
                manifest.repository.name,
                manifest.repository_root,
                manifest.knowledge.location,
                manifest.revision,
                serde_json::to_string(&manifest.exports).unwrap_or_default(),
                manifest.audit.status,
                manifest.generated_at,
                expires,
                deps_str,
            ],
        )?;
        Ok(())
    }

    /// Unregister a repository by UUID.
    pub fn unregister(&self, uuid: &Uuid) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let uuid_str = uuid.to_string();
        let affected = conn.execute("DELETE FROM repository_cache WHERE uuid = ?1", params![uuid_str])?;
        Ok(affected > 0)
    }

    /// List all cached repository entries.
    pub fn list(&self) -> Result<Vec<CachedRepoMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, uuid, name, repository_root, knowledge_db, revision, exports, audit, last_sync, expires, dependencies
             FROM repository_cache ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            let uuid_str: String = row.get(1)?;
            let uuid = Uuid::parse_str(&uuid_str).unwrap_or_default();
            let exports_str: String = row.get(6)?;
            let exports: Vec<String> =
                serde_json::from_str(&exports_str).unwrap_or_default();
            let deps_str: String = row.get::<_, String>(10).unwrap_or_default();
            let dependencies: Vec<String> =
                serde_json::from_str(&deps_str).unwrap_or_default();
            Ok(CachedRepoMetadata {
                repository: RepoIdentity {
                    id: row.get(0)?,
                    name: row.get(2)?,
                    uuid,
                },
                revision: row.get::<_, i64>(5)? as u64,
                repository_root: row.get(3)?,
                knowledge: schemas::manifest::KnowledgeLocation {
                    location: row.get(4)?,
                },
                exports,
                audit: row.get(7)?,
                last_sync: row.get(8)?,
                expires: row.get(9)?,
                dependencies,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get a single cached entry by UUID.
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Result<Option<CachedRepoMetadata>> {
        let conn = self.conn.lock().unwrap();
        let uuid_str = uuid.to_string();
        let mut stmt = conn.prepare(
            "SELECT id, uuid, name, repository_root, knowledge_db, revision, exports, audit, last_sync, expires, dependencies
             FROM repository_cache WHERE uuid = ?1",
        )?;
        let mut rows = stmt.query_map(params![uuid_str], |row| {
            let u: String = row.get(1)?;
            let uuid = Uuid::parse_str(&u).unwrap_or_default();
            let exports_str: String = row.get(6)?;
            let exports: Vec<String> =
                serde_json::from_str(&exports_str).unwrap_or_default();
            let deps_str: String = row.get::<_, String>(10).unwrap_or_default();
            let dependencies: Vec<String> =
                serde_json::from_str(&deps_str).unwrap_or_default();
            Ok(CachedRepoMetadata {
                repository: RepoIdentity {
                    id: row.get(0)?,
                    name: row.get(2)?,
                    uuid,
                },
                revision: row.get::<_, i64>(5)? as u64,
                repository_root: row.get(3)?,
                knowledge: schemas::manifest::KnowledgeLocation {
                    location: row.get(4)?,
                },
                exports,
                audit: row.get(7)?,
                last_sync: row.get(8)?,
                expires: row.get(9)?,
                dependencies,
            })
        })?;
        match rows.next() {
            Some(Ok(meta)) => Ok(Some(meta)),
            _ => Ok(None),
        }
    }

    /// Get a single cached entry by ID.
    pub fn get_by_id(&self, id: &str) -> Result<Option<CachedRepoMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, uuid, name, repository_root, knowledge_db, revision, exports, audit, last_sync, expires, dependencies
             FROM repository_cache WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            let uuid_str: String = row.get(1)?;
            let uuid = Uuid::parse_str(&uuid_str).unwrap_or_default();
            let exports_str: String = row.get(6)?;
            let exports: Vec<String> =
                serde_json::from_str(&exports_str).unwrap_or_default();
            let deps_str: String = row.get::<_, String>(10).unwrap_or_default();
            let dependencies: Vec<String> =
                serde_json::from_str(&deps_str).unwrap_or_default();
            Ok(CachedRepoMetadata {
                repository: RepoIdentity {
                    id: row.get(0)?,
                    name: row.get(2)?,
                    uuid,
                },
                revision: row.get::<_, i64>(5)? as u64,
                repository_root: row.get(3)?,
                knowledge: schemas::manifest::KnowledgeLocation {
                    location: row.get(4)?,
                },
                exports,
                audit: row.get(7)?,
                last_sync: row.get(8)?,
                expires: row.get(9)?,
                dependencies,
            })
        })?;
        match rows.next() {
            Some(Ok(meta)) => Ok(Some(meta)),
            _ => Ok(None),
        }
    }

    /// Write a cached metadata entry (upsert).
    pub fn cache_write(&self, meta: &CachedRepoMetadata) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let uuid_str = meta.repository.uuid.to_string();
        let exports_str = serde_json::to_string(&meta.exports)?;
        let deps_str = serde_json::to_string(&meta.dependencies)?;
        conn.execute(
            "INSERT OR REPLACE INTO repository_cache (id, uuid, name, repository_root, knowledge_db, revision, exports, audit, last_sync, expires, dependencies)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                meta.repository.id,
                uuid_str,
                meta.repository.name,
                meta.repository_root,
                meta.knowledge.location,
                meta.revision as i64,
                exports_str,
                meta.audit,
                meta.last_sync,
                meta.expires,
                deps_str,
            ],
        )?;
        Ok(())
    }

    /// Read a cached metadata entry by ID.
    pub fn cache_read(&self, id: &str) -> Result<Option<CachedRepoMetadata>> {
        self.get_by_id(id)
    }

    /// List all cached entry IDs.
    pub fn cache_list(&self) -> Result<Vec<(String, PathBuf)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM repository_cache ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok((id, PathBuf::new()))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Remove a cached entry by ID.
    pub fn cache_remove(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM repository_cache WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemas::manifest::KnowledgeLocation;

    fn test_db() -> RegistryDb {
        RegistryDb::open_in_memory().unwrap()
    }

    fn sample_meta(id: &str) -> CachedRepoMetadata {
        CachedRepoMetadata {
            repository: RepoIdentity {
                id: id.to_string(),
                name: format!("Repo-{}", id),
                uuid: Uuid::new_v4(),
            },
            revision: 1,
            repository_root: format!("/tmp/{}", id),
            knowledge: KnowledgeLocation {
                location: format!("/tmp/{}/.samgraha/knowledge.db", id),
            },
            exports: vec!["arch".to_string()],
            audit: "PASS".to_string(),
            last_sync: "2026-06-27T12:00:00Z".to_string(),
            expires: "2026-06-28T12:00:00Z".to_string(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn test_registry_db_write_read() {
        let db = test_db();
        let meta = sample_meta("test-repo");
        db.cache_write(&meta).unwrap();
        let loaded = db.cache_read("test-repo").unwrap().unwrap();
        assert_eq!(loaded.repository.id, "test-repo");
        assert_eq!(loaded.repository.uuid, meta.repository.uuid);
    }

    #[test]
    fn test_registry_db_list() {
        let db = test_db();
        db.cache_write(&sample_meta("a")).unwrap();
        db.cache_write(&sample_meta("b")).unwrap();
        let all = db.list().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_registry_db_unregister() {
        let db = test_db();
        let meta = sample_meta("unreg");
        let uuid = meta.repository.uuid;
        db.cache_write(&meta).unwrap();
        assert!(db.unregister(&uuid).unwrap());
        assert!(db.cache_read("unreg").unwrap().is_none());
    }

    #[test]
    fn test_registry_db_get_by_uuid() {
        let db = test_db();
        let meta = sample_meta("by-uuid");
        let uuid = meta.repository.uuid;
        db.cache_write(&meta).unwrap();
        let loaded = db.get_by_uuid(&uuid).unwrap().unwrap();
        assert_eq!(loaded.repository.id, "by-uuid");
    }

    #[test]
    fn test_registry_db_cache_remove() {
        let db = test_db();
        db.cache_write(&sample_meta("remove-me")).unwrap();
        db.cache_remove("remove-me").unwrap();
        assert!(db.cache_read("remove-me").unwrap().is_none());
    }
}
