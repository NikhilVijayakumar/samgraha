-- knowledge.db — discovery-only domain mirror: a flat list of domains a
-- standard declares, linked to usecases via `usecase.domain_id`. Domains
-- are metadata for filtering and grouping usecases; samgraha never
-- interprets their meaning beyond sorting by `sort_order`.

CREATE TABLE IF NOT EXISTS domain (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    key         TEXT    NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, key)
);
