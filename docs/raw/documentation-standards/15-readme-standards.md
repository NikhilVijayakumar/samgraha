# README Standard

## Table of Contents
- [Purpose](#purpose)
- [Project Name](#project-name)
- [Short Description](#short-description)
- [Overview](#overview)
- [Key Capabilities](#key-capabilities)
- [Documentation Structure](#documentation-structure)
- [Installation](#installation)
- [Build](#build)
- [Configuration](#configuration)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)
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
- [Repository Structure](#repository-structure)
- [Usage](#usage)
- [Related](#related)
- [Repository Overview](#repository-overview)
- [Documentation Navigation](#documentation-navigation)
- [Getting Started](#getting-started)
- [Ecosystem Context](#ecosystem-context)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why README Documentation exists — its role as the primary entry point to a repository within the documentation ecosystem
> **out_of_scope:** Detailed feature specifications, architecture documentation, engineering decisions, API documentation, source code explanations
> **contributes:** Establishes the root intent that all README sections derive from and constrains what a README may contain
> **relationships:** Derived from the ecosystem need for a single entry point; referenced by every other documentation standard as the navigation root
> **responsibilities:** Define README Documentation's reason for being, its boundary with other standards, and its role as an entry point rather than comprehensive documentation
> **generation_rules:** State what README Documentation is; explain what it defines and what it does not; reference the broader documentation ecosystem
> **enhancement_rules:** Strengthen clarity of scope boundaries; remove overlap with downstream standards; keep stable over time
> **validation_rules:** Purpose is clearly defined; no detailed documentation present; boundary with other standards is explicit
> **audit_rules:** Must exist; must not contain feature lists or implementation details; must define what README Documentation is and is not

This document defines the standard for README documentation within the engineering documentation ecosystem.

A README is the primary entry point to a repository.

Its purpose is to provide a concise overview of the project, explain its purpose, describe how the repository is organized, and guide readers toward detailed documentation.

The README introduces the project.

It does not replace project documentation.

---

## Project Name

> **semantic_type:** `project_name`
> **scope:** The canonical name of the project as it appears in the repository, package manifests, and documentation
> **out_of_scope:** Version numbers, codenames, internal aliases, marketing names, organization names
> **contributes:** Establishes the project identity that all other README sections refer back to
> **relationships:** Derived from Vision(01); referenced by Repository Overview and Documentation Navigation
> **responsibilities:** Provide the single authoritative name for the project used across all documentation and tooling
> **generation_rules:** Use the name from package manifests or repository metadata; keep it consistent across all documentation; do not abbreviate
> **enhancement_rules:** Update only when the project is officially renamed; ensure consistency across all references
> **validation_rules:** Name matches package manifests; name is consistent across documentation; no conflicting names present
> **audit_rules:** Must exist; must match the name in package.json, setup.py, or equivalent; must not be ambiguous

*(To be written by the domain expert. This section provides the project's canonical name.)*

---

## Short Description

> **semantic_type:** `short_description`
> **scope:** A one- or two-sentence summary of what the project does, suitable for search results and social previews
> **out_of_scope:** Feature lists, installation instructions, architecture details, lengthy explanations
> **contributes:** Gives readers an immediate understanding of the project's purpose before they read further
> **relationships:** Derived from Vision(01); feeds Overview and Repository Overview; referenced by AI systems for quick classification
> **responsibilities:** Provide a concise, accurate summary that enables rapid project assessment
> **generation_rules:** Write one to two sentences; state what the project does and who it is for; avoid jargon; keep under 200 characters when possible
> **enhancement_rules:** Update when the project scope changes; keep concise; remove marketing language
> **validation_rules:** Description is accurate; description is concise; description enables quick understanding; no feature lists present
> **audit_rules:** Must exist; must be one to two sentences; must not exceed 200 characters; must not contain installation instructions

*(To be written by the domain expert. This section provides a one-sentence summary of the project.)*

---

## Overview

> **semantic_type:** `overview`
> **scope:** A high-level narrative of what the project is, the problem it solves, and why it exists
> **out_of_scope:** Detailed feature specifications, implementation details, architecture diagrams, API documentation
> **contributes:** Provides the conceptual foundation that readers need before diving into specific sections
> **relationships:** Derived from Vision(01); feeds Key Capabilities and Repository Overview; referenced by Getting Started
> **responsibilities:** Explain the project's purpose, the problem it addresses, and its approach at a high level
> **generation_rules:** Write a short paragraph; explain the problem and solution; keep high-level; reference Vision for deeper context
> **enhancement_rules:** Update when project scope changes; keep concise; remove implementation details that creep in
> **validation_rules:** Overview is accurate; overview is concise; overview explains the problem and solution; no implementation details present
> **audit_rules:** Must exist; must explain the problem the project solves; must not contain implementation details; must be concise

*(To be written by the domain expert. This section provides a high-level overview of the project.)*

---

## Key Capabilities

> **semantic_type:** `key_capabilities`
> **scope:** The main capabilities or features the project provides, listed at a high level without implementation detail
> **out_of_scope:** Feature specifications, API details, implementation approaches, technical architecture, code examples
> **contributes:** Helps readers quickly identify whether the project meets their needs
> **relationships:** Derived from Vision(01) goals; feeds Feature Documentation; referenced by Repository Overview
> **responsibilities:** List the primary capabilities the project offers in a scannable format
> **generation_rules:** List three to seven key capabilities; use short descriptive phrases; keep each capability independent; avoid technical jargon
> **enhancement_rules:** Add capabilities when scope expands; remove capabilities that are deprecated; keep the list scannable
> **validation_rules:** Capabilities are accurate; capabilities are high-level; capabilities are scannable; no implementation details present
> **audit_rules:** Must exist; must list between three and seven capabilities; must not contain implementation details; must be scannable

*(To be written by the domain expert. This section lists the project's main capabilities.)*

---

## Documentation Structure

> **semantic_type:** `documentation_structure`
> **scope:** How the project's documentation is organized, where to find detailed documentation, and how documents relate to each other
> **out_of_scope:** Detailed content of any specific document, document templates, documentation tooling configuration
> **contributes:** Enables readers to navigate from the README to the specific documentation they need
> **relationships:** References all documentation standards; feeds Documentation Navigation; connected to Documentation Folder
> **responsibilities:** Describe the documentation layout, list key documents, and explain how to navigate between them
> **generation_rules:** List the documentation directories and key files; explain the organization principle; provide navigation guidance
> **enhancement_rules:** Update when documentation structure changes; keep the list current; add new sections as documentation grows
> **validation_rules:** Documentation structure matches actual files; navigation guidance is accurate; all key documents are listed
> **audit_rules:** Must exist; must match actual documentation layout; must list all key documents; navigation guidance must be accurate

*(To be written by the domain expert. This section describes how documentation is organized.)*

---

## Installation

> **semantic_type:** `installation`
> **scope:** The steps required to install the project and its dependencies on a developer's machine
> **out_of_scope:** Build instructions, configuration details, development setup, troubleshooting guides, deployment procedures
> **contributes:** Enables developers to get the project running with minimal friction
> **relationships:** Derived from Build(14); feeds Getting Started; referenced by Development and Contributing
> **responsibilities:** Provide clear, tested installation steps that work on a clean machine
> **generation_rules:** List prerequisites first; provide step-by-step commands; verify each step works; include version requirements
> **enhancement_rules:** Update when dependencies change; add platform-specific instructions as needed; keep steps minimal
> **validation_rules:** Installation steps work on a clean machine; prerequisites are listed; commands are correct; no unnecessary steps present
> **audit_rules:** Must exist; must include prerequisites; must provide step-by-step instructions; must be tested on a clean machine

*(To be written by the domain expert. This section provides installation instructions.)*

---

## Build

> **semantic_type:** `build`
> **scope:** How to build the project from source, including build commands, prerequisites, and expected outcomes
> **out_of_scope:** CI/CD pipeline configuration, deployment procedures, release processes, build system internals
> **contributes:** Enables developers to produce a working build from source
> **relationships:** Derived from Build(14); feeds Getting Started; referenced by Development and Contributing
> **responsibilities:** Provide clear build instructions that produce a working artifact
> **generation_rules:** List build commands; state prerequisites; describe expected output; include common build targets
> **enhancement_rules:** Update when build system changes; add new build targets as needed; keep instructions minimal
> **validation_rules:** Build commands work; prerequisites are listed; expected output is described; no unnecessary complexity present
> **audit_rules:** Must exist; must provide build commands; must state prerequisites; must describe expected output

*(To be written by the domain expert. This section explains how to build the project.)*

---

## Configuration

> **semantic_type:** `configuration`
> **scope:** Configuration options, environment variables, settings files, and customization points available to users and developers
> **out_of_scope:** Default configuration values without context, internal configuration mechanisms, configuration file syntax details
> **contributes:** Enables users and developers to customize the project's behavior without reading source code
> **relationships:** Derived from Engineering(07); feeds Usage; referenced by Development and Getting Started
> **responsibilities:** List all configuration options with their purpose, default values, and valid ranges
> **generation_rules:** List configuration options by category; state defaults; explain valid values; provide examples
> **enhancement_rules:** Add options when new configuration is added; update defaults when they change; remove deprecated options
> **validation_rules:** All configuration options are documented; defaults are stated; valid values are described; examples are provided
> **audit_rules:** Must exist if the project has configuration; must list all options; must state defaults; must provide examples

*(To be written by the domain expert. This section documents configuration options.)*

---

## Development

> **semantic_type:** `development`
> **scope:** How to set up a development environment, run tests, and contribute to the project's development workflow
> **out_of_scope:** Feature implementation details, architecture decisions, coding standards details, deployment procedures
> **contributes:** Enables new contributors to become productive quickly
> **relationships:** Derived from Engineering(07); feeds Contributing; referenced by Getting Started and Repository Structure
> **responsibilities:** Describe the development workflow, testing approach, and local development setup
> **generation_rules:** Describe the development environment setup; explain how to run tests; list development commands; reference coding standards
> **enhancement_rules:** Update when workflow changes; add new development commands as needed; keep instructions current
> **validation_rules:** Development setup instructions work; test commands are correct; workflow is described; references to other docs are accurate
> **audit_rules:** Must exist; must describe development setup; must explain how to run tests; must reference coding standards

*(To be written by the domain expert. This section describes the development workflow.)*

---

## Contributing

> **semantic_type:** `contributing`
> **scope:** How to contribute to the project, including the contribution process, code review expectations, and quality standards
> **out_of_scope:** Development environment setup details, build instructions, feature implementation specifics
> **contributes:** Lowers the barrier to contribution by making the process explicit
> **relationships:** Derived from Engineering(07); references Development; feeds Related and Documentation Navigation
> **responsibilities:** Describe the contribution process, code review expectations, and quality standards for contributions
> **generation_rules:** Describe the contribution workflow; explain code review process; list quality standards; reference development setup
> **enhancement_rules:** Update when contribution process changes; add new quality standards as needed; keep process clear
> **validation_rules:** Contribution process is described; code review expectations are stated; quality standards are listed; references are accurate
> **audit_rules:** Must exist; must describe contribution workflow; must explain code review process; must list quality standards

*(To be written by the domain expert. This section explains how to contribute.)*

---

## License

> **semantic_type:** `license`
> **scope:** The project's license, copyright notices, and legal terms governing use and distribution
> **out_of_scope:** License comparison guides, legal advice, license compatibility analysis, third-party license details
> **contributes:** Makes the legal terms of use immediately clear to users and contributors
> **relationships:** May reference External Context for third-party licenses; standalone legal section
> **responsibilities:** State the project's license clearly and provide the full license text or a link to it
> **generation_rules:** State the license name; link to or include the full license text; include copyright notices if applicable
> **enhancement_rules:** Update only when the license changes; keep the license text current; ensure copyright notices are accurate
> **validation_rules:** License is stated; license text is accessible; copyright notices are present if applicable; no legal ambiguity
> **audit_rules:** Must exist if the project has a license; must state the license name; must provide access to full license text

*(To be written by the domain expert. This section specifies the project's license.)*

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

> **semantic_type:** `repository_structure`
> **scope:** The high-level organization of the repository — what the major directories contain and their purpose
> **out_of_scope:** Detailed file listings, internal module organization, implementation details, code architecture
> **contributes:** Helps readers understand where to find specific types of files and how the codebase is organized
> **relationships:** References Documentation Structure; feeds Getting Started and Development; connected to Documentation Folder
> **responsibilities:** Describe the purpose of major directories and provide a high-level map of the repository
> **generation_rules:** List major directories with one-sentence descriptions; keep descriptions high-level; focus on purpose not contents
> **enhancement_rules:** Update when directory structure changes; add new directories as needed; remove references to deleted directories
> **validation_rules:** Directory descriptions match actual structure; descriptions are high-level; no implementation details present
> **audit_rules:** Must exist; must describe major directories; descriptions must be high-level; must match actual repository structure

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

> **semantic_type:** `usage`
> **scope:** How to use the project after installation — basic commands, common workflows, and typical usage patterns
> **out_of_scope:** Advanced configuration, API documentation, internal implementation details, development workflows
> **contributes:** Enables users to start using the project immediately after installation
> **relationships:** Derived from Build(14) and Installation; feeds Getting Started; referenced by Configuration
> **responsibilities:** Provide clear, working usage examples that demonstrate the project's primary functions
> **generation_rules:** Provide basic usage examples; show common commands; include expected output; keep examples minimal
> **enhancement_rules:** Update when CLI or API changes; add new usage patterns as needed; keep examples working
> **validation_rules:** Usage examples work; commands are correct; expected output is shown; examples cover primary functions
> **audit_rules:** Must exist; must provide working examples; must cover primary functions; must show expected output

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

> **semantic_type:** `getting_started`
> **scope:** The fastest path from zero to a working project — prerequisites, installation, build, and first run in one place
> **out_of_scope:** Detailed development workflows, contribution guidelines, architecture documentation, advanced configuration
> **contributes:** Provides the onboarding entry point that minimizes time to first successful run
> **relationships:** Aggregates Installation, Build, and Usage; references Development and Contributing; feeds Repository Overview
> **responsibilities:** Provide a concise, linear path from clone to running project with all necessary steps
> **generation_rules:** List prerequisites; provide step-by-step commands; verify each step works; include expected outcomes
> **enhancement_rules:** Update when setup process changes; keep steps minimal; add platform-specific notes as needed
> **validation_rules:** Steps work on a clean machine; prerequisites are complete; commands are correct; expected outcomes are described
> **audit_rules:** Must exist; must provide a complete path from zero to running; must be tested; must not skip steps

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
