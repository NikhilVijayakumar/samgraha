-- knowledge.db — produced output tracking: opt-in, per standard. Records
-- artifacts produced by a script step's execution (the `artifacts[]` field
-- in the JSON envelope). Each artifact's type is a relation into
-- artifact_type (16-artifact_type.sql), not free text — but unlike
-- asset_kind/template_type, samgraha find-or-creates the type row itself
-- rather than requiring it pre-declared (§16). Location is absolute and
-- expected to live under `<samgraha_dir>/output/` (§3.12) but samgraha
-- does not enforce this — it only records where the script said it wrote.

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
