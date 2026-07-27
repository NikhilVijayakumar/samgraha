//! `standards.db` — global, cross-repo standard registry. Lives at
//! `~/.samgraha/standards.db` (via `common::env::mcp_dir()`). Contains
//! `standard_registry` (§2.1) and `operation_log` (§8.1). Mirrors
//! `RegistryDb`'s open/migration-versioning pattern exactly.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::sync::Mutex;
use tracing::info;

const STANDARDS_MIGRATIONS: &[&str] = &[STD_V1];

/// STD_V1 — creates `standard_registry` (§2.1) and `operation_log` (§8.1).
const STD_V1: &str = "
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS standard_registry (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT    NOT NULL UNIQUE,
    category       TEXT    NOT NULL,
    subcategory    TEXT,
    source_path    TEXT    NOT NULL,
    is_abstract    INTEGER NOT NULL DEFAULT 0,
    extends        TEXT,
    version        TEXT    NOT NULL DEFAULT '0.0.0',
    description    TEXT    NOT NULL DEFAULT '',
    metadata_json  TEXT    NOT NULL DEFAULT '{}',
    verify_status  TEXT    NOT NULL DEFAULT 'unverified'
                   CHECK (verify_status IN ('unverified','passed','failed')),
    verified_at    TEXT,
    registered_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_standard_registry_category ON standard_registry(category, subcategory);

CREATE TABLE IF NOT EXISTS operation_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    operation   TEXT    NOT NULL,
    standard    TEXT    NOT NULL,
    repo_root   TEXT,
    scope       TEXT    NOT NULL DEFAULT 'global' CHECK (scope IN ('global','repo')),
    status      TEXT    NOT NULL,
    detail_json TEXT    NOT NULL DEFAULT '{}',
    occurred_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
";

pub struct StandardsDb {
    conn: Mutex<Connection>,
}

impl StandardsDb {
    /// Open or create the global standards database at `~/.samgraha/standards.db`.
    pub fn open() -> Result<Self> {
        let path = common::env::mcp_dir().join("standards.db");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open standards db at {}", path.display()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = Self { conn: Mutex::new(conn) };
        store.run_migrations()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = Self { conn: Mutex::new(conn) };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        for (i, migration) in STANDARDS_MIGRATIONS.iter().enumerate() {
            let version = i as i64 + 1;
            if version > current_version {
                info!("Applying standards migration STD_V{}", version);
                conn.execute_batch(migration)?;
                conn.execute(
                    "INSERT INTO _schema_version (version, applied_at) VALUES (?1, datetime('now'))",
                    params![version],
                )?;
            }
        }
        Ok(())
    }

    /// Upsert a standard into the global registry (§2.1).
    pub fn upsert_standard(
        &self,
        name: &str,
        category: &str,
        subcategory: Option<&str>,
        source_path: &str,
        is_abstract: bool,
        extends: Option<&str>,
        version: &str,
        description: &str,
        metadata_json: &str,
        verify_status: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO standard_registry (name, category, subcategory, source_path, is_abstract, extends, version, description, metadata_json, verify_status, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
             ON CONFLICT(name) DO UPDATE SET
                category = excluded.category,
                subcategory = excluded.subcategory,
                source_path = excluded.source_path,
                is_abstract = excluded.is_abstract,
                extends = excluded.extends,
                version = excluded.version,
                description = excluded.description,
                metadata_json = excluded.metadata_json,
                verify_status = excluded.verify_status,
                updated_at = datetime('now')",
            params![name, category, subcategory, source_path, is_abstract as i64, extends, version, description, metadata_json, verify_status],
        )?;
        Ok(())
    }

    /// Get a standard by name from the global registry.
    pub fn get_standard(&self, name: &str) -> Result<Option<StandardRegistryRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, category, subcategory, source_path, is_abstract, extends, version, description, metadata_json, verify_status, verified_at, registered_at, updated_at
             FROM standard_registry WHERE name = ?1",
        )?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(StandardRegistryRow {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                subcategory: row.get(3)?,
                source_path: row.get(4)?,
                is_abstract: row.get::<_, i64>(5)? != 0,
                extends: row.get(6)?,
                version: row.get(7)?,
                description: row.get(8)?,
                metadata_json: row.get(9)?,
                verify_status: row.get(10)?,
                verified_at: row.get(11)?,
                registered_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?;
        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            _ => Ok(None),
        }
    }

    /// List standards, optionally filtered by category/subcategory.
    pub fn list_standards(
        &self,
        category: Option<&str>,
        subcategory: Option<&str>,
    ) -> Result<Vec<StandardRegistryRow>> {
        let conn = self.conn.lock().unwrap();
        let (sql, param_values): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match (category, subcategory) {
            (Some(cat), Some(sub)) => (
                "SELECT id, name, category, subcategory, source_path, is_abstract, extends, version, description, metadata_json, verify_status, verified_at, registered_at, updated_at
                 FROM standard_registry WHERE category = ?1 AND subcategory = ?2 ORDER BY name",
                vec![Box::new(cat.to_string()), Box::new(sub.to_string())],
            ),
            (Some(cat), None) => (
                "SELECT id, name, category, subcategory, source_path, is_abstract, extends, version, description, metadata_json, verify_status, verified_at, registered_at, updated_at
                 FROM standard_registry WHERE category = ?1 ORDER BY name",
                vec![Box::new(cat.to_string())],
            ),
            _ => (
                "SELECT id, name, category, subcategory, source_path, is_abstract, extends, version, description, metadata_json, verify_status, verified_at, registered_at, updated_at
                 FROM standard_registry ORDER BY name",
                vec![],
            ),
        };
        let mut stmt = conn.prepare(sql)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(StandardRegistryRow {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                subcategory: row.get(3)?,
                source_path: row.get(4)?,
                is_abstract: row.get::<_, i64>(5)? != 0,
                extends: row.get(6)?,
                version: row.get(7)?,
                description: row.get(8)?,
                metadata_json: row.get(9)?,
                verify_status: row.get(10)?,
                verified_at: row.get(11)?,
                registered_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Update verify_status and verified_at for a standard.
    pub fn set_verify_status(&self, name: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE standard_registry SET verify_status = ?1, verified_at = datetime('now'), updated_at = datetime('now') WHERE name = ?2",
            params![status, name],
        )?;
        Ok(())
    }

    /// Log an operation (§8.1).
    pub fn log_operation(
        &self,
        operation: &str,
        standard: &str,
        repo_root: Option<&str>,
        scope: &str,
        status: &str,
        detail_json: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO operation_log (operation, standard, repo_root, scope, status, detail_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![operation, standard, repo_root, scope, status, detail_json],
        )?;
        Ok(())
    }

    /// Full wipe of `standards.db`: drops all tables, re-runs migrations,
    /// and deletes the `mcp_dir()/registry/` file tree (§3.1/§3.7).
    /// Used by Phase 2's `ensure_current_schema` equivalent for
    /// `standards.db` — a complete reset, no partial state.
    pub fn reset(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
        conn.execute_batch("DROP TABLE IF EXISTS operation_log;")?;
        conn.execute_batch("DROP TABLE IF EXISTS standard_registry;")?;
        conn.execute_batch("DROP TABLE IF EXISTS _schema_version;")?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        drop(conn);
        // Delete the mcp-registry file tree (§3.7)
        let registry_dir = common::env::mcp_dir().join("registry");
        if registry_dir.exists() {
            std::fs::remove_dir_all(&registry_dir)?;
        }
        // Re-run migrations to recreate tables
        self.run_migrations()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StandardRegistryRow {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub source_path: String,
    pub is_abstract: bool,
    pub extends: Option<String>,
    pub version: String,
    pub description: String,
    pub metadata_json: String,
    pub verify_status: String,
    pub verified_at: Option<String>,
    pub registered_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> StandardsDb {
        StandardsDb::open_in_memory().unwrap()
    }

    #[test]
    fn upsert_and_get_standard() {
        let db = test_db();
        db.upsert_standard(
            "pcems_2026", "academic", None, "/tmp/pcems_2026",
            false, None, "1.0.0", "A test standard", "{}", "unverified",
        ).unwrap();
        let row = db.get_standard("pcems_2026").unwrap().unwrap();
        assert_eq!(row.name, "pcems_2026");
        assert_eq!(row.category, "academic");
        assert_eq!(row.version, "1.0.0");
        assert_eq!(row.verify_status, "unverified");
    }

    #[test]
    fn upsert_overwrites_existing() {
        let db = test_db();
        db.upsert_standard(
            "std-a", "dev", None, "/tmp/a",
            false, None, "0.1.0", "v1", "{}", "unverified",
        ).unwrap();
        db.upsert_standard(
            "std-a", "dev", Some("tools"), "/tmp/a2",
            true, None, "0.2.0", "v2", "{}", "passed",
        ).unwrap();
        let row = db.get_standard("std-a").unwrap().unwrap();
        assert_eq!(row.version, "0.2.0");
        assert!(row.is_abstract);
        assert_eq!(row.subcategory.as_deref(), Some("tools"));
    }

    #[test]
    fn list_standards_filters_by_category() {
        let db = test_db();
        db.upsert_standard("a", "dev", None, "/a", false, None, "0.0.0", "", "{}", "unverified").unwrap();
        db.upsert_standard("b", "academic", None, "/b", false, None, "0.0.0", "", "{}", "unverified").unwrap();
        db.upsert_standard("c", "dev", Some("tools"), "/c", false, None, "0.0.0", "", "{}", "unverified").unwrap();

        let dev = db.list_standards(Some("dev"), None).unwrap();
        assert_eq!(dev.len(), 2);

        let dev_tools = db.list_standards(Some("dev"), Some("tools")).unwrap();
        assert_eq!(dev_tools.len(), 1);
        assert_eq!(dev_tools[0].name, "c");

        let all = db.list_standards(None, None).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn set_verify_status() {
        let db = test_db();
        db.upsert_standard(
            "std-v", "dev", None, "/v", false, None, "0.0.0", "", "{}", "unverified",
        ).unwrap();
        db.set_verify_status("std-v", "passed").unwrap();
        let row = db.get_standard("std-v").unwrap().unwrap();
        assert_eq!(row.verify_status, "passed");
        assert!(row.verified_at.is_some());
    }

    #[test]
    fn log_operation_inserts_row() {
        let db = test_db();
        db.log_operation("register_globally", "std-a", None, "global", "ok", "{}").unwrap();
        db.log_operation("seed", "std-a", Some("/repo"), "repo", "ok", "{}").unwrap();
        // Verify rows exist by reading them back (no public list method, but we can count)
        let conn = db.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM operation_log", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn migrations_idempotent() {
        let db = test_db();
        // Second open against same in-memory DB is a fresh DB, but
        // calling open_in_memory twice proves the migration runner doesn't crash
        let db2 = StandardsDb::open_in_memory().unwrap();
        db2.upsert_standard(
            "x", "dev", None, "/x", false, None, "0.0.0", "", "{}", "unverified",
        ).unwrap();
        // Original db still works
        db.upsert_standard(
            "y", "dev", None, "/y", false, None, "0.0.0", "", "{}", "unverified",
        ).unwrap();
    }
}
