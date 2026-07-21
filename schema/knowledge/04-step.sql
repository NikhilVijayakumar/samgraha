-- knowledge.db — one row per step in a usecase's ordered sequence.
-- `kind` is the only thing samgraha branches on: 'deterministic' (samgraha
-- runs a script itself, no model needed) or 'semantic' (samgraha stages
-- data + a prompt and hands reasoning to whichever model is driving the
-- calling MCP client — never itself). Ordering is a plain integer, not a
-- dependency graph: a pre-script -> semantic -> post-script triad is just
-- three consecutive steps, no special-casing needed.

CREATE TABLE IF NOT EXISTS step (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    usecase_id  INTEGER NOT NULL REFERENCES usecase(id) ON DELETE CASCADE,
    step_order  INTEGER NOT NULL,
    kind        TEXT    NOT NULL CHECK (kind IN ('deterministic','semantic')),
    description TEXT    NOT NULL DEFAULT '',
    UNIQUE(usecase_id, step_order)
);
