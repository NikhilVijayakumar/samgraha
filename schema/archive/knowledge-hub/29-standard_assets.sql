-- Standard assets catalog: every named script or prompt asset a standard
-- ships, independent of which phase(s) reference it. Lets an LLM (or
-- samgraha's dispatcher) resolve a name to a path and read the one-line
-- purpose without inspecting the filesystem.
--
-- Populated from system.yaml's `catalog:` block at register_standard
-- time and propagated to consumer repos via sync_knowledge_system.

CREATE TABLE IF NOT EXISTS standard_assets (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id  INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,   -- e.g. "audit-python", "leaderboard-prompt"
    kind         TEXT    NOT NULL CHECK (kind IN ('script','prompt')),
    path         TEXT    NOT NULL,   -- relative to .samgraha/scripts/ or /templates/ or /audit_analysis/
    purpose      TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard_id, name)
);
CREATE INDEX IF NOT EXISTS idx_standard_assets_standard ON standard_assets(standard_id, kind);
