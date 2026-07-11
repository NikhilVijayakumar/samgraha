# Architecture Standard

This section details the Architecture Standard.

## Purpose

This document defines the standard for Architecture Documentation within the engineering documentation ecosystem.

Architecture Documentation describes the structural organization of a system.

Unlike other documentation types, Architecture is not expected to be represented by a single document.

Instead, it is a structured collection of related documents that collectively describe the responsibilities, boundaries, interactions, and organization of the system.

Architecture explains **how responsibilities are organized**.

It does not explain implementation details.

---

# Required Sections

Every Architecture document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| System Overview | `system_overview` | ✓ | Overview, Architecture Overview |
| Component Model | `component_model` | ✓ | Components, Component Architecture |
| Communication | `communication_paths` | ✓ | Communication Paths, Component Communication |
| Data Flow | `data_flow` | ✓ | Data Movement, Information Flow |
| Security | `security_considerations` | ✓ | Security Architecture, Security Model |
| Purpose | `purpose` | | Overview, Summary |
| Rationale | `rationale` | | Decision Rationale, Architectural Decisions, Why |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

# Responsibilities

Architecture Documentation is responsible for describing:

* System structure
* Components
* Responsibilities
* Ownership boundaries
* Communication
* Dependencies
* Data flow
* Runtime organization
* Deployment organization
* Architectural constraints
* System invariants

Architecture defines how the system is organized.

It does not define how it is implemented.

---

# Scope

Architecture Documentation may include:

* System Overview
* Component Model
* Module Boundaries
* Runtime Boundaries
* Communication Architecture
* Data Flow
* Security Architecture
* Deployment Architecture
* Persistence Architecture
* Plugin Architecture
* Integration Architecture
* Lifecycle Documentation
* Architectural Invariants

Projects should include only the architectural topics relevant to the repository.

Architecture documentation is intentionally modular.

---

# Out of Scope

Architecture Documentation must not describe:

* Product vision
* Feature requirements
* User experience
* UI design
* Source code
* Algorithms
* Class implementations
* Function implementations
* Programming syntax
* Library APIs
* Configuration files
* Build scripts

These belong to downstream documentation.

---

# Architecture as a Documentation Collection

Architecture is a collection of related documents.

Example:

```text
architecture/

    system-overview.md

    component-model.md

    runtime-boundary.md

    communication.md

    persistence.md

    deployment.md
```

Each document should describe one architectural concern.

Responsibilities should not overlap.

---

# Single Responsibility

Every architecture document should have one primary responsibility.

Examples:

* Runtime Boundary
* Plugin Lifecycle
* Persistence
* Security
* Communication

Large architecture documents should be decomposed into smaller focused documents.

---

# Inputs

Architecture Documentation derives from:

* Vision
* Technology Decisions
* Platform Constraints
* Engineering Principles

Architecture is independent of Feature Documentation and Feature Design.

Architecture should not derive from implementation.

---

# Outputs

Architecture provides direction for:

* Feature Technical Design
* Engineering Decisions
* Implementation
* Testing Standards
* Validation
* Documentation Audits

Implementation should conform to Architecture.

---

# Traceability

Architecture should remain traceable.

```text
Vision
    │
    ├──────────────────────────┐
    ↓                          ↓
Feature chain            Architecture Documentation
(Feature → Feature Design)    (technology decisions,
    │                          platform constraints,
    │                          engineering principles)
    └──────────────────────────┘
                ↓
      Feature Technical Design
      (Feature + Architecture + optional External Context)
                ↓
          Engineering
                ↓
          Implementation
```

Architecture provides the structural foundation that Feature Technical Design applies to specific features.

---

# Relationships

| Document | Relationship |
|---|---|
| Philosophy | Architecture is guided by Philosophy |
| Security | Architecture is guided by Security's threat model (once registered) |
| Engineering | Soft, non-mandatory alignment — most frameworks expect an architecture, none require one first |
| Feature Technical Design | Architecture constrains it |
| External Context | May be referenced when integration constraints affect structure |

---

# Required Characteristics

Architecture Documentation should be:

* Consistent across components
* Traceable to Philosophy and Security
* Technology-independent at the system level
* Boundary-respecting
* Stable
* Reviewable independent of any single feature

---

# Architectural Boundaries

Architecture should define:

* Responsibility ownership
* Component boundaries
* Communication boundaries
* Data ownership
* Runtime ownership
* Security boundaries
* Extension boundaries

Boundaries should be explicit.

Implicit architecture should be avoided.

---

# Architectural Principles

Architecture should promote:

* Separation of concerns
* High cohesion
* Low coupling
* Explicit ownership
* Stable interfaces
* Predictable communication
* Clear dependencies
* Replaceable components

