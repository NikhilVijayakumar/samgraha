# Feature Standard

This section details the Feature Standard.

## Purpose

This document defines the standard for Feature Documentation within the engineering documentation ecosystem.

Feature Documentation describes the functional capabilities of a product.

Unlike traditional requirements documents, Feature Documentation is organized as a collection of **atomic feature specifications**.

Each document describes exactly one feature.

Each feature should be understandable, reviewable, implementable, and testable independently.

Feature Documentation explains **what the product must do**.

It does not describe implementation, architecture, or engineering decisions.

---

# Required Sections

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

# Responsibilities

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

# Scope

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

# Prohibited Content

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

# Feature as an Atomic Specification

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

# Atomicity

Every feature should satisfy the following principles:

* One feature
* One responsibility
* One purpose
* One implementation objective
* One acceptance boundary

Large features should be decomposed into multiple independent feature documents.

---

# Independence

A feature should remain understandable without requiring unrelated feature documents.

Cross-feature references should be used only when functional relationships exist.

Features should minimize coupling.

---

# Inputs

Feature Documentation derives from:

* Vision
* Product Goals
* Business Requirements
* User Needs

Feature Documentation should not derive from implementation.

---

# Outputs

Feature Documentation provides direction for:

* Feature Design
* Architecture
* Feature Technical Design
* Engineering
* Testing
* Validation

Every implementation should trace back to one or more feature specifications.

---

# Traceability

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

# Relationships

| Document         | Relationship                                                    |
| ---------------- | --------------------------------------------------------------- |
| Vision           | Every feature derives from Vision                               |
| Design           | Feature Design applies design principles                        |
| Architecture     | Architecture organizes feature realization                      |
| Engineering      | Engineering explains implementation choices                     |
| External Context | May be referenced when external behavior influences the feature |

---

# Feature Principles

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

# Technology Independence

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

# External Context

A feature may reference External Context when external systems influence functional behavior.

Examples include:

* Authentication providers
* Cloud services
* External protocols
* Platform capabilities

Feature Documentation should reference External Context rather than duplicate it.

---

# Quality Requirements

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

# Validation Rules

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

# Common Mistakes

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

# Generation Rules

When generating Feature Documentation:

* Create one document per feature.
* Keep the feature atomic.
* Describe capabilities before workflows.
* Focus on business behavior.
* Avoid implementation terminology.
* Define clear feature boundaries.
* Maintain traceability to Vision.

---

# Enhancement Rules

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

# Audit Rules

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

# Success Criteria

Feature Documentation is successful when:

* Every product capability has exactly one feature specification.
* Features are independently understandable.
* Features are independently implementable.
* Features are independently testable.
* Features align with the Vision.
* Downstream documentation can be created without redefining feature intent.
* AI systems can implement individual capabilities without loading unrelated features.

---

# Non-Goals

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

# Profiles

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

# Summary

Feature Documentation is a collection of **atomic functional specifications**.

Each feature document defines exactly one product capability, remains technology independent, and provides the foundation for downstream design, architecture, engineering, and implementation.

The objective is to maximize cohesion, minimize coupling, and ensure every feature can be independently understood, reviewed, implemented, tested, and maintained.

---

# Documentation Folder

Feature documents live under:

```text
docs/raw/feature/
```

---

## Usage

Written by product owners/engineers before design or implementation starts — one file per capability. Use `samgraha audit --domain feature` to confirm every feature has Functional Requirements and Acceptance Criteria before it moves to Feature Design.

## Related

- [Vision Standard](vision.md) — every feature derives from Vision
- [Feature Design Standard](feature-design.md) — user-centered design for this feature
- [Feature Technical Standard](feature-technical.md) — architectural realization of this feature
- [Standards Reference Standard](standards.md) — how this standard itself is documented
