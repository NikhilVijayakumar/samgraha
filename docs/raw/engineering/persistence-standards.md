# Persistence Standards

## Purpose

This document defines the persistence architecture for the Saṃgraha platform.

Persistence stores compiled documentation knowledge for query and audit. The persistence layer is local-first, deterministic, and repository-scoped.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Storage Engine: SQLite

SQLite is the persistence engine for all platform data. Rationale is documented in Technology Selection.

### Connection Management

- Single writer, multiple readers
- WAL mode for concurrent read access during writes
- Connection pooling is not required — single-process access pattern
- Connections use busy timeout with reasonable retry (5 seconds)

### Schema Management

- Schema is versioned and managed through migration files
- Migrations are sequences of SQL statements stored in the registry crate
- Migrations are applied atomically at first connection
- Migration state is tracked in a `_schema_version` table
- Schema version is verified on every startup
- Backward compatibility is maintained for at least one major version

The platform maintains two independent migration chains — `KNOWLEDGE_MIGRATIONS` (V1–V7) for the knowledge registry and `REGISTRY_MIGRATIONS` (REG_V1+) for the repository registry. Both use the same inline `const &[&str]` pattern in `crates/registry/src/migration.rs`. This is an intentional deviation from file-per-migration convention for simplicity at current scale.

Migration file naming (documented convention, not implemented): `V<number>__<description>.sql`

```
V1__initial_schema.sql
V2__add_document_tags.sql
V3__add_audit_results.sql
```

---

## Core Tables

### Documents

```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,           -- Relative path from repo root
    hash TEXT NOT NULL,                  -- SHA-256 of file content
    standard TEXT NOT NULL,              -- Documentation standard (adr, prd, spec, doc)
    title TEXT NOT NULL,                 -- Extracted title
    body TEXT NOT NULL,                  -- Parsed document body
    metadata TEXT NOT NULL DEFAULT '{}', -- Extracted metadata (JSON)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_documents_standard ON documents(standard);
CREATE INDEX idx_documents_path ON documents(path);
```

### Relationships

```sql
CREATE TABLE relationships (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL REFERENCES documents(id),
    target_id INTEGER NOT NULL REFERENCES documents(id),
    rel_type TEXT NOT NULL,             -- "derives-from", "references", "implements", etc.
    metadata TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_relationships_source ON relationships(source_id);
CREATE INDEX idx_relationships_target ON relationships(target_id);
```

### Audit Results

```sql
CREATE TABLE audit_results (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id),
    check_id TEXT NOT NULL,             -- Unique audit check identifier
    severity TEXT NOT NULL,             -- "error", "warning", "suggestion"
    message TEXT NOT NULL,
    location TEXT,                      -- File location relevant to the finding
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_audit_document ON audit_results(document_id);
```

### Glossary

```sql
CREATE TABLE glossary (
    id INTEGER PRIMARY KEY,
    term TEXT NOT NULL UNIQUE,
    definition TEXT NOT NULL,
    source_document_id INTEGER REFERENCES documents(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## Knowledge Packages

Knowledge Packages are portable archives of compiled knowledge for distribution and import.

### Format

```yaml
# knowledge-package.yaml
format: knowledge-package/v1
package:
  name: "saṃgraha-core-docs"
  version: "1.0.0"
  description: "Core saṃgraha documentation"
  repository: "https://github.com/example/samgraha"
  documents: 42
  created: "2026-03-15T10:30:00Z"
  hash: "sha256:e3b0c44298fc1c149afbf4c8996fb924..."
```

### Contents

```
knowledge-package/
├── package.yaml             # Package manifest
├── documents/               # Compiled documents
│   ├── doc_001.json
│   ├── doc_002.json
│   └── ...
├── registry.db              # SQLite database (optional, for full import)
└── signatures/              # Cryptographic signatures (optional)
    └── package.sig
```

### Import/Export

- Import validates package integrity (hash verification)
- Import merges documents by path — existing documents are updated
- Import does not delete documents not in the package
- Export includes all compiled documents for the repository
- Export excludes the SQLite database by default (included with --full flag)

---

## Data Integrity

### Content Hashing

- Every document is identified by its SHA-256 hash
- On re-compilation, documents with unchanged hashes are skipped
- The registry verifies hash consistency on read
- Knowledge Package distribution verifies hash on import

### Corruption Detection

On startup, the registry checks:

1. Database integrity via `PRAGMA integrity_check`
2. Document hash consistency for a random sample of 100 documents
3. Foreign key consistency

Corruption errors are reported immediately. Recovery is from the last knowledge package export.

---

## Backup and Recovery

- Knowledge Packages serve as the backup format
- Automatic export on significant registry changes (configurable)
- Export is safe — uses SQLite backup API (read-only consistent snapshot)
- Recovery: import knowledge package, re-compile documents

---

## Thread Safety

- The SQLite connection uses a mutex for write access
- Reads are concurrent (WAL mode)
- Long-running operations report progress
- Interrupt signal (Ctrl+C) safely closes the database connection

---

## Traceability

This document derives from:

- Architecture: Data Flow
- Architecture: Security Architecture
- Technology Selection

Persistence Standards provide the foundation for:

- Registry Implementation
- Knowledge Package Implementation

## Build Standards

Persistence layer is built with release optimizations for SQLite. Build profiles support bundled and system SQLite linkage. See [Build Standards](build-standards.md) for persistence-specific build configuration.

## Testing Standards

Persistence correctness is verified through integration tests covering CRUD operations, migration paths, and concurrent access patterns. See [Testing Standards](testing-standards.md) for testing framework.

Traceability:

```
Architecture → Technology Selection → Persistence Standards → Implementation
```
