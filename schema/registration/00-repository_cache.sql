-- registry.db — repository registration cache. Unchanged from the
-- existing implementation (crates/registry/src/migration.rs's REG_V1/
-- REG_V2, REGISTRY_MIGRATIONS) — mirrored here as the canonical reference
-- copy, not a new schema. Disposable/rebuildable from dependency
-- manifests, not authoritative data.

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
    expires TEXT NOT NULL,
    dependencies TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_repo_cache_uuid ON repository_cache(uuid);
