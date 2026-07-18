-- System plans — stores the init script's phase-wise plan output.
-- Matches §8.4 of generic-script-architecture-proposal.md.
--
-- The plan JSON is static per system (same for every repo running it).
-- "Where is this repo right now" is computed on demand by joining
-- system_plans with script_runs, not stored here.

CREATE TABLE system_plans (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id     INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint TEXT   NOT NULL,  -- same convention as script_runs
    system_name     TEXT    NOT NULL,  -- e.g. "rust_dev"
    plan_json       TEXT    NOT NULL,  -- full init output JSON (§8.4 shape)
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(standard_id, repo_fingerprint, system_name)
);

CREATE INDEX idx_system_plans_lookup ON system_plans(standard_id, repo_fingerprint, system_name);
