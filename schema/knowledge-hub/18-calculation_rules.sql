-- Calculation rules: one row per scoring bucket a standard defines
-- (deterministic_document, deterministic_section, semantic_document,
-- semantic_section, final_score, trend). Each bucket names its own
-- calculation_method (weighted_pass_rate, weighted_sum, trend_comparison,
-- ...); final_score's weighted_sum inputs live in calculation_inputs.
--
-- tolerance_* / min_samples / fallback_* columns are only meaningful for
-- the 'trend' bucket (trend_comparison's two-tier per_standard -> global ->
-- floor tolerance resolution); NULL otherwise.
--
-- scope is NULL for cross-cutting buckets (e.g. final_score, trend) and
-- otherwise whatever granularity the standard's own calculation file names
-- (document, section, domain, ...). No fixed vocabulary here, deliberately —
-- a CHECK enumerating known scope values would be exactly the kind of
-- standard-shaped assumption this table exists to avoid (found the hard way:
-- python_hackathon's calculation/aggregation/domain/*.yaml declares
-- "scope: domain", which an earlier ('document','section')-only CHECK
-- rejected outright).

CREATE TABLE calculation_rules (
    id                    INTEGER PRIMARY KEY,
    standard_id           INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    bucket                TEXT NOT NULL,
    calculation_method    TEXT NOT NULL,
    scope                 TEXT,
    formula               TEXT NOT NULL,
    rollup                TEXT,
    tolerance_method      TEXT,
    tolerance_k           REAL,
    tolerance_floor       REAL,
    tolerance_scope       TEXT,
    min_samples           INTEGER,
    fallback_scope        TEXT,
    fallback_min_samples  INTEGER,
    note                  TEXT,
    UNIQUE(standard_id, bucket)
);
