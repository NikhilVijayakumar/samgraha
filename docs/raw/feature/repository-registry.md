# Repository Registry

This section details the Repository Registry.

## Purpose

The Repository Registry is the authoritative catalog of repository metadata.

It manages repository registration, identity, manifests, status, metadata cache, synchronization, and dependency topology.

The Repository Registry operates on a separate track from compiled engineering knowledge. It reads only Repository Manifests. It never opens or reads compiled knowledge databases.

The Registry is a compile-time and synchronization artifact. It is never consulted during runtime query resolution.

---

## Functional Requirements

This section details the Functional Requirements.

## FR1. Repository Registration

The Repository Registry shall support repository registration.

Registration records:

* repository identity (UUID, ID, name)
* repository location
* repository revision
* compiler version
* audit status
* exported documentation domains
* repository capabilities
* declared dependencies
* workspace membership
* synchronization history

Registration shall not modify repository contents.

---

## FR2. Repository Manifest Storage

The Repository Registry shall store Repository Manifests.

Each manifest entry contains:

* repository metadata from `manifest.json`
* revision tracking
* synchronization timestamps

The Registry shall detect manifest changes between synchronizations.

The Registry never stores:

* documents
* compiled knowledge
* search indexes
* embeddings

---

## FR3. Repository Identity

Every repository shall be identified by a stable UUID.

Repository identity includes:

* `uuid` — globally unique, never changes
* `id` — short identifier, may change on rename
* `name` — human-readable display name, may change

The UUID is generated once at `samgraha init`. It is committed to version control and never overwritten.

The Registry uses UUID as the authoritative identity key. ID and name are mutable display fields.

---

## FR4. Repository Status

The Repository Registry shall compute repository status on demand.

| Status | Meaning |
|---|---|
| `REGISTERED` | Known to Registry, metadata current |
| `SYNC_REQUIRED` | Manifest has changed since last sync |
| `STALE_METADATA` | Metadata cache has expired |
| `STALE_KNOWLEDGE` | Compiled knowledge revision behind manifest |
| `AUDIT_FAILED` | Last audit produced errors |
| `MISSING` | Repository path does not exist |
| `UNAVAILABLE` | Path exists but not accessible |

Status is computed from cached metadata. It is never persisted.

---

## FR5. Metadata Cache

The Repository Registry shall maintain a metadata cache.

The cache stores dependency metadata in a local SQLite database (`.samgraha/registry.db`).

The `repository_cache` table stores:

* repository identity (UUID, ID, name)
* revision
* repository root
* knowledge database location
* exported domains
* audit status
* synchronization timestamp
* expiration timestamp

Cache behavior is configured in `samgraha.toml`:

```toml
[resolver]
metadata_cache = true
metadata_ttl = "24h"
auto_refresh = true
registry_type = "file"
```

Expired metadata is never silently trusted. Cache miss at runtime degrades gracefully.

---

## FR6. Synchronization

The Repository Registry shall support explicit metadata synchronization.

Synchronization transfers only manifests:

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

Engineering knowledge never leaves the repository during synchronization.

---

## FR7. Dependency Graph

The Repository Registry shall maintain a dependency graph of registered repositories.

The graph supports:

* direct dependency resolution
* transitive dependency resolution
* cycle detection
* dependency completeness verification

The dependency graph contains only repository metadata. It never contains engineering knowledge.

---

## FR8. Repository Discovery Integration

The Repository Registry shall accept discovered repositories from Repository Discovery.

Registration may be triggered by:

* explicit `samgraha registry register`
* discovered repository approval through Workspace Management
* configuration-based auto-registration

The Registry shall not automatically register repositories without confirmation.

---

## Business Rules

* The Repository Registry reads only Repository Manifests.
* The Repository Registry never opens or reads compiled knowledge databases.
* The Repository Registry is a compile-time and synchronization artifact.
* The Repository Registry is never consulted during runtime query resolution.
* Metadata is disposable and refreshable.
* Repository UUID is stable across renames and relocations.
* Synchronization transfers metadata, not documentation.
* Resolution uses local metadata cache, never direct Registry contact.

---

## Registry Lifecycle

```text
Knowledge Compiler
        │
        ▼
Repository Manifest
        │
        ▼
samgraha registry sync
        │
        ▼
Repository Registry
        │
        ├── Registration
        ├── Manifest Storage
        ├── Status Computation
        ├── Metadata Cache
        └── Dependency Graph
        │
        ▼
Metadata Cache (.samgraha/registry.db)
        │
        ▼
Knowledge Resolver (cache only)
```

---

## Inputs

The Repository Registry consumes:

* Repository Manifests (`manifest.json`)
* repository configuration
* workspace configuration
* discovery results
* sync commands

---

## Outputs

The Repository Registry provides:

* repository registration
* repository discovery
* manifest storage
* status information
* metadata cache entries
* dependency topology
* synchronization history

Outputs are consumed by compile-time and administrative tooling.

---

## Constraints

The Repository Registry shall:

* read only Repository Manifests
* never open knowledge databases
* never contain engineering knowledge
* remain disposable and rebuildable from manifests
* support offline operation
* operate deterministically
* support large workspaces

---

## Dependencies

The Repository Registry depends upon:

* Knowledge Compiler (produces manifests)
* Repository Configuration (identity, resolver settings)
* Workspace Management (membership)
* Repository Discovery (new repositories)

The Repository Registry provides metadata to:

* Metadata Cache
* Knowledge Resolver (indirectly through cache)
* Engineering CLI
* MCP Adapter

---

## Non-Goals

The Repository Registry does not:

* compile documentation
* store compiled knowledge
* execute audits
* perform enrichment
* resolve runtime queries
* manage documentation
* own engineering knowledge

Those responsibilities belong to their respective platform components.

---

## Future Extensions

The Repository Registry should support future capabilities, including:

* remote HTTP registry backend
* organization-scoped registries
* registry federation
* event-driven synchronization
* repository health monitoring
* cross-workspace discovery
* scheduled refresh

Future capabilities should integrate without changing the logical Registry model.

---

## Acceptance Criteria

The feature is successful when:

* repositories can be registered and discovered
* repository metadata is accurately stored and retrievable
* status correctly reflects repository state
* synchronization transfers only manifests, never knowledge
* metadata cache is disposable and refreshable
* runtime resolution never contacts the Registry
* registry contents are fully rebuildable from manifests

---

## Traceability

This feature derives from the following architectural commitments:

* **Repositories own their knowledge.** The Registry owns metadata, never knowledge.
* **The Registry reads only manifests.** Knowledge databases are never opened.
* **Metadata is disposable.** Registry state can be regenerated at any time.
* **Identity is stable.** UUID survives renames and relocations.
* **Local-first.** Resolution uses cached metadata, never direct Registry contact.

**Traceability**

Architecture → Feature: Repository Registry
Vision → Architecture: Repository Registry Architecture
