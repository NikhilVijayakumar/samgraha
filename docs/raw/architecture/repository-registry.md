# Repository Registry Architecture

This section details the Repository Registry Architecture.

## Purpose

This document defines the architecture of the Repository Registry — the authoritative catalog of repository metadata within Saṃgraha.

The Repository Registry is responsible for repository discovery, identity management, manifest storage, dependency topology, and synchronization history.

It operates on an entirely separate track from compiled engineering knowledge. The Repository Registry reads only Repository Manifests. It never opens or reads compiled knowledge databases.

---

## System Overview

The Saṃgraha platform follows a layered architecture with four primary layers: Documentation Standards, Knowledge Services, Knowledge Compilation, and Knowledge Runtime. See [System Overview](system-overview.md) for the complete architecture description, platform layers, logical components, and architectural principles.

## Component Model

The system is composed of logical components organized by responsibility: Documentation Standards, Knowledge Services, Knowledge Compiler, Knowledge Enrichment, Knowledge Registry, Repository Registry, Knowledge Runtime, Transport Adapters, and Provider Integrations. See [Component Model](component-model.md) for detailed component responsibilities, dependencies, and interaction contracts.

# Architecture Philosophy

The Repository Registry follows a strict two-track separation.

**Track A — Metadata (compile-time only):**

Documentation Standards → Documentation → Knowledge Compiler → Repository Manifest → Repository Registry → Synchronization

**Track B — Knowledge (runtime):**

Knowledge Registry → Knowledge Resolver → Knowledge Package → Knowledge Runtime → Development Tools

The two tracks never intersect.

The Repository Registry is a compile-time and synchronization artifact. It is never consulted during knowledge resolution or runtime queries.

The Knowledge Resolver reads compiled knowledge databases directly. It never contacts the Repository Registry during resolution.

---

# Design Principles

1. **Repositories own their knowledge.** The Registry owns metadata, never engineering knowledge.
2. **Manifests only.** The Registry reads Repository Manifests. It never opens knowledge databases.
3. **No duplication.** Compiled knowledge is never copied into the Registry.
4. **Local-first.** Resolution operates from local metadata cache.
5. **Metadata is disposable.** Registry state can be regenerated from manifests at any time.
6. **Identity stability.** Repository UUID survives renames and relocations.
7. **Explicit sync.** Registry updates happen through explicit user commands, not automatically at runtime.

---

# Registry Data Model

The Registry maintains the following information for each registered repository.

| Property | Description | Source |
|---|---|---|
| Repository UUID | Stable unique identifier (never changes) | Generated at `samgraha init` |
| Repository ID | Short identifier (may change on rename) | From manifest |
| Repository Name | Human-readable display name | From manifest |
| Repository Root | Local filesystem path | From manifest |
| Repository Revision | Latest compiled revision number | From manifest |
| Compiler Version | Compiler used for latest compilation | From manifest |
| Audit Status | Latest audit outcome (PASS/FAIL/ERROR) | From manifest |
| Exported Domains | Documentation domains the repository provides | Auto-derived from compiled docs |
| Capabilities | Operations the repository supports | Auto-derived from configuration |
| Dependencies | Declared repository dependencies (by ID) | From manifest |
| Last Synchronization | Timestamp of last Registry update | Computed by Registry |

The UUID is the authoritative identity key. ID and name are mutable display fields.

---

# Repository Identity

Every repository declares a stable identity in `samgraha.toml`.

```toml
[repository]
id = "astra"
name = "Astra"
uuid = "f47ac10b-58cc-4372-a567-0e02b2c3d479"
```

| Field | Description | Stability |
|---|---|---|
| `id` | Short identifier used in manifests and dependency declarations | May change on rename |
| `name` | Human-readable display name | May change |
| `uuid` | Globally unique identifier | Never changes |

The UUID is generated once at `samgraha init` and committed to version control.

When a repository is renamed:

```
Before: id = "prana",  uuid = "abc-123"
After:  id = "prana2", uuid = "abc-123"
```

The Registry updates the ID and name. The UUID remains the same. All existing dependency references that use the UUID continue to resolve.

---

# Repository Manifest

Every successful compilation produces a Repository Manifest as an explicit compiler output.

The manifest is the synchronization artifact exchanged with the Repository Registry. It describes the repository. It never contains engineering knowledge.

**Location:** `.samgraha/manifest.json`

**Format:** JSON

Example:

```json
{
  "repository": {
    "id": "astra",
    "name": "Astra",
    "uuid": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
  },
  "revision": 52,
  "compiler": {
    "version": "1.0.0"
  },
  "audit": {
    "status": "PASS",
    "last_audit": "2026-06-27T12:00:00Z"
  },
  "repository_root": "D:/Projects/astra",
  "knowledge": {
    "location": ".samgraha/knowledge.db"
  },
  "exports": ["architecture", "feature", "design", "standards"],
  "capabilities": ["compile", "audit", "semantic-audit", "enrichment", "mcp"],
  "dependencies": ["prati", "tantra"],
  "generated_at": "2026-06-27T12:01:00Z"
}
```

The manifest never contains:

* documents
* chunks
* search indexes
* embeddings
* engineering knowledge

Both `knowledge.db` and `manifest.json` are outputs of the same compilation. The manifest is written only when compilation succeeds.

---

# Repository Status

Repository Status is computed on demand from cached metadata. It is never persisted.

| Status | Meaning |
|---|---|
| `REGISTERED` | Known to Registry, metadata current |
| `SYNC_REQUIRED` | Registered but manifest has changed since last sync |
| `STALE_METADATA` | Metadata cache has expired (TTL elapsed) |
| `STALE_KNOWLEDGE` | Compiled knowledge revision behind manifest revision |
| `AUDIT_FAILED` | Last audit produced errors |
| `MISSING` | Repository path does not exist |
| `UNAVAILABLE` | Path exists but not accessible |

