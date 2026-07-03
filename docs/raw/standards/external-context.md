# External Context Standard

This section details the External Context Standard.

## Purpose

This document defines the standard for External Context Documentation within the engineering documentation ecosystem.

External Context Documentation describes knowledge that exists **outside the current repository** but is required to correctly understand, design, implement, or maintain the repository.

Unlike traditional dependency documentation, External Context does not document package dependencies.

Instead, it documents **knowledge dependencies**.

Each External Context document describes one external system, library, platform, protocol, or project that materially influences the repository.

Projects may contain zero, one, or many External Context documents depending on their dependencies.

---

# Responsibilities

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

# Scope

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

# Out of Scope

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

# External Context as an Atomic Collection

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

# Atomicity

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

# Dependency Types

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

# Inputs

External Context derives from:

* External documentation
* Project documentation
* Architecture
* Engineering decisions

External Context should summarize relevant knowledge.

It should not replace the original documentation.

---

# Outputs

External Context provides guidance for:

* Features
* Feature Design
* Architecture
* Feature Technical Design
* Engineering
* Implementation

Any document may reference External Context rather than duplicating external knowledge.

---

# Traceability

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

# Relationships

| Document                 | Relationship                               |
| ------------------------ | ------------------------------------------ |
| Vision                   | Usually independent                        |
| Feature                  | May reference functional capabilities      |
| Feature Design           | May reference design constraints           |
| Architecture             | May reference architectural constraints    |
| Feature Technical Design | Frequently references External Context     |
| Engineering              | Frequently references technology rationale |

---

# Knowledge Dependency Principles

External Context should:

* Explain why the dependency exists.
* Explain which capabilities are used.
* Explain architectural constraints.
* Explain implementation implications.
* Reference authoritative documentation.
* Avoid duplication.

The purpose is understanding, not documentation ownership.

---

# Documentation Ownership

External Context does **not** own the external documentation.

It should:

* summarize
* explain relevance
* define dependency boundaries
* reference the authoritative source

If the external project changes, External Context should be updated accordingly.

---

# Technology Selection

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

# Quality Requirements

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

# Validation Rules

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

# Common Mistakes

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

# Generation Rules

When generating External Context:

* Create one document per dependency.
* Explain why the dependency exists.
* Explain how it influences the repository.
* Reference authoritative documentation.
* Keep documentation concise.
* Avoid duplication.
* Focus on repository relevance rather than completeness.

---

# Enhancement Rules

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

# Audit Rules

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

# Success Criteria

External Context Documentation is successful when:

* Engineers understand why external dependencies exist.
* AI systems understand how external systems influence implementation.
* Internal documentation avoids duplicating external knowledge.
* Repository-specific dependency decisions are clear.
* Contributors know where authoritative documentation resides.
* External changes can be accommodated without rewriting internal documentation.

---

# Non-Goals

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

# Summary

External Context Documentation is a collection of atomic knowledge documents that describe external dependencies required to understand and implement a repository.

Its purpose is not to replace external documentation, but to explain **why a dependency exists, how it influences the repository, and where the authoritative knowledge resides**, allowing project documentation to remain concise, maintainable, and free of unnecessary duplication.

---

# Documentation Folder

External Context documents live under:

```text
docs/raw/external-context/
```
