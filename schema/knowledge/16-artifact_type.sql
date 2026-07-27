-- knowledge.db — per-standard catalog of artifact.type values (e.g. image,
-- diagram, dataset, model). Unlike asset_kind/template_type, rows here are
-- not declared upfront by a seeder — they're find-or-created by samgraha
-- itself (services::register_standard::get_or_create_lookup) whenever a
-- script's result envelope reports an artifact type not seen before for
-- this standard, since a script's runtime output vocabulary can't be
-- predicted at registration time. Never deleted by delete_existing —
-- artifacts and their types are a historical output record that survives
-- a standard re-registering.

CREATE TABLE IF NOT EXISTS artifact_type (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);
