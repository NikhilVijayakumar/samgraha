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
-- scope is NULL for the two cross-cutting buckets (final_score, trend) —
-- the CHECK spells out "OR scope IS NULL" explicitly rather than relying
-- on SQLite's default CHECK-ignores-NULL behavior, so the same constraint
-- means the same thing if this schema is ever ported to a database that
-- enforces CHECK against NULL (e.g. some CHECK dialects do).

CREATE TABLE calculation_rules (
    id                    INTEGER PRIMARY KEY,
    standard_id           INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    bucket                TEXT NOT NULL,
    calculation_method    TEXT NOT NULL,
    scope                 TEXT CHECK (scope IS NULL OR scope IN ('document','section')),
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
