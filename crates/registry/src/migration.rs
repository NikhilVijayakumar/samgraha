/// Knowledge registry migrations — create `knowledge.db` tables.
pub const KNOWLEDGE_MIGRATIONS: &[&str] = &[V1, V2, V3, V4, V5, V6, V7];

/// Repository registry migrations — create `registry.db` tables.
pub const REGISTRY_MIGRATIONS: &[&str] = &[REG_V1];

/// Backward compat alias.
pub const MIGRATIONS: &[&str] = KNOWLEDGE_MIGRATIONS;

const V1: &str = "
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL,
    standard TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_documents_standard ON documents(standard);
CREATE INDEX IF NOT EXISTS idx_documents_path ON documents(path);
CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);

CREATE TABLE IF NOT EXISTS relationships (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL REFERENCES documents(id),
    target_id INTEGER NOT NULL REFERENCES documents(id),
    rel_type TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_relationships_source ON relationships(source_id);
CREATE INDEX IF NOT EXISTS idx_relationships_target ON relationships(target_id);

CREATE TABLE IF NOT EXISTS audit_results (
    id INTEGER PRIMARY KEY,
    document_id INTEGER REFERENCES documents(id),
    check_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    location TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_document ON audit_results(document_id);

CREATE TABLE IF NOT EXISTS glossary (
    id INTEGER PRIMARY KEY,
    term TEXT NOT NULL UNIQUE,
    definition TEXT NOT NULL,
    source_document_id INTEGER REFERENCES documents(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS enrichment (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id),
    artifact_type TEXT NOT NULL,
    content TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_enrichment_document ON enrichment(document_id);

CREATE TABLE IF NOT EXISTS build_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

const V2: &str = "
CREATE TABLE IF NOT EXISTS search_index (
    term TEXT NOT NULL,
    document_id INTEGER NOT NULL REFERENCES documents(id),
    frequency INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (term, document_id)
);

CREATE INDEX IF NOT EXISTS idx_search_term ON search_index(term);
";

const V3: &str = "
CREATE TABLE IF NOT EXISTS knowledge_packages (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    manifest TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const V4: &str = "
CREATE TABLE IF NOT EXISTS document_sections (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    semantic_type TEXT NOT NULL,
    canonical_name TEXT NOT NULL,
    content TEXT NOT NULL,
    required INTEGER NOT NULL DEFAULT 0,
    section_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_sections_semantic_type ON document_sections(semantic_type);
CREATE INDEX IF NOT EXISTS idx_sections_document_id ON document_sections(document_id);
CREATE INDEX IF NOT EXISTS idx_sections_type_doc ON document_sections(semantic_type, document_id);
";

const V5: &str = "
ALTER TABLE document_sections ADD COLUMN parent_id INTEGER REFERENCES document_sections(id);

CREATE INDEX IF NOT EXISTS idx_sections_parent_id ON document_sections(parent_id);
";

const V6: &str = "
CREATE TABLE IF NOT EXISTS graph_nodes (
    urn TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    title TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_graph_nodes_document ON graph_nodes(document_id);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_type ON graph_nodes(node_type);

CREATE TABLE IF NOT EXISTS graph_edges (
    id INTEGER PRIMARY KEY,
    source_urn TEXT NOT NULL REFERENCES graph_nodes(urn) ON DELETE CASCADE,
    target_urn TEXT NOT NULL REFERENCES graph_nodes(urn) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_graph_edges_source ON graph_edges(source_urn);
CREATE INDEX IF NOT EXISTS idx_graph_edges_target ON graph_edges(target_urn);
CREATE INDEX IF NOT EXISTS idx_graph_edges_type ON graph_edges(edge_type);
";

const V7: &str = "
ALTER TABLE documents ADD COLUMN quality TEXT NOT NULL DEFAULT '{}';
";

/// REG_V1 — repository registry tables for `.samgraha/registry.db`.
///
/// Stores cached dependency metadata in a single `repository_cache` table,
/// indexed by UUID for fast lookup during dependency resolution.
/// Supersedes the Phase 1-5 JSON file approach (`.samgraha/dependencies/*.meta.json`).
/// The cache is disposable — fully rebuildable from dependency manifests.
const REG_V1: &str = "
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS repository_cache (
    id TEXT PRIMARY KEY,
    uuid TEXT NOT NULL,
    name TEXT NOT NULL,
    repository_root TEXT NOT NULL,
    knowledge_db TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0,
    exports TEXT NOT NULL DEFAULT '[]',
    audit TEXT NOT NULL DEFAULT 'PASS',
    last_sync TEXT NOT NULL,
    expires TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_repo_cache_uuid ON repository_cache(uuid);
";
