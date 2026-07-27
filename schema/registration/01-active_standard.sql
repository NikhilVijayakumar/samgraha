-- registry.db — which knowledge standard (and version) is currently
-- activated in this repo's `.samgraha/`. One standard is active per repo
-- at a time — switching standards means deleting `.samgraha/` and
-- re-registering, so a single-row (`id = 1`) table is enough, same
-- singleton pattern knowledge.db's own `_core_schema_epoch` table uses.
-- Mirrors registry::migration::REG_V3 exactly. Replaces knowledge.db's old
-- per-repo `standard` table (removed — see core_schema.rs's
-- CORE_SCHEMA_EPOCH bump to 2): this fact belongs with repo registration
-- bookkeeping, not inside the workflow-data schema a standard's own seeder
-- writes into.

CREATE TABLE IF NOT EXISTS active_standard (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    name          TEXT    NOT NULL,
    category      TEXT    NOT NULL DEFAULT '',
    subcategory   TEXT,
    extends       TEXT,
    version       TEXT    NOT NULL DEFAULT '0.0.0',
    metadata_json TEXT    NOT NULL DEFAULT '{}',
    activated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
