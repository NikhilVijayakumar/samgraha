-- Script check dependencies: a check's depends_on list as real edges
-- instead of a comma-separated or JSON list column.

CREATE TABLE script_check_dependencies (
    id                   INTEGER PRIMARY KEY,
    script_check_id      INTEGER NOT NULL REFERENCES script_checks(id),
    depends_on_check_id  INTEGER NOT NULL REFERENCES script_checks(id),
    UNIQUE(script_check_id, depends_on_check_id)
);
