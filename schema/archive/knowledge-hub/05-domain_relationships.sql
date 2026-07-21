-- Domain relationships: the derivation graph edges between domains of the
-- same standard (e.g. vision --derives--> feature). A domain can have more
-- than one parent; each parent gets its own row.
--
-- mutual: the edge holds in both directions (e.g. a non-mandatory soft
-- alignment between two domains that both expect the other).
-- enforce_order: within the same tier, from_domain must complete before
-- to_domain starts, even though the tier itself doesn't gate on this edge
-- (the one documented exception: External Context before Engineering).

CREATE TABLE domain_relationships (
    id                    INTEGER PRIMARY KEY,
    standard_id           INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    from_domain_id        INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    to_domain_id          INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    relationship_type_id  INTEGER NOT NULL REFERENCES relationship_types(id) ON DELETE CASCADE,
    mutual                INTEGER NOT NULL DEFAULT 0,
    enforce_order         INTEGER NOT NULL DEFAULT 0,
    note                  TEXT,
    UNIQUE(standard_id, from_domain_id, to_domain_id, relationship_type_id)
);
