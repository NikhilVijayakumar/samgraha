//! `knowledge.db`'s samgraha-owned execution schema — mirrors
//! `schema/knowledge/*.sql` exactly. Samgraha creates and migrates every
//! table here. It never creates or migrates whatever tables a standard's
//! own scripts add to the same file — `custom_data_tables` only catalogs
//! that they exist.

use anyhow::Result;
use crate::migration::RESERVED_TABLE_NAMES;
use rusqlite::Connection;

/// Epoch counter — bumped by hand in code exactly when the
/// `RESERVED_TABLE_NAMES` table shape changes. `ensure_current_schema`
/// compares this against the stored epoch; a mismatch triggers a full
/// wipe-and-recreate of every reserved table. Most releases don't touch
/// this shape, so most releases leave existing standard-catalog data
/// intact.
pub const CORE_SCHEMA_EPOCH: i64 = 3;

/// Applied in order, same discipline as `REGISTRY_MIGRATIONS`/the old
/// `KNOWLEDGE_MIGRATIONS`: never edit a past entry, only add the next one.
pub const CORE_MIGRATIONS: &[&str] = &[CORE_V1, CORE_V2];

/// Version-gated migration runner for `knowledge.db`. Reads the current
/// version from `_schema_version` (created by `CORE_V1`) and applies only
/// migrations past that version. Safe to call on every `register_standard`
/// / `run_script_step` invocation — `CREATE TABLE IF NOT EXISTS` statements
/// are idempotent by construction, and `ALTER TABLE ADD COLUMN` statements
/// in later versions are gated by the version check, so a second call
/// against an already-migrated database never crashes on "duplicate column".
pub fn run_core_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(CORE_V1)?; // V1 is all IF NOT EXISTS — always safe
    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    // Apply migrations past V1 (V1 already applied above, idempotent)
    for (i, migration) in CORE_MIGRATIONS.iter().enumerate().skip(1) {
        let version = i as i64 + 1;
        if version > current_version {
            conn.execute_batch(migration)?;
            conn.execute(
                "INSERT INTO _schema_version (version, applied_at) VALUES (?1, datetime('now'))",
                rusqlite::params![version],
            )?;
        }
    }
    Ok(())
}

/// Ensure the database's reserved-table shape matches `CORE_SCHEMA_EPOCH`.
/// Called on every `knowledge.db` open (same call site `run_core_migrations`
/// already occupies). If the stored epoch differs from `CORE_SCHEMA_EPOCH`,
/// all `RESERVED_TABLE_NAMES` tables are dropped and recreated from scratch
/// — no in-place migration, no partial state.
pub fn ensure_current_schema(conn: &Connection) -> Result<()> {
    // Ensure the epoch-tracking table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _core_schema_epoch (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            epoch INTEGER NOT NULL DEFAULT 0
        );
        INSERT OR IGNORE INTO _core_schema_epoch (id, epoch) VALUES (1, 0);",
    )?;

    let stored_epoch: i64 = conn
        .query_row("SELECT epoch FROM _core_schema_epoch WHERE id = 1", [], |r| r.get(0))
        .unwrap_or(0);

    if stored_epoch != CORE_SCHEMA_EPOCH {
        reset_samgraha_tables(conn)?;
        // Re-create the epoch table (reset dropped it) and set the new epoch
        conn.execute_batch(
            "CREATE TABLE _core_schema_epoch (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                epoch INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO _core_schema_epoch (id, epoch) VALUES (1, 0);",
        )?;
        conn.execute(
            "UPDATE _core_schema_epoch SET epoch = ?1 WHERE id = 1",
            rusqlite::params![CORE_SCHEMA_EPOCH],
        )?;
    }

    // Always run incremental migrations on top (idempotent)
    run_core_migrations(conn)?;

    Ok(())
}

