-- Systems: the outer use-case / tenant.
-- "samgraha-documentation" is one row; "hackathon-eval", "research-paper-publishing"
-- are siblings — each gets its own row, no schema change.
--
-- Exactly one system may have is_default = 1 (enforced by the partial unique
-- index below). A repo with no row in repo_registrations uses this one.

CREATE TABLE systems (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    is_default  INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX ux_systems_one_default ON systems(is_default) WHERE is_default = 1;
