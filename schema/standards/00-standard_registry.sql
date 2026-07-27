-- standards.db — global, cross-repo standard registry. Lives at
-- `~/.samgraha/standards.db` (via `common::env::mcp_dir()`). Each row
-- represents a standard that has been registered globally — the source
-- path now points at a local copy under `mcp_dir()/registry/`, never at
-- an external repo (§3.7). The `verify_status` field tracks whether the
-- standard passed its structural verify-gate (smoke_test).

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
