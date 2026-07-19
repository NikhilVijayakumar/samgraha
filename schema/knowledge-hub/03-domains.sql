-- Domains: canonical domain catalog for a standard, with tier assignment.
-- Replaces the free-text "domain" columns used elsewhere with a real row a
-- system registers once — e.g. "vision" (tier 1), "architecture" (tier 2).
-- Adding a domain to a standard = one new row here, never a migration.

CREATE TABLE domains (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    key          TEXT NOT NULL,
    name         TEXT NOT NULL,
    tier         INTEGER NOT NULL,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    description  TEXT,
    content_kind TEXT NOT NULL DEFAULT 'documentation',
    UNIQUE(standard_id, key)
);
