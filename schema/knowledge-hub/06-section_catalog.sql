-- Section catalog: the canonical required/optional sections for a domain
-- (e.g. vision has purpose, vision_statement, problem, solution, ...).
-- This is the documentation content section rules registry — what a
-- document generator must produce and what an auditor checks for presence,
-- independent of any single document instance.
--
-- aliases: comma-separated alternate heading spellings matched
-- case-insensitively (e.g. "Purpose" might also be authored as "Overview"
-- in some documents) — added for crates/'s existing fuzzy section-matching
-- behavior (StandardDefinition.find_section_type), which needs this and
-- has no other source. NULL/empty until knowledge-hub's source content
-- (documentation-standards/, templates/generation/) is authored with an
-- explicit aliases field per section — the column exists ahead of that
-- authoring work so a DB-backed StandardRegistry doesn't ship with
-- structurally worse section matching than the builtin one it replaces.

CREATE TABLE section_catalog (
    id            INTEGER PRIMARY KEY,
    domain_id     INTEGER NOT NULL REFERENCES domains(id),
    semantic_type TEXT NOT NULL,
    name          TEXT NOT NULL,
    sort_order    INTEGER NOT NULL DEFAULT 0,
    mandatory     INTEGER NOT NULL DEFAULT 1,
    aliases       TEXT,
    UNIQUE(domain_id, semantic_type)
);
