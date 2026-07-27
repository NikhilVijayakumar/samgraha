-- knowledge.db — generic proposal tracking: opt-in, per standard. Links a
-- usecase to a template and an execution; tracks status (draft/final/
-- archived) and an optional output location. Samgraha never interprets
-- proposal content — it only records the lifecycle.

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
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
