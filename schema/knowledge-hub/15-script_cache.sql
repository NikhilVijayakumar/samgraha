-- Script cache: stores the last execution result per script check per repo
-- fingerprint. Used by the orchestrator to decide reuse-vs-rerun. One row
-- per (script_check, repo_fingerprint) pair — a new run overwrites the
-- previous row for that pair.

CREATE TABLE script_cache (
    id                INTEGER PRIMARY KEY,
    script_check_id   INTEGER NOT NULL REFERENCES script_checks(id) ON DELETE CASCADE,
    repo_fingerprint  TEXT NOT NULL,
    status            TEXT NOT NULL CHECK (status IN ('pass','fail','error','not_applicable')),
    executed_at       TEXT NOT NULL,
    result_json       TEXT NOT NULL,
    UNIQUE(script_check_id, repo_fingerprint)
);
