# Engineering Standards

This section details the Engineering Standards.

## Purpose

This document defines the standard for Engineering Documentation within the engineering documentation ecosystem.

Engineering Documentation describes the repository-wide engineering decisions, implementation standards, technology selection rationale, development conventions, and operational practices required to realize the documented architecture.

Unlike Feature Technical Design, Engineering Documentation is **not feature specific**.

Instead, it provides reusable engineering knowledge that governs the implementation of the entire repository.

Engineering Documentation explains **why the repository is engineered this way**.

It does not describe feature implementations.

---

# Responsibilities

Engineering Documentation is responsible for defining:

* Engineering principles
* Implementation standards
* Technology selection rationale
* Dependency standard
* Repository organization
* Development conventions
* Build standard
* Testing standard
* Deployment standard
* Packaging standard
* Persistence standard
* Configuration standard
* Security standard
* Performance standard
* Observability standard
* Versioning standard
* Release standard
* Migration standard
* Engineering constraints

Engineering Documentation establishes reusable engineering decisions for the repository.

---

# Scope

Engineering Documentation may include:

* Engineering Principles
* Implementation Standards
* Technology Selection
* Dependency Standards
* Repository Structure
* Build Standards
* Packaging Standards
* Testing Standards
* Deployment Standards
* Persistence Standards
* Configuration Standards
* Security Standards
* Performance Standards
* Logging Standards
* Monitoring Standards
* Localization Standards
* Plugin Standards
* Release Standards
* Migration Standards
* Operational Guidelines

Projects should document only the engineering domains relevant to the repository.

Engineering Documentation is intentionally modular.

---

# Out of Scope

Engineering Documentation must not describe:

* Product Vision
* Product Features
* Feature Design
* Feature Technical Design
* Shared Architecture
* Individual feature implementations
* Source code
* Algorithms
* Class implementations
* Function implementations
* Business requirements

Engineering explains implementation standard.

It does not contain implementation itself.

---

# Engineering as a Documentation Collection

Engineering Documentation is a collection of focused engineering documents.

Example:

```text
engineering/

    engineering-principles.md

    implementation-standards.md

    technology-selection.md

    dependency-standards.md

    repository-structure.md

    build-standards.md

    testing-standards.md

    deployment-standards.md

    persistence-standards.md

    security-standards.md

    performance-standards.md

    localization-standards.md

    packaging-standards.md
```

Each document should describe one engineering concern.

Responsibilities should not overlap.

---

# Single Responsibility

Every Engineering document should describe one reusable engineering concern.

Examples include:

* Build Standards
* Dependency Standards
* Repository Structure
* Testing Standards
* Deployment Standards
* Persistence Standards
* Security Standards

Large documents should be decomposed into smaller focused documents.

---

# Inputs

Engineering Documentation derives from:

* Architecture Documentation
* Feature Technical Design
* Relevant External Context
* Engineering Principles
* Organizational Standards

Engineering Documentation should not derive from implementation.

---

# Outputs

Engineering Documentation provides direction for:

* Source Code Implementation
* Repository Organization
* Build Pipelines
* Testing
* Deployment
* Continuous Integration
* Continuous Delivery
* Operational Procedures

Implementation should conform to the documented engineering standards.

---

# Traceability

Engineering Documentation remains traceable.

```text
Vision
    ↓
Feature
    ↓
Feature Design
    ↓
Architecture
    ↓
Relevant External Context
    ↓
Feature Technical Design
    ↓
Engineering
    ↓
Implementation
```

Engineering translates architectural intent into repository-wide implementation practices.

---

# Relationships

| Document                 | Relationship                                             |
| ------------------------ | -------------------------------------------------------- |
| Architecture             | Applies architectural decisions to engineering practices |
| Feature Technical Design | Provides feature-specific engineering context            |
| External Context         | Applies external engineering constraints                 |
| Implementation           | Realizes engineering standards                          |

---

# Engineering Principles

Engineering Documentation should establish reusable principles such as:

* Documentation First
* Architecture First
* Convention over Configuration
* Explicit Configuration
* Minimal Dependencies
* Repository Isolation
* Local First
* Offline First
* Deterministic Builds
* Separation of Concerns
* Progressive Enhancement
* Secure by Default
* Fail Fast
* Observable Systems

Projects may define additional engineering principles appropriate to their domain.

---

# Technology Selection

Engineering Documentation should explain **why** technologies were selected.

Examples include:

