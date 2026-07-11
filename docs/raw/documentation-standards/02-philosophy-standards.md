# Philosophy Standard

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
- [Quality Requirements](#quality-requirements)

---


## Purpose

This document defines the standard for Philosophy documentation within the engineering documentation ecosystem.

Philosophy Documentation establishes the product's guiding principles, values, and the deliberate trade-offs that shape every downstream decision.

Unlike Vision, which explains **why** the product exists, Philosophy explains **how the people building it choose to think and decide**.

It does not describe features, architecture, or engineering decisions.

It shapes the judgment that produces them.

---

## Required Sections

Every Philosophy document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Principles | `guiding_principles` | ✓ | Core Principles, Design Principles |
| Values | `values` | ✓ | Core Values, What We Value |
| Trade-offs | `tradeoffs` | ✓ | Trade offs, Tradeoffs, Deliberate Trade-offs |
| Purpose | `purpose` | | Overview, Summary |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Philosophy Documentation aims to:

* Give the team a shared way to resolve ambiguous decisions.
* Make trade-offs explicit rather than implicit.
* Keep Architecture and Design principled rather than arbitrary.
* Remain true across many features and technology changes.

---

## Non-Goals

Philosophy Documentation does not define:

* Product vision or direction
* Feature specifications
* Architecture
* Technology selection

These responsibilities belong to other documentation standards.

---

## Success Criteria

Philosophy Documentation is successful when:

* Engineers can resolve an ambiguous design decision by referring back to it.
* Architecture and Design documents can cite it to justify a principle.
* It remains true even as specific features and technologies change.

---

## Responsibilities

Philosophy Documentation is responsible for defining:

* Guiding principles
* Core values
* Deliberate trade-offs
* Decision-making posture
* What the product optimizes for
* What the product deliberately does not optimize for

Philosophy provides the reasoning framework every other standard is judged against.

---

## Scope

Philosophy Documentation may describe:

* Engineering philosophy
* Product philosophy
* Design philosophy
* Trade-offs between competing goods (e.g. simplicity vs. flexibility)
* Values that resolve ambiguous decisions
* Postures toward risk, speed, and correctness

Philosophy should remain abstract enough to outlive any single feature or technology choice.

---

## Out of Scope

Philosophy Documentation must not describe:

* Product vision or long-term direction
* Feature specifications
* Architecture
* Technology selection
* Source code
* Specific technology choices

These belong to other documentation standards.

---

## Inputs

Philosophy Documentation derives from:

* Vision
* Organizational values
* Prior engineering experience and lessons learned

Philosophy should not derive from implementation details.

---

## Outputs

Philosophy Documentation provides direction for:

* Architecture
* Design
* Engineering decisions

Every architectural or design principle should be traceable to a value or trade-off documented here.

---

## Traceability

```text
Vision
    ↓
Philosophy
    │
    ├──────────────┐
    ↓              ↓
Architecture     Design
```

Philosophy guides Architecture and Design without dictating their specifics.

---

## Relationships

| Document     | Relationship                               |
| ------------ | ------------------------------------------ |
| Vision       | Inspired by Vision                          |
| Feature      | Shapes feature decision-making              |
| Architecture | Guides architectural principles              |
| Design       | Guides design principles                     |
| Security     | Shapes security decision-making              |
| Engineering  | Shapes engineering principles and trade-offs |

---

## Required Characteristics

Philosophy Documentation should be:

* Stable
* Abstract
* Honest about trade-offs
* Memorable
* Technology-independent
* Actionable when a decision is ambiguous

---

## Audit Rules

An audit should verify:

* Principles are documented (`phil-001`, error).
* Values are documented (`phil-002`, warning).
* Trade-offs are documented (`phil-003`, suggestion).

---

## Validation Rules

Philosophy Documentation is considered valid when:

* Principles are documented and distinguishable from Vision.
* Values are explicit.
* Trade-offs are named honestly, including what is deliberately sacrificed.
* No implementation or technology-specific content appears.

---

## Generation Rules

When generating Philosophy Documentation:

* Start from Vision, then ask "how do we decide when two good options conflict?"
* Name the trade-off explicitly — what is chosen, and what is deliberately given up.
* Keep principles technology-independent.

---

## Enhancement Rules

When enhancing Philosophy Documentation:

* Replace aspirational statements with named trade-offs.
* Remove content that duplicates Vision.
* Keep principles few and memorable rather than exhaustive.

---

## Summary

Philosophy is the decision-making posture behind the product: the principles, values, and named trade-offs that Architecture and Design are built on. It stays stable while features and technology change underneath it.

---

## Common Mistakes

Examples of incorrect Philosophy content include:

* Restating Vision instead of explaining decision-making posture.
* Listing aspirational values with no named trade-off.
* Describing specific technology choices.
* Describing features instead of the reasoning behind them.

---

## Documentation Folder

Philosophy documents live under:

```text
docs/raw/philosophy/
```

---

## Usage

Written rarely, revisited when a recurring disagreement reveals the team lacks a shared principle to resolve it. Use `samgraha audit --domain philosophy` to confirm Principles, Values, and Trade-offs are all present — a Philosophy doc missing a Trade-offs section is usually just restated Vision.

## Related

- [Vision Standard](01-vision-standards.md) — Philosophy is inspired by Vision
- [Architecture Standard](05-architecture-standards.md) — guided by Philosophy
- [Design Standard](06-design-standards.md) — guided by Philosophy
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Quality Requirements

Philosophy Documentation should be:

* Stable
* Abstract enough to survive technology changes
* Concrete enough to resolve real disagreements
* Honest about trade-offs (not aspirational marketing)

---
