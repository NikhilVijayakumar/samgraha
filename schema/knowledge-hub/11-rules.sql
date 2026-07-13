-- Rules: one row per rule (not per file). rule_key is the rule's own id
-- within its domain (e.g. "vis-doc-001") — self-contained, unlike the old
-- "path/to/file.yaml#rule_id" pointer, which made the schema depend on this
-- repo's own directory layout. description/condition/message hold the
-- rule's real content, not just its scoring metadata.
--
-- evidence_type names the evidence extractor (section_presence,
-- cross_reference, keyword_absence, llm_judgment, script_result, ...);
-- its variable-shape parameters live in rule_evidence_params.
-- script_check_id is set when evidence_type = 'script_result'.
--
-- rule_key is only unique against its own (section, scope, kind) — short
-- ids like "C1"/"C2"/"C3" are reused by design across every section's
-- semantic rubric in a domain (each section file grades its own C1), and
-- document-scope rules independently reuse short ids too. section_catalog_id
-- is NULL for every document-scope rule and for a domain's section-scope
-- fallback rule, so scope/kind are in the key too — otherwise a document-
-- scope and a fallback section-scope rule sharing a short id would collide.
--
-- section_catalog_key exists because SQL NULL is never equal to NULL: a
-- plain UNIQUE(..., section_catalog_id, ...) does NOT dedupe two
-- document-scope rows (section_catalog_id NULL in both) even if every
-- other column matches — every re-insert looks novel to SQLite and
-- ON CONFLICT silently never fires, so a loader re-run duplicates every
-- document-scope and fallback rule instead of updating it (found by
-- actually re-running the loader twice and diffing row counts — not a
-- hypothetical). section_catalog_key coalesces NULL to 0 (real ids start
-- at 1, so 0 never collides with a real one) so two document-scope rows
-- with the same key really do conflict.
--
-- is_fallback marks a domain's section-scope fallback rule (source:
-- audit/semantic/section/{domain}/generic.md) — applies to any section in
-- that domain lacking its own specific rubric. Redundant with the
-- (section_catalog_id IS NULL AND scope = 'section') convention for
-- uniqueness purposes (already enforced above); this column exists so the
-- audit engine can query it directly instead of knowing that convention.

CREATE TABLE rules (
    id                  INTEGER PRIMARY KEY,
    standard_id         INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    domain_id           INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    section_catalog_id  INTEGER REFERENCES section_catalog(id) ON DELETE CASCADE,
    rule_key            TEXT NOT NULL,
    kind                TEXT NOT NULL CHECK (kind IN ('deterministic','semantic')),
    scope               TEXT NOT NULL CHECK (scope IN ('document','section')),
    description         TEXT NOT NULL,
    condition           TEXT NOT NULL,
    message             TEXT NOT NULL,
    severity            TEXT NOT NULL CHECK (severity IN ('error','warning','suggestion','info')),
    weight              REAL NOT NULL DEFAULT 1.0,
    mandatory           INTEGER NOT NULL DEFAULT 0,
    evidence_type       TEXT NOT NULL,
    script_check_id     INTEGER REFERENCES script_checks(id) ON DELETE CASCADE,
    is_fallback         INTEGER NOT NULL DEFAULT 0,
    section_catalog_key INTEGER GENERATED ALWAYS AS (COALESCE(section_catalog_id, 0)) VIRTUAL,
    UNIQUE(standard_id, domain_id, section_catalog_key, scope, kind, rule_key)
);
