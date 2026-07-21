-- knowledge.db — every semantic-step prompt a standard ships. Content is
-- stored inline (not a path) — read once at register time, handed to the
-- calling agent verbatim at dispatch time. No file-sync/staleness concern
-- once it's in this table; re-registering the standard replaces it.

CREATE TABLE IF NOT EXISTS prompt (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    purpose  TEXT    NOT NULL DEFAULT '',
    content  TEXT    NOT NULL,   -- the actual prompt/rubric text, verbatim
    UNIQUE(standard, name)
);
