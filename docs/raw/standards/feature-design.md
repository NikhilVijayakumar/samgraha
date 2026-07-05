# Feature Design Standard

This section details the Feature Design Standard.

## Purpose

This document defines the standard for Feature Design Documentation within the engineering documentation ecosystem.

Feature Design translates a single Feature Specification into a complete user-centered design by applying the shared principles defined by the Design Documentation together with any relevant External Context.

Feature Design is **not** reusable design guidance.

It is the application of reusable design principles to one specific feature.

Every Feature Design document has a strict one-to-one relationship with a Feature Specification.

Feature Design explains **how users should experience a feature**.

It does not define implementation, architecture, or engineering decisions.

---

# Required Sections

Every Feature Design document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| User Experience | `user_experience` | ✓ | UX, User Flow |
| Workflow | `workflow` | ✓ | User Workflow, Flow |
| States | `states` | ✓ | UI States, Application States, State Transitions |
| Purpose | `purpose` | | Overview, Summary |
| Non-Goals | `non_goals` | | Non Goals, Out of Scope, Not In Scope |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

# Responsibilities

Feature Design is responsible for defining:

* User experience
* User workflow
* User journey
* Interaction model
* Navigation within the feature
* Information presentation
* User feedback
* Validation behavior
* Empty states
* Loading states
* Success states
* Error experience
* Accessibility application
* Localization considerations
* Responsive behavior
* User-facing constraints introduced by external systems

Feature Design transforms functional requirements into a complete user experience.

---

# Scope

Feature Design may describe:

* User interactions
* User workflows
* Navigation
* Screen relationships
* Interaction states
* Validation behavior
* Error behavior
* Empty states
* Loading states
* Responsive behavior
* Accessibility behavior
* Localization behavior
* User guidance
* User feedback

Every Feature Design document should remain focused on one feature.

---

# Out of Scope

Feature Design must not describe:

* Product Vision
* Design Philosophy
* Design Principles
* Architecture
* Component responsibilities
* Runtime behavior
* Programming languages
* Frameworks
* APIs
* Databases
* Algorithms
* Source code
* Engineering decisions

Shared Design belongs to Design Documentation.

Implementation belongs to downstream documentation.

---

# One-to-One Mapping

Every Feature Specification should have exactly one corresponding Feature Design document.

Example:

```text
features/

    authentication.md

feature-design/

    authentication.md
```

Both documents describe the same feature from different perspectives.

No Feature Design should describe multiple unrelated features.

---

# Design Principle Application

Feature Design applies the reusable principles defined by Design Documentation.

Examples include:

* Design Philosophy
* Interaction Principles
* Navigation Principles
* Accessibility Principles
* Localization Principles
* Visual Language
* Content Guidelines
* Responsive Design Principles

Feature Design should reference these principles rather than redefining them.

---

# External Context Application

Feature Design should identify any External Context that influences the user experience of the feature.

Examples include:

* Native operating system interaction patterns
* Authentication provider user flows
* Platform UX guidelines
* Accessibility standards
* Localization requirements
* Regulatory user-facing requirements
* Third-party interaction constraints

External Context should be referenced rather than duplicated.

Only context relevant to the feature should be included.

---

# Inputs

Feature Design derives from:

* Feature Specification
* Design Documentation
* Relevant External Context
* Product Constraints

Feature Design should not derive from implementation.

---

# Outputs

Feature Design provides direction for:

* Architecture
* Feature Technical Design
* UI Prototypes
* Mockups
* User Acceptance Testing
* Engineering

Implementation should realize the documented feature design.

---

# Traceability

Feature Design remains traceable.

```text
Vision
    │
    ├──────────────────────┐
    ↓                      ↓
Feature                  Design
    │                      │
    │              External Context (optional)
    │                      │
    └──────────────────────┘
               ↓
          Feature Design
               ↓
      Feature Technical Design
      (converges with Architecture)
               ↓
           Engineering
               ↓
         Implementation
```

Every Feature Design should trace directly to exactly one Feature Specification.

---

# Relationships

| Document                 | Relationship                             |
| ------------------------ | ---------------------------------------- |
| Feature                  | One-to-one mapping                       |
| Design                   | Applies shared design principles         |
| External Context         | Applies user-facing external constraints |
| Architecture             | Provides structural realization          |
| Feature Technical Design | Provides technical realization           |
| Engineering              | Explains implementation decisions        |

---

# Feature Design Principles

Every Feature Design should be:

