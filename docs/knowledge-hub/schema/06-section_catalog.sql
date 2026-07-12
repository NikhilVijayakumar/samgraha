-- Section catalog: the canonical required/optional sections for a domain
-- (e.g. vision has purpose, vision_statement, problem, solution, ...).
-- This is the documentation content section rules registry — what a
-- document generator must produce and what an auditor checks for presence,
-- independent of any single document instance.

CREATE TABLE section_catalog (
    id            INTEGER PRIMARY KEY,
    domain_id     INTEGER NOT NULL REFERENCES domains(id),
    semantic_type TEXT NOT NULL,
    name          TEXT NOT NULL,
    sort_order    INTEGER NOT NULL DEFAULT 0,
    mandatory     INTEGER NOT NULL DEFAULT 1,
    UNIQUE(domain_id, semantic_type)
);
