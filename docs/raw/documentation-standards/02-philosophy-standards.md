# Philosophy Standard

## Table of Contents
- [Purpose](#purpose)
- [Principles](#principles)
- [Values](#values)
- [Trade-offs](#trade-offs)
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

> **semantic_type:** `purpose`
> **scope:** Why Philosophy documentation exists — its role in the documentation ecosystem
> **out_of_scope:** Specific principles, values, trade-offs, implementation details, feature lists
> **contributes:** Establishes the reason for Philosophy's existence and its relationship to other standards
> **relationships:** Philosophy(02) is Tier 1; derives from Vision(01); guides Architecture(05), Design(06), Engineering(07)
> **responsibilities:** Explain what Philosophy documentation is and why it matters for downstream decisions
> **generation_rules:** State the purpose before the scope; explain the "why" before the "what"; avoid implementation language
> **enhancement_rules:** Strengthen the connection to Vision; clarify the distinction from other standards
> **validation_rules:** Purpose is clearly defined; no implementation details present; distinguishes Philosophy from Vision and other standards
> **audit_rules:** Must exist; must not contain implementation details; must explain Philosophy's role in the ecosystem

This document defines the standard for Philosophy documentation within the engineering documentation ecosystem.

Philosophy Documentation establishes the product's guiding principles, values, and the deliberate trade-offs that shape every downstream decision.

Unlike Vision, which explains **why** the product exists, Philosophy explains **how the people building it choose to think and decide**.

It does not describe features, architecture, or engineering decisions.

It shapes the judgment that produces them.

---

## Principles

> **semantic_type:** `guiding_principles`
> **scope:** The core principles that guide decision-making — enduring values that influence every downstream choice
> **out_of_scope:** Implementation rules, coding standards, architectural patterns, framework-specific guidelines
> **contributes:** Provides the reasoning framework that Architecture and Design are built on
> **relationships:** Derived from Vision(01); referenced by Architecture(05) and Design(06) for decision-making
> **responsibilities:** Define principles that remain true even as specific features and technologies change
> **generation_rules:** Extract from the product's values; express as principles, not rules; use memorable phrasing
> **enhancement_rules:** Add principles when new values emerge; remove principles that have become obsolete; keep core intent
> **validation_rules:** Principles are technology-independent; stable across features; memorable; actionable when a decision is ambiguous
> **audit_rules:** Must exist; must not reference specific technologies; must be evaluable against real decisions; must be stable

*(To be written by the product owner. This section defines the guiding principles of the product.)*

---

## Values

> **semantic_type:** `values`
> **scope:** The core values the product optimizes for — what the product deliberately chooses to prioritize
> **out_of_scope:** Feature priorities, implementation choices, technology preferences, temporary goals
> **contributes:** Makes explicit what the product values most, enabling consistent trade-off decisions
> **relationships:** Derived from Vision(01); informs Principles; referenced by Design(06) for user experience decisions
> **responsibilities:** Name the values explicitly; make trade-offs visible; ensure values are stable and memorable
> **generation_rules:** Start from the Vision; ask "what do we optimize for when two good options conflict?" Name the value
> **enhancement_rules:** Add values when new priorities emerge; remove values that have become obsolete; keep values stable
> **validation_rules:** Values are explicit and named; stable over time; actionable when making trade-offs; not aspirational platitudes
> **audit_rules:** Must exist; must be explicit and named; must be actionable; must not be aspirational without substance

*(To be written by the product owner. This section defines the core values of the product.)*

---

## Trade-offs

> **semantic_type:** `tradeoffs`
> **scope:** The deliberate trade-offs the product makes — what it chooses NOT to optimize for, and why
> **out_of_scope:** Accidental trade-offs, temporary compromises, implementation constraints, technology limitations
> **contributes:** Makes implicit trade-offs explicit; prevents downstream standards from contradicting product priorities
> **relationships:** Derived from Values; referenced by Architecture(05) and Engineering(07) for constraint decisions
> **responsibilities:** Name the trade-offs explicitly; explain why each trade-off was made; ensure downstream standards respect them
> **generation_rules:** For each value, ask "what does this value cost us?" Name the trade-off and the reason
> **enhancement_rules:** Add trade-offs when new priorities emerge; remove trade-offs that are no longer relevant; keep reasons current
> **validation_rules:** Trade-offs are explicit and named; reasons are clear; downstream standards can reference them; stable over time
> **audit_rules:** Must exist; must name trade-offs explicitly; must provide reasons; must be referenceable by downstream standards

*(To be written by the product owner. This section defines the deliberate trade-offs the product makes.)*

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
