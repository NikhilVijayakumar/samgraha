-- knowledge.db — per-standard catalog of valid template.type values (e.g.
-- markdown, html, email). Declared by the standard's own seeder before it
-- inserts template rows referencing type_id — same pattern as
-- `domain`/`usecase.domain_id`. Mirrors registry::core_schema::CORE_V2
-- exactly.

CREATE TABLE IF NOT EXISTS template_type (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);
