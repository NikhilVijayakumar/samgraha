-- standards.db — operation log: one row per registration, update, seed,
-- or other significant operation. The `scope` column distinguishes
-- global operations (registering a standard globally) from repo-scoped
-- ones (seeding a standard into a specific repository). The `detail_json`
-- column carries operation-specific metadata as a JSON blob.

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