Projects may define additional architectural principles.

---

# Technology Independence

Architecture should remain implementation independent whenever practical.

Architecture should describe:

* responsibilities

instead of

* frameworks

Architecture may reference technologies only when they are architecturally significant.

Example:

Acceptable

* Electron Main Process
* Browser Process
* Plugin Runtime

Not acceptable

* React Hooks
* Axios
* SQLite API
* Rust syntax

Technology selection belongs in Engineering Documentation.

---

# Cross-Repository Architecture

If a repository depends on another repository:

Architecture should define:

* ownership boundaries
* interaction contracts
* communication model

Architecture should not duplicate another repository's architecture.

Instead, reference the relevant documentation.

---

# Quality Requirements

Architecture Documentation should be:

* Modular
* Cohesive
* Traceable
* Technology appropriate
* Responsibility driven
* Consistent
* Maintainable
* Scalable
* Reviewable

Architecture should evolve through decomposition rather than document growth.

---

# Validation Rules

Architecture Documentation is considered valid when:

* Responsibilities are clearly defined.
* Every architectural concern has a documented owner.
* Boundaries are explicit.
* Communication paths are documented.
* Component responsibilities do not overlap.
* Architectural documents remain modular.
* Architecture remains traceable to Features.
* No implementation details dominate architectural descriptions.

Validation applies to the architecture collection rather than individual files.

---

# Common Mistakes

Examples include:

* Mixing architecture with implementation.
* Large monolithic architecture documents.
* Undefined ownership.
* Hidden communication paths.
* Missing boundaries.
* Technology-driven architecture.
* Duplicated architectural responsibilities.
* Documenting source code instead of architecture.

These should be reported during audits.

---

# Generation Rules

When generating Architecture Documentation:

* Decompose by architectural responsibility.
* Prefer multiple focused documents over one large document.
* Describe responsibilities before technologies.
* Define boundaries explicitly.
* Reference external architectures rather than duplicating them.
* Maintain traceability to Features.

Architecture generation should optimize for maintainability rather than document count.

---

# Enhancement Rules

When enhancing Architecture Documentation:

* Improve separation of concerns.
* Split oversized documents.
* Remove duplicated responsibilities.
* Strengthen ownership definitions.
* Clarify communication.
* Improve traceability.
* Remove implementation leakage.
* Preserve architectural intent.

Architecture should become clearer through refinement.

---

# Audit Rules

An audit should verify:

* Architecture is modular.
* Responsibilities are clearly separated.
* Ownership is explicit.
* Boundaries are documented.
* Communication paths are understandable.
* Documents do not duplicate one another.
* Architecture supports Feature Technical Design without depending on Feature Documentation.
* Architecture avoids implementation detail.
* Cross-repository references are used instead of duplication.

Architecture quality is evaluated across the complete documentation collection.

---

# Success Criteria

Architecture Documentation is successful when:

* Engineers understand how the system is organized.
* Responsibilities are unambiguous.
* New contributors can locate architectural information easily.
* Components can evolve independently.
* Implementation decisions remain consistent with architectural intent.
* AI systems can understand the system organization without reading source code.

---

# Goals

Architecture Documentation aims to:

* Give the system a single authoritative structural description.
* Make component boundaries and responsibilities explicit.
* Let Feature Technical Design constrain itself against one source of truth.
* Keep structure stable while individual features change.

---

# Non-Goals

Architecture Documentation does not attempt to define:

* Product purpose
* Feature specifications
* Engineering decisions
* Technology selection
* Source code
* Algorithms
* Build configuration
* Library usage

These responsibilities belong to other documentation standards.

---

# Summary

Architecture Documentation is the structural specification of the system.

It is a modular collection of focused documents that collectively describe system organization, responsibilities, ownership, boundaries, and communication.

The objective is not to document every implementation detail, but to provide a clear, maintainable, and traceable architectural model that guides engineering and implementation throughout the lifecycle of the project.

---

# Documentation Folder

Architecture documents live under:

```text
docs/raw/architecture/
```

---

## Usage

Written by architects/senior engineers when a system's structure changes; read by anyone writing Feature Technical Design (which constrains itself to what Architecture permits). Use `samgraha compile --domain architecture` after adding a new architecture document, and `samgraha audit --domain architecture` to catch missing System Overview / Component Model / Security sections before review.

## Related

- [Feature Technical Standard](10-feature-technical-standards.md) — constrained by Architecture
- [Engineering Standard](07-engineering-standards.md) — guided by Architecture
- [Standards Reference Standard](standards.md) — how this standard itself is documented