* Why Rust
* Why Kotlin
* Why Electron
* Why React
* Why SQLite
* Why TOML
* Why MCP
* Why gRPC
* Why Local Storage

Technology selection should describe engineering rationale rather than implementation details.

---

# External Context Application

Engineering Documentation should identify External Context that influences repository-wide engineering decisions.

Examples include:

* Internal frameworks
* Runtime platforms
* Build systems
* Packaging tools
* Platform capabilities
* Shared protocols
* Organizational engineering standards

External Context should be referenced rather than duplicated.

Only repository-wide engineering dependencies should be documented.

---

# Quality Requirements

Engineering Documentation should be:

* Modular
* Reusable
* Repository-wide
* Technology focused
* Rationale driven
* Maintainable
* Traceable
* Consistent

Engineering decisions should remain stable unless engineering standard changes.

---

# Validation Rules

Engineering Documentation is considered valid when:

* Engineering principles are documented.
* Repository-wide standards are clearly defined.
* Technology choices include engineering rationale.
* External engineering constraints are identified.
* Documents remain modular.
* Feature-specific implementation details are absent.
* Source code is not documented.
* Engineering standards remain traceable to Architecture.

Validation applies to the Engineering Documentation collection.

---

# Common Mistakes

Examples include:

* Documenting feature implementations.
* Rewriting Architecture Documentation.
* Embedding source code.
* Explaining algorithms.
* Duplicating External Context.
* Mixing repository-wide standard with feature-specific engineering.
* Describing implementation instead of engineering rationale.

These should be reported during audits.

---

# Generation Rules

When generating Engineering Documentation:

* Focus on repository-wide engineering decisions.
* Explain why engineering decisions exist.
* Organize documentation by engineering concern.
* Apply Architecture consistently.
* Apply relevant External Context.
* Reference rather than duplicate shared documentation.
* Avoid implementation details.
* Preserve engineering consistency.

---

# Enhancement Rules

When enhancing Engineering Documentation:

* Improve engineering rationale.
* Strengthen repository-wide consistency.
* Remove duplicated architectural information.
* Remove duplicated External Context.
* Improve traceability.
* Improve modularity.
* Preserve engineering intent.

Engineering Documentation should become easier to understand and maintain without changing implementation behavior.

---

# Audit Rules

An audit should verify:

* Repository-wide engineering principles are documented.
* Technology selection includes rationale.
* Engineering standards align with Architecture.
* Relevant External Context has been applied.
* Documents remain modular.
* Responsibilities do not overlap.
* Feature-specific engineering is absent.
* Source code is not documented.
* Engineering rationale is explicit rather than implied.

Engineering quality is evaluated across the complete Engineering Documentation collection.

---

# Success Criteria

Engineering Documentation is successful when:

* Engineers understand why the repository is engineered as it is.
* Technology choices are justified.
* Repository-wide engineering practices are consistent.
* New contributors can understand engineering decisions without reading source code.
* AI systems can implement features while respecting repository engineering conventions.
* Engineering decisions remain stable, traceable, and maintainable throughout the project lifecycle.

---

# Non-Goals

Engineering Documentation does not define:

* Product Vision
* Feature Specifications
* Feature Design
* Feature Technical Design
* Shared Architecture
* Source Code
* Algorithms
* API Implementations
* Business Logic

These responsibilities belong to other documentation standards.

---

# Summary

Engineering Documentation is the repository-wide engineering specification.

It is a modular collection of documents that explains the engineering principles, implementation standards, technology selection rationale, development conventions, and operational practices required to realize the documented architecture.

Its purpose is to provide a reusable engineering foundation that guides implementation consistently across the repository while remaining independent of feature-specific implementations and source code.

---

# Documentation Folder

Engineering documents live under:

```text
docs/raw/engineering/
```

---

# Implementation Folder

Engineering Documentation must declare the repository's implementation folder.

One engineering document must include a section titled **Repository Structure** that:

* names the top-level folder containing all implementation source code
* explains the rationale for that folder name and layout
* describes the high-level module or crate organization within it

This declaration is the authoritative source for the `implementation-audit`. The audit reads this section to locate source code before auditing. If the declaration is absent, the implementation audit cannot proceed.

Example structure for the declaration:

```
## Repository Structure

Implementation lives under `<folder>/`.

Rationale: <reason for folder name and layout>.

Top-level modules:
- `<folder>/module-a/` — <responsibility>
- `<folder>/module-b/` — <responsibility>
```

The implementation folder name and rationale must be kept current whenever the source layout changes.
