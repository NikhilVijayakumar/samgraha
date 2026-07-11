# Engineering Standards

## Table of Contents
- [Purpose](#purpose)
- [Build Standards](#build-standards)
- [Testing Standards](#testing-standards)
- [Code Standards](#code-standards)
- [Constraints](#constraints)
- [Required Sections](#required-sections)
- [Goals](#goals)
- [Non-Goals](#non-goals)
- [Success Criteria](#success-criteria)
- [Responsibilities](#responsibilities)
- [Scope](#scope)
- [Out of Scope](#out-of-scope)
- [Inputs](#inputs)
- [Outputs](#outputs)
- [Traceability](#traceability)
- [Relationships](#relationships)
- [Required Characteristics](#required-characteristics)
- [Audit Rules](#audit-rules)
- [Validation Rules](#validation-rules)
- [Generation Rules](#generation-rules)
- [Enhancement Rules](#enhancement-rules)
- [Summary](#summary)
- [Common Mistakes](#common-mistakes)
- [Documentation Folder](#documentation-folder)
- [Implementation Folder](#implementation-folder)
- [Repository Structure](#repository-structure)
- [Usage](#usage)
- [Related](#related)
- [Engineering as a Documentation Collection](#engineering-as-a-documentation-collection)
- [Single Responsibility](#single-responsibility)
- [Engineering Principles](#engineering-principles)
- [Technology Selection](#technology-selection)
- [External Context Application](#external-context-application)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Engineering Documentation exists — its role in the documentation ecosystem and what it uniquely covers
> **out_of_scope:** Specific engineering standards, technology choices, implementation details, feature-specific engineering
> **contributes:** Establishes the root intent that all engineering sections derive from and constrains what engineering docs may contain
> **relationships:** Purpose(01) root; derived from Architecture(05); governs all engineering sections
> **responsibilities:** Define Engineering Documentation's reason for being in terms of its relationship to the broader documentation ecosystem
> **generation_rules:** Start with the role in the ecosystem; distinguish from Feature Technical Design; explain the "why" before the "what"
> **enhancement_rules:** Strengthen the distinction from adjacent standards; remove ambiguity about scope boundaries
> **validation_rules:** Purpose is clearly defined; boundaries with adjacent standards are explicit; no implementation detail leakage
> **audit_rules:** Must exist; must not contain specific technology choices; must distinguish Engineering from Feature Technical Design

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
## Purpose

[1–2 paragraphs explaining this document's role in the documentation ecosystem,
its scope boundaries, and how it differs from adjacent standards]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Architecture(05), adjacent documentation standards

This document defines the standard for Engineering Documentation within the engineering documentation ecosystem.

Engineering Documentation describes the repository-wide engineering decisions, implementation standards, technology selection rationale, development conventions, and operational practices required to realize the documented architecture.

Unlike Feature Technical Design, Engineering Documentation is **not feature specific**.

Instead, it provides reusable engineering knowledge that governs the implementation of the entire repository.

Engineering Documentation explains **why the repository is engineered this way**.

It does not describe feature implementations.

---

## Build Standards

> **semantic_type:** `build_standards`
> **scope:** Repository-wide build process, CI/CD pipeline, and build tooling standards
> **out_of_scope:** Feature-specific build steps, deployment scripts, release automation details
> **contributes:** Ensures consistent, reproducible builds across the entire repository
> **relationships:** Derived from Architecture(05); referenced by Implementation for build conformance; connected to Testing Standards
> **responsibilities:** Define the build toolchain, pipeline stages, and build quality gates
> **generation_rules:** Identify the build system and pipeline stages; explain rationale for each stage; document quality gates
> **enhancement_rules:** Add stages when new concerns emerge; remove obsolete steps; preserve pipeline stability
> **validation_rules:** Build process is documented end-to-end; quality gates are explicit; rationale is present for each choice
> **audit_rules:** Must exist; must document the build toolchain; must include pipeline stages; must connect to Architecture

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

```markdown
## Build Standards

> [metadata block]

### Build System

[1–2 paragraphs: build tool, configuration, rationale for choice]

### Pipeline Stages

> **diagram:** flowchart of pipeline stages

[1 paragraph per stage explaining purpose, inputs, outputs, quality gates]

### Quality Gates

[Optional: criteria that must pass before proceeding]
```

**Required subsections:** Build System, Pipeline Stages
**Optional subsections:** Quality Gates
**Required diagrams:** Pipeline flowchart
**Required cross-references:** Architecture(05), Testing Standards

*(To be written by the domain expert. This section defines the repository-wide build process, CI/CD pipeline, and build tooling standards.)*

---

## Testing Standards

> **semantic_type:** `testing_standards`
> **scope:** Repository-wide testing strategy, test types, coverage expectations, and test tooling standards
> **out_of_scope:** Feature-specific test cases, individual test implementations, test data details
> **contributes:** Ensures consistent, comprehensive testing practices across the entire repository
> **relationships:** Derived from Architecture(05) and Build Standards; referenced by Implementation for test conformance
> **responsibilities:** Define the testing strategy, test types, coverage expectations, and test tooling
> **generation_rules:** Identify test types and their purpose; define coverage expectations; document test tooling and conventions
> **enhancement_rules:** Add test types when new concerns emerge; refine coverage expectations; preserve testing stability
> **validation_rules:** Testing strategy is documented; coverage expectations are explicit; tooling is identified; rationale is present
> **audit_rules:** Must exist; must document the testing strategy; must include test types; must connect to Architecture

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

```markdown
## Testing Standards

> [metadata block]

### Test Types

[1 paragraph per test type: unit, integration, e2e — purpose, scope, tooling]

### Coverage Expectations

[1 paragraph: coverage targets, what is measured, thresholds]

### Test Tooling

[Optional: test runner, assertion libraries, mocking frameworks]
```

**Required subsections:** Test Types, Coverage Expectations
**Optional subsections:** Test Tooling
**Required diagrams:** Test strategy flowchart
**Required cross-references:** Architecture(05), Build Standards

*(To be written by the domain expert. This section defines the repository-wide testing strategy, test types, and testing tooling standards.)*

---

## Code Standards

> **semantic_type:** `code_standards`
> **scope:** Repository-wide coding style, conventions, linting rules, and code quality standards
> **out_of_scope:** Feature-specific implementation patterns, algorithm details, API usage examples
> **contributes:** Ensures consistent, readable, maintainable code across the entire repository
> **relationships:** Derived from Engineering Principles; referenced by Implementation for code conformance; connected to Testing Standards
> **responsibilities:** Define coding style, naming conventions, linting rules, and code quality expectations
> **generation_rules:** Identify the language-specific style guide; document linting configuration; explain non-obvious conventions
> **enhancement_rules:** Add conventions when new patterns emerge; remove obsolete rules; preserve style stability
> **validation_rules:** Coding standards are documented; conventions are explicit; tooling is identified; rationale is present
> **audit_rules:** Must exist; must document coding conventions; must identify linting tooling; must connect to Engineering Principles

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
## Code Standards

> [metadata block]

### Style Guide

[1 paragraph: language-specific style reference, key rules]

### Linting Configuration

[1 paragraph: linter tool, config file location, key rules]

### Naming Conventions

[Optional: naming patterns for files, modules, functions, variables]
```

**Required subsections:** Style Guide, Linting Configuration
**Optional subsections:** Naming Conventions
**Required diagrams:** none
**Required cross-references:** Engineering Principles

*(To be written by the domain expert. This section defines the repository-wide coding style, conventions, and code quality standards.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Non-functional requirements and engineering limitations that bound all implementation decisions
> **out_of_scope:** Feature-specific constraints, business rules, UI requirements, user-facing limitations
> **contributes:** Provides the hard boundaries within which all engineering decisions must operate
> **relationships:** Derived from Architecture(05) and External Context; referenced by Technology Selection and all engineering standards
> **responsibilities:** Define non-functional requirements, performance bounds, security constraints, and regulatory limitations
> **generation_rules:** Extract from Architecture and External Context; categorize by type (performance, security, compliance); make constraints verifiable
> **enhancement_rules:** Add constraints when new external requirements emerge; remove obsolete constraints; preserve constraint clarity
> **validation_rules:** Constraints are documented; constraints are verifiable; constraints connect to their source; no contradictions between constraints
> **audit_rules:** Must exist; must document non-functional requirements; must be verifiable; must connect to Architecture or External Context

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
## Constraints

> [metadata block]

[1–2 paragraphs explaining non-functional requirements and engineering
limitations, categorized by type (performance, security, compliance),
each with source attribution and verifiability]

### Performance Constraints

[Optional: latency, throughput, memory bounds]

### Security Constraints

[Optional: authentication, encryption, access control requirements]

### Compliance Constraints

[Optional: regulatory, organizational policy requirements]
```

**Required subsections:** none
**Optional subsections:** Performance Constraints, Security Constraints, Compliance Constraints
**Required diagrams:** none
**Required cross-references:** Architecture(05), External Context

*(To be written by the domain expert. This section defines the non-functional requirements and engineering limitations that bound all implementation decisions.)*

---

## Required Sections

Every Engineering document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases | Content Requirements |
|---------|--------------|----------|---------|----------------------|
| Engineering Principles | `guiding_principles` | ✓ | Principles, Core Principles | Technology-independent values, stable across features, memorable phrasing |
| Technology Selection | `rationale` | ✓ | Technology Choices, Technology Rationale, Why | Rationale for each choice; not a bare list; connected to Architecture and External Context |
| Build Standards | `build_standards` | ✓ | Build, Build Process, CI/CD | Build system, pipeline stages, quality gates, rationale for each stage |
| Testing Standards | `testing_standards` | ✓ | Testing, Test Strategy | Test types, coverage expectations, test tooling, rationale |
| Purpose | `purpose` | | Overview, Summary | Document's role in ecosystem, scope boundaries, relationship to adjacent standards |
| Code Standards | `code_standards` | | Coding Standards, Code Style | Style guide, linting configuration, naming conventions, rationale |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements | Non-functional requirements categorized by type, verifiable, connected to source |
| Traceability | `traceability` | | Traces To, Derived From | Derivation diagram, upstream/downstream list, non-contradiction rule |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Engineering Documentation aims to:

* Give the codebase one authoritative source for technology rationale and repo-wide standards.
* Make build, test, and code-quality expectations explicit.
* Keep engineering decisions traceable to Architecture and Security rather than arbitrary.

---

## Non-Goals

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

## Success Criteria

Engineering Documentation is successful when:

* Engineers understand why the repository is engineered as it is.
* Technology choices are justified.
* Repository-wide engineering practices are consistent.
* New contributors can understand engineering decisions without reading source code.
* AI systems can implement features while respecting repository engineering conventions.
* Engineering decisions remain stable, traceable, and maintainable throughout the project lifecycle.

---

## Responsibilities

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

## Scope

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

## Out of Scope

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

## Inputs

Engineering Documentation derives from:

* Architecture Documentation
* Feature Technical Design
* Relevant External Context
* Security Requirements
* Engineering Principles
* Organizational Standards

Engineering Documentation should not derive from implementation.

---

## Outputs

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

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Engineering Documentation connects to the documentation hierarchy — derivation chain from Architecture through Engineering to Implementation
> **out_of_scope:** Implementation traceability, test traceability, version history, bug tracking
> **contributes:** Makes engineering decisions' lineage visible and verifiable across the documentation ecosystem
> **relationships:** Engineering(07) sits between Feature Technical Design(10) and Implementation; derived from Architecture(05)
> **responsibilities:** Show the derivation path from Architecture to Engineering to Implementation; assert no downstream document contradicts engineering standards
> **generation_rules:** Use the derivation diagram; list upstream sources and downstream consumers; state non-contradiction constraint
> **enhancement_rules:** Update the diagram when new standards are added; ensure derivation paths remain accurate
> **validation_rules:** Derivation paths are complete; no orphaned standards; non-contradiction rule is stated
> **audit_rules:** Must exist; must include derivation diagram; must list upstream and downstream standards; must state non-contradiction constraint

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

```markdown
## Traceability

> [metadata block]

[1 paragraph stating the non-contradiction constraint and traceability principle]

> **diagram:** flowchart showing derivation chain

### Upstream Sources

[List of upstream documents that feed into Engineering]

### Downstream Consumers

[List of downstream documents that derive from Engineering]
```

**Required subsections:** Upstream Sources, Downstream Consumers
**Optional subsections:** none
**Required diagrams:** Derivation chain flowchart
**Required cross-references:** Architecture(05), Feature Technical Design(10), Implementation

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

## Relationships

| Document                 | Relationship                                             |
| ------------------------ | -------------------------------------------------------- |
| Architecture             | Applies architectural decisions to engineering practices |
| Feature Technical Design | Provides feature-specific engineering context            |
| Security                 | Applies security requirements to engineering practices   |
| External Context         | Applies external engineering constraints                 |
| Implementation           | Realizes engineering standards                          |

---

## Required Characteristics

Engineering Documentation should be:

* Rationale-driven, not just prescriptive
* Consistent across the repository
* Reviewable independent of any single feature
* Traceable to Architecture and Security
* Stable
* Convention-respecting

---

## Audit Rules

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

## Validation Rules

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

## Generation Rules

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

## Enhancement Rules

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

## Summary

Engineering Documentation is the repository-wide engineering specification.

It is a modular collection of documents that explains the engineering principles, implementation standards, technology selection rationale, development conventions, and operational practices required to realize the documented architecture.

Its purpose is to provide a reusable engineering foundation that guides implementation consistently across the repository while remaining independent of feature-specific implementations and source code.

---

## Common Mistakes

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

## Documentation Folder

Engineering documents live under:

```text
docs/raw/engineering/
```

---

## Implementation Folder

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

---

## Usage

Written and maintained by senior engineers/tech leads as repository-wide decisions are made; read by every contributor before touching build, test, or deployment tooling. Use `samgraha audit --domain engineering` to confirm Technology Selection includes rationale (not just a list of choices) and that the Repository Structure declaration required by the `implementation-audit` is present and current.

## Related

- [Architecture Standard](05-architecture-standards.md) — engineering practices apply architectural decisions
- [Feature Technical Standard](10-feature-technical-standards.md) — feature-specific engineering context
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Engineering as a Documentation Collection

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

## Single Responsibility

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

## Engineering Principles

> **semantic_type:** `guiding_principles`
> **scope:** Repository-wide engineering principles that govern all implementation decisions — values that survive technology changes
> **out_of_scope:** Technology-specific rules, framework guidelines, coding standards, feature-specific principles
> **contributes:** Provides the stable judgment foundation for all engineering decisions across the repository
> **relationships:** Derived from Architecture(05) and Vision(01); referenced by Technology Selection and all engineering standards
> **responsibilities:** Define principles that remain true even as specific technologies and features change
> **generation_rules:** Extract from architectural intent and product philosophy; express as stable values; use memorable phrasing; keep count manageable
> **enhancement_rules:** Add principles when new engineering concerns emerge; remove obsolete principles; preserve core intent
> **validation_rules:** Principles are technology-independent; stable across features; memorable; actionable when an engineering decision is ambiguous
> **audit_rules:** Must exist; must not reference specific technologies; must be evaluable against real engineering decisions; must be stable

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
## Engineering Principles

> [metadata block]

[1 paragraph explaining how principles guide engineering decisions]

[bulleted list of principles, each as a memorable phrase with 1–sentence explanation]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Architecture(05), Vision(01)

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

## Technology Selection

> **semantic_type:** `rationale`
> **scope:** Why specific technologies were selected — engineering rationale for language, framework, database, and tool choices
> **out_of_scope:** Implementation details, API usage, code examples, feature-specific technology decisions, migration plans
> **contributes:** Justifies every technology choice so engineers and AI systems understand the reasoning, not just the list
> **relationships:** Derived from Architecture(05) and External Context; referenced by Implementation for technology conformance
> **responsibilities:** Explain the engineering rationale behind each technology choice; ensure rationale is stable and auditable
> **generation_rules:** Start from architectural constraints and external context; explain "why" for each choice; group by engineering concern
> **enhancement_rules:** Strengthen rationale when new context emerges; remove outdated justification; preserve decision stability
> **validation_rules:** Every technology choice has rationale; rationale is engineering-focused, not business-focused; rationale is stable
> **audit_rules:** Must exist; must include rationale for each choice; must not be a bare list; must connect to Architecture and External Context

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** extensive
> **diagram_requirements:** none

```markdown
## Technology Selection

> [metadata block]

### [Technology Category]

[1 paragraph explaining why this technology was chosen,
connected to architectural constraints and external context]

[repeat for each technology category: Language, Framework, Database, Tooling, etc.]
```

**Required subsections:** One per technology category (Language, Framework, Database, Tooling as applicable)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Architecture(05), External Context, Constraints

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

## External Context Application

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

## Quality Requirements

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
