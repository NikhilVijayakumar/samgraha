# External Context Standard

## Table of Contents
- [Purpose](#purpose)
- [Integration Contract](#integration-contract)
- [Constraints](#constraints)
- [Dependencies](#dependencies)
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
- [Usage](#usage)
- [Related](#related)
- [External Context as an Atomic Collection](#external-context-as-an-atomic-collection)
- [Atomicity](#atomicity)
- [Dependency Types](#dependency-types)
- [Internal Dependencies](#internal-dependencies)
- [External Platforms](#external-platforms)
- [Standards and Protocols](#standards-and-protocols)
- [Domain Knowledge](#domain-knowledge)
- [Knowledge Dependency Principles](#knowledge-dependency-principles)
- [Documentation Ownership](#documentation-ownership)
- [Technology Selection](#technology-selection)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why External Context Documentation exists — its role in capturing knowledge dependencies that live outside the repository but materially influence implementation
> **out_of_scope:** Internal architecture, feature specifications, package management, implementation details, API reference manuals
> **contributes:** Establishes the root intent for all External Context sections and distinguishes knowledge dependencies from traditional dependency documentation
> **relationships:** Derived from project integrations; feeds Feature Technical Design(10) and Engineering(07); referenced by Architecture(05) for constraint awareness
> **responsibilities:** Define External Context Documentation's reason for being and its boundary within the documentation ecosystem
> **generation_rules:** State what External Context is; explain what it documents and what it does not; distinguish from package dependency documentation
> **enhancement_rules:** Strengthen scope boundaries; remove overlap with Engineering or Architecture standards; keep stable over time
> **validation_rules:** Purpose is clearly defined; no internal architecture present; boundary with other standards is explicit
> **audit_rules:** Must exist; must not contain internal design or implementation; must define what External Context is and is not

This document defines the standard for External Context Documentation within the engineering documentation ecosystem.

External Context Documentation describes knowledge that exists **outside the current repository** but is required to correctly understand, design, implement, or maintain the repository.

Unlike traditional dependency documentation, External Context does not document package dependencies.

Instead, it documents **knowledge dependencies**.

Each External Context document describes one external system, library, platform, protocol, or project that materially influences the repository.

Projects may contain zero, one, or many External Context documents depending on their dependencies.

---

## Integration Contract

> **semantic_type:** `integration_contract`
> **scope:** The formal interface agreement between the repository and the external system — APIs, protocols, data formats, authentication, and communication patterns
> **out_of_scope:** Internal implementation of the integration, code-level details, build configuration, library versioning
> **contributes:** Makes the external system's contract explicit so downstream design and engineering can implement integrations correctly
> **relationships:** Consumed by Feature Technical Design(10) and Engineering(07); references Constraints; may be referenced by Architecture(05)
> **responsibilities:** Define the integration surface: endpoints, protocols, data formats, authentication mechanisms, error behaviors, and versioning policies
> **generation_rules:** Identify the integration surface from external documentation; describe contract elements in implementation-neutral terms; reference authoritative API docs
> **enhancement_rules:** Update when external contracts change; clarify ambiguous contract elements; remove internal implementation details that leaked in
> **validation_rules:** Contract covers all integration points used by the repository; external documentation is referenced; no internal implementation details present
> **audit_rules:** Must exist for each external dependency with a programmatic interface; must reference authoritative documentation; must not contain code or implementation specifics

*(To be written by the integrating engineer. This section defines the formal interface contract with the external system.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Limitations and boundaries imposed by the external system — rate limits, platform requirements, protocol restrictions, licensing, compliance obligations
> **out_of_scope:** Internal project constraints, design decisions, implementation trade-offs, technology selection rationale
> **contributes:** Makes external constraints visible so downstream standards can design within known boundaries
> **relationships:** May reference or be referenced by Feature(04) Constraints; feeds Architecture(05) and Feature Design(09); referenced by Engineering(07)
> **responsibilities:** Document every constraint the external system imposes on the repository, including functional, performance, legal, and compliance constraints
> **generation_rules:** Identify constraints from external documentation and platform characteristics; categorize by type (functional, performance, legal, compliance); state each as a clear limitation
> **enhancement_rules:** Add constraints when external systems change; remove constraints that no longer apply; clarify ambiguous constraint descriptions
> **validation_rules:** Constraints are real, sourced from the external system, clearly stated, and categorized; no internal design decisions disguised as constraints
> **audit_rules:** Must exist if the external system imposes constraints; must not contain internal design decisions; must be sourced from external documentation

*(To be written by the integrating engineer. This section defines the limitations imposed by the external dependency.)*

---

## Dependencies

> **semantic_type:** `dependencies`
> **scope:** What this external dependency itself depends on — transitive requirements, platform prerequisites, required companion systems
> **out_of_scope:** Internal project dependencies, package version management, build-time dependencies, development tooling
> **contributes:** Makes transitive dependency chains visible so integration planning accounts for the full dependency surface
> **relationships:** May reference other External Context documents; feeds Architecture(05) and Engineering(07); connects to dependency resolution workflows
> **responsibilities:** List transitive dependencies required by the external system, including platform prerequisites and required companion services
> **generation_rules:** Identify transitive requirements from external documentation; distinguish runtime from build-time dependencies; note version or platform requirements
> **enhancement_rules:** Add transitive dependencies when discovered; remove dependencies that are not relevant to the repository; clarify dependency criticality
> **validation_rules:** Dependencies are real and relevant to the repository; dependency criticality is noted; no internal dependencies disguised as external
> **audit_rules:** Must exist if the external system has transitive requirements relevant to the repository; must not list internal project dependencies

*(To be written by the integrating engineer. This section defines what the external dependency itself requires.)*

---

## Required Sections

Every External Context document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Purpose | `purpose` | ✓ | Overview, Summary |
| Integration Contract | `integration_contract` | ✓ | Contract, API Contract, Interface |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements |
| Dependencies | `dependencies` | | Dependency, Depends On |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

External Context Documentation aims to:

* Document why each external dependency exists and what it's used for.
* Prevent tribal knowledge about integrations from living only in one engineer's head.
* Make external constraints traceable to the features and technical designs they affect.

---

## Non-Goals

External Context does not define:

* Internal Features
* Internal Design
* Internal Architecture
* Engineering Decisions
* Package Management
* Dependency Versions
* Source Code
* API Reference Manuals

These responsibilities belong to other documentation standards or to the external project's own documentation.

---

## Success Criteria

External Context Documentation is successful when:

* Engineers understand why external dependencies exist.
* AI systems understand how external systems influence implementation.
* Internal documentation avoids duplicating external knowledge.
* Repository-specific dependency decisions are clear.
* Contributors know where authoritative documentation resides.
* External changes can be accommodated without rewriting internal documentation.

---

## Responsibilities

External Context Documentation is responsible for describing:

* External systems
* External libraries
* Internal shared projects
* Shared frameworks
* Shared protocols
* External platforms
* Integration contracts
* Knowledge dependencies
* Constraints introduced by external systems
* Referenced documentation

External Context explains **what external knowledge contributors must understand**.

It does not replace the documentation owned by the external project.

---

## Scope

External Context may describe:

* Internal shared libraries
* Internal frameworks
* Shared runtime platforms
* Shared protocols
* External APIs
* Operating system services
* Platform capabilities
* AI platforms
* Communication standards
* File formats
* Industry specifications
* Regulatory standards

Each document should describe one external dependency.

---

## Out of Scope

External Context must not describe:

* Internal repository architecture
* Internal feature specifications
* Source code
* Package manifests
* Build configuration
* Version management
* Library APIs
* Installation procedures
* Complete third-party documentation

External Context should summarize and reference.

It should not duplicate.

---

## Inputs

External Context derives from:

* External documentation
* Project documentation
* Architecture
* Engineering decisions

External Context should summarize relevant knowledge.

It should not replace the original documentation.

---

## Outputs

External Context provides guidance for:

* Features
* Feature Design
* Architecture
* Feature Technical Design
* Engineering
* Implementation

Any document may reference External Context rather than duplicating external knowledge.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How External Context connects to the documentation hierarchy — which downstream standards consume external knowledge and how
> **out_of_scope:** Internal implementation traceability, version history, change tracking, test traceability
> **contributes:** Makes External Context's influence visible and verifiable across the documentation ecosystem
> **relationships:** Informed by external documentation; feeds Feature Technical Design(10), Engineering(07), and Architecture(05); may be referenced by Feature(04) and Feature Design(09)
> **responsibilities:** Show which documentation domains consume External Context; assert that downstream standards reference rather than duplicate external knowledge
> **generation_rules:** Use the tier diagram showing External Context's lateral influence; list which standards may reference External Context; state the non-duplication rule
> **enhancement_rules:** Update the diagram when new standards reference External Context; ensure influence paths remain accurate
> **validation_rules:** Influence paths are complete; no orphaned references; non-duplication rule is stated
> **audit_rules:** Must exist; must include tier diagram; must list consuming standards; must state that External Context informs but does not redefine downstream documentation

External Context supports multiple documentation domains.

```text
Vision

↓

Features

↓

Feature Design

↓

Architecture

↓

Feature Technical Design

↓

Engineering

         ↑

 External Context
```

External Context informs documentation.

It does not redefine it.

---

## Relationships

| Document                 | Relationship                               |
| ------------------------ | ------------------------------------------ |
| Vision                   | Usually independent                        |
| Feature                  | May reference functional capabilities      |
| Feature Design           | May reference design constraints           |
| Architecture             | May reference architectural constraints    |
| Feature Technical Design | Frequently references External Context     |
| Engineering              | Frequently references technology rationale |

---

## Required Characteristics

External Context Documentation should be:

* Accurate to the actual external system, not aspirational
* Current — reviewed when the dependency's contract changes
* Traceable to the feature/technical design it affects
* Ownership-clear (who's responsible for tracking upstream changes)
* Non-duplicative of the dependency's own documentation

---

## Audit Rules

An audit should verify:

* External dependencies are documented only when necessary.
* One document describes one dependency.
* Dependency purpose is clearly explained.
* Constraints are documented.
* External documentation is referenced rather than duplicated.
* Repository relevance is obvious.
* No internal architecture has leaked into External Context.

External Context quality is evaluated individually and across the documentation collection.

---

## Validation Rules

External Context is considered valid when:

* One document describes one external dependency.
* The dependency is clearly identified.
* The purpose of the dependency is explained.
* Constraints are documented.
* Repository usage is described.
* External documentation is referenced where appropriate.
* Duplicate documentation has been avoided.

Validation applies independently to each External Context document.

---

## Generation Rules

When generating External Context:

* Create one document per dependency.
* Explain why the dependency exists.
* Explain how it influences the repository.
* Reference authoritative documentation.
* Keep documentation concise.
* Avoid duplication.
* Focus on repository relevance rather than completeness.

---

## Enhancement Rules

When enhancing External Context:

* Improve clarity.
* Improve dependency justification.
* Remove duplicated documentation.
* Strengthen repository relevance.
* Improve references.
* Split oversized documents.
* Preserve dependency intent.

External Context should become easier to understand and maintain.

---

## Summary

External Context Documentation is a collection of atomic knowledge documents that describe external dependencies required to understand and implement a repository.

Its purpose is not to replace external documentation, but to explain **why a dependency exists, how it influences the repository, and where the authoritative knowledge resides**, allowing project documentation to remain concise, maintainable, and free of unnecessary duplication.

---

## Common Mistakes

Examples include:

* Copying vendor documentation.
* Documenting package manifests.
* Explaining every API.
* Duplicating Architecture.
* Duplicating Engineering.
* Describing implementation.
* Mixing multiple unrelated dependencies.

These should be reported during audits.

---

## Documentation Folder

External Context documents live under:

```text
docs/raw/external-context/
```

---

## Usage

Written by whoever integrates a new external dependency, one document per dependency, only when the dependency materially influences implementation. Use `samgraha search --domain external-context` to check whether a dependency is already documented before writing a new one, avoiding duplicate External Context files for the same system.

## Related

- [Feature Technical Standard](10-feature-technical-standards.md) — frequently references External Context
- [Engineering Standard](07-engineering-standards.md) — frequently references technology rationale
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## External Context as an Atomic Collection

External Context Documentation is organized as a collection of independent documents.

Example:

```text
external-context/

    astra.md

    prana.md

    mcp.md

    electron.md

    sqlite.md
```

Each document describes one external dependency.

Projects may have:

* No External Context
* One External Context
* Multiple External Context documents

depending on project requirements.

---

## Atomicity

Every External Context document should describe one external dependency.

Examples include:

* Astra
* Prana
* MCP
* Electron
* SQLite
* Google Drive
* OAuth

Large documents should be decomposed into multiple focused documents.

---

## Dependency Types

External Context may describe:

## Internal Dependencies

Projects maintained within the same engineering ecosystem.

Examples:

* Astra
* Prana
* Prati
* Saṃgraha

---

## External Platforms

External systems that influence implementation.

Examples:

* Electron
* Android
* Windows
* Azure

---

## Standards and Protocols

Shared specifications.

Examples:

* MCP
* OAuth
* OpenAPI
* JSON Schema

---

## Domain Knowledge

Knowledge required to correctly implement the project.

Examples:

* Accessibility Guidelines
* Security Standards
* Regulatory Requirements

---

## Knowledge Dependency Principles

External Context should:

* Explain why the dependency exists.
* Explain which capabilities are used.
* Explain architectural constraints.
* Explain implementation implications.
* Reference authoritative documentation.
* Avoid duplication.

The purpose is understanding, not documentation ownership.

---

## Documentation Ownership

External Context does **not** own the external documentation.

It should:

* summarize
* explain relevance
* define dependency boundaries
* reference the authoritative source

If the external project changes, External Context should be updated accordingly.

---

## Technology Selection

External Context should document technologies only when they materially influence implementation.

Examples:

Appropriate:

* Astra Framework
* Prana Runtime
* MCP
* Electron
* SQLite

Generally unnecessary:

* React
* Axios
* Lodash
* Zod

Widely understood libraries should not require External Context unless project-specific conventions exist.

---

## Quality Requirements

External Context should be:

* Atomic
* Concise
* Relevant
* Traceable
* Maintainable
* Repository focused
* Easy to reference
* Free of duplication

Each document should justify why the dependency matters.

---
