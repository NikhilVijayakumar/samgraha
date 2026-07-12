-- Relationship types: the closed vocabulary of edge types a standard's
-- domain graph is allowed to use (e.g. "derives", "inspires", "guides",
-- "informs"). Scoped per standard so a different system can define its own
-- vocabulary instead of inheriting one it doesn't need.
--
-- tier_gating: 'strict' means the edge blocks tier advancement until the
-- source domain clears; 'none' means the edge is informational/non-blocking
-- (e.g. a soft-alignment or a citation).

CREATE TABLE relationship_types (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    name         TEXT NOT NULL,
    tier_gating  TEXT NOT NULL CHECK (tier_gating IN ('strict','none')),
    description  TEXT,
    UNIQUE(standard_id, name)
);
