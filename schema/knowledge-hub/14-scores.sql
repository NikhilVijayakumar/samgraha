-- Scores: one row per document per audit run — the four bucket scores plus
-- the final rollup. See calculation_rules for the per-standard formula that
-- produced each value:
--
--   final_score = sum(bucket_score / 100 * bucket_weight)
--
-- weights come from calculation_inputs, scoped to the standard's
-- final_score row in calculation_rules — not hardcoded here, so a
-- different system can weight its buckets differently.
--
-- Individual finding detail stays in audit_results; this table carries
-- only the aggregated scores and pointers.

CREATE TABLE scores (
    id                     INTEGER PRIMARY KEY,
    document_id            INTEGER NOT NULL REFERENCES documents(id),
    deterministic_whole    REAL,
    deterministic_section  REAL,
    semantic_whole         REAL,
    semantic_section       REAL,
    final_score            REAL,
    created_at             TEXT NOT NULL
);