/// Drop and recreate every table in `RESERVED_TABLE_NAMES`. The DB must
/// have foreign keys disabled during this operation (DROP order matters).
/// After this call the DB is in a clean state — no standard-catalog data,
/// schema version set to the current CORE_MIGRATIONS level.
pub fn reset_samgraha_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
    for table in RESERVED_TABLE_NAMES {
        conn.execute_batch(&format!("DROP TABLE IF EXISTS {table};"))?;
    }
    // Also drop the epoch table so ensure_current_schema re-creates it
    conn.execute_batch("DROP TABLE IF EXISTS _core_schema_epoch;")?;
    // Drop any indexes that belong to reserved tables
    conn.execute_batch(
        "DROP INDEX IF EXISTS idx_execution_step_repo;
         DROP INDEX IF EXISTS idx_artifact_execution;",
    )?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(())
}

const CORE_V1: &str = "
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS usecase (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    data        TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(standard, name)
);

CREATE TABLE IF NOT EXISTS script (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    location TEXT    NOT NULL,
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);

CREATE TABLE IF NOT EXISTS prompt (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    purpose  TEXT    NOT NULL DEFAULT '',
    content  TEXT    NOT NULL,
    UNIQUE(standard, name)
);

CREATE TABLE IF NOT EXISTS step (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    usecase_id  INTEGER NOT NULL REFERENCES usecase(id) ON DELETE CASCADE,
    step_order  INTEGER NOT NULL,
    kind        TEXT    NOT NULL CHECK (kind IN ('deterministic','semantic')),
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(usecase_id, step_order)
);

CREATE TABLE IF NOT EXISTS step_script (
    step_id   INTEGER NOT NULL REFERENCES step(id) ON DELETE CASCADE,
    script_id INTEGER NOT NULL REFERENCES script(id) ON DELETE CASCADE,
    PRIMARY KEY (step_id, script_id)
);

CREATE TABLE IF NOT EXISTS step_prompt (
    step_id   INTEGER NOT NULL REFERENCES step(id) ON DELETE CASCADE,
    prompt_id INTEGER NOT NULL REFERENCES prompt(id) ON DELETE CASCADE,
    PRIMARY KEY (step_id, prompt_id)
);

CREATE TABLE IF NOT EXISTS execution (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    step_id    INTEGER NOT NULL REFERENCES step(id) ON DELETE CASCADE,
    repo_root  TEXT    NOT NULL,
    status     TEXT    NOT NULL DEFAULT 'ok',
    timestamp  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_execution_step_repo ON execution(step_id, repo_root);

CREATE TABLE IF NOT EXISTS custom_data_tables (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    standard        TEXT    NOT NULL,
    table_name      TEXT    NOT NULL,
    purpose         TEXT    NOT NULL DEFAULT '',
    owner_script_id INTEGER REFERENCES script(id) ON DELETE SET NULL,
    shape_json      TEXT,
    UNIQUE(standard, table_name)
);
";

/// CORE_V2 — additive schema for knowledge standard management. Adds
/// `git_detail`, `domain`, `standard_asset`, `template`, `proposal`,
/// `artifact` tables plus `execution.git_detail_id` and `usecase.domain_id`
/// columns. All `CREATE TABLE IF NOT EXISTS` except the two
/// `ALTER TABLE ADD COLUMN` statements, which are version-gated by
/// `run_core_migrations`.
///
/// Previously also created a `standard` table (per-repo catalog metadata:
/// name/category/subcategory/extends/version). Removed — one standard is
/// active per repo at a time now, so that fact lives in registry.db's
/// `active_standard` table instead (`schema/registration/01-active_standard.sql`),
/// not duplicated inside `knowledge.db`'s workflow-data schema. This is the
/// shape change `CORE_SCHEMA_EPOCH` bumped from 1 to 2 for.
///
/// `standard_asset.kind`, `template.type`, and `artifact.type` were free
/// `TEXT` columns — no relational integrity, any string was accepted.
/// Bumped to epoch 3: each now has its own per-standard lookup table
/// (`asset_kind`, `template_type`, `artifact_type` — same shape and
/// `UNIQUE(standard, name)` pattern `domain` already uses for
/// `usecase.domain_id`) and a `*_id` foreign key instead of free text.
/// `asset_kind`/`template_type` rows are declared by a standard's own
/// seeder before it references them (same responsibility as `domain`) —
/// declaring, then referencing, in one script run, no ordering hazard.
/// `artifact_type` is different: artifact rows come from a script's
/// *runtime* output (`run_script_step`'s `result.artifacts[]`), whose
/// vocabulary can't be predicted at registration time, so samgraha itself
/// find-or-creates the `artifact_type` row on demand
/// (`register_standard::get_or_create_lookup`) rather than requiring a
/// seeder to pre-declare every possible output type — failing a step's
/// whole execution over a missing catalog row would be a worse failure
/// mode than growing the catalog as new types show up.
const CORE_V2: &str = "
-- §2.10 — git provenance
CREATE TABLE IF NOT EXISTS git_detail (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_root   TEXT    NOT NULL,
    commit_sha  TEXT    NOT NULL,
    branch      TEXT    NOT NULL DEFAULT '',
    dirty       INTEGER NOT NULL DEFAULT 0,
    captured_at TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(repo_root, commit_sha, dirty)
);
ALTER TABLE execution ADD COLUMN git_detail_id INTEGER REFERENCES git_detail(id);

-- §2.11 — discovery-only domain mirror
CREATE TABLE IF NOT EXISTS domain (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    key         TEXT    NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, key)
);
ALTER TABLE usecase ADD COLUMN domain_id INTEGER REFERENCES domain(id);

