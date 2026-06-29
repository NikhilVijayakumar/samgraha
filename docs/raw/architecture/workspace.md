# Workspace Architecture

## Purpose

This document defines the Workspace Architecture of the Saṃgraha platform.

A Workspace organizes multiple engineering repositories into a coherent knowledge ecosystem while preserving repository ownership and isolation.

The Workspace Architecture enables cross-repository engineering knowledge, dependency resolution, and repository-aware AI assistance without compromising the independence of individual repositories.

This document defines the logical organization of workspaces.

Implementation details are documented separately.

---

## System Overview

The Saṃgraha platform follows a layered architecture with four primary layers: Documentation Standards, Knowledge Services, Knowledge Compilation, and Knowledge Runtime. See [System Overview](system-overview.md) for the complete architecture description, platform layers, logical components, and architectural principles.

## Component Model

The system is composed of logical components organized by responsibility: Documentation Standards, Knowledge Services, Knowledge Compiler, Knowledge Enrichment, Knowledge Registry, Repository Registry, Knowledge Runtime, Transport Adapters, and Provider Integrations. See [Component Model](component-model.md) for detailed component responsibilities, dependencies, and interaction contracts.

# Workspace Philosophy

Repositories should remain independently owned.

Knowledge should be shared intentionally rather than implicitly.

A Workspace provides a shared engineering context while preserving repository autonomy.

The Workspace is an organizational boundary rather than a persistence boundary.

---

# Workspace Model

A Workspace contains one or more repositories.

```text
Workspace
    │
    ├── Repository A
    ├── Repository B
    ├── Repository C
    └── Repository N
```

Each repository remains independently buildable.

Each repository maintains its own documentation.

Each repository produces its own compiled knowledge.

---

# Repository Model

Every repository consists of:

```text
Repository
      │
      ├── Documentation
      ├── Documentation Standards
      ├── Compiled Knowledge
      └── Repository Configuration
```

Repositories remain self-contained.

Repository knowledge may be consumed by other repositories only through explicitly declared dependencies.

---

# Knowledge Registry

Each repository owns its own Knowledge Registry.

```text
Workspace
      │
      ├── Repository A → .samgraha/knowledge.db
      ├── Repository B → .samgraha/knowledge.db
      ├── Repository C → .samgraha/knowledge.db
      └── Repository N → .samgraha/knowledge.db
```

Repositories do not share a single centralized Knowledge Registry. Each repository compiles and owns its compiled knowledge independently.

Cross-repository knowledge is assembled at runtime by the Knowledge Resolver from individual `knowledge.db` files according to declared dependencies.

Each registry remains a generated artifact. Documentation remains the authoritative source.

---

# Repository Registry

The Repository Registry operates alongside the Knowledge Registry at the workspace level.

```text
Workspace
      │
      ▼
Repository Registry
      │
      ├── Repository A (manifest, status)
      ├── Repository B (manifest, status)
      ├── Repository C (manifest, status)
      └── Repository N (manifest, status)
```

The Repository Registry tracks:

* workspace membership
* repository identity (UUID, ID, name)
* repository revision and status
* dependency topology
* synchronization history

Default location: `.samgraha/registry.db` within the workspace root.

The Repository Registry reads only Repository Manifests. It never opens compiled knowledge databases.

---

# Repository Knowledge Packages

Every repository receives its own Knowledge Package.

A Knowledge Package contains:

- repository knowledge
- dependency knowledge
- verification metadata
- repository relationships
- optional derived knowledge

Knowledge Packages are optimized for runtime consumption.

---

# Dependency Resolution

Repositories communicate through explicit dependencies.

```text
Repository A
      │
      ├──────► Repository B
      │
      └──────► Repository C
```

Dependencies define:

- visibility
- repository relationships
- knowledge availability
- compilation order

Undeclared dependencies are not permitted.

---

# Workspace Compilation

Workspace compilation proceeds independently for each repository.

