-- Score bands: rating thresholds scoped per standard (e.g. Excellent
-- 95-100, Very Good 90-94, ... Needs Improvement 0-69), so a different
-- system can define its own bands instead of inheriting one it doesn't
-- need. Applies to final_score; individual bucket scores use the same
-- bands for their own trend displays.

CREATE TABLE score_bands (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    rating       TEXT NOT NULL,
    min_score    REAL NOT NULL,
    max_score    REAL NOT NULL,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    UNIQUE(standard_id, rating)
);
