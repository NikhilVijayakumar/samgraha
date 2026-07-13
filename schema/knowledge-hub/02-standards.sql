-- Standards: pluggable rule sets scoped to a system.
-- A system owns one or more standards. "documentation-standards" belongs to
-- the samgraha-documentation system; a hackathon system might own several
-- ("frontend-v1", "backend-v1", "qa-v1") side by side.

CREATE TABLE standards (
    id          INTEGER PRIMARY KEY,
    system_id   INTEGER NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    description TEXT,
    UNIQUE(system_id, name, version)
);
