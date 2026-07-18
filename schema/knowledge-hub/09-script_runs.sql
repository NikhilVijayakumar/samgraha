-- Run-tracking + expiry — one row per phase/check execution.
-- Matches §8.5 of generic-script-architecture-proposal.md exactly.
--
-- validity check (generic, samgraha-side):
--   expiry_rule_json NULL  → always valid
--   type: "ttl"           → valid while now < expires_at
--   type: "head_commit"   → valid while current repo HEAD == head_commit_at_run
--
-- Re-running the same phase upserts (ON CONFLICT): the new run replaces
-- the old one, matching §7.4's idempotent/backfill principle.

CREATE TABLE script_runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id         INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint    TEXT    NOT NULL,  -- {check_name}-{repo_root} convention
    capability          TEXT    NOT NULL,  -- validate|calculate|report|scaffold|plan-generation|init|<future>
    phase_or_check_key  TEXT    NOT NULL,  -- init's phase id, or an existing check_name
    ran_at              TEXT    NOT NULL DEFAULT (datetime('now')),  -- ISO8601
    expiry_rule_json    TEXT,              -- NULL = never expires; else e.g.
                                           -- {"type":"ttl","seconds":86400}
                                           -- or {"type":"head_commit"}
    expires_at          TEXT,              -- precomputed for ttl-type rules; NULL otherwise
    head_commit_at_run  TEXT,              -- git HEAD sha at run time, for head_commit-type rules
    UNIQUE(standard_id, repo_fingerprint, capability, phase_or_check_key)
);

CREATE INDEX idx_script_runs_validity ON script_runs(standard_id, repo_fingerprint, capability);
CREATE INDEX idx_script_runs_expires ON script_runs(expires_at) WHERE expires_at IS NOT NULL;