`STALE_METADATA` and `STALE_KNOWLEDGE` are distinct states. Metadata expiry and knowledge staleness are independent conditions.

`MISSING` and `UNAVAILABLE` are also distinct. Missing means the path does not exist. Unavailable means the path exists but cannot be opened.

Status derives from:

* revision comparison (manifest revision vs cached revision)
* metadata lifetime (TTL vs last sync timestamp)
* audit status (from manifest)
* path accessibility check

---

# Metadata Cache

Each repository maintains a local SQLite-backed cache of dependency metadata.

```
.samgraha/
    registry.db          ← SQLite database with repository_cache table
```

The cache stores `CachedRepoMetadata` entries — repository identity, revision, root path, knowledge database location, exports, audit status, sync timestamp, and TTL expiry. Same fields as the JSON prototype, now backed by an indexed SQLite table.

The `repository_cache` table schema:

```sql
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
```

No engineering knowledge is duplicated. The cache is disposable — fully rebuildable from dependency manifests.

---

# Metadata Lifetime

Metadata is intentionally temporary.

Configuration in `samgraha.toml`:

```toml
[resolver]
metadata_cache = true
metadata_ttl = "24h"
auto_refresh = true
registry_type = "file"
```

When metadata expires:

1. If `auto_refresh = true`, repository attempts refresh from local Registry.
2. Cache is updated.
3. Resolution continues.

Expired metadata is never silently trusted indefinitely.

Runtime resolution never contacts the Registry directly. Cache miss degrades gracefully — reports missing dependency, does not abort the request.

---

# Synchronization

Synchronization exchanges metadata only.

```
Repository
    ↓
Compile
    ↓
manifest.json generated
    ↓
samgraha registry sync
    ↓
Repository Registry updated
```

Engineering knowledge never leaves the repository.

The Registry receives only the Repository Manifest.

---

# Resolution Pipeline

The resolution pipeline is split at the compile/runtime boundary.

## Compile-Time

Registry contact is permitted during and immediately after compilation.

```
Compile
    ↓
Success
    ↓
Write knowledge.db
    ↓
Increment revision
    ↓
Generate manifest.json
    ↓
[if auto_refresh = true] Update local Registry
    ↓
[explicit] samgraha registry sync → Registry updated
```

## Runtime

No Registry contact. No network calls.

```
User Query
    ↓
Knowledge Resolver
    ↓
Metadata Cache lookup
    ├── hit  → Read knowledge.db → Knowledge Package → Runtime
    └── miss → Report missing dependency, degrade gracefully
```

The Repository Registry is never in the runtime query path.

---

# Extension Model

The Repository Registry is accessed through a `RegistryClient` interface with these operations:

| Operation | Description |
|---|---|
| `register` | Add a repository manifest to the registry |
| `unregister` | Remove a repository by UUID |
| `sync` | Update a manifest to match current state |
| `discover` | Query repositories by search criteria |
| `get_metadata` | Retrieve metadata for a single repository |
| `list` | Enumerate all known repositories |

The interface is implemented by:

| Implementation | When | Backend |
|---|---|---|
| `FileRegistryClient` | Phase 1 — built | Local SQLite, per-workspace |
| `HttpRegistryClient` | Future | Remote HTTP, org-wide |

The trait decouples Registry consumers from storage. Consumers depend on the trait, not the implementation.

---

# CLI Integration

Repository management is exposed through the `samgraha registry` subcommand.

```text
samgraha registry register
samgraha registry unregister
samgraha registry sync
samgraha registry refresh
samgraha registry status
samgraha registry list
samgraha registry resolve runtime
```

---

# MCP Integration

The MCP adapter routes both knowledge and registry operations.

| Method | Route |
|---|---|
| `search`, `get_sections`, `get_document`, `compile`, `audit`, `info` | Knowledge Runtime |
| `register_repository`, `unregister_repository`, `synchronize_repository`, `resolve_dependencies`, `list_repositories`, `repository_status`, `workspace_status` | Registry Client |

Single adapter, dual routing.

---

# Design Properties

* **Local-first.** Registry operates on local SQLite by default.
* **Offline-first.** Normal operation never requires network.
* **Metadata-driven.** Registry tracks metadata, not knowledge.
* **Deterministic.** Same input always produces same Registry state.
* **Disposable.** Registry can be rebuilt from manifests at any time.
* **Stable identity.** UUID survives renames and relocations.
* **Explicit sync.** Registry updates happen on user command.
* **Extensible.** `RegistryClient` trait supports future backends.

---

# Technology Independence

The Repository Registry architecture intentionally avoids implementation technologies.

Storage engines, serialization formats, synchronization protocols, transport layers, and programming language constructs belong to Engineering Documentation.

This document defines the logical Registry architecture only.

---

# Traceability

This document derives from:

* Vision — Global Engineering Knowledge
* Vision — Repository-Owned Knowledge
* Vision — Deterministic Engineering
* Architecture: System Overview
* Architecture: Component Model

This document provides architectural context for:

* Feature: Repository Registry
* Feature: Knowledge Resolution
* Feature: Knowledge Package
* Feature: Repository Discovery
* Feature: Repository Configuration
* Engineering: Repository Registry Implementation

## Security

The Repository Registry operates outside the runtime query path and manages repository metadata only. Registry data is a generated artifact and never an authority boundary violation. See [Security Architecture](security-architecture.md) for registry-specific security requirements.

Traceability:

```text
Vision
    ↓
System Overview
    ↓
Component Model
    ↓
Repository Registry Architecture
    ↓
Feature
    ↓
Engineering
    ↓
Implementation
```
