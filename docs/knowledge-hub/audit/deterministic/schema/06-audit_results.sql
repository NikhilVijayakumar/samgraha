-- Audit results: one row per rule evaluated against one target (a document
-- or a section). evidence is a JSON blob whose shape depends on the rule's
-- evidence.type extractor (section_presence, cross_reference, keyword_absence,
-- llm_judgment — see §10.3 of proposal.md).

CREATE TABLE audit_results (
    id           INTEGER PRIMARY KEY,
    target_id    INTEGER NOT NULL,
    target_kind  TEXT NOT NULL CHECK (target_kind IN ('document','section')),
    rule_id      INTEGER NOT NULL REFERENCES rules(id),
    score        REAL NOT NULL,
    evidence     TEXT,             -- JSON blob
    created_at   TEXT NOT NULL
);
