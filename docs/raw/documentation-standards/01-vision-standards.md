# Vision Standard

## Table of Contents
- [Purpose](#purpose)
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
- [Technology Independence](#technology-independence)
- [Product Philosophy](#product-philosophy)
- [Guiding Principles](#guiding-principles)
- [Quality Requirements](#quality-requirements)

---


## Purpose

This document defines the standard for Vision documentation within the engineering documentation ecosystem.

A Vision document establishes the long-term purpose, direction, and identity of a product or repository.

It defines **why** the product exists.

It does not define implementation, architecture, or engineering decisions.

All downstream documentation ultimately derives from the Vision.

---

## Required Sections

Every Vision document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Purpose | `purpose` | ✓ | Overview, Summary |
| Vision | `vision_statement` | ✓ | Long-Term Vision, The Vision |
| Problem | `problem` | ✓ | Problem Statement, The Problem |
| Solution | `solution` | ✓ | The Solution, Our Solution |
| Target Audience | `target_audience` | ✓ | Audience, Who Is This For |
| Platform Pillars | `pillars` | | Pillars, Foundations, Core Pillars |
| Philosophy | `philosophy` | | Product Philosophy, Design Philosophy |
| Guiding Principles | `guiding_principles` | | Principles, Core Principles |
| Success Criteria | `success_criteria` | | Acceptance Criteria, Definition of Done |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

A Vision document aims to:

* Give every downstream document a single, stable source of "why" to trace back to.
* Let a new contributor understand product intent in minutes, without reading code.
* Let engineers evaluate a proposed feature or architecture change against a stated purpose.
* Outlive individual features, technology choices, and implementation cycles.

---

## Non-Goals

The Vision does not attempt to define:

* Product requirements
* Feature specifications
* User stories
* Technical architecture
* Technology stack
* Implementation standards
* Source code organization

These responsibilities belong to other documentation standards.

---

## Success Criteria

A Vision document is successful when:

* Engineers understand the long-term purpose of the project.
* Product decisions can be evaluated against the Vision.
* Features naturally derive from the Vision.
* Architecture supports the Vision without redefining it.
* Engineering decisions remain aligned with product goals.
* AI systems can infer product intent without reading implementation documents.

---

## Responsibilities

A Vision document is responsible for defining:

* Product purpose
* Long-term direction
* Core objectives
* Intended value
* Target users or consumers
* Guiding principles
* Product identity

The Vision provides the strategic foundation for every subsequent engineering decision.

---

## Scope

A Vision document should describe:

* Why the product exists
* What problem it intends to solve
* The long-term purpose of the project
* The intended value delivered
* The desired future state
* Product philosophy
* Product principles
* Success vision

The Vision should remain stable throughout the product lifecycle.

---

## Out of Scope

A Vision document must not describe:

* Features
* User workflows
* UI layouts
* Architecture
* Components
* Technology choices
* Programming languages
* Frameworks
* Databases
* APIs
* Algorithms
* Build systems
* Source code
* Library selection
* Implementation details

These belong in downstream documentation.

---

## Inputs

A Vision document may consider:

* Business objectives
* Product goals
* Market needs
* User problems
* Organizational direction

The Vision should not depend on implementation documentation.

---

## Outputs

A Vision document provides direction for:

* Feature documentation
* Feature Design
* Architecture
* Engineering Decisions
* Product Roadmaps
* Documentation audits

Every Feature should be traceable to the Vision.

---

## Traceability

The Vision is the root of the documentation hierarchy.

```text
Tier 0: Vision (Purpose)
    ├──→ Tier 1: Features (What)
    │         └──→ Tier 2: Feature Design (How — UX)
    │                    └──→ Tier 3: Feature Technical Design ←──┐
    ├──→ Tier 1: Philosophy (Principles)                          │
    ├──→ Tier 2: Architecture (Structure) ────────────────────────┤
    └──→ Tier 2: External Context (Constraints) ──────────────────┤
                                                                   ↓
                                              Tier 3: Feature Technical Design
                                                          ↓
                                              Tier 4: Engineering (Standards)
                                                          ↓
                                              Tier 5: Implementation (Code)
```

No downstream document should contradict the Vision.

---

## Relationships

| Document         | Relationship                                    |
| ---------------- | ----------------------------------------------- |
| Features         | Derived from Vision                             |
| Feature Design   | Supports Vision through Features                |
| Architecture     | Realizes Vision through system organization     |
| Engineering      | Supports Vision through implementation standards |
| External Context | Independent                                     |
| README           | Summarizes the repository using the Vision      |

---

## Required Characteristics

A Vision document should be:

* Stable
* Technology independent
* Product focused
* Long-term oriented
* Implementation independent
* Easy to understand
* Concise
* Inspirational
* Actionable

---

## Audit Rules

An audit should verify:

* The Vision explains why the product exists.
* The document is technology independent.
* No implementation details appear.
* Product philosophy is present.
* Guiding principles are documented.
* Downstream documentation remains consistent with the Vision.
* The Vision remains stable and future-oriented.

Any implementation detail should be reported as a standards violation.

---

## Validation Rules

A Vision document is considered valid if:

* The purpose is clearly defined.
* The long-term objective is explicit.
* Product philosophy is documented.
* Guiding principles are identified.
* No implementation details are present.
* No architectural decisions are described.
* No feature specifications are included.
* The document can guide future feature development.

---

## Generation Rules

When generating a Vision document:

* Focus on purpose before capability.
* Explain the problem before the solution.
* Describe long-term value.
* Avoid implementation language.
* Write from the product perspective.
* Prefer stable concepts over temporary goals.
* Keep technology decisions separate.

---

## Enhancement Rules

When enhancing a Vision document:

* Improve clarity.
* Strengthen long-term direction.
* Remove implementation leakage.
* Remove architectural discussion.
* Eliminate duplicated feature descriptions.
* Improve consistency with product philosophy.
* Preserve existing intent.

Enhancements should refine—not redefine—the Vision.

---

## Summary

The Vision is the highest-level engineering artifact within the documentation ecosystem.

Its responsibility is to communicate **why** the product exists and the long-term direction it should follow.

Every downstream document should refine the Vision without redefining it, ensuring that engineering decisions remain aligned with enduring product intent rather than temporary implementation choices.

---

## Common Mistakes

Examples of incorrect Vision content include:

* Listing application features.
* Explaining UI behavior.
* Discussing databases.
* Selecting programming languages.
* Describing frameworks.
* Explaining APIs.
* Introducing architecture diagrams.
* Including implementation plans.

These belong in downstream documentation.

---

## Documentation Folder

Vision documents live under:

```text
docs/raw/vision/
```

---

## Usage

Vision is written once per product and revised rarely — product owners author it; everyone else reads it before writing Features, since every Feature must trace back to the Vision. Use `samgraha compile --domain vision` to validate structure, and `samgraha search --domain vision` (or the MCP `search` tool) to pull Vision context into an AI-assisted feature-writing session.

## Related

- [Feature Standard](04-feature-standards.md) — every Feature derives from Vision
- [Philosophy Standard](02-philosophy-standards.md) — inspires Vision's guiding principles
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Technology Independence

The Vision should remain independent of implementation technologies.

The following should generally never appear:

* Programming languages
* Frameworks
* Libraries
* Databases
* Infrastructure
* Build systems
* Cloud providers

Technology decisions evolve.

Vision should not.

---

## Product Philosophy

A Vision should communicate the philosophy that guides product decisions.

Examples include:

* Documentation First
* Privacy First
* Local First
* Offline First
* Accessibility First
* Developer Experience
* Simplicity
* Reliability

These describe values rather than implementation.

---

## Guiding Principles

Vision should define enduring principles that influence future decisions.

Examples:

* Human-centered design
* AI-assisted engineering
* Open standards
* Predictable behavior
* Long-term maintainability

Principles should remain stable even as features evolve.

---

## Quality Requirements

A Vision document should:

* Clearly explain why the product exists.
* Communicate long-term direction.
* Inspire engineering decisions.
* Remain understandable without technical knowledge.
* Avoid implementation discussion.
* Remain stable over time.
* Provide sufficient guidance for feature definition.

---
