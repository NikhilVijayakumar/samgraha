-- Sections: one row per section of a document.
-- semantic_type maps to the type declared in documentation-standards/
-- (e.g. "purpose", "system_overview", "traceability").

CREATE TABLE sections (
    id            INTEGER PRIMARY KEY,
    document_id   INTEGER NOT NULL REFERENCES documents(id),
    semantic_type TEXT NOT NULL,
    name          TEXT NOT NULL,
    sort_order    INTEGER NOT NULL DEFAULT 0,
    UNIQUE(document_id, semantic_type)
);
