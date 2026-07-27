-- knowledge.db — generic rendering template catalog: opt-in, per standard.
-- Templates store content inline (read once at register time, not a path
-- to be re-read later). Each template's type is a relation into
-- template_type (13-template_type.sql), not free text — the standard's
-- own seeder declares valid types before referencing them here. Name is
-- unique within its standard.

CREATE TABLE IF NOT EXISTS template (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    type_id  INTEGER NOT NULL REFERENCES template_type(id),
    content  TEXT    NOT NULL,
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);
