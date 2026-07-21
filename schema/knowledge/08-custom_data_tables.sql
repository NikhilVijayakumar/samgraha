-- knowledge.db — catalog of custom tables a standard's own scripts
-- create/own inside this same knowledge.db file. Samgraha never creates,
-- migrates, or manages the data of these tables — it only records that
-- they exist, why, and (once introspected) what they look like, so an
-- LLM or samgraha's own tooling can discover them without inspecting the
-- filesystem or guessing. `owner_script` references the script table
-- (script.id) that runs CREATE TABLE / owns the data — not a raw path.

CREATE TABLE IF NOT EXISTS custom_data_tables (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    standard        TEXT    NOT NULL,
    table_name      TEXT    NOT NULL,
    purpose         TEXT    NOT NULL DEFAULT '',
    owner_script_id INTEGER REFERENCES script(id) ON DELETE SET NULL,
    shape_json      TEXT,   -- PRAGMA table_info(table_name) result, cached;
                            -- NULL until the table actually exists and has
                            -- been introspected at least once
    UNIQUE(standard, table_name)
);
