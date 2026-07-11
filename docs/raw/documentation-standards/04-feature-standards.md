# Feature Standard

## Table of Contents
- [Purpose](#purpose)
- [Functional Requirements](#functional-requirements)
- [Business Rules](#business-rules)
- [Constraints](#constraints)
- [Dependencies](#dependencies)
- [Acceptance Criteria](#acceptance-criteria)
- [Future Extensions](#future-extensions)
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
- [Prohibited Content](#prohibited-content)
- [Feature as an Atomic Specification](#feature-as-an-atomic-specification)
- [Atomicity](#atomicity)
- [Independence](#independence)
- [Feature Principles](#feature-principles)
- [Technology Independence](#technology-independence)
- [External Context](#external-context)
- [Quality Requirements](#quality-requirements)
- [Profiles](#profiles)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Feature Documentation exists — its role in defining product capabilities independent of implementation
> **out_of_scope:** Feature specifications, architecture details, implementation approaches, technology choices
> **contributes:** Establishes the root intent for all Feature Documentation sections and downstream standards
> **relationships:** Derived from Vision(01); feeds Feature Design(09) and Feature Technical Design(10)
> **responsibilities:** Define Feature Documentation's reason for being and its boundary within the documentation ecosystem
> **generation_rules:** State what Feature Documentation is; explain what it defines and what it does not; reference the broader ecosystem
> **enhancement_rules:** Strengthen clarity of scope boundaries; remove overlap with downstream standards; keep stable over time
> **validation_rules:** Purpose is clearly defined; no feature specifications present; boundary with other standards is explicit
> **audit_rules:** Must exist; must not contain feature lists; must define what Feature Documentation is and is not

This document defines the standard for Feature Documentation within the engineering documentation ecosystem.

Feature Documentation describes the functional capabilities of a product.

Unlike traditional requirements documents, Feature Documentation is organized as a collection of **atomic feature specifications**.

Each document describes exactly one feature.

Each feature should be understandable, reviewable, implementable, and testable independently.

Feature Documentation explains **what the product must do**.

It does not describe implementation, architecture, or engineering decisions.

---

## Functional Requirements

> **semantic_type:** `functional_requirements`
> **scope:** The complete set of functional behaviors the feature must exhibit — what the product does for the user
> **out_of_scope:** Implementation details, technical design, API contracts, database schemas, UI layout
> **contributes:** Defines the core deliverable of every feature document — the "what" that drives all downstream design
> **relationships:** Derived from Vision(01) goals; feeds Feature Design(09) and Feature Technical Design(10); referenced by Acceptance Criteria
> **responsibilities:** List every functional behavior the feature must support, written as testable statements independent of technology
> **generation_rules:** Write one requirement per behavior; use shall/must language; keep each requirement atomic and independently verifiable
> **enhancement_rules:** Add requirements when scope clarifies; remove requirements that duplicate other features; split compound requirements
> **validation_rules:** Every requirement is atomic, testable, technology-independent, and traceable to Vision; no implementation leakage
> **audit_rules:** Must exist; must not contain implementation details; each requirement must be independently testable; must not reference specific technologies

*(To be written by the domain expert. This section defines what the feature must do.)*

---

## Business Rules

> **semantic_type:** `business_rules`
> **scope:** The business logic and rules that govern feature behavior — the constraints on how the feature operates
> **out_of_scope:** Technical validation, input sanitization, database constraints, API rate limits
> **contributes:** Captures domain knowledge that cannot be inferred from functional requirements alone
> **relationships:** Derived from Business Requirements and Vision(01); feeds Feature Design(09); referenced by Acceptance Criteria
> **responsibilities:** Document every business rule that governs feature behavior, including conditional logic and edge cases
> **generation_rules:** Express rules as conditional statements; reference the business domain; keep rules independent of implementation
> **enhancement_rules:** Add rules when domain knowledge clarifies; remove rules that are technical constraints; consolidate overlapping rules
> **validation_rules:** Rules are complete, unambiguous, technology-independent, and testable; no implementation details present
> **audit_rules:** Must exist if the feature has business logic; must not contain technical constraints; must be testable

*(To be written by the domain expert. This section defines the business logic governing the feature.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** The limitations and boundaries the feature must operate within — regulatory, business, or technical constraints
> **out_of_scope:** Implementation details, technology choices, framework limitations, performance benchmarks
> **contributes:** Defines the boundary conditions within which the feature must function
> **relationships:** May reference External Context; feeds Feature Design(09) and Architecture(05); referenced by Acceptance Criteria
> **responsibilities:** Document every constraint that limits how the feature can be designed or implemented
> **generation_rules:** List constraints by category (regulatory, business, technical); state each constraint as a clear limitation
> **enhancement_rules:** Add constraints when new regulations or policies apply; remove constraints that no longer apply; clarify ambiguous constraints
> **validation_rules:** Constraints are complete, clear, technology-independent, and enforceable; no implementation details present
> **audit_rules:** Must exist if the feature has constraints; must not contain implementation details; must be enforceable

*(To be written by the domain expert. This section defines the limitations the feature must operate within.)*

---

## Dependencies

> **semantic_type:** `dependencies`
> **scope:** Other features, systems, or capabilities the feature relies on to function — the relationships that create coupling
> **out_of_scope:** Implementation dependencies, library versions, framework dependencies, build dependencies
> **contributes:** Makes feature coupling explicit so downstream design can address integration points
> **relationships:** References other Feature documents; feeds Architecture(05) and Feature Technical Design(10); may reference External Context
> **responsibilities:** List every feature, system, or capability this feature depends on, with the nature of the dependency
> **generation_rules:** List dependencies by name and type; state the nature of each dependency; distinguish functional from data dependencies
> **enhancement_rules:** Add dependencies when new coupling is discovered; remove dependencies when features are decoupled; clarify dependency nature
> **validation_rules:** Dependencies are real and documented; nature of each dependency is stated; no circular dependencies exist
> **audit_rules:** Must exist if the feature has dependencies; must not list implementation dependencies; must state the nature of each dependency

*(To be written by the domain expert. This section defines what other capabilities the feature relies on.)*

---

## Acceptance Criteria

> **semantic_type:** `acceptance_criteria`
> **scope:** The verifiable conditions that confirm the feature is complete and correct — the definition of done for the feature
> **out_of_scope:** Test implementation details, code coverage targets, performance benchmarks, deployment criteria
> **contributes:** Provides the testable contract that validates feature completeness and correctness
> **relationships:** Derived from Functional Requirements and Business Rules; feeds Testing and Validation; referenced by Audit Rules
> **responsibilities:** Define every verifiable condition that confirms the feature meets its requirements and business rules
> **generation_rules:** Derive from each functional requirement and business rule; write as given/when/then or checklist; keep each criterion independently verifiable
> **enhancement_rules:** Add criteria when requirements clarify; remove criteria that duplicate others; ensure every requirement has a corresponding criterion
> **validation_rules:** Every requirement has at least one acceptance criterion; criteria are testable; no implementation details present
> **audit_rules:** Must exist; must cover all functional requirements; must be independently testable; must not reference implementation details

*(To be written by the domain expert. This section defines how to verify the feature is complete.)*

---

## Future Extensions

> **semantic_type:** `future_extensions`
> **scope:** Known future work or planned extensions for the feature — what is explicitly deferred to a later iteration
> **out_of_scope:** Current feature requirements, implementation plans, technology roadmap, release schedule
> **contributes:** Documents intentional deferrals so downstream standards know what is planned but not yet scoped
> **relationships:** References Vision(01) goals; feeds Feature Design(09) planning; may reference Backlog items
> **responsibilities:** List known future work with enough context to understand why it was deferred and when it should be revisited
> **generation_rules:** List deferred work with rationale; keep descriptions brief; reference the triggering condition for revisiting
> **enhancement_rules:** Remove items that are now in scope; add items when deferral decisions are made; update triggering conditions
> **validation_rules:** Items are genuinely deferred (not forgotten); rationale is clear; triggering conditions are stated
> **audit_rules:** Must exist if future work is known; must not contain current requirements; must include rationale for deferral

*(To be written by the domain expert. This section defines what extensions are planned for the future.)*

---

## Required Sections

Every Feature document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

Section headings use level-2 markdown (`## Section Name`).

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Purpose | `purpose` | ✓ | Overview, Summary |
| Functional Requirements | `functional_requirements` | ✓ | Requirements, FRs, Functional Reqs, Feature Requirements |
| Business Rules | `business_rules` | | Rules, Business Logic |
| Inputs | `inputs` | | Input, Input Data |
| Outputs | `outputs` | | Output, Output Data |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements |
| Dependencies | `dependencies` | | Dependency, Depends On |
| Acceptance Criteria | `acceptance_criteria` | ✓ | Success Criteria, Definition of Done, Criteria |
| Non-Goals | `non_goals` | | Non Goals, Out of Scope |
| Future Extensions | `future_extensions` | | Future Work, Roadmap |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

Sections marked required that are absent produce a compile diagnostic (knowledge is still generated).

---

## Goals

Feature Documentation aims to:

* Give every capability a single authoritative functional specification.
* Make each feature independently reviewable and implementable.
* Keep functional requirements decoupled from technology choice.
* Make every feature traceable back to Vision and Philosophy.

---

## Non-Goals

> **semantic_type:** `non_goals`
> **scope:** What Feature Documentation explicitly does not define — boundaries that prevent scope creep into other standards
> **out_of_scope:** Goals, vision statements, success criteria, any positive framing of intent
> **contributes:** Prevents Feature Documentation from conflating with Architecture, Design, or Engineering standards
> **relationships:** Mirrors Goals; referenced by Audit Rules to detect scope violations in feature documents
> **responsibilities:** List every responsibility that belongs to a different documentation standard, with ownership assigned
> **generation_rules:** Invert the Goals list; reference the specific downstream standard that owns each excluded responsibility
> **enhancement_rules:** Add new exclusions when scope creep is detected; remove exclusions for responsibilities that migrate to Feature Documentation
> **validation_rules:** All excluded responsibilities have a clear owner in another standard; no ambiguity about where responsibility lies
> **audit_rules:** Must exist; each excluded item must name its owning standard; must not contradict the Goals list

Feature Documentation does not define:

* Product Vision
* Design Principles
* Architecture
* Technical Design
* Engineering Decisions
* Technology Selection
* Source Code
* APIs

These responsibilities belong to other documentation standards.

---

## Success Criteria

Feature Documentation is successful when:

* Every product capability has exactly one feature specification.
* Features are independently understandable.
* Features are independently implementable.
* Features are independently testable.
* Features align with the Vision.
* Downstream documentation can be created without redefining feature intent.
* AI systems can implement individual capabilities without loading unrelated features.

---

## Responsibilities

Feature Documentation is responsible for defining:

* Functional capability
* Feature objective
* User value
* Functional requirements
* Business rules
* Inputs
* Outputs
* Success criteria
* Constraints
* Dependencies on other features (when necessary)

Feature Documentation defines product capabilities.

It does not define how they are implemented.

---

## Scope

Feature Documentation may describe:

* Functional behavior
* Business logic
* User expectations
* Validation rules
* Error conditions
* Acceptance criteria
* Functional constraints
* Feature interactions
* Functional dependencies

Each feature should remain focused on one capability.

---

## Out of Scope

Feature Documentation must not describe:

* Architecture
* Technical implementation
* Programming languages, frameworks, or libraries
* APIs
* Databases
* Source code
* UI implementation
* System components
* Communication protocols

These belong to Architecture, Feature Technical Design, Engineering, Feature Design, or Implementation respectively — see Prohibited Content below for the full rationale table.

---

## Inputs

> **semantic_type:** `inputs`
> **scope:** What feeds Feature Documentation — the upstream sources from which feature specifications derive
> **out_of_scope:** Implementation sources, code analysis, technical constraints, architecture decisions
> **contributes:** Defines the derivation chain so every feature can be traced to its originating intent
> **relationships:** Consumes Vision(01) outputs; may reference External Context; referenced by Traceability sections
> **responsibilities:** List every valid source of feature specifications and assert that implementation is not a source
> **generation_rules:** List sources by name; state the derivation direction; assert that Feature Documentation should not derive from implementation
> **enhancement_rules:** Add new sources when upstream standards are added; remove sources that no longer feed Feature Documentation
> **validation_rules:** All listed sources exist as upstream standards; derivation direction is correct; no circular dependencies
> **audit_rules:** Must exist; must list at least Vision as a source; must state that implementation is not a source; must not reference downstream standards as inputs

Feature Documentation derives from:

* Vision
* Product Goals
* Business Requirements
* User Needs

Feature Documentation should not derive from implementation.

---

## Outputs

> **semantic_type:** `outputs`
> **scope:** What Feature Documentation feeds — the downstream standards that consume feature specifications
> **out_of_scope:** Upstream sources, Vision content, implementation details
> **contributes:** Makes explicit which downstream standards depend on Feature Documentation for their input
> **relationships:** Feeds Feature Design(09), Architecture(05), Feature Technical Design(10), Engineering(07), and Testing
> **responsibilities:** List every downstream standard that consumes Feature Documentation and state the nature of the dependency
> **generation_rules:** List downstream consumers by name and standard number; state what each consumer derives from Feature Documentation
> **enhancement_rules:** Add new consumers when downstream standards are added; update relationships when standards are reorganized
> **validation_rules:** All listed consumers exist; the nature of dependency is stated; no upstream standards listed as consumers
> **audit_rules:** Must exist; must list at least Feature Design as a consumer; must not list upstream standards as consumers

Feature Documentation provides direction for:

* Feature Design
* Architecture
* Feature Technical Design
* Engineering
* Testing
* Validation

Every implementation should trace back to one or more feature specifications.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Feature Documentation connects to the documentation hierarchy — the derivation chain from Vision to implementation
> **out_of_scope:** Implementation traceability, test traceability, bug tracking, version history
> **contributes:** Makes Feature Documentation's influence visible and verifiable across the entire documentation ecosystem
> **relationships:** Vision(01) → Feature(04) → Feature Design(09) → Feature Technical Design(10) → Engineering(07) → Implementation
> **responsibilities:** Show the derivation path from Vision through Feature to downstream standards; assert that no downstream document contradicts Feature intent
> **generation_rules:** Use the tier model diagram; list which standards derive from Features; state the non-contradiction rule
> **enhancement_rules:** Update the diagram when new standards are added; ensure derivation paths remain accurate
> **validation_rules:** Derivation paths are complete; no orphaned standards; non-contradiction rule is stated
> **audit_rules:** Must exist; must include tier diagram; must list downstream standards; must state non-contradiction constraint

Feature Documentation should remain traceable.

```text
Vision

↓

Feature

↓

Feature Design

↓

Architecture

↓

Feature Technical Design

↓

Engineering

↓

Implementation
```

Every feature should support the documented Vision.

---

## Relationships

| Document         | Relationship                                                    |
| ---------------- | --------------------------------------------------------------- |
| Vision           | Every feature derives from Vision                               |
| Design           | Feature Design applies design principles                        |
| Architecture     | Architecture organizes feature realization                      |
| Engineering      | Engineering explains implementation choices                     |
| External Context | May be referenced when external behavior influences the feature |

---

## Required Characteristics

A Feature document should be:

* Atomic
* Independent
* Cohesive
* Traceable to Vision
* Testable (acceptance criteria are checkable)
* Technology-independent

---

## Audit Rules

An audit should verify:

* Each document describes one feature.
* Feature responsibilities are clear.
* Features remain technology independent.
* Business rules are complete.
* Acceptance criteria exist.
* Features remain traceable to Vision.
* No implementation or architectural decisions appear.
* Features are independently understandable and implementable.

Feature quality is evaluated per document and across the feature collection.

---

## Validation Rules

Feature Documentation is considered valid when:

* One document describes one feature.
* Feature purpose is clearly defined.
* Functional requirements are complete.
* Business rules are documented.
* Acceptance criteria are present.
* Technology decisions are absent.
* Features remain traceable to Vision.
* Feature boundaries are explicit.

Validation applies to every feature independently.

---

## Generation Rules

When generating Feature Documentation:

* Create one document per feature.
* Keep the feature atomic.
* Describe capabilities before workflows.
* Focus on business behavior.
* Avoid implementation terminology.
* Define clear feature boundaries.
* Maintain traceability to Vision.

---

## Enhancement Rules

When enhancing Feature Documentation:

* Improve atomicity.
* Split oversized features.
* Remove implementation leakage.
* Clarify business behavior.
* Improve traceability.
* Remove duplicated functionality.
* Preserve feature intent.

Features should become smaller, clearer, and more independent through refinement.

---

## Summary

Feature Documentation is a collection of **atomic functional specifications**.

Each feature document defines exactly one product capability, remains technology independent, and provides the foundation for downstream design, architecture, engineering, and implementation.

The objective is to maximize cohesion, minimize coupling, and ensure every feature can be independently understood, reviewed, implemented, tested, and maintained.

---

## Common Mistakes

Examples include:

* Combining multiple unrelated features.
* Introducing implementation details.
* Explaining architecture.
* Embedding UI implementation.
* Discussing frameworks.
* Mixing business requirements with engineering decisions.
* Creating features that cannot be implemented independently.

These should be reported during audits.

---

## Documentation Folder

Feature documents live under:

```text
docs/raw/feature/
```

---

## Usage

Written by product owners/engineers before design or implementation starts — one file per capability. Use `samgraha audit --domain feature` to confirm every feature has Functional Requirements and Acceptance Criteria before it moves to Feature Design.

## Related

- [Vision Standard](01-vision-standards.md) — every feature derives from Vision
- [Feature Design Standard](09-feature-design-standards.md) — user-centered design for this feature
- [Feature Technical Standard](10-feature-technical-standards.md) — architectural realization of this feature
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Prohibited Content

Feature Documentation must not describe:

| Prohibited | Rationale |
|------------|-----------|
| Architecture | Belongs to Architecture Documentation |
| Technical implementation | Belongs to Feature Technical Design |
| Programming languages | Belongs to Engineering Documentation |
| Frameworks | Belongs to Engineering Documentation |
| Libraries | Belongs to Engineering Documentation |
| APIs | Belongs to Feature Technical Design |
| Databases | Belongs to Engineering Documentation |
| Source code | Belongs to Implementation |
| UI implementation | Belongs to Feature Design |
| System components | Belongs to Architecture Documentation |
| Communication protocols | Belongs to Architecture Documentation |

Prohibited content detected during compilation produces a diagnostic.

---

## Feature as an Atomic Specification

Feature Documentation is a collection of atomic specifications.

Example:

```text
features/

    authentication.md

    localization.md

    plugin-management.md

    project-registry.md

    settings.md
```

Each document represents one feature.

A feature should not describe multiple unrelated capabilities.

---

## Atomicity

Every feature should satisfy the following principles:

* One feature
* One responsibility
* One purpose
* One implementation objective
* One acceptance boundary

Large features should be decomposed into multiple independent feature documents.

---

## Independence

A feature should remain understandable without requiring unrelated feature documents.

Cross-feature references should be used only when functional relationships exist.

Features should minimize coupling.

---

## Feature Principles

Every feature should be:

* Atomic
* Independent
* Cohesive
* Traceable
* Testable
* Reviewable
* Technology Independent
* User Focused

Features should maximize cohesion while minimizing coupling.

---

## Technology Independence

Feature Documentation should remain technology independent.

Feature documents should describe:

* capabilities
* expected behavior
* business rules

instead of

* frameworks
* programming languages
* databases
* APIs
* implementation patterns

Technology decisions belong in Engineering Documentation.

---

## External Context

A feature may reference External Context when external systems influence functional behavior.

Examples include:

* Authentication providers
* Cloud services
* External protocols
* Platform capabilities

Feature Documentation should reference External Context rather than duplicate it.

---

## Quality Requirements

Feature Documentation should be:

* Atomic
* Complete
* Unambiguous
* Technology Independent
* Testable
* Traceable
* Maintainable
* Consistent

Every feature should remain focused on one capability.

---

## Profiles

Feature Documentation defines the following package profiles.
Each profile specifies which sections to include when packaging for a specific consumer.

```yaml
profiles:
  implementation:
    include:
      - functional_requirements
      - business_rules
      - constraints
      - dependencies

  review:
    include:
      - purpose
      - acceptance_criteria
      - traceability

  architecture:
    include:
      - constraints
      - dependencies
```

Profiles are consumed by the Knowledge Package service. New profiles may be added as consumer needs emerge.

---
