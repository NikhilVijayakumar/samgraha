-- Plan-generation semantic input — stores the LLM's determination of
-- "what needs generating for this repo, in this workflow."
-- Matches §8.3 of generic-script-architecture-proposal.md exactly.
--
-- Re-running the semantic-determination step upserts (ON CONFLICT):
-- the current input_json shifts into previous_input_json before
-- overwriting, giving a one-generation-back hedge for iteration.

CREATE TABLE plan_generation_inputs (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id      INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint TEXT    NOT NULL,  -- same convention as script_runs
    workflow_id      TEXT    NOT NULL,  -- matches an id in init's phase plan (§8.4)
    domain_key       TEXT,              -- NULL = plan-level, not domain-specific
    instance_key     TEXT,              -- NULL unless domain is multi-instance (§4 item 7)
    input_json       TEXT    NOT NULL,  -- the semantic determination's own output, opaque to samgraha
    previous_input_json TEXT,           -- prior input_json, shifted here by the upsert
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),  -- ISO8601
    UNIQUE(standard_id, repo_fingerprint, workflow_id, domain_key, instance_key)
);

CREATE INDEX idx_plan_gen_inputs_lookup ON plan_generation_inputs(standard_id, repo_fingerprint, workflow_id);
