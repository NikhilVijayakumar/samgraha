-- Rules: one row per check, pointing at a YAML file that defines what the
-- check evaluates. rule_ref is a path relative to the knowledge-hub root,
-- e.g. "audit/deterministic/document/05-architecture.yaml" or
-- "audit/deterministic/section/architecture/01-purpose.yaml".
--
-- The schema does not interpret rule_ref — the engine loads the file and
-- executes its condition (deterministic) or sends it to the LLM (semantic).

CREATE TABLE rules (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    domain       TEXT NOT NULL,
    section_type TEXT,            -- NULL for whole-document rules
    kind         TEXT NOT NULL CHECK (kind IN ('deterministic','semantic')),
    scope        TEXT NOT NULL CHECK (scope IN ('document','section')),
    mandatory    INTEGER NOT NULL DEFAULT 0,
    weight       REAL NOT NULL DEFAULT 1.0,
    rule_ref     TEXT NOT NULL
);
