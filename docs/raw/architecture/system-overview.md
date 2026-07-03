# System Overview

Saṃgraha is a Knowledge Engineering Platform that transforms engineering documentation into verified, deterministic engineering knowledge for humans and AI systems.

## Purpose

This document provides a high-level architectural overview of Saṃgraha.

Saṃgraha is a Knowledge Engineering Platform that transforms engineering documentation into verified, deterministic engineering knowledge for humans and AI systems.

Rather than operating as an application server, Saṃgraha behaves as a documentation compiler and knowledge platform. Documentation is the authoritative source of engineering intent. Compiled knowledge is an optimized representation of that documentation. Runtime services expose compiled knowledge without interpreting or modifying the original documentation.

This document describes the logical organization of the platform.

Implementation details are documented separately.

---

## Component Model

The system is composed of the following logical components: Documentation Standards, Knowledge Services, Knowledge Compiler, Knowledge Enrichment, Knowledge Registry, Repository Registry, Knowledge Runtime, Transport Adapters, and Provider Integrations. Each component has defined responsibilities and dependencies. See [Component Model](component-model.md) for detailed breakdown.

# Platform Overview

Saṃgraha is organized into four architectural layers.

```text
Documentation Standards
          │
          ▼
Knowledge Services
          │
          ▼
Knowledge Compilation
          │         │
          ▼         ▼
Knowledge      Repository
 Registry       Registry
(Knowledge    (Metadata
  Track)        Track)
          │         │
          └────┬────┘
               ▼
     Knowledge Runtime
```

Each layer has a single responsibility.

Together they transform engineering documentation into deterministic engineering knowledge.

Compilation produces two distinct outputs that follow separate, non-intersecting tracks. The knowledge track serves runtime queries. The metadata track serves synchronization and discovery.

---

# Architectural Layers

Saṃgraha is organized into architectural layers that separate concerns, enforce dependency direction, and establish clear boundaries between platform capabilities.

## Documentation Standards

Documentation Standards define engineering contracts.

Every documentation domain specifies:

* Purpose
* Responsibilities
* Scope
* Relationships
* Validation Rules
* Audit Rules
* Generation Rules
* Enhancement Rules
* Success Criteria

Documentation Standards define engineering quality independently of implementation.

---

## Knowledge Services

Knowledge Services execute engineering operations using Documentation Standards.

Examples include:

* Documentation Generation
* Documentation Audit
* Documentation Validation
* Documentation Enhancement
* Knowledge Search
* Repository Analysis
* Dependency Analysis
* Cross-Reference Analysis
* Knowledge Explanation
* Knowledge Compilation

Knowledge Services remain deterministic whenever possible.

Every service derives its behavior from Documentation Standards.

---

## Knowledge Compilation

Knowledge Compilation transforms documentation into optimized engineering knowledge.

Every successful compilation produces two distinct outputs:

| Output | Purpose |
|---|---|
| Compiled knowledge database | Engineering knowledge for search, retrieval, runtime |
| Repository manifest | Repository metadata for synchronization, discovery |

The compiled knowledge contains:

* Structured metadata
* Search indexes
* Cross-document relationships
* Traceability information
* Knowledge packages
* Optional semantic enrichments

The repository manifest contains only repository metadata — identity, revision, capabilities, exports, dependencies.

Compilation is deterministic.

Generated artifacts are disposable.

Documentation remains the single source of truth.

---

## Knowledge Runtime

The Knowledge Runtime exposes compiled engineering knowledge to development tools.

The runtime provides:

* Repository-aware retrieval
* Cross-repository knowledge resolution
* Progressive context loading
* Knowledge Services
* Deterministic execution

The runtime exposes services through one or more transport adapters.

---

# Logical Components

The platform is composed of the following logical components.

```text
Documentation Standards
            │
            ▼
     Knowledge Services
            │
     ┌──────┴──────┐
     │             │
     ▼             ▼
Knowledge      Knowledge
Compiler       Enrichment
     │             │
     └──────┬──────┘
            │
     ┌──────┴──────┐
     ▼             ▼
Knowledge     Repository
 Registry      Registry
(Knownledge   (Metadata
  Track)        Track)
     │             │
     └──────┬──────┘
            ▼
   Knowledge Runtime
            │
     ┌──────┴────────────┐
     ▼                   ▼
 CLI Adapter        MCP Adapter
            │
            ▼
 AI Engineering Tools
```

