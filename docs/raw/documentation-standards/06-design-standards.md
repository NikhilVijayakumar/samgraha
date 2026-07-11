# Design Standard

## Table of Contents
- [Purpose](#purpose)
- [UX Principles](#ux-principles)
- [Accessibility](#accessibility)
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
- [Audit Rules](#audit-rules)
- [Validation Rules](#validation-rules)
- [Generation Rules](#generation-rules)
- [Enhancement Rules](#enhancement-rules)
- [Summary](#summary)
- [Common Mistakes](#common-mistakes)
- [Documentation Folder](#documentation-folder)
- [Usage](#usage)
- [Related](#related)
- [Design as a Documentation Collection](#design-as-a-documentation-collection)
- [Single Responsibility](#single-responsibility)
- [Design Principles](#design-principles)
- [Technology Independence](#technology-independence)
- [Cross-Repository Design](#cross-repository-design)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Design Documentation exists — its role in establishing reusable design language across the product ecosystem
> **out_of_scope:** Feature-specific design decisions, implementation details, technology choices, component specifications
> **contributes:** Establishes the root intent that all design sections and downstream Feature Design documents derive from
> **relationships:** References Vision(01) and Philosophy(02); feeds Feature Design(09) and Architecture(05)
> **responsibilities:** Define what Design Documentation is, what it covers, and how it relates to other documentation standards
> **generation_rules:** Start with the problem Design Documentation solves; state what it defines and what it does not; avoid feature-specific language
> **enhancement_rules:** Strengthen clarity without adding scope; remove ambiguity; ensure purpose survives feature and technology changes
> **validation_rules:** Purpose is clearly defined; no implementation details present; distinguishable from Feature Design; technology-independent
> **audit_rules:** Must exist; must not contain feature lists; must not reference specific technologies; must distinguish Design from Feature Design

This document defines the standard for Design Documentation within the engineering documentation ecosystem.

Design Documentation establishes the shared design language, design principles, interaction philosophy, and user experience standards that govern an entire product or product ecosystem.

Unlike Feature Design, Design Documentation is **not feature specific**.

Instead, it provides reusable guidance that ensures every feature delivers a consistent and predictable user experience.

Design Documentation defines **how products should be designed**.

It does not describe how individual features behave.

---

## UX Principles

> **semantic_type:** `ux_principles`
> **scope:** User experience principles that govern how users interact with the product — interaction philosophy, navigation, feedback, and discoverability
> **out_of_scope:** Visual design details, accessibility requirements, implementation frameworks, component specifications, feature-specific workflows
> **contributes:** Ensures every feature delivers a consistent, predictable, and user-centered experience across the product ecosystem
> **relationships:** Derived from Design Principles; referenced by Feature Design(09) for interaction decisions; complements Accessibility standards
> **responsibilities:** Define reusable UX principles covering interaction patterns, navigation philosophy, feedback mechanisms, and discoverability
> **generation_rules:** Extract from user-centered values; describe interaction philosophy at the product level; avoid feature-specific or technology-specific language
> **enhancement_rules:** Strengthen user-centeredness; remove feature-specific UX guidance; ensure principles remain technology-independent
> **validation_rules:** UX principles are reusable across features; technology-independent; actionable for designers and engineers; user-focused
> **audit_rules:** Must exist; must not reference specific UI frameworks; must not describe feature-specific interactions; must be technology-independent

*(To be written by the domain expert. This section defines user experience principles governing interaction, navigation, feedback, and discoverability.)*

---

## Accessibility

> **semantic_type:** `accessibility`
> **scope:** Accessibility standards and principles ensuring the product is usable by people of all abilities — inclusive design, assistive technology support, and compliance targets
> **out_of_scope:** Feature-specific accessibility implementations, component-level ARIA patterns, testing scripts, code-level accessibility fixes
> **contributes:** Ensures every feature and design decision considers accessibility by default, reducing retrofitting costs and expanding user reach
> **relationships:** Complements UX Principles; referenced by Feature Design(09) and Engineering(07) for compliance; aligns with legal and organizational standards
> **responsibilities:** Define accessibility principles, compliance targets, inclusive design guidelines, and assistive technology requirements
> **generation_rules:** Reference applicable standards (WCAG, etc.); state principles at the product level; describe inclusive design philosophy before technical requirements
> **enhancement_rules:** Update compliance targets when standards evolve; strengthen inclusive design guidance; remove feature-specific accessibility rules
> **validation_rules:** Accessibility standards are documented; compliance targets are defined; principles are technology-independent; applicable to the product domain
> **audit_rules:** Must exist if the product has a user interface; must reference applicable standards; must not contain component-level ARIA patterns; must be technology-independent

*(To be written by the domain expert. This section defines accessibility standards and inclusive design principles for the product.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Design-level limitations and non-functional requirements that bound how the product can be designed — platform constraints, regulatory requirements, and organizational mandates
> **out_of_scope:** Technical architecture constraints, implementation limitations, coding standards, infrastructure requirements
> **contributes:** Provides guardrails that Feature Design and Architecture must respect; prevents designs that violate organizational or regulatory requirements
> **relationships:** Derived from Vision(01) and External Context; referenced by Feature Design(09) and Architecture(05) as binding constraints
> **responsibilities:** Document design-level constraints including platform requirements, regulatory obligations, organizational mandates, and performance boundaries
> **generation_rules:** Extract from Vision, regulatory requirements, and organizational standards; state as binding constraints, not suggestions; distinguish from preferences
> **enhancement_rules:** Add constraints when new regulations or mandates emerge; remove obsolete constraints; ensure constraints remain binding and current
> **validation_rules:** Constraints are binding and actionable; clearly distinguished from preferences; current and applicable; technology-independent where possible
> **audit_rules:** Must exist if the product has regulatory or platform constraints; must state binding nature; must not conflate constraints with preferences

*(To be written by the domain expert. This section defines design-level constraints, non-functional requirements, and regulatory or organizational mandates.)*

---

## Required Sections

Every Design document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Design Principles | `design_principles` | ✓ | Principles, Core Design |
| UX Principles | `ux_principles` | ✓ | User Experience Principles, UX Guidelines |
| Accessibility | `accessibility` | ✓ | A11y, Accessibility Standards |
| Purpose | `purpose` | | Overview, Summary |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Design Documentation aims to:

* Give every feature-specific design a shared set of reusable principles instead of reinventing them.
* Keep the product's user experience consistent across features.
* Separate reusable design guidance from any single feature's design.

---

## Non-Goals

Design Documentation does not define:

* Product requirements
* Feature specifications
* Feature workflows
* Screen implementations
* Technical architecture
* Engineering decisions
* Technology selection
* Source code
* Component implementation

These responsibilities belong to other documentation standards.

---

## Success Criteria

Design Documentation is successful when:

* Every feature follows a common design language.
* Multiple applications exhibit consistent user experiences.
* Designers and engineers share a common understanding of design principles.
* Feature Design can be created without redefining design philosophy.
* Architecture complements rather than replaces design.
* AI systems can consistently apply design principles across repositories.

---

## Responsibilities

Design Documentation is responsible for defining:

* Product Design Philosophy
* Design Principles
* User Experience Principles
* Interaction Principles
* Information Design Principles
* Navigation Principles
* Accessibility Principles
* Localization Principles
* Visual Language
* Design System Guidelines
* Cross-Application Consistency
* Content Design Principles

Design Documentation establishes common design rules that every feature should follow.

---

## Scope

Design Documentation may include:

* Design Philosophy
* Product Design Principles
* User Experience Principles
* Interaction Guidelines
* Navigation Standards
* Information Architecture Principles
* Visual Design Language
* Typography Guidelines
* Color Principles
* Spacing Principles
* Iconography Principles
* Accessibility Standards
* Localization Standards
* Content Guidelines
* Error Communication Principles
* Notification Principles
* Responsive Design Principles
* Animation Principles
* Design Tokens
* Component Usage Principles

Projects should document only the design domains relevant to the repository.

Design Documentation is intentionally modular.

---

## Out of Scope

Design Documentation must not describe:

* Product Vision
* Feature Requirements
* Feature Workflows
* Feature Behavior
* Screen Specifications
* User Stories
* Technical Architecture
* Technology Selection
* Frameworks
* Source Code
* APIs
* Algorithms
* Implementation Details

These responsibilities belong to other documentation standards.

---

## Inputs

Design Documentation derives from:

* Vision
* Philosophy (02)
* User Experience Goals
* Product Principles
* Organizational Standards

Design Documentation should not derive from feature implementation.

---

## Outputs

Design Documentation provides guidance for:

* Feature Design
* Architecture
* Engineering Decisions
* Design Systems
* UI Libraries
* Mockups
* Prototypes
* Product Development

Every Feature Design should align with the Design Documentation.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Design Documentation connects to the documentation hierarchy — the derivation chain from Vision through Design to Feature Design
> **out_of_scope:** Implementation traceability, test traceability, bug tracking, version history, code-level tracing
> **contributes:** Makes Design's influence visible and verifiable across the entire documentation ecosystem
> **relationships:** Derived from Vision(01); feeds Feature Design(09) and Architecture(05); referenced by Feature Technical Design(10)
> **responsibilities:** Show the derivation path from Vision to Design; show how Design feeds Feature Design and downstream standards
> **generation_rules:** Use the tier diagram; list which standards derive from or reference Design; assert the non-contradiction rule
> **enhancement_rules:** Update the diagram when new standards are added; ensure derivation paths remain accurate
> **validation_rules:** Derivation paths are complete; no orphaned standards; non-contradiction rule is stated
> **audit_rules:** Must exist; must include tier diagram; must list downstream standards; must state non-contradiction constraint

Design Documentation remains traceable.

```text
Vision
    │
    ├────────────────────────┐
    ↓                        ↓
Feature                Design Documentation
    │                        │
    │                External Context (optional)
    │                        │
    └────────────────────────┘
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

Design principles influence every downstream design decision.

---

## Relationships

| Document       | Relationship                       |
| -------------- | ---------------------------------- |
| Vision         | Design supports the product vision |
| Feature        | Independent                        |
| Feature Design | Derived from Design principles     |
| Architecture   | Complements Design                 |
| Engineering    | Influenced by Design constraints   |
| README         | May reference Design philosophy    |

---

## Required Characteristics

Design Documentation should be:

* Consistent across features
* Reusable rather than feature-specific
* Accessible by default
* Technology-independent
* User-centered
* Stable across releases

---

## Audit Rules

An audit should verify:

* Design principles are reusable.
* Design remains technology independent.
* Feature-specific behavior has not been introduced.
* Design philosophy is clearly defined.
* Accessibility guidance exists where appropriate.
* Localization guidance exists where appropriate.
* Documents remain modular.
* Responsibilities do not overlap.
* Cross-repository reuse is encouraged.

Design quality is evaluated across the complete Design Documentation collection.

---

## Validation Rules

Design Documentation is considered valid when:

* Design philosophy is clearly defined.
* Design principles are documented.
* User experience principles are reusable.
* Accessibility guidance is documented where applicable.
* Localization guidance exists where applicable.
* Design remains implementation independent.
* Design documents remain modular.
* No feature-specific workflows dominate the documentation.

Validation applies to the Design Documentation collection rather than individual files.

---

## Generation Rules

When generating Design Documentation:

* Focus on reusable design principles.
* Describe product-wide guidance.
* Separate principles from feature behavior.
* Keep documents modular.
* Avoid implementation terminology.
* Maintain technology independence.
* Optimize for reuse across multiple features and applications.

---

## Enhancement Rules

When enhancing Design Documentation:

* Improve clarity.
* Strengthen design consistency.
* Remove feature-specific content.
* Remove implementation leakage.
* Improve organization.
* Split oversized documents.
* Preserve design philosophy.
* Strengthen cross-product reuse.

Design should become more reusable through refinement.

---

## Summary

Design Documentation is the reusable design knowledge of the product or product ecosystem.

It is a modular collection of documents that defines design philosophy, interaction principles, accessibility, localization, navigation, visual language, and other shared design standards.

Its purpose is to establish a consistent design language that guides every feature, architecture, and implementation while remaining independent of both feature-specific behavior and implementation technologies.

---

## Common Mistakes

Examples include:

* Describing individual feature workflows.
* Documenting application screens.
* Embedding mockups for specific features.
* Explaining implementation.
* Discussing frontend frameworks.
* Mixing architecture with design.
* Mixing feature behavior with design philosophy.

These should be reported during audits.

---

## Documentation Folder

Design documents live under:

```text
docs/raw/design/
```

---

## Usage

Written once and shared across every feature — designers/UX leads author it, Feature Design authors reference it rather than redefine it. Use `samgraha search --domain design` to pull shared design principles into a Feature Design writing session, and `samgraha audit --domain design` to catch feature-specific content that leaked in.

## Related

- [Feature Design Standard](09-feature-design-standards.md) — applies these principles to one feature
- [Philosophy Standard](02-philosophy-standards.md) — guides Design's principles
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Design as a Documentation Collection

Design Documentation is a collection of related documents.

Example:

```text
design/

    philosophy/

        design-philosophy.md

        design-principles.md

    interaction/

        interaction-principles.md

        navigation-principles.md

        feedback-principles.md

    accessibility/

        accessibility-principles.md

    localization/

        localization-principles.md

    visual/

        color-principles.md

        typography-principles.md

        spacing-principles.md

        iconography-principles.md

    content/

        writing-guidelines.md

        terminology.md

    responsive/

        responsive-principles.md
```

Each document should describe one design concern.

Responsibilities should not overlap.

---

## Single Responsibility

Every Design document should describe one reusable design concern.

Examples include:

* Navigation Principles
* Accessibility Principles
* Localization Principles
* Visual Language
* Design Philosophy
* Typography
* Content Guidelines
* Interaction Principles

Large documents should be decomposed into smaller focused documents.

---

## Design Principles

> **semantic_type:** `design_principles`
> **scope:** Core reusable design principles that govern all design decisions — consistency, simplicity, predictability, accessibility, and other foundational values
> **out_of_scope:** Feature-specific design rules, implementation guidelines, technology-specific patterns, component specifications
> **contributes:** Provides the shared design language that every Feature Design references rather than redefining
> **relationships:** Derived from Vision(01) Philosophy; referenced by Feature Design(09), UX Principles, and Accessibility; guides Architecture(05)
> **responsibilities:** Define principles that remain stable across features, applications, and technology changes
> **generation_rules:** Extract from product philosophy; express as memorable, stable values; use concrete examples where helpful; keep at the product level
> **enhancement_rules:** Add principles when new design concerns emerge; merge overlapping principles; strengthen clarity without adding scope
> **validation_rules:** Principles are technology-independent; reusable across features; memorable; actionable when a design decision is ambiguous
> **audit_rules:** Must exist; must not reference specific technologies; must not describe feature-specific behavior; must be evaluable against real design decisions

Design Documentation should establish reusable principles such as:

* Consistency
* Simplicity
* Predictability
* Accessibility
* Discoverability
* Learnability
* Inclusiveness
* Feedback
* Flexibility
* Progressive Disclosure
* Error Prevention
* User Empowerment

Projects may define additional principles appropriate to their domain.

---

## Technology Independence

Design Documentation should remain implementation independent.

It should describe:

* user experience
* interaction philosophy
* presentation principles
* communication principles

rather than

* UI frameworks
* CSS libraries
* frontend frameworks
* component implementations
* rendering technologies

Technology belongs to Engineering.

---

## Cross-Repository Design

Design Documentation may be shared across multiple repositories.

Examples include:

* Shared Design Language
* Shared Accessibility Standards
* Shared Localization Standards
* Shared Navigation Principles
* Shared Design System Guidelines

Repositories should reference shared Design Documentation rather than duplicating it.

---

## Quality Requirements

Design Documentation should be:

* Reusable
* Modular
* Consistent
* Technology Independent
* Product Focused
* Human Centered
* Maintainable
* Reviewable
* Scalable

Design principles should remain stable over time.

---
