-- Workflow phases: one row per phase in a use case's init plan (§8.4).
--
-- Replaces the embedded phases array that system_plans.plan_json used to
-- hold as a JSON blob. Dependencies are real edges in
-- workflow_phase_dependencies (§2.1 of schema-redesign-proposal.md), not
-- an embedded depends_on array.
--
-- script_name / pre_script / post_script hold catalog names resolved
-- against standard_assets at dispatch time, not raw filesystem paths.
-- instruction is the catalog name of a prompt file (kind='prompt' in
-- standard_assets) handed verbatim to the calling agent for semantic steps.

CREATE TABLE workflow_phases (
    id                INTEGER PRIMARY KEY,
    use_case_id       INTEGER NOT NULL REFERENCES workflow_use_cases(id) ON DELETE CASCADE,
    phase_id          TEXT    NOT NULL,  -- e.g. "tier1-generate"
    sort_order        INTEGER NOT NULL DEFAULT 0,  -- preserves original JSON array order
    kind              TEXT    NOT NULL CHECK (kind IN ('semantic','script')),
    description       TEXT,
    script_name       TEXT,              -- catalog name resolved via standard_assets; NULL for kind='semantic' without a script
    pre_script        TEXT,              -- catalog name, optional
    post_script       TEXT,              -- catalog name, optional
    instruction       TEXT,              -- catalog name of prompt (kind='prompt'); for semantic steps only
    expiry_rule_json  TEXT,              -- same shape as script_runs.expiry_rule_json; NULL = never expires
    UNIQUE(use_case_id, phase_id)
);
