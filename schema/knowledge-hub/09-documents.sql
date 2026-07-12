-- Documents: one row per audited document, tagged to the standard and
-- domain it belongs to. The same physical file can appear under different
-- standards if it's relevant to multiple rule sets (rare but possible).

CREATE TABLE documents (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    domain_id    INTEGER NOT NULL REFERENCES domains(id),
    title        TEXT,
    path         TEXT NOT NULL,
    UNIQUE(standard_id, path)
);
