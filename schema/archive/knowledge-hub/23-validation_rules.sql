-- Validation rules: a standard's own scoring-pipeline integrity checks
-- (calculation/validation/scoring_validation.yaml — weight sums, score
-- bounds, domain counts, ...). `rule` is prose describing the check, same
-- relationship `condition` has to `evidence` on the `rules` table — it's
-- documentation for a human/LLM reading the check, not something this
-- schema (or any Rust code) parses as an expression language. No CHECK
-- constraint on `severity` or anything else here — same reasoning
-- `calculation_rules.bucket` has none: a standard's own vocabulary, not
-- samgraha's to enumerate.

CREATE TABLE validation_rules (
    id                INTEGER PRIMARY KEY,
    standard_id       INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    check_key         TEXT NOT NULL,
    name              TEXT NOT NULL,
    description       TEXT,
    rule              TEXT NOT NULL,
    severity          TEXT,
    invalidate_audit  INTEGER NOT NULL DEFAULT 0,
    sort_order        INTEGER NOT NULL DEFAULT 0,
    UNIQUE(standard_id, check_key)
);
