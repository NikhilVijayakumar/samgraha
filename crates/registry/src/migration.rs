//! `registry.db` migrations (repository registration — unchanged from
//! before this pivot) plus the reserved-name list a standard's own custom
//! tables must not collide with in `knowledge.db`.
//!
//! The old `KNOWLEDGE_MIGRATIONS` (V1-V36, the doc-audit/report schema —
//! `documents`, `relationships`, `pipeline_reports`, etc.) is archived at
//! `archive/crates/registry-legacy/migration.rs`. `knowledge.db`'s live
//! schema is now `crate::core_schema::CORE_MIGRATIONS`.

/// Table names reserved by samgraha's own `knowledge.db` schema
/// (`core_schema::CORE_MIGRATIONS`). A standard's `custom_data_tables`
/// entries must not collide with any name here — enforced at
/// `register_standard` time.
pub const RESERVED_TABLE_NAMES: &[&str] = &[
    "_schema_version",
    "usecase",
    "script",
    "prompt",
    "step",
    "step_script",
    "step_prompt",
    "execution",
    "custom_data_tables",
];

pub const REGISTRY_MIGRATIONS: &[&str] = &[REG_V1, REG_V2];

/// REG_V1 — repository registry tables for `.samgraha/registry.db`.
/// Stores cached dependency metadata in a single `repository_cache` table,
/// indexed by UUID for fast lookup during dependency resolution.
/// The cache is disposable — fully rebuildable from dependency manifests.
const REG_V1: &str = "
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS repository_cache (
    id TEXT PRIMARY KEY,
    uuid TEXT NOT NULL,
    name TEXT NOT NULL,
    repository_root TEXT NOT NULL,
    knowledge_db TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0,
    exports TEXT NOT NULL DEFAULT '[]',
    audit TEXT NOT NULL DEFAULT 'PASS',
    last_sync TEXT NOT NULL,
    expires TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_repo_cache_uuid ON repository_cache(uuid);
";

/// REG_V2 — add `dependencies` column for cached transitive dependency
/// names. Enables offline resolution without reading dependency manifests
/// on cache hit.
const REG_V2: &str = "
ALTER TABLE repository_cache ADD COLUMN dependencies TEXT NOT NULL DEFAULT '[]';
";

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn registry_migrations_apply_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        for m in REGISTRY_MIGRATIONS {
            conn.execute_batch(m).unwrap();
        }
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='repository_cache'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1);
    }

    #[test]
    fn reserved_table_names_covers_every_core_table() {
        for expected in [
            "usecase", "script", "prompt", "step", "step_script",
            "step_prompt", "execution", "custom_data_tables",
        ] {
            assert!(
                RESERVED_TABLE_NAMES.contains(&expected),
                "RESERVED_TABLE_NAMES missing '{expected}' — a standard could pick this name"
            );
        }
    }
}
