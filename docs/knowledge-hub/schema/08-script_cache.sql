-- Script cache: stores the last execution result per check per repo.
-- Used by the orchestrator to decide reuse-vs-rerun (§2e of proposal.md).
-- One row per (check, repo_fingerprint) pair — a new run overwrites the
-- previous row for that pair.

CREATE TABLE script_cache (
    id              INTEGER PRIMARY KEY,
    check_name      TEXT NOT NULL,
    domain          TEXT NOT NULL,
    repo_fingerprint TEXT NOT NULL,
    status          TEXT NOT NULL CHECK (status IN ('pass','fail','error','not_applicable')),
    executed_at     TEXT NOT NULL,
    result_json     TEXT NOT NULL,            -- full JSON output from the script
    UNIQUE(check_name, repo_fingerprint)
);
