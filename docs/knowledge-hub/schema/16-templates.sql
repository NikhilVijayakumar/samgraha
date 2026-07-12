-- Templates: full body storage for generation templates
-- (templates/generation/document|section) and audit report templates
-- (templates/audit/{deterministic,semantic}/{document,section},
-- templates/audit/summary). content holds the entire markdown/Jinja2 body
-- — the template's real content, not a path to it.
--
-- audit_bucket is NULL when kind = 'generation'; section_catalog_id is NULL
-- for document-scope templates. The CHECK spells out "OR audit_bucket IS
-- NULL" explicitly for the same portability reason as
-- calculation_rules.scope — see that file's comment.
--
-- section_catalog_key / audit_bucket_key: same SQL-NULL-never-equals-NULL
-- fix as rules.section_catalog_key (see that file's comment) — a plain
-- UNIQUE including the raw nullable section_catalog_id/audit_bucket
-- columns never dedupes document-scope or generation-kind rows on a
-- loader re-run. Coalesced to 0 / '' (real ids start at 1, real
-- audit_bucket values are non-empty), so re-inserts of the same NULL-bearing
-- row correctly conflict instead of duplicating.
--
-- source_file is a nullable, non-load-bearing debugging breadcrumb (e.g.
-- "generation/document/01-vision.md") — nothing looks a row up by it, it
-- only helps trace a bad row back to the file it came from during
-- ingestion debugging. Not a dependency the schema relies on.

CREATE TABLE templates (
    id                  INTEGER PRIMARY KEY,
    standard_id         INTEGER NOT NULL REFERENCES standards(id),
    domain_id           INTEGER NOT NULL REFERENCES domains(id),
    section_catalog_id  INTEGER REFERENCES section_catalog(id),
    kind                TEXT NOT NULL CHECK (kind IN ('generation','audit_report')),
    audit_bucket        TEXT CHECK (audit_bucket IS NULL OR audit_bucket IN ('deterministic','semantic','summary')),
    scope               TEXT NOT NULL CHECK (scope IN ('document','section')),
    name                TEXT NOT NULL,
    content             TEXT NOT NULL,
    sort_order          INTEGER NOT NULL DEFAULT 0,
    source_file         TEXT,
    section_catalog_key INTEGER GENERATED ALWAYS AS (COALESCE(section_catalog_id, 0)) VIRTUAL,
    audit_bucket_key    TEXT GENERATED ALWAYS AS (COALESCE(audit_bucket, '')) VIRTUAL,
    UNIQUE(standard_id, domain_id, section_catalog_key, kind, audit_bucket_key, scope)
);
