# README Standard

## Purpose

This document defines the standard for README documentation within the engineering documentation ecosystem.

A README is the primary entry point to a repository.

Its purpose is to provide a concise overview of the project, explain its purpose, describe how the repository is organized, and guide readers toward detailed documentation.

The README introduces the project.

It does not replace project documentation.

---

## Required Sections

A README should typically include:

* Project Name
* Short Description
* Overview
* Purpose
* Key Capabilities
* Repository Structure
* Documentation Structure
* Getting Started
* Installation
* Build
* Usage
* Configuration
* Development
* Contributing
* License (if applicable)

Projects may extend these sections where appropriate.

---

## Goals

README aims to:

* Let a new visitor understand what the project is and how to run it within minutes.
* Route deeper questions to the right documentation instead of duplicating it.
* Keep install/run instructions accurate to the finished Build.

---

## Non-Goals

The README does not attempt to replace:

* Vision documentation
* Feature documentation
* Design documentation
* Architecture documentation
* Engineering documentation
* API documentation
* External Context
* Source code documentation

Its responsibility is to introduce the repository and direct readers to the appropriate documentation.

---

## Success Criteria

A README is successful when:

* A new developer understands the repository within a few minutes.
* AI systems can identify the repository purpose quickly.
* Readers know where to find detailed documentation.
* Repository setup is straightforward.
* Documentation remains organized rather than duplicated.
* The README serves as the canonical entry point to the repository.

---

## Responsibilities

A README is responsible for:

* Introducing the project
* Explaining the repository purpose
* Providing high-level project context
* Helping readers navigate documentation
* Explaining how to build and use the project
* Introducing repository structure
* Listing prerequisites
* Providing quick-start guidance

The README should minimize the time required for a developer to understand the repository.

---

## Scope

A README should describe:

* Project overview
* Repository purpose
* High-level capabilities
* Repository structure
* Documentation structure
* Installation
* Build instructions
* Usage
* Configuration
* Development workflow
* Links to detailed documentation

A README should summarize.

It should not become the documentation itself.

---

## Out of Scope

A README must not contain detailed:

* Feature specifications
* Architecture documentation
* Engineering decisions
* Technical designs
* API specifications
* External Context
* Ownership rules
* Source code explanations
* Complete tutorials

These belong in dedicated documentation.

---

## Inputs

A README may reference:

* Vision
* Feature documentation
* Architecture documentation
* Engineering documentation
* External Context
* Project configuration

The README summarizes these documents.

It does not replace them.

---

## Outputs

A README provides entry points to:

* Vision
* Features
* Design
* Architecture
* Engineering
* External Context
* Development Guides
* Build Instructions
* Contribution Guides

---

## Traceability

The README should reference project documentation without duplicating it.

```text
README
    │
    ├── Vision
    ├── Features
    ├── Design
    ├── Architecture
    ├── Engineering
    ├── External Context
    └── Development
```

The README is a navigation document.

---

## Relationships

| Document         | Relationship                          |
| ---------------- | ------------------------------------- |
| Vision           | Summarizes project purpose            |
| Features         | Summarizes capabilities               |
| Design           | References design documentation       |
| Architecture     | References architecture documentation |
| Engineering      | References implementation standards    |
| External Context | References external technologies      |
| Implementation   | Points to executable entry points     |

---

## Required Characteristics

A README should be:

* Welcoming to a first-time visitor
* Accurate to the current Build
* Current — not describing a removed or planned feature as if it exists
* Concise
* Navigable — routes deeper questions to the right documentation
* Actionable — install/run instructions actually work

---

## Audit Rules

An audit should verify:

* The project purpose is immediately understandable.
* Repository responsibilities are clearly explained.
* Documentation navigation exists.
* Installation guidance is appropriate.
* Repository structure is described.
* README does not duplicate detailed documentation.
* Ecosystem relationships are explained when applicable.
* Links to documentation remain accurate.

README bloat should be reported as a standards violation.

---

## Validation Rules

A README is considered valid if:

* The project purpose is clear.
* Repository responsibilities are explained.
* Documentation navigation is present.
* Installation instructions exist where applicable.
* Repository structure is described.
* Links to detailed documentation are available.
* No detailed implementation documentation has been duplicated.

---

## Generation Rules

When generating a README:

* Introduce the project before explaining usage.
* Summarize rather than duplicate documentation.
* Keep the overview concise.
* Provide clear navigation.
* Explain repository structure.
* Highlight ecosystem relationships.
* Focus on developer onboarding.

---

## Enhancement Rules

When enhancing a README:

* Improve readability.
* Improve navigation.
* Remove duplicated documentation.
* Clarify project purpose.
* Improve onboarding.
* Keep documentation references current.
* Preserve project identity.

Enhancements should simplify repository discovery rather than increase documentation volume.

---

## Summary

The README is the front door of the repository.

Its responsibility is to provide orientation, onboarding, and navigation rather than comprehensive technical documentation.

A well-designed README enables both humans and AI systems to quickly understand the repository, locate detailed documentation, and begin contributing without overwhelming them with implementation details.

---

## Common Mistakes

Examples of incorrect README content include:

* Copying complete feature documentation.
* Embedding architecture documents.
* Including engineering decision records.
* Documenting every API.
* Explaining every source file.
* Duplicating External Context.
* Becoming a project wiki.

The README should remain an entry point rather than comprehensive documentation.

---

## Documentation Folder

The README lives at the repository root:

```text
README.md
```

There is exactly one README per repository. It is not placed under `docs/raw/`.

---

## Repository Structure

The README should explain the purpose of major directories.

Examples:

* docs/
* src/
* tests/
* scripts/
* examples/

Only high-level descriptions are required.

---

## Usage

Written once at repository creation, updated whenever the repository's purpose, structure, or getting-started steps change. Use `samgraha audit --domain readme` to check the README has a title and a getting-started section before merging.

## Related

- [Vision Standard](01-vision-standards.md) — README summarizes project purpose from Vision
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Repository Overview

The README should explain:

* What the repository contains.
* What role it plays within the ecosystem.
* How it relates to other repositories.
* Who should use it.

Readers should understand the repository within a few minutes.

---

## Documentation Navigation

The README should guide readers toward detailed documentation.

Example navigation:

```text
Vision
Features
Architecture
Engineering
External Context
API
Development Guide
```

The README should act as a documentation index.

---

## Getting Started

The README should provide enough information to begin using or contributing to the repository.

Examples include:

* Prerequisites
* Installation
* Build
* Running the project
* Development environment
* Testing

Detailed procedures should be placed in dedicated guides.

---

## Ecosystem Context

If the repository belongs to a larger ecosystem, the README should explain:

* Repository role
* Upstream dependencies
* Downstream consumers
* Related repositories

This provides context without duplicating architecture documentation.

---

## Quality Requirements

A README should:

* Be concise
* Be easy to navigate
* Introduce the repository quickly
* Reference detailed documentation
* Avoid unnecessary duplication
* Remain beginner friendly
* Support both humans and AI systems

---
