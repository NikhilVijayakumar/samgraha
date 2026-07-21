-- Workflow phase dependencies: real edges, not a JSON array — same pattern
-- domain_relationships already established, applied at phase granularity.
-- (§2.1 of schema-redesign-proposal.md.)

CREATE TABLE workflow_phase_dependencies (
    id                    INTEGER PRIMARY KEY,
    phase_id              INTEGER NOT NULL REFERENCES workflow_phases(id) ON DELETE CASCADE,
    depends_on_phase_id   INTEGER NOT NULL REFERENCES workflow_phases(id) ON DELETE CASCADE,
    UNIQUE(phase_id, depends_on_phase_id)
);
