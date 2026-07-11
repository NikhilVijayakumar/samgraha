-- Scores: one row per document per audit run — the four report scores plus
-- the final rollup. See §5 of proposal.md for the scoring formula:
--
--   final_score = (deterministic_whole/100 * 25)
--               + (deterministic_section/100 * 25)
--               + (semantic_whole/100 * 25)
--               + (semantic_section/100 * 25)
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
