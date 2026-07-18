-- Workflow phases: one row per phase in a use case's init plan (§8.4).
--
-- Replaces the embedded phases array that system_plans.plan_json used to
-- hold as a JSON blob. Dependencies are real edges in
-- workflow_phase_dependencies (§2.1 of schema-redesign-proposal.md), not
-- an embedded depends_on array.

CREATE TABLE workflow_phases (
    id                INTEGER PRIMARY KEY,
    use_case_id       INTEGER NOT NULL REFERENCES workflow_use_cases(id) ON DELETE CASCADE,
    phase_id          TEXT    NOT NULL,  -- e.g. "tier1-generate"
    sort_order        INTEGER NOT NULL DEFAULT 0,  -- preserves original JSON array order
    kind              TEXT    NOT NULL CHECK (kind IN ('semantic','script')),
    description       TEXT,
    script_path       TEXT,              -- NULL for kind='semantic'
    pre_script        TEXT,
    post_script       TEXT,
    expiry_rule_json  TEXT,              -- same shape as script_runs.expiry_rule_json; NULL = never expires
    UNIQUE(use_case_id, phase_id)
);