-- Per-standard catalog of valid standard_asset.kind values (e.g. config,
-- guide, plan) — declared by the standard's own seeder before it inserts
-- standard_asset rows referencing kind_id, same pattern as domain.
CREATE TABLE IF NOT EXISTS asset_kind (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);

-- §2.9 — standard-shipped content catalog (plan/guide/config)
CREATE TABLE IF NOT EXISTS standard_asset (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    kind_id  INTEGER NOT NULL REFERENCES asset_kind(id),
    name     TEXT    NOT NULL,
    location TEXT    NOT NULL,
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, kind_id, name)
);

-- Per-standard catalog of valid template.type values (e.g. markdown,
-- html, email) — declared by the standard's own seeder before it inserts
-- template rows referencing type_id, same pattern as domain.
CREATE TABLE IF NOT EXISTS template_type (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);

-- §2.7 — generic rendering template catalog (opt-in, per standard)
CREATE TABLE IF NOT EXISTS template (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    type_id  INTEGER NOT NULL REFERENCES template_type(id),
    content  TEXT    NOT NULL,
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);

-- §2.8 — generic proposal tracking (opt-in, per standard)
CREATE TABLE IF NOT EXISTS proposal (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    standard     TEXT    NOT NULL,
    usecase_id   INTEGER NOT NULL REFERENCES usecase(id) ON DELETE CASCADE,
    template_id  INTEGER REFERENCES template(id) ON DELETE SET NULL,
    execution_id INTEGER REFERENCES execution(id) ON DELETE SET NULL,
    title        TEXT    NOT NULL,
    status       TEXT    NOT NULL DEFAULT 'draft'
                 CHECK (status IN ('draft','final','archived')),
    location     TEXT,
    metadata_json TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Per-standard catalog of artifact.type values (e.g. image, diagram,
-- dataset, model). Unlike asset_kind/template_type, rows here are not
-- declared upfront by a seeder — they're find-or-created by samgraha
-- itself (register_standard::get_or_create_lookup) whenever a script's
-- result envelope reports an artifact type not seen before for this
-- standard. Never deleted by delete_existing — artifacts and their types
-- are a historical output record that survives a standard re-registering.
CREATE TABLE IF NOT EXISTS artifact_type (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);

-- §2.9 — produced output tracking (opt-in, per standard)
CREATE TABLE IF NOT EXISTS artifact (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    standard     TEXT    NOT NULL,
    execution_id INTEGER REFERENCES execution(id) ON DELETE SET NULL,
    type_id      INTEGER NOT NULL REFERENCES artifact_type(id),
    name         TEXT    NOT NULL,
    location     TEXT    NOT NULL,
    purpose      TEXT    NOT NULL DEFAULT '',
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_artifact_execution ON artifact(execution_id);
";

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn run_migrations(conn: &Connection) {
        run_core_migrations(conn).unwrap();
    }

    #[test]
    fn core_migrations_apply_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn);
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for expected in [
            "usecase", "script", "prompt", "step", "step_script",
            "step_prompt", "execution", "custom_data_tables",
            "git_detail", "domain", "asset_kind", "standard_asset",
            "template_type", "template", "proposal",
            "artifact_type", "artifact",
        ] {
            assert!(tables.contains(&expected.to_string()), "missing table {expected}");
        }
        assert!(!tables.contains(&"standard".to_string()), "'standard' should no longer be a knowledge.db table — it lives in registry.db's active_standard table now");
    }

    #[test]
    fn step_kind_check_rejects_invalid_value() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn);
        conn.execute(
            "INSERT INTO usecase (standard, name) VALUES ('test-std', 'uc1')",
            [],
        )
        .unwrap();
        let err = conn.execute(
            "INSERT INTO step (usecase_id, step_order, kind) VALUES (1, 1, 'bogus')",
            [],
        );
        assert!(err.is_err(), "expected CHECK constraint to reject invalid kind");
    }

    #[test]
    fn usecase_name_unique_per_standard() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn);
        conn.execute(
            "INSERT INTO usecase (standard, name) VALUES ('std-a', 'scoring')",
            [],
        )
        .unwrap();
        // Same name, different standard — allowed.
        conn.execute(
            "INSERT INTO usecase (standard, name) VALUES ('std-b', 'scoring')",
            [],
        )
        .unwrap();
        // Same name, same standard — rejected.
        let err = conn.execute(
            "INSERT INTO usecase (standard, name) VALUES ('std-a', 'scoring')",
            [],
        );
        assert!(err.is_err());
    }

    #[test]
    fn custom_data_tables_owner_script_nullable_on_delete() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn);
        conn.execute(
            "INSERT INTO script (standard, name, location) VALUES ('std-a', 'db-setup', 'script/db.py')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_data_tables (standard, table_name, owner_script_id) \
             VALUES ('std-a', 'hackathon_scores', 1)",
            [],
        )
        .unwrap();
        conn.execute("DELETE FROM script WHERE id = 1", []).unwrap();
        let owner: Option<i64> = conn
            .query_row(
                "SELECT owner_script_id FROM custom_data_tables WHERE table_name = 'hackathon_scores'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(owner.is_none(), "owner_script_id should be nulled, not block delete");
    }

    #[test]
    fn run_core_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_core_migrations(&conn).unwrap();
        // Second call must not crash (ALTER TABLE ADD COLUMN gated by version check)
        run_core_migrations(&conn).unwrap();
        // Version should be 2 (both V1 and V2 applied once)
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM _schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn ensure_current_schema_creates_epoch_table() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_current_schema(&conn).unwrap();
        let epoch: i64 = conn
            .query_row("SELECT epoch FROM _core_schema_epoch WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(epoch, CORE_SCHEMA_EPOCH);
    }

    #[test]
    fn ensure_current_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_current_schema(&conn).unwrap();
        // Insert some data
        conn.execute(
            "INSERT INTO usecase (standard, name) VALUES ('test', 'uc1')",
            [],
        )
        .unwrap();
        // Second call with same epoch should NOT wipe data
        ensure_current_schema(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM usecase", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "same epoch should not reset tables");
    }

    #[test]
    fn reset_samgraha_tables_drops_all_reserved_tables() {
        let conn = Connection::open_in_memory().unwrap();
        run_core_migrations(&conn).unwrap();
        conn.execute(
            "INSERT INTO usecase (standard, name) VALUES ('test', 'uc1')",
            [],
        )
        .unwrap();
        reset_samgraha_tables(&conn).unwrap();
        // Tables should not exist after reset
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='usecase'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(exists, 0, "usecase table should be dropped after reset");
    }
}
