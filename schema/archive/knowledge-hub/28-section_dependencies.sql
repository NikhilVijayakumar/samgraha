-- Section dependencies: declares which sections must be generated before
-- others within the same domain. Operates at section_catalog granularity,
-- one level deeper than domain_relationships. Used by `generate` (§3 of
-- content-generation-mcp-proposal.md) to order section-level generation
-- tasks and reject store_generated_content for sections whose dependencies
-- haven't been stored yet.

CREATE TABLE IF NOT EXISTS section_dependencies (
    id                     INTEGER PRIMARY KEY,
    standard_id            INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    domain_id              INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    section_catalog_id     INTEGER NOT NULL REFERENCES section_catalog(id) ON DELETE CASCADE,
    depends_on_section_id  INTEGER NOT NULL REFERENCES section_catalog(id) ON DELETE CASCADE,
    UNIQUE(standard_id, section_catalog_id, depends_on_section_id)
);
CREATE INDEX IF NOT EXISTS idx_section_dependencies_domain ON section_dependencies(domain_id);
CREATE INDEX IF NOT EXISTS idx_section_dependencies_section ON section_dependencies(section_catalog_id);
