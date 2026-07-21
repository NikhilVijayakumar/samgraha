-- knowledge.db — execution log: one row per time a step actually ran.
-- Added two columns beyond the original (id, step_id, timestamp) spec:
-- `repo_root` and `status`. Reasoning: a usecase/step is defined once per
-- standard, but executed per target repo (python_hackathon scoring N
-- participant repos from Heimdall is the concrete case this came up in)
-- — without recording which repo a run was against, "has this step run"
-- is unanswerable for more than one target. `status` records whether the
-- run's script/agent-reported result succeeded, so a failed run doesn't
-- read as "already done." Flagging both as additions, not silently
-- assumed — the rest of the table is exactly as specified.

CREATE TABLE IF NOT EXISTS execution (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    step_id    INTEGER NOT NULL REFERENCES step(id) ON DELETE CASCADE,
    repo_root  TEXT    NOT NULL,
    status     TEXT    NOT NULL DEFAULT 'ok',
    timestamp  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_execution_step_repo ON execution(step_id, repo_root);
