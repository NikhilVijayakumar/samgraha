-- Workflow use cases: one row per use case a system's init plan declares
-- (§8.4 of generic-script-architecture-proposal.md).
--
-- Scoped by standard_id — each system's init plan declares its own use
-- cases. Phases live in workflow_phases (child table), not here.

CREATE TABLE workflow_use_cases (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    use_case_id  TEXT    NOT NULL,  -- e.g. "repo_new-case_1_no_documentation"
    label        TEXT    NOT NULL,
    UNIQUE(standard_id, use_case_id)
);
