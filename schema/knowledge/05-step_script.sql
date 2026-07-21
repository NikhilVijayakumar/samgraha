-- knowledge.db — maps a deterministic step to the script it runs. One
-- row per step; step.kind must be 'deterministic' for a row to exist
-- here (enforced in application code, not a CHECK constraint, since
-- SQLite can't cross-reference another table's column in a CHECK).

CREATE TABLE IF NOT EXISTS step_script (
    step_id   INTEGER NOT NULL REFERENCES step(id) ON DELETE CASCADE,
    script_id INTEGER NOT NULL REFERENCES script(id) ON DELETE CASCADE,
    PRIMARY KEY (step_id, script_id)
);
