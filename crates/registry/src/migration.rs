pub const MIGRATIONS: &[&str] = &[V1, V2, V3, V4];

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
