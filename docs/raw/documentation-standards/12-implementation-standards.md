# Implementation Standard

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
- [Generation Rules](#generation-rules)
- [Enhancement Rules](#enhancement-rules)
- [Audit Rules](#audit-rules)
- [Validation Rules](#validation-rules)
- [Summary](#summary)
- [Common Mistakes](#common-mistakes)
- [Documentation Folder](#documentation-folder)
- [Usage](#usage)
- [Related](#related)
- [Revision History](#revision-history)
- [Relationship to Engineering's Code Standards](#relationship-to-engineerings-code-standards)
- [One-to-One Mapping](#one-to-one-mapping)
- [Quality Requirements](#quality-requirements)

---


## Purpose

This document defines the standard for Implementation documentation within the engineering documentation ecosystem.

Implementation is the as-built record of a single Feature Technical Design (10) once Prototype (11) has de-risked the approach: what was actually built, where it diverged from the plan, and why.

Every Implementation document has a strict one-to-one relationship with a Feature Technical document — the same relationship Feature Technical has with Feature.

Implementation explains **what was actually built**, not what was planned or how the repository builds code in general.

It does not restate Engineering's repo-wide Code Standards, and it is not a second copy of Feature Technical Design.

---

## Required Sections

Every Implementation document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Deviations From Plan | `deviations` | ✓ | Deviations, Changes From Design, What Changed |
| Module Boundaries | `module_boundaries` | ✓ | Modules, Component Boundaries |
| Known Debt | `known_debt` | | Technical Debt, Shortcuts Taken |
| Purpose | `purpose` | | Overview, Summary |
| Constraints | `constraints` | | Limitations |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Implementation aims to:

* Give every shipped feature an honest as-built record.
* Make deviations from plan visible instead of hidden in commit history.
* Give Build(13) something specific to package.

---

## Non-Goals

Implementation does not define:

* Product Vision
* Security threat models
* Architecture
* Repo-wide technology rationale or code style
* Repo-wide build/CI/CD mechanics
* Packaging or release policy

These responsibilities belong to other documentation standards.

---

## Success Criteria

Implementation is successful when:

* Every shipped Feature Technical Design has a corresponding as-built record.
* An engineer can determine what actually got built without reading source code.
* Deviations from plan are traceable to a specific reason.
* Build(13) can package with confidence in what Implementation says was actually delivered.

---

## Responsibilities

Implementation is responsible for recording:

* What was actually built for one Feature Technical Design
* Where the build diverged from the plan, and why
* Module and component boundaries as realized in code
* Known debt or shortcuts taken, and their intended resolution
* Which validated Prototype approach (if any) was carried into the real build

Implementation closes the loop between what was planned and what exists — it is the single place an auditor or future engineer can check "did we build what we said we'd build."

---

## Scope

Implementation may describe:

* Deviations from Feature Technical Design and their rationale
* Module/component boundaries as actually realized
* Known debt, shortcuts, or deferred work
* Which Prototype findings were carried forward or discarded
* Notable implementation decisions not anticipated by the plan

Every Implementation document should remain scoped to the one Feature Technical Design it realizes.

---

## Out of Scope

Implementation must not describe:

* Product Vision
* Design Philosophy
* Security threat models or principles
* Architecture
* Repo-wide technology rationale
* Repo-wide code style or quality rules
* Repo-wide build/CI/CD mechanics
* User experience

Repo-wide rules belong to Engineering. Structural realization belongs to Architecture and Feature Technical. Packaging belongs to Build.

---

## Inputs

Implementation derives from:

* Feature Technical Design (10) — the plan being realized
* Engineering (07) — technology rationale and Code Standards it must honor
* Prototype (11) — validated approach, where one exists

---

## Outputs

Implementation provides:

* An as-built record for Build (13) to package
* A record future engineers can check against the original plan

---

## Traceability

```text
Feature Technical (10) ──┐
                          ├──> Implementation (12)
Prototype (11) ───────────┘         │
                                     ↓
                                  Build (13)
```

Every Implementation should trace directly to exactly one Feature Technical Design, and record which Prototype (if any) validated the approach it followed.

---

## Relationships

| Document | Relationship |
|---|---|
| Feature Technical | One-to-one mapping — Implementation is the as-built record of this plan |
| Prototype | Implementation records which validated approach (if any) it followed |
| Engineering | Implementation honors Engineering's technology rationale and Code Standards without restating them |
| Build | Build packages what Implementation produced |

---

## Required Characteristics

Implementation should be:

* Accurate to what was actually built
* Traceable to exactly one Feature Technical Design
* Honest about known debt
* Specific about deviations, not vague
* Written after the fact, not speculatively

---

## Generation Rules

When generating Implementation documentation:

* Record what was actually built, not what was planned.
* Map implementation decisions to the Feature Technical Design.
* Document deviations from the plan with rationale.
* Include verification results (tests, reviews, audits).
* Keep the record concise — implementation is evidence, not narrative.

---

## Enhancement Rules

When enhancing Implementation documentation:

* Update when the implementation changes materially.
* Verify traceability to Feature Technical Design remains accurate.
* Remove references to features that no longer exist.
* Preserve the record of past decisions even if superseded.
* Ensure new deviations are documented with rationale.

---

## Audit Rules

An audit should verify:

* A one-to-one mapping exists between Feature Technical and Implementation.
* Deviations From Plan is present (error if missing).
* Module Boundaries is present (error if missing).
* No repo-wide Engineering content is duplicated (warning if found).
* Known debt, if any, is documented rather than omitted (warning if suspected missing).

---

## Validation Rules

Implementation is considered valid when:

* One document corresponds to one Feature Technical Design.
* Deviations from the plan are documented with rationale, or the section explicitly states there were none.
* Module boundaries as built are described.
* No repo-wide Code Standards content appears.
* No re-statement of Feature Technical Design content that didn't change.

---

## Summary

Implementation is the as-built counterpart to Feature Technical Design — a one-to-one record of what was actually built for one feature, where it diverged from plan, and what debt remains, written after Prototype has de-risked the approach and Engineering's repo-wide rationale has been honored.

---

## Common Mistakes

Examples include:

* Writing Implementation before the feature is actually built.
* Copying Feature Technical Design verbatim instead of recording what changed.
* Describing repo-wide code style instead of feature-specific deviations.
* Omitting known debt because it's inconvenient to admit.
* Covering multiple features in one Implementation document.

These should be reported during audits.

---

## Documentation Folder

Implementation documents live under:

```text
docs/raw/implementation/
```

---

## Usage

Written one-to-one with a Feature Technical Design, after the feature ships, by whoever built it. Once registered in code, use `samgraha audit --domain implementation` to confirm every shipped Feature Technical Design has a matching as-built record and that no repo-wide Engineering content leaked in.

## Related

- [Feature Technical Standard](10-feature-technical-standards.md) — one-to-one mapping, the plan this documents the realization of
- [Prototype Standard](11-prototype-standards.md) — validated approach this may follow
- [Engineering Standard](07-engineering-standards.md) — repo-wide technology rationale and Code Standards this honors without restating
- [Build Standard](13-build-standards.md) — packages what this produces

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| Draft | — | — | Initial proposal. No `StandardDefinition` for `implementation` exists in `crates/standards/src/builtin.rs` yet. Required Sections, audit rules, and relationships below are proposed. |

## Relationship to Engineering's Code Standards

Engineering(07)'s optional **Code Standards** section is repo-wide: style, quality, and conventions that apply to every line of code, written once and stable. Implementation(12) is per-feature: the as-built record of one Feature Technical Design, written after that feature ships. They don't overlap because they operate at different scopes — repo-wide policy versus per-feature realization — the same split Security(03) draws against Architecture's, Engineering's, and Feature Technical's own Security sections.

| Layer | Owns | Derives From |
|---|---|---|
| Engineering — Code Standards | Repo-wide style, quality, and conventions — once | Engineering's own technology rationale |
| Implementation (12) | What was actually built for one feature, and how it diverged from plan | Feature Technical(10) it realizes, honoring Engineering(07)'s rationale and Code Standards |

An Implementation document that restates repo-wide style rules instead of documenting feature-specific deviations should be flagged during audit — that content belongs in Engineering.

---

## One-to-One Mapping

Every Feature Technical Design should have exactly one corresponding Implementation document, the same pattern Feature Design and Feature Technical each keep with Feature.

Example:

```text
feature-technical/

    authentication.md

implementation/

    authentication.md
```

No Implementation document should describe multiple unrelated Feature Technical Designs.

---

## Quality Requirements

Implementation should be:

* Written after the feature ships, not speculatively
* Specific about deviations — not just "some things changed"
* Traceable to exactly one Feature Technical Design
* Free of repo-wide policy that belongs in Engineering
* Honest about known debt rather than silent about it

---
