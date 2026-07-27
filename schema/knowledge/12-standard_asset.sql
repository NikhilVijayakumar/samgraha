-- knowledge.db — standard-shipped content catalog: tracks plan/guide/config
-- files a standard provides. Each asset's kind is a relation into
-- asset_kind (11-asset_kind.sql), not free text — the standard's own
-- seeder declares valid kinds before referencing them here. Samgraha
-- never reads or interprets the asset's content — it only records where
-- to find it. Paths are absolutized after seeding (§3.11).

CREATE TABLE IF NOT EXISTS standard_asset (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    kind_id  INTEGER NOT NULL REFERENCES asset_kind(id),
    name     TEXT    NOT NULL,
    location TEXT    NOT NULL,
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, kind_id, name)
);
