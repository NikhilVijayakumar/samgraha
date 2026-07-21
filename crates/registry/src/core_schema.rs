//! `knowledge.db`'s samgraha-owned execution schema — mirrors
//! `schema/knowledge/*.sql` exactly. Samgraha creates and migrates every
//! table here. It never creates or migrates whatever tables a standard's
//! own scripts add to the same file — `custom_data_tables` only catalogs
//! that they exist.

/// Applied in order, same discipline as `REGISTRY_MIGRATIONS`/the old
/// `KNOWLEDGE_MIGRATIONS`: never edit a past entry, only add the next one.
pub const CORE_MIGRATIONS: &[&str] = &[CORE_V1];

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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn run_migrations(conn: &Connection) {
        for m in CORE_MIGRATIONS {
            conn.execute_batch(m).unwrap();
        }
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
        ] {
            assert!(tables.contains(&expected.to_string()), "missing table {expected}");
        }
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
}
