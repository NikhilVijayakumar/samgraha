-- Sections: one row per section of a document. section_catalog_id points
-- at the canonical section this instance fulfills (e.g. "purpose" under
-- the vision domain's catalog).

CREATE TABLE sections (
    id                  INTEGER PRIMARY KEY,
    document_id         INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    section_catalog_id  INTEGER NOT NULL REFERENCES section_catalog(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    sort_order          INTEGER NOT NULL DEFAULT 0,
    UNIQUE(document_id, section_catalog_id)
);
