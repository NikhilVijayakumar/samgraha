# Persistence Architecture

## Purpose

This document defines the persistence architecture of the Saṃgraha platform.

Persistence Architecture describes how engineering knowledge is organized, owned, and maintained throughout its lifecycle.

Rather than focusing on storage technologies, this document defines the architectural responsibilities, persistence boundaries, and lifecycle of persistent engineering knowledge.

Implementation details are documented separately.

---

# Persistence Philosophy

Saṃgraha persists engineering knowledge rather than application state.

Documentation remains the authoritative source.

Persistent storage contains compiled representations of engineering knowledge.

Compiled knowledge is disposable.

It can always be regenerated from documentation.

Persistence should never become the source of truth.

---

# Persistence Model

The platform distinguishes three categories of information.

```text
Documentation
        │
        ▼
Compiled Knowledge
        │
        ▼
Runtime State
```

Each category has different ownership and lifecycle.

---

## Documentation

Documentation is persistent.

Documentation is authoritative.

Documentation is managed by engineers and version control.

---

## Compiled Knowledge

Compiled knowledge is persistent but derived.

Compiled knowledge exists to optimize retrieval and runtime execution.

Compiled knowledge may be regenerated at any time.

---

## Runtime State

Runtime state is transient.

Runtime state exists only while requests are executing.

Runtime state should never become persistent engineering knowledge.

---

# Persistence Boundaries

Saṃgraha defines explicit persistence boundaries across two tracks.

```text
KNOWLEDGE TRACK

Repository Documentation
          │
          ▼
Knowledge Compiler
          │
          ▼
Knowledge Registry (.samgraha/knowledge.db)
          │
          ▼
Knowledge Runtime


METADATA TRACK

Repository Documentation
          │
          ▼
Knowledge Compiler
          │
          ▼
Repository Manifest (.samgraha/manifest.json)
          │
          ▼
Repository Registry (.samgraha/registry.db)
          │
          ▼
Metadata Cache (.samgraha/dependencies/*.meta.json)
```

The two tracks never intersect.

Only the Knowledge Compiler may create or update persistent engineering knowledge or repository manifests.

The Knowledge Runtime consumes compiled knowledge. It never modifies it.

The Metadata Cache is written by the Knowledge Resolver as a disposable performance optimization. Cache files may be deleted and regenerated from the Repository Registry at any time.

---

# Persistent Components

The platform persists several categories of engineering information.

## Documentation Metadata

Repository metadata describing engineering documentation.

Examples include:

* document identity
* documentation domain
* ownership
* lifecycle status
* relationships

---

## Compiled Knowledge

Compiled representations of documentation.

Examples include:

* structured document representation
* retrieval metadata
* knowledge chunks
* cross-references
* dependency relationships

---

## Verification Metadata

Knowledge verification information.

Examples include:

* audit status
* validation status
* traceability
* compilation metadata
* documentation standard version

---

## Knowledge Enrichment

Optional derived engineering knowledge.

Examples include:

* summaries
* keywords
* glossary entries
* semantic relationships
* embeddings

Derived knowledge remains non-authoritative.

---

# Knowledge Registry

The Knowledge Registry is the persistent storage boundary for compiled engineering knowledge.

Its responsibilities include:

* storing compiled knowledge
* storing document metadata
* storing traceability
* storing verification metadata
* supporting deterministic retrieval

The Knowledge Registry never stores repository metadata, manifests, or dependency information. Those belong to the Repository Registry.

The registry should never store undocumented engineering intent.

---

# Repository Registry

The Repository Registry is the persistent storage boundary for repository metadata.

Its responsibilities include:

* storing repository identity (UUID, ID, name)
* storing repository manifests
* storing dependency topology
* storing synchronization history
* storing workspace membership

The Repository Registry never stores compiled engineering knowledge. It reads only Repository Manifests.

Repository Registry storage is disposable. It can be fully rebuilt from manifests at any time.

---

# Persistence Lifecycle

Engineering knowledge follows a deterministic lifecycle across two tracks.

```text
KNOWLEDGE TRACK

Documentation
        │
        ▼
Compilation
        │
        ▼
Knowledge Registry
        │
        ▼
Knowledge Runtime
        │
        ▼
Development Tools


METADATA TRACK

Documentation
        │
        ▼
Compilation
        │
        ▼
Repository Manifest
        │
        ▼
Repository Registry
        │
        ▼
Metadata Cache
        │
        ▼
Knowledge Resolver
```

Compilation is the only stage that modifies persistent knowledge.

Runtime remains read-only.

---

# Repository Isolation

Persistent knowledge is isolated by repository.

Repositories share knowledge only through explicitly declared dependencies.

The persistence layer should preserve repository boundaries.

Knowledge ownership remains explicit.

---

# Persistence Principles

## Documentation First

Documentation remains authoritative.

Persistent knowledge is always derived.

---

## Disposable Artifacts

Compiled knowledge should always be reproducible.

Persistent artifacts may be deleted and regenerated.

---

## Deterministic Persistence

Identical documentation should always produce identical persistent knowledge.

---

## Repository Awareness

Persistent knowledge should preserve repository identity and dependency relationships.

---

## Traceability

Persistent knowledge should retain explicit relationships to:

* source documentation
* Documentation Standards
* audit results
* compilation metadata

---

## Read-Only Runtime

Persistent knowledge should never be modified during runtime execution.

---

## Offline First

Persistence should operate entirely on local storage.

External services remain optional.

---

# Technology Independence

The Persistence Architecture intentionally avoids implementation technologies.

Database engines, schemas, indexes, query strategies, storage formats, and optimization techniques belong to Engineering Documentation.

This document defines architectural persistence responsibilities only.

---

# Traceability

This document derives from:

* Vision
* Documentation Philosophy
* System Overview
* Runtime Boundary
* Security Architecture

This document provides architectural context for:

* Engineering Persistence Strategy
* Knowledge Registry
* Knowledge Runtime

Supporting features include:

* Markdown Compilation
* Knowledge Registry
* Incremental Compilation
* Knowledge Search
* Knowledge Enrichment

Traceability:

```text
Vision
    ↓
Documentation Philosophy
    ↓
System Overview
    ↓
Persistence Architecture
    ↓
Engineering
    ↓
Implementation
```