Each component has one primary responsibility.

Components communicate only through well-defined interfaces.

---

# Knowledge Flow

Engineering knowledge progresses through deterministic stages.

```text
Documentation Standards
            │
            ▼
Project Documentation
            │
            ▼
Knowledge Services
     │
     ├── Generate
     ├── Audit
     ├── Validate
     └── Enhance
            │
            ▼
Knowledge Compiler
            │
     ┌──────┴──────┐
     ▼             ▼
Knowledge     Repository
 Registry      Registry
(Knowledge   (Metadata
  Track)       Track)
     │             │
     │             ▼
     │     Registry Sync
     │             │
     ▼             │
Knowledge          │
 Runtime           │
     │             │
     └─────┬───────┘
           ▼
   Development Tools
```

Documentation is never modified during runtime.

Runtime services consume compiled knowledge only.

---

# Logical Component Responsibilities

| Component               | Responsibility                                                          |
| ----------------------- | ----------------------------------------------------------------------- |
| Documentation Standards | Define engineering contracts                                            |
| Knowledge Services      | Execute documentation operations using standards                        |
| Knowledge Compiler      | Transform documentation into deterministic knowledge                    |
| Knowledge Enrichment    | Generate optional summaries, keywords, embeddings, and derived metadata |
| Knowledge Registry      | Persist and query compiled knowledge                                    |
| Repository Registry     | Repository registration, discovery, manifest storage, sync history      |
| Knowledge Runtime       | Expose compiled knowledge and orchestrate Knowledge Services            |
| CLI Adapter             | Command-line access to the runtime                                      |
| MCP Adapter             | Model Context Protocol interface for AI engineering tools               |
| Provider Integrations   | Optional integrations with AI providers                                 |
| Shared Schemas          | Common models used across all components                                |

---

# Architectural Principles

Saṃgraha follows these architectural principles.

### Documentation First

Documentation is the authoritative engineering specification.

Implementation realizes documentation.

---

### Separation of Responsibilities

Every component has one architectural responsibility.

Responsibilities do not overlap.

---

### Standards Before Services

Knowledge Services derive their behavior from Documentation Standards.

Services should never define engineering rules independently.

---

### Deterministic Compilation

The same documentation always produces the same compiled knowledge.

Compilation contains no hidden state.

---

### Runtime Independence

The Knowledge Runtime operates entirely on compiled knowledge.

It never depends on source documentation during execution.

---

### Extensibility

Platform capabilities are designed to be extensible.

New standards, services, audits, validators, generators, and transport adapters should be added without modifying existing components.

---

### Offline First

Compilation and deterministic Knowledge Services operate without requiring network connectivity or AI providers.

AI provides optional enrichment rather than mandatory functionality.

---

# Technology Mapping

The current reference implementation is organized as a Rust workspace.

```text
samgraha/
crates/
    common/        # shared configuration, utilities
    schemas/       # shared domain types
    standards/     # Documentation Standards
    services/      # Knowledge Services + Knowledge Runtime
    compiler/      # Knowledge Compiler
    registry/      # Knowledge Registry + Repository Registry
    audit/         # Audit Framework
    providers/     # Provider Integrations
    cli/           # CLI Adapter
    mcp/           # MCP Adapter
tests/
```

Note: `KnowledgeRuntime` is implemented within `crates/services/src/runtime/`. There is no standalone `runtime/` crate.

This organization is an implementation detail of the current platform.

The logical architecture remains independent of programming language or repository layout.

---

# Traceability

This document derives from:

* Vision
* Documentation Philosophy

This document provides architectural context for:

* Component Model
* Runtime Boundary
* Communication
* Extension Model
* Persistence
* Security Architecture
* Repository Registry Architecture

Supporting features include:

* Documentation Standards
* Knowledge Services
* Markdown Compilation
* Knowledge Registry
* Repository Registry
* Knowledge Runtime
* Workspace Support
* Knowledge Search
* Knowledge Enrichment
* Automated Audit

## Security

Security is a cross-cutting architectural concern. Each component enforces security within its boundary. See [Security Architecture](security-architecture.md) for the complete security model, including trust boundaries, threat categories, and component responsibilities.

Traceability:

```text
Vision
    ↓
Documentation Philosophy
    ↓
System Overview
    ↓
Architecture ― Repository Registry Architecture
    ↓
Feature ― Repository Registry
    ↓
Engineering
    ↓
Implementation
```
