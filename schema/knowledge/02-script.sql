-- knowledge.db — every deterministic script a standard ships. `location`
-- is a path samgraha resolves and runs via the existing script contract
-- (--repo-root/--in/--out, common/env.rs's run_capability_script) — the
-- one thing samgraha imposes. What the script does is the standard's own
-- business, samgraha never interprets it.

CREATE TABLE IF NOT EXISTS script (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    location TEXT    NOT NULL,   -- path on disk, resolved at dispatch time
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);