* User-centered
* Feature-specific
* Cohesive
* Traceable
* Technology independent
* Consistent with Design Documentation
* Consistent with External Context
* Independently reviewable
* Independently implementable

---

# Technology Independence

Feature Design should remain implementation independent.

Feature Design should describe:

* user interaction
* user workflow
* expected behavior
* information presentation
* accessibility behavior
* localization behavior

rather than:

* frontend frameworks
* component libraries
* APIs
* source code
* implementation patterns

Technology belongs in Engineering Documentation.

---

# Quality Requirements

Feature Design should be:

* Complete
* User-centered
* Consistent
* Accessible
* Localizable
* Technology independent
* Traceable
* Atomic

Each document should remain focused on one feature.

---

# Validation Rules

Feature Design is considered valid when:

* One document corresponds to one Feature.
* User workflows are complete.
* Shared Design Documentation has been applied.
* Relevant External Context has been identified.
* Accessibility considerations exist where applicable.
* Localization considerations exist where applicable.
* User-facing external constraints are documented.
* No implementation details appear.
* Feature Design remains traceable to the Feature Specification.
* Shared Design and External Context are referenced rather than duplicated.

---

# Common Mistakes

Examples include:

* Combining multiple features.
* Rewriting Design Documentation.
* Rewriting External Context.
* Introducing architecture.
* Discussing implementation.
* Defining frameworks.
* Embedding source code.
* Ignoring shared design principles.
* Ignoring external UX constraints.

These should be reported during audits.

---

# Generation Rules

When generating Feature Design:

* Start from the Feature Specification.
* Apply Design Documentation.
* Apply relevant External Context.
* Focus on user experience.
* Keep one document per feature.
* Maintain technology independence.
* Reference shared Design Documentation.
* Reference External Context rather than duplicating it.
* Do not redefine reusable design guidance.

---

# Enhancement Rules

When enhancing Feature Design:

* Improve workflow clarity.
* Improve user experience.
* Strengthen consistency with Design Documentation.
* Strengthen consistency with External Context.
* Remove duplicated design principles.
* Remove duplicated external documentation.
* Remove implementation leakage.
* Improve traceability.
* Preserve feature intent.

Feature Design should become clearer without changing feature requirements.

---

# Audit Rules

An audit should verify:

* A one-to-one mapping exists between Feature and Feature Design.
* Shared Design Documentation has been applied.
* Relevant External Context has been identified.
* User workflows are complete.
* Accessibility has been considered.
* Localization has been considered where applicable.
* User-facing external constraints are reflected.
* Design remains technology independent.
* No implementation details appear.
* No architectural decisions appear.
* Design Documentation and External Context are referenced instead of duplicated.

Feature Design quality is evaluated individually and across the feature collection.

---

# Success Criteria

Feature Design is successful when:

* Every Feature has one corresponding Feature Design.
* Shared Design principles are consistently applied.
* Relevant External Context has been incorporated where necessary.
* Users experience consistent interactions across the product.
* Engineers understand how the feature should behave.
* Architecture can realize the design without redefining user behavior.
* AI systems can implement user experience consistently without inventing workflows or violating external platform conventions.

---

# Non-Goals

Feature Design does not define:

* Product Vision
* Shared Design Principles
* Feature Requirements
* Architecture
* Technical Design
* Engineering Decisions
* Technology Selection
* Source Code

These responsibilities belong to other documentation standards.

---

# Summary

Feature Design Documentation is the design realization of a single Feature Specification.

Each document maintains a strict one-to-one relationship with its corresponding Feature, applying the shared principles defined by the Design Documentation together with any relevant External Context to describe how users should experience that feature.

Its purpose is to bridge functional requirements and system architecture while preserving consistency, usability, accessibility, localization, platform conventions, and technology independence across the product ecosystem.

---

# Documentation Folder

Feature Design documents live under:

```text
docs/raw/feature-design/
```

---

## Usage

Written one-to-one with a Feature, by whoever owns UX for that feature, after the Feature spec exists and before Feature Technical Design starts. Use `samgraha audit --domain feature-design` to confirm every Feature has a matching Feature Design and that shared Design principles were applied rather than redefined.

## Related

- [Feature Standard](feature.md) — one-to-one mapping
- [Design Standard](design.md) — shared principles this standard applies
- [Feature Technical Standard](feature-technical.md) — technical realization of this design
- [Standards Reference Standard](standards.md) — how this standard itself is documented
