-- knowledge.db — a usecase a knowledge standard declares. One row per
-- usecase; a standard can declare more than one. Everything below this
-- table is samgraha's own orchestration schema — meaning, data shape, and
-- storage of what a usecase actually produces is entirely the standard's
-- concern (see custom_data_tables, 08-custom_data_tables.sql).

CREATE TABLE IF NOT EXISTS usecase (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    standard    TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    data        TEXT    NOT NULL DEFAULT '{}',  -- additional data, opaque to samgraha
    UNIQUE(standard, name)
);
