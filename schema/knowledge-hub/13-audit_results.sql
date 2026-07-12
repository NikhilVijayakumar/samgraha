-- Audit results: one row per rule evaluated against one target (a document
-- or a section). evidence is a JSON blob whose shape depends on the rule's
-- evidence_type extractor (section_presence, cross_reference,
-- keyword_absence, llm_judgment, script_result, ...). Unlike rule
-- definitions, this is per-run execution output, not system definition —
-- its shape is inherently variant per rule, so it stays JSON rather than
-- being forced into fixed columns or an EAV table.
--
-- script_result evidence carries the script's own metrics and evidence
-- arrays (see script_checks.result_schema). The script's exit code is NOT
-- stored here — exit 0 means "check ran" (pass/fail/not_applicable lives in
-- status); exit 1 means "script itself failed to execute" (error status).

CREATE TABLE audit_results (
    id           INTEGER PRIMARY KEY,
    target_id    INTEGER NOT NULL,
    target_kind  TEXT NOT NULL CHECK (target_kind IN ('document','section')),
    rule_id      INTEGER NOT NULL REFERENCES rules(id),
    score        REAL NOT NULL,
    evidence     TEXT,
    created_at   TEXT NOT NULL
);
