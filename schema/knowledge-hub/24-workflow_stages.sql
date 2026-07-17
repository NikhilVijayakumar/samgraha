-- Workflow stages: a standard's plan/core/loop.yaml `stages:` list, one row
-- per stage in declared order. Each real stage dict has exactly one key
-- (the stage type — "repository", "audit", "calculate", "aggregate",
-- "normalize", "validate", "report", ...) whose value is that stage's own
-- flat param dict (type, scope, parallel, source, on_failure, templates,
-- cap, description, ...). No fixed stage-type vocabulary here — same
-- reasoning `calculation_rules.bucket` has none — a standard's own
-- workflow shape, not samgraha's to enumerate. params_json is a plain JSON
-- blob rather than a child param-rows table (unlike rule_evidence_params):
-- every observed stage's params are flat scalars, nothing needs SQL to
-- query into one individually.

CREATE TABLE workflow_stages (
    id            INTEGER PRIMARY KEY,
    standard_id   INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    sort_order    INTEGER NOT NULL,
    stage_type    TEXT NOT NULL,
    params_json   TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_workflow_stages_standard ON workflow_stages(standard_id, sort_order);
