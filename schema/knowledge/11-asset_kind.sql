-- knowledge.db — per-standard catalog of valid standard_asset.kind values
-- (e.g. config, guide, plan). Declared by the standard's own seeder before
-- it inserts standard_asset rows referencing kind_id — same pattern as
-- `domain`/`usecase.domain_id`. Mirrors registry::core_schema::CORE_V2
-- exactly.

CREATE TABLE IF NOT EXISTS asset_kind (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);
