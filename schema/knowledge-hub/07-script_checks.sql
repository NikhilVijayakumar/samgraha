-- Script checks: registry of script-backed audit checks (the
-- manifest.yaml + schema.json pairs). domain_id is NULL for checks that
-- apply across every domain (the "_generic" bucket, e.g. traceability-refs
-- checks that aren't specific to one domain).
--
-- result_schema holds the check's own JSON-Schema document body verbatim —
-- that is the artifact's native format, not a shortcut around modeling; the
-- check's own identity (name, domain, timeout, network requirement) is real
-- columns, not text buried in that schema.

CREATE TABLE script_checks (
    id                INTEGER PRIMARY KEY,
    standard_id       INTEGER NOT NULL REFERENCES standards(id),
    domain_id         INTEGER REFERENCES domains(id),
    check_name        TEXT NOT NULL,
    category          TEXT,
    timeout_seconds   INTEGER NOT NULL DEFAULT 60,
    requires_network  INTEGER NOT NULL DEFAULT 0,
    result_schema     TEXT NOT NULL,
    description       TEXT,
    UNIQUE(standard_id, check_name)
);
