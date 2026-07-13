-- Plan settings: per-standard tier-loop orchestration config (one row per
-- standard). threshold_rating is a real composite FK into that same
-- standard's own score_bands — not a magic string that could drift out of
-- sync with the bands it's supposed to reference.
--
-- The composite FK means a standard's score_bands rows must exist before
-- its plan_settings row can be inserted — a silent load-order dependency,
-- not just a numbering coincidence. A loader (or anyone hand-seeding data)
-- must populate score_bands first.

CREATE TABLE plan_settings (
    id                INTEGER PRIMARY KEY,
    standard_id       INTEGER NOT NULL UNIQUE REFERENCES standards(id) ON DELETE CASCADE,
    threshold_rating  TEXT NOT NULL,
    max_iterations    INTEGER NOT NULL DEFAULT 5,
    fallback          TEXT NOT NULL DEFAULT 'human_review',
    note              TEXT,
    FOREIGN KEY (standard_id, threshold_rating) REFERENCES score_bands(standard_id, rating) ON DELETE CASCADE
);
