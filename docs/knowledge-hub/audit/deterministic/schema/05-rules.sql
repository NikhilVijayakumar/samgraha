-- Rules: one row per rule (not per file). Each YAML file contains a rules:
-- list; every entry in that list becomes its own row here. rule_ref uses
-- the format "path/to/file.yaml#rule_id" so the engine knows which rule
-- within the file this row represents.
--
-- mandatory, weight, and severity are per-rule properties that can differ
-- between rules in the same file (e.g. 3 mandatory + 1 optional in
-- vision/01-purpose.yaml).

CREATE TABLE rules (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    domain       TEXT NOT NULL,
    section_type TEXT,            -- NULL for whole-document rules
    kind         TEXT NOT NULL CHECK (kind IN ('deterministic','semantic')),
    scope        TEXT NOT NULL CHECK (scope IN ('document','section')),
    mandatory    INTEGER NOT NULL DEFAULT 0,
    weight       REAL NOT NULL DEFAULT 1.0,
    severity     TEXT NOT NULL CHECK (severity IN ('Critical','Warning','Suggestion')),
    rule_ref     TEXT NOT NULL     -- "path/to/file.yaml#rule_id"
);
