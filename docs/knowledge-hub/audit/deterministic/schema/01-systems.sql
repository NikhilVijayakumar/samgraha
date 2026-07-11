-- Systems: the outer use-case / tenant.
-- "samgraha-documentation" is one row; "hackathon-eval", "research-paper-publishing"
-- are siblings — each gets its own row, no schema change.

CREATE TABLE systems (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    description TEXT
);
