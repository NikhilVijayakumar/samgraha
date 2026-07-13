-- Standard docs: the documentation-standards/*.md content itself, one row
-- per domain. This is the human-readable spec a domain's rules, templates,
-- and section catalog are all derived from.
--
-- source_file is a nullable, non-load-bearing debugging breadcrumb (e.g.
-- "01-vision-standards.md") — see templates.source_file for the same note.

CREATE TABLE standard_docs (
    id           INTEGER PRIMARY KEY,
    domain_id    INTEGER NOT NULL UNIQUE REFERENCES domains(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    content      TEXT NOT NULL,
    source_file  TEXT
);