```text
Repository Documentation
          │
          ▼
Knowledge Compilation
          │
     ┌────┴────┐
     ▼         ▼
Repository  Repository
 Knowledge   Manifest
     │         │
     ▼         ▼
Knowledge  Repository
 Registry   Registry
```

Compilation produces two outputs: compiled knowledge for the Knowledge Registry and a repository manifest for the Repository Registry.

Compilation should preserve repository ownership.

Workspace compilation should never merge repositories into a single engineering model.

---

# Workspace Resolution

The Knowledge Runtime resolves engineering knowledge within the Workspace.

Resolution includes:

- active repository
- dependency repositories
- repository boundaries
- Knowledge Packages

Runtime requests should receive only knowledge relevant to the active repository.

---

# Repository Isolation

Repositories remain isolated by default.

A repository may access another repository only through explicitly declared dependencies.

Repository isolation applies to:

- documentation
- compiled knowledge
- runtime context
- Knowledge Services

Workspace membership does not imply unrestricted access.

---

# Workspace Lifecycle

Knowledge progresses through the Workspace lifecycle.

```text
Repository
      │
      ▼
Compilation
      │
     ┌────┴────┐
     ▼         ▼
Knowledge  Repository
 Registry   Registry
     │         │
     ▼         ▼
Knowledge  Registry
 Package     Sync
     │         │
     ▼         │
Knowledge      │
 Runtime       │
     │         │
     └────┬────┘
          ▼
 Development
   Tools
```

Every repository follows the same lifecycle.

The knowledge track serves runtime delivery. The metadata track serves synchronization and discovery.

---

# Workspace Principles

## Repository Independence

Repositories should remain independently buildable and independently understandable.

---

## Explicit Dependencies

Knowledge sharing requires declared dependencies.

Implicit repository discovery is not permitted.

---

## Repository Ownership

Each repository owns its own engineering knowledge.

Ownership is never transferred to the Workspace.

---

## Shared Knowledge

Shared engineering knowledge is always derived from repository documentation.

The Workspace never becomes the authoritative source.

---

## Deterministic Resolution

The same repository and dependency graph should always produce the same runtime context.

---

## Progressive Context

Development tools should receive only the engineering knowledge required for the active repository.

Repository context should expand only through declared dependencies.

---

## Disposable Knowledge Registry

The Knowledge Registry remains a generated artifact.

It may be regenerated from repository documentation at any time.

---

## Registry Separation

The Repository Registry and Knowledge Registry are distinct components with different responsibilities.

The Repository Registry tracks repository metadata. The Knowledge Registry stores compiled engineering knowledge.

The two registries never intersect. The Repository Registry never reads knowledge databases. The Knowledge Registry never manages repository identity.

---

# Workspace Scalability

The Workspace Architecture supports:

- single-repository projects
- multi-repository products
- shared engineering libraries
- platform ecosystems
- organization-wide knowledge registries

The architectural model remains unchanged regardless of Workspace size.

---

# Technology Independence

This document intentionally avoids implementation technologies.

Workspace discovery, configuration files, storage engines, dependency formats, synchronization mechanisms, and registry implementations belong to Engineering Documentation.

Workspace Architecture defines only the logical organization of repositories and engineering knowledge.

---

# Traceability

This document derives from:

- Vision
- Documentation Philosophy
- System Overview
- Knowledge Flow
- Component Model
- Persistence Architecture

This document provides architectural context for:

- Knowledge Registry
- Knowledge Runtime
- Deployment Architecture
- Engineering Workspace Strategy

Supporting features include:

- Workspace Support
- Knowledge Registry
- Repository Registry
- Knowledge Runtime
- Repository Resolution
- Knowledge Packages
- External Context

## Security

Workspaces maintain strict repository isolation. Knowledge is shared only through declared dependencies. See [Security Architecture](security-architecture.md) for workspace security requirements and cross-repository access control.

Traceability:

```text
Vision
    ↓
Documentation Philosophy
    ↓
System Overview
    ↓
Workspace Architecture
    ↓
Architecture ― Repository Registry Architecture
    ↓
Engineering
    ↓
Implementation
```