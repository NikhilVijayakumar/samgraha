# Prototype Standard

## Table of Contents
- [Purpose](#purpose)
- [Mock APIs](#mock-apis)
- [Data Model](#data-model)
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
- [Generation Rules](#generation-rules)
- [Enhancement Rules](#enhancement-rules)
- [Audit Rules](#audit-rules)
- [Validation Rules](#validation-rules)
- [Summary](#summary)
- [Common Mistakes](#common-mistakes)
- [Documentation Folder](#documentation-folder)
- [Usage](#usage)
- [Related](#related)
- [Prohibited Content](#prohibited-content)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Prototype documentation exists — its role as executable simulation before production
> **out_of_scope:** Implementation details, production code, architecture, long-term data governance
> **contributes:** Establishes Prototype as the disposable simulation that validates Feature Design and Feature Technical Design
> **relationships:** Validates Feature Design(09) and Feature Technical Design(10); feeds confidence to Engineering(07) and Implementation(12)
> **responsibilities:** Define what a Prototype is, its falsifiable question, and its place in the documentation lifecycle
> **generation_rules:** State the falsifiable question first; explain that Prototype is disposable; emphasize it comes before Implementation
> **enhancement_rules:** Strengthen the connection to Feature Design and Feature Technical; clarify the distinction from production
> **validation_rules:** Purpose clearly defines Prototype's role; distinguishes from production; references the documents it validates
> **audit_rules:** Must exist; must state falsifiable question; must emphasize disposable nature

This document defines the standard for Prototype Documentation within the engineering documentation ecosystem.

A Prototype is an executable simulation of the application built to answer a specific falsifiable question or de-risk a specific unknown before production engineering commits to an approach.

A Prototype is **not** production implementation.

It is a disposable artifact scoped to one question, evaluated once, and discarded or replaced.

Prototype validates Feature Design and Feature Technical Design by exercising them against real interaction before Engineering builds the production version.

---

## Required Sections

Every Prototype document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Scope | `scope` | ✓ | Coverage, What Is Covered |
| Mock APIs | `mock_apis` | ✓ | Mocked APIs, API Contracts, Simulated APIs |
| Data Model | `data_model` | ✓ | Data Structures, Schema |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements |
| Traceability | `traceability` | | Traces To, Derived From |
| Purpose | `purpose` | | Overview, Summary |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Mock APIs

> **semantic_type:** `mock_apis`
> **scope:** All external dependencies that are mocked or stubbed in the prototype and their contracts
> **out_of_scope:** Production API specifications, real network endpoints, authentication credentials
> **contributes:** Defines which parts of the system are simulated so stakeholders and engineers know what is real vs. fake
> **relationships:** Constrained by Scope (which dependencies are in/out); feeds the data model with mock response structures
> **responsibilities:** List each mocked dependency; document the mock contract (request/response); specify fidelity level
> **generation_rules:** For each external dependency in scope, write a mock contract; clarify what is simulated and what is real
> **enhancement_rules:** Add mocks for newly identified dependencies; increase fidelity where required; remove unused mocks
> **validation_rules:** Every external dependency has a mock or stub; no real endpoints are called; no production credentials present
> **audit_rules:** Must exist (error if missing); no real production endpoints or credentials; each mock has a fidelity indicator

> **⚠️ IMPORTANT:** This section is required by the Prototype standard but does not yet exist as a heading. Please add it when drafting a real Prototype document.

---

## Data Model

> **semantic_type:** `data_model`
> **scope:** Minimal data structures, schema, and seed data needed to exercise the prototype scenario
> **out_of_scope:** Production database schema, full data governance, long-term persistence layer design
> **contributes:** Ensures the prototype has enough data to be realistic without reproducing production data
> **relationships:** Defined by what mock APIs return; used to keep the prototype independently runnable
> **responsibilities:** Define minimal data structures; provide seed data for the scenario; ensure no PII or production data is included
> **generation_rules:** Start from the scenario; define data structures only at the level needed for simulation; use seed data not production data
> **enhancement_rules:** Add missing structures; replace placeholders with realistic but fake data; remove unused fields
> **validation_rules:** Data model is minimal; no production data or PII is present; seed data exercises the scenario
> **audit_rules:** Must exist (error if missing); must contain no production data or PII; must be minimal for the scenario

> **⚠️ IMPORTANT:** This section is required by the Prototype standard but does not yet exist as a heading. Please add it when drafting a real Prototype document.

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Limitations and non-functional requirements that bound the prototype's design
> **out_of_scope:** Production performance targets, production security posture, long-term maintainability requirements
> **contributes:** Makes explicit what the prototype cannot do or chooses not to simulate
> **relationships:** Refines Scope with hard boundaries; may reference Limitations in the prototype design
> **responsibilities:** Document technical limitations; document constraints on the simulation approach; distinguish constraints from preferences
> **generation_rules:** List each limitation; state whether it is a hard constraint or a known shortcoming; explain the impact on decision-making
> **enhancement_rules:** Add new constraints discovered during implementation; separate environmental constraints from design decisions
> **validation_rules:** Constraints are clearly stated; no production requirements are presented as prototype constraints
> **audit_rules:** Must exist; each constraint must be verifiable; no production performance or security constraints

> **⚠️ IMPORTANT:** This section is required by the Prototype standard but does not yet exist as a heading. Please add it when drafting a real Prototype document.

## Goals

Prototype aims to:

* Answer one falsifiable question before production engineering commits.
* Keep the simulation scoped and disposable.
* Give stakeholders something to react to before real cost is spent.

---

## Non-Goals

Prototype does not define:

* Product Vision
* Architecture
* Production Engineering Decisions
* Long-term Data Governance

These responsibilities belong to other documentation standards.

---

## Success Criteria

Prototype is successful when:

* Stakeholders can evaluate the falsifiable question the prototype was built to answer.
* Feature Design and/or Feature Technical Design gain (or lose) confidence as a direct result.
* The prototype is discarded or explicitly promoted, not left to rot as accidental production code.

---

## Responsibilities

Prototype is responsible for defining:

* The falsifiable question the simulation answers
* Success and failure thresholds for that question
* What is simulated versus what is real
* Mock or stubbed external dependencies
* A minimal data model sufficient to exercise the scenario
* Known limitations and assumptions

Prototype turns a Feature Design or Feature Technical Design into something a stakeholder can react to before it's built for real.

---

## Scope

> **semantic_type:** `scope`
> **scope:** What the Prototype covers — in-scope behavior, fidelity levels, simulated flows
> **out_of_scope:** Production implementation, architecture, product vision, domain definitions
> **contributes:** Ensures each Prototype remains deliberately bounded to one falsifiable question
> **relationships:** Feeds from Goals and Success Criteria; constrains what mock APIs and data model must cover
> **responsibilities:** Define scope boundaries explicitly; document fidelity level per scope item; ensure no scope creep
> **generation_rules:** Start from the falsifiable question; list in-scope items with fidelity level; explicitly tag out-of-scope areas
> **enhancement_rules:** Tighten scope boundaries; add fidelity levels where missing; remove items that crept in
> **validation_rules:** Scope is documented with in-scope and out-of-scope lists; every scope item has a fidelity level
> **audit_rules:** Must exist (error if missing); must not include production implementation concerns

Prototype may describe:

* In-scope and out-of-scope behavior
* Fidelity level per scope item (mocked, stubbed, partial, full)
* Simulated interaction flows
* Mocked external dependencies
* Minimal data model and seed data
* Known limitations and assumptions

Every Prototype should remain scoped to one falsifiable question.

---

## Out of Scope

Prototype must not describe:

* Product Vision
* Design Philosophy
* Architecture
* Production data governance
* Production security posture
* Production performance targets
* Long-term maintainability

Production concerns belong to Feature Technical Design and Engineering. A Prototype that tries to be production-ready has stopped being a prototype.

---

## Inputs

Prototype derives from:

* Feature Design — the UX it validates
* Feature Technical Design — the technical plan it validates

Prototype should not derive from Implementation — it comes before Implementation, not after.

---

## Outputs

Prototype provides:

* Confidence (or lack of it) that Feature Design and Feature Technical Design are viable
* A validated approach that Implementation may follow

Prototype code itself is not an output — it's discarded once the question is answered.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Prototype relates to upstream and downstream documents in the documentation ecosystem
> **out_of_scope:** Full document dependency graph; source code trace links; internal section traceability
> **contributes:** Shows that every Prototype is derived from and validates specific upstream documents
> **relationships:** Indicates that Prototype receives from Feature Design(09) and Feature Technical(10); feeds confidence to Implementation(12)
> **responsibilities:** Document which upstream documents this Prototype validates; show the derivation chain
> **generation_rules:** Use a text diagram; list the documents that feed this Prototype and the documents it validates
> **enhancement_rules:** Add missing trace links; clarify the validation direction; ensure traceability is specific
> **validation_rules:** Traceability diagram shows upstream and downstream relationships; every link is accurate
> **audit_rules:** Must exist; must trace to upstream documents; must not be generic or disconnected

```text
Feature Design ──┐
                  ├──> Prototype ── validates ──> confidence to proceed
Feature Technical ┘
```

Every Prototype should trace to the Feature Design and/or Feature Technical document it validates.

---

## Relationships

| Document          | Relationship                    |
| ------------------ | -------------------------------- |
| Feature Design     | Prototype validates it            |
| Feature Technical   | Prototype validates it            |

---

## Required Characteristics

Prototype should be:

* Scoped to one falsifiable question
* Disposable
* Independently runnable without external network
* Free of production credentials or PII
* Honest about fidelity level per scope item

---

## Generation Rules

When generating Prototype documentation:

* State the falsifiable question the prototype answers.
* Define success criteria before building the prototype.
* Document scope boundaries and excluded areas.
* Record the decision made from prototype results.
* Mark the prototype as disposable — not production code.

---

## Enhancement Rules

When enhancing Prototype documentation:

* Verify the original question was answered.
* Update results if the prototype is re-executed.
* Ensure the disposition (adopt/reject/iterate) is recorded.
* Remove or archive prototypes that are no longer relevant.
* Preserve the reasoning behind go/no-go decisions.

---

## Audit Rules

An audit should verify:

* Scope is present (error if missing).
* Mock APIs are documented (warning if missing).
* The prototype is disposable — no production implementation claims (warning if violated).

---

## Validation Rules

Prototype is considered valid when:

* Scope is documented with in-scope and out-of-scope lists.
* Every external dependency has a mock or stub.
* The data model is minimal and free of production data or PII.
* The question the prototype answers has explicit success and failure thresholds.
* No production implementation is presented as final.

---

## Summary

Prototype Documentation is a disposable, executable simulation scoped to one falsifiable question, built to validate a Feature Design or Feature Technical Design before Engineering commits to production implementation.

---

## Common Mistakes

Examples include:

* Building a prototype with no stated question ("see if it works").
* Letting scope grow into a second production effort.
* Mocks that silently call real production endpoints.
* Treating the prototype's disposable code as reusable.
* Embedding production credentials in mock configuration.

These should be reported during audits.

---

## Documentation Folder

Prototype documents live under:

```text
docs/raw/prototype/
```

---

## Usage

Written when a Feature Design or Feature Technical decision carries enough risk to warrant proof before production engineering starts. Use `samgraha audit --domain prototype` to confirm scope and mock APIs are documented and no production implementation leaked in.

## Related

- [Feature Design Standard](09-feature-design-standards.md) — one of the documents this standard validates
- [Feature Technical Standard](10-feature-technical-standards.md) — the other document this standard validates
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Prohibited Content

A Prototype document must not contain:

* Production implementation

Prototype code is disposable by definition; presenting it as production-ready is a documentation defect, not a strength.

---

## Quality Requirements

Prototype should be:

* Scoped to one falsifiable question
* Explicit about success/failure thresholds
* Disposable
* Independently runnable without external network
* Free of production credentials or PII

---
