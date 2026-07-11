# Implementation Standard

## Table of Contents
- [Purpose](#purpose)
- [Required Sections](#required-sections)
- [Deviations From Plan](#deviations-from-plan)
- [Module Boundaries](#module-boundaries)
- [Known Debt](#known-debt)
- [Constraints](#constraints)
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

> **semantic_type:** `purpose`
> **scope:** Why the Implementation standard exists — its role as the as-built record of a Feature Technical Design
> **out_of_scope:** Feature lists, architecture decisions, repo-wide code standards, implementation mechanics
> **contributes:** Establishes Implementation's identity as the honest post-build counterpart to Feature Technical Design (10)
> **relationships:** Derived from the ecosystem need to close the loop between planned and built; referenced by Build(13) and audit pipelines
> **responsibilities:** Define what Implementation is, what it is not, and how it relates to Feature Technical and Prototype
> **generation_rules:** Start with the relationship to Feature Technical Design; state the one-to-one constraint; distinguish from Engineering's repo-wide scope
> **enhancement_rules:** Strengthen the distinction from Feature Technical without losing the connection; remove any overlap with repo-wide Engineering scope
> **validation_rules:** Purpose is clearly defined; no implementation details present; no repo-wide policy leaked in; one-to-one relationship stated
> **audit_rules:** Must exist; must not contain feature lists or technology choices; must state the one-to-one relationship with Feature Technical Design

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

## Deviations From Plan

> **semantic_type:** `deviations`
> **scope:** Every material divergence between the Feature Technical Design plan and what was actually built, with rationale
> **out_of_scope:** Repo-wide code style changes, build configuration tweaks, unrelated refactors, implementation details without rationale
> **contributes:** Makes the gap between plan and reality visible and auditable; prevents hidden drift
> **relationships:** Compares directly against Feature Technical Design (10); may reference Prototype (11) findings that caused the deviation
> **responsibilities:** List each deviation, explain why it occurred, and note whether Prototype informed the change
> **generation_rules:** Walk the Feature Technical Design plan section by section; note every material change; write rationale for each; if none exist, explicitly state so
> **enhancement_rules:** Add newly discovered deviations; refine rationale; remove non-material items only with documented justification
> **validation_rules:** Each deviation has rationale; deviations are material, not trivial; section exists even if empty (states no deviations)
> **audit_rules:** Must exist as a section (error if missing); must not be absent silently; rationale required for each entry

*(To be populated after the feature ships. This section records every material deviation from the Feature Technical Design with rationale.)*

---

## Module Boundaries

> **semantic_type:** `module_boundaries`
> **scope:** Module and component boundaries as actually realized in code — the structural truth of what was built
> **out_of_scope:** Architectural intent from Feature Technical Design, planned boundaries that were never implemented, repo-wide module conventions
> **contributes:** Provides the as-built structural record that Build (13) and future engineers can verify against
> **relationships:** Realizes the boundaries planned in Feature Technical Design (10); may differ from Architecture's (05) intended structure
> **responsibilities:** Describe modules, their responsibilities, interfaces, and how they map to the planned boundaries
> **generation_rules:** Document modules as they exist in code; map to Feature Technical Design plan; note where structure diverged from plan
> **enhancement_rules:** Update when module structure changes materially; keep boundary descriptions aligned with actual code; note divergences from plan
> **validation_rules:** Boundaries described match actual code; divergences from plan noted; each module has a stated responsibility
> **audit_rules:** Must exist as a section (error if missing); must describe actual code structure, not planned structure

*(To be populated after the feature ships. This section describes module and component boundaries as actually realized in code.)*

---

## Known Debt

> **semantic_type:** `known_debt`
> **scope:** Technical debt, shortcuts taken, and deferred work — what was consciously traded away and the intended resolution
> **out_of_scope:** Permanent architectural tradeoffs, repo-wide quality standards, implementation details without debt implications
> **contributes:** Makes technical debt visible and trackable rather than hidden; enables informed future decisions
> **relationships:** May reference shortcuts from Feature Technical Design (10) tradeoffs; feeds future work tracked by Backlog or Engineering(07)
> **responsibilities:** Document each known debt item, its impact, and the intended resolution path
> **generation_rules:** List every conscious shortcut; explain the tradeoff made; state the intended resolution; note impact if unresolved
> **enhancement_rules:** Add newly identified debt; update resolution status; remove items that have been resolved
> **validation_rules:** Each debt item has an impact statement and resolution intent; debt is honest, not minimized
> **audit_rules:** Must not silently omit known debt; warning if suspected missing; each entry must have resolution intent

*(To be populated after the feature ships. This section records known debt, shortcuts taken, and their intended resolution.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Limitations that shaped the implementation — runtime, operational, or resource constraints that affected what was built
> **out_of_scope:** Architectural constraints from Architecture (05), technology rationale from Engineering (07), repo-wide policy
> **contributes:** Records the practical boundaries the implementation operated within, explaining why certain choices were made
> **relationships:** May reflect constraints from Feature Technical Design (10); influenced by Engineering (07) technology rationale
> **responsibilities:** Document constraints that materially affected the implementation and how they were addressed
> **generation_rules:** Identify constraints that shaped decisions; explain how they were addressed; distinguish from architectural constraints
> **enhancement_rules:** Add newly identified constraints; update resolution status; remove constraints that no longer apply
> **validation_rules:** Constraints are real and documented; each has a resolution or acceptance; not duplicating architectural constraints
> **audit_rules:** Must not duplicate repo-wide Engineering constraints; must reflect per-feature constraints only

*(To be populated after the feature ships. This section documents constraints that materially shaped the implementation.)*

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

> **semantic_type:** `traceability`
> **scope:** How this Implementation traces back to its Feature Technical Design and forward to Build
> **out_of_scope:** Detailed code lineage, commit history, CI/CD pipeline tracing, cross-feature dependencies
> **contributes:** Enables auditability by linking the as-built record to the plan it realized and the package it feeds
> **relationships:** Traces to Feature Technical Design (10); records which Prototype (11) validated the approach; feeds Build (13)
> **responsibilities:** Establish the one-to-one trace link and record which Prototype findings (if any) were carried forward
> **generation_rules:** Map to the single Feature Technical Design; note Prototype involvement; show forward link to Build
> **enhancement_rules:** Verify trace links remain accurate after changes; update Prototype reference if approach source changes
> **validation_rules:** Exactly one Feature Technical Design traced; Build forward link present; Prototype noted when applicable
> **audit_rules:** Must trace to exactly one Feature Technical Design; must not reference multiple unrelated plans

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
