# Feature Design Standard

## Table of Contents
- [Purpose](#purpose)
- [User Experience](#user-experience)
- [Workflow](#workflow)
- [States](#states)
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
- [One-to-One Mapping](#one-to-one-mapping)
- [Design Principle Application](#design-principle-application)
- [External Context Application](#external-context-application)
- [Feature Design Principles](#feature-design-principles)
- [Technology Independence](#technology-independence)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Feature Design Documentation exists — its role in translating a single Feature Specification into a user-centered design by applying shared Design principles and External Context
> **out_of_scope:** Feature specifications, implementation details, architecture decisions, technology choices, reusable design guidance
> **contributes:** Establishes the root intent for all Feature Design sections and downstream standards; grounds every UX decision in the documentation ecosystem
> **relationships:** Derived from Vision(01), Feature(04), and Design(06); feeds Feature Technical Design(10) and Engineering(07)
> **responsibilities:** Define what Feature Design is, what it covers, what it does not cover, and how it relates to other documentation standards
> **generation_rules:** State what Feature Design is; explain what it defines and what it does not; reference the one-to-one relationship with Feature; reference the broader ecosystem
> **enhancement_rules:** Strengthen clarity of scope boundaries; remove overlap with downstream standards; keep stable over time
> **validation_rules:** Purpose is clearly defined; no implementation details present; one-to-one relationship with Feature is stated; boundary with Design and Architecture is explicit
> **audit_rules:** Must exist; must not contain feature specifications; must define what Feature Design is and is not; must state the one-to-one relationship

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
[1 paragraph: what Feature Design is and its role in the documentation ecosystem]
[1 paragraph: what it defines — user-centered design for a single feature]
[1 paragraph: what it does not define — no implementation, architecture, or technology]
[1 sentence: one-to-one relationship with Feature Specification]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** none

This document defines the standard for Feature Design Documentation within the engineering documentation ecosystem.

Feature Design translates a single Feature Specification into a complete user-centered design by applying the shared principles defined by the Design Documentation together with any relevant External Context.

Feature Design is **not** reusable design guidance.

It is the application of reusable design principles to one specific feature.

Every Feature Design document has a strict one-to-one relationship with a Feature Specification.

Feature Design explains **how users should experience a feature**.

It does not define implementation, architecture, or engineering decisions.

---

## User Experience

> **semantic_type:** `user_experience`
> **scope:** The complete user-facing experience of the feature — how users discover, interact with, and complete the feature's purpose from start to finish
> **out_of_scope:** Implementation details, framework choices, component internals, technical architecture, backend logic
> **contributes:** Defines the primary deliverable of Feature Design — the authoritative UX specification that Architecture and Engineering must realize
> **relationships:** Derived from Feature Specification functional requirements; applies Design(06) UX Principles and Accessibility; feeds Feature Technical Design(10) and UI Prototypes
> **responsibilities:** Describe the complete user experience including discovery, interaction, feedback, error handling, empty states, loading states, and success states
> **generation_rules:** Start from Feature Specification functional requirements; apply Design UX Principles; describe user interactions without referencing technology; include all states and transitions
> **enhancement_rules:** Improve clarity of user interactions; remove implementation leakage; strengthen consistency with Design Principles; ensure all states are covered
> **validation_rules:** User experience is complete and technology-independent; all functional requirements have corresponding UX behaviors; Design Principles are applied not redefined
> **audit_rules:** Must exist; must cover all functional requirements; must not reference specific technologies or frameworks; must include error, empty, and loading states

### Template

> **minimum_content:** 2 paragraphs plus subsections
> **length_guidance:** extensive
> **diagram_requirements:** flowchart

```markdown
### User Experience

> **semantic_type:** `user_experience`
> **scope:** ...
> **out_of_scope:** ...
> **contributes:** ...
> **relationships:** ...
> **responsibilities:** ...
> **generation_rules:** ...
> **enhancement_rules:** ...
> **validation_rules:** ...
> **audit_rules:** ...

[Introduction: overall UX intent for the feature, derived from Feature Specification]

### Discovery

[How users first encounter and access the feature]

### Primary Interaction

[Core user flow — what users do to accomplish the feature's purpose]

### Feedback and Response

[How the system communicates results of user actions]

### Error Handling

[What users see and can do when something goes wrong]

### Empty State

[What users see when there is no data or content to display]

### Loading State

[What users see while the system is processing]

### Success State

[What users see when the task completes successfully]

### Accessibility

[How the experience accommodates assistive technologies]

### Localization

[How the experience adapts for different languages and regions]
```

**Required subsections:** Discovery, Primary Interaction, Feedback and Response, Error Handling, Empty State, Loading State, Success State
**Optional subsections:** Accessibility, Localization
**Required diagrams:** flowchart (primary user interaction flow)
**Required cross-references:** Feature Specification, Design Documentation UX Principles

*(To be written by the domain expert. This section defines the complete user experience for the feature.)*

---

## Workflow

> **semantic_type:** `workflow`
> **scope:** The step-by-step user workflow for completing the feature's primary task — the ordered sequence of user actions and system responses
> **out_of_scope:** Implementation sequence, API call order, database transaction flow, internal processing steps
> **contributes:** Provides the behavioral contract that engineers implement and testers validate; ensures users can complete the feature end-to-end
> **relationships:** Derived from Feature Specification functional requirements and User Experience; feeds Feature Technical Design(10) and Acceptance Criteria
> **responsibilities:** Document every user workflow as a clear sequence of user actions and system responses, including branching and error recovery
> **generation_rules:** Start from the Feature Specification; map each functional requirement to a workflow step; describe actions from the user's perspective; include decision points and error paths
> **enhancement_rules:** Simplify workflow steps; remove unnecessary steps; clarify branching logic; ensure every functional requirement maps to at least one workflow
> **validation_rules:** Workflows are complete and unambiguous; every functional requirement is covered; error recovery is documented; no implementation details present
> **audit_rules:** Must exist; must cover all primary user tasks; must include error recovery; must not reference implementation internals; must be written from user perspective

### Template

> **minimum_content:** 1 workflow with at least 3 steps
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

```markdown
### Workflow

> **semantic_type:** `workflow`
> **scope:** ...
> **out_of_scope:** ...
> **contributes:** ...
> **relationships:** ...
> **responsibilities:** ...
> **generation_rules:** ...
> **enhancement_rules:** ...
> **validation_rules:** ...
> **audit_rules:** ...

### Primary Workflow

[Step-by-step sequence of user actions and system responses]

1. User does X → System responds with Y
2. User does A → System responds with B
3. ...

### Alternative Workflows

[Branching paths for different user choices or conditions]

### Error Recovery

[What happens and what users can do when a step fails]
```

**Required subsections:** Primary Workflow, Error Recovery
**Optional subsections:** Alternative Workflows
**Required diagrams:** flowchart (primary workflow)
**Required cross-references:** Feature Specification, User Experience

*(To be written by the domain expert. This section defines the step-by-step user workflow for the feature.)*

---

## States

> **semantic_type:** `states`
> **scope:** The observable states of the feature's UI and behavior — what users see and experience in each state and how transitions between states occur
> **out_of_scope:** Internal application state management, database state, caching strategies, memory management, component lifecycle
> **contributes:** Ensures every possible feature state is designed rather than left to implementation improvisation; prevents missing states from reaching production
> **relationships:** Derived from User Experience and Workflow; referenced by Feature Technical Design(10) for implementation; validated against Acceptance Criteria
> **responsibilities:** Define every observable state including initial, active, waiting, empty, error, success, and terminal states; document transitions between them
> **generation_rules:** Enumerate all observable states from the User Experience; define what users see in each state; document valid transitions; include empty and error states explicitly
> **enhancement_rules:** Add missing states discovered during implementation; clarify transition triggers; remove states that duplicate other features; ensure every transition has a clear trigger
> **validation_rules:** All observable states are documented; transitions are explicit; empty and error states exist for every applicable state; no internal state management described
> **audit_rules:** Must exist; must include empty and error states; must document all state transitions; must not reference internal state management; must be observable from user perspective

### Template

> **minimum_content:** state table plus transition descriptions
> **length_guidance:** moderate
> **diagram_requirements:** state

```markdown
### States

> **semantic_type:** `states`
> **scope:** ...
> **out_of_scope:** ...
> **contributes:** ...
> **relationships:** ...
> **responsibilities:** ...
> **generation_rules:** ...
> **enhancement_rules:** ...
> **validation_rules:** ...
> **audit_rules:** ...

### State Definitions

| State | Description | What User Sees |
|-------|-------------|----------------|
| Initial | ... | ... |
| Active | ... | ... |
| Empty | ... | ... |
| Error | ... | ... |
| Success | ... | ... |

### State Transitions

| From | To | Trigger | User Action |
|------|----|---------|-------------|
| Initial | Active | ... | ... |
| Active | Success | ... | ... |
| Active | Error | ... | ... |
| ... | ... | ... | ... |
```

**Required subsections:** State Definitions, State Transitions
**Optional subsections:** none
**Required diagrams:** state (state transition diagram)
**Required cross-references:** User Experience, Workflow

*(To be written by the domain expert. This section defines the observable states and state transitions for the feature.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** User-facing constraints introduced by external systems — limitations on the feature's design that originate from platform, regulatory, or external system requirements
> **out_of_scope:** Implementation constraints, technical limitations, framework restrictions, coding standards, infrastructure requirements
> **contributes:** Makes external design limitations explicit so Architecture and Engineering do not discover them late; prevents designs that violate platform or regulatory requirements
> **relationships:** Derived from External Context and Platform requirements; referenced by Feature Technical Design(10) and Architecture(05) as binding constraints
> **responsibilities:** Document every user-facing constraint that limits how the feature can be designed, including platform rules, regulatory requirements, and external system behaviors
> **generation_rules:** Identify constraints from External Context and platform requirements; state each constraint as a binding limitation; distinguish constraints from preferences; cite the source
> **enhancement_rules:** Add constraints when new external requirements emerge; remove constraints that no longer apply; clarify ambiguous constraints; update source citations
> **validation_rules:** Constraints are explicit and sourced; binding constraints are distinguished from preferences; no implementation constraints included; applicable to the feature
> **audit_rules:** Must exist if the feature has external constraints; must cite the source of each constraint; must not describe implementation limitations; must distinguish hard from soft constraints

### Template

> **minimum_content:** constraint list with sources
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
### Constraints

> **semantic_type:** `constraints`
> **scope:** ...
> **out_of_scope:** ...
> **contributes:** ...
> **relationships:** ...
> **responsibilities:** ...
> **generation_rules:** ...
> **enhancement_rules:** ...
> **validation_rules:** ...
> **audit_rules:** ...

| Constraint | Type | Source | Impact on Design |
|-----------|------|--------|-----------------|
| [description] | Hard/Advisory | [External Context source] | [what it prevents or requires] |
| ... | ... | ... | ... |
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** External Context, Platform requirements

*(To be written by the domain expert. This section defines user-facing constraints from external systems.)*

---

## Required Sections

Every Feature Design document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases | Content Requirements |
|---------|--------------|----------|---------|---------------------|
| User Experience | `user_experience` | ✓ | UX, User Flow | Complete user-facing experience covering discovery, interaction, feedback, errors, empty, loading, and success states |
| Workflow | `workflow` | ✓ | User Workflow, Flow | Ordered sequence of user actions and system responses including branching and error recovery |
| States | `states` | ✓ | UI States, Application States, State Transitions | Every observable UI state and all valid transitions between them |
| Purpose | `purpose` | | Overview, Summary | Definition of Feature Design, scope boundaries, and one-to-one relationship with Feature |
| Non-Goals | `non_goals` | | Non Goals, Out of Scope, Not In Scope | Excluded responsibilities with the owning standard identified for each |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements | External user-facing constraints with source citation and binding vs. advisory distinction |
| Traceability | `traceability` | | Traces To, Derived From | Derivation chain, one-to-one mapping assertion, and downstream consumers |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Feature Design aims to:

* Give every feature a single authoritative user-experience specification.
* Keep UX decisions traceable to shared Design principles rather than invented per feature.
* Bridge functional requirements and architecture without prescribing implementation.

---

## Non-Goals

> **semantic_type:** `non_goals`
> **scope:** What Feature Design Documentation explicitly does not define — boundaries that prevent scope creep into other documentation standards
> **out_of_scope:** Goals, success criteria, any positive framing of intent; this section only lists exclusions
> **contributes:** Prevents Feature Design from conflating with Architecture, Engineering, or Design Documentation; clarifies ownership of excluded responsibilities
> **relationships:** Mirrors Goals; referenced by Audit Rules to detect scope violations in Feature Design documents
> **responsibilities:** List every responsibility that belongs to a different documentation standard, with the owning standard identified
> **generation_rules:** Invert the Goals list; reference the specific downstream standard that owns each excluded responsibility; keep the list current with ecosystem changes
> **enhancement_rules:** Add new exclusions when scope creep is detected; remove exclusions for responsibilities that migrate to Feature Design; ensure no ambiguity in ownership
> **validation_rules:** All excluded responsibilities have a clear owner in another standard; no overlap between excluded items and defined scope
> **audit_rules:** Must exist; each excluded item must name its owning standard; must not contradict the Goals list; must not contain positive scope statements

### Template

> **minimum_content:** bulleted list with owning standards
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
### Non-Goals

> **semantic_type:** `non_goals`
> **scope:** ...
> **out_of_scope:** ...
> **contributes:** ...
> **relationships:** ...
> **responsibilities:** ...
> **generation_rules:** ...
> **enhancement_rules:** ...
> **validation_rules:** ...
> **audit_rules:** ...

Feature Design does not define:

* [Responsibility] — owned by [Standard Name]
* [Responsibility] — owned by [Standard Name]
* ...
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Goals, owning standard for each excluded item

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

## Success Criteria

Feature Design is successful when:

* Every Feature has one corresponding Feature Design.
* Shared Design principles are consistently applied.
* Relevant External Context has been incorporated where necessary.
* Users experience consistent interactions across the product.
* Engineers understand how the feature should behave.
* Architecture can realize the design without redefining user behavior.
* AI systems can implement user experience consistently without inventing workflows or violating external platform conventions.

---

## Responsibilities

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

## Scope

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

## Out of Scope

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

## Inputs

Feature Design derives from:

* Feature Specification
* Design Documentation
* Relevant External Context
* Product Constraints

Feature Design should not derive from implementation.

---

## Outputs

Feature Design provides direction for:

* Architecture
* Feature Technical Design
* UI Prototypes
* Mockups
* User Acceptance Testing
* Engineering

Implementation should realize the documented feature design.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Feature Design connects to the documentation hierarchy — the derivation chain from Vision through Feature and Design to Engineering
> **out_of_scope:** Implementation traceability, test traceability, bug tracking, version history, code-level tracing
> **contributes:** Makes Feature Design's influence visible and verifiable; ensures every design decision can be traced to its originating feature and shared design principles
> **relationships:** Derived from Feature(04) and Design(06); feeds Feature Technical Design(10); constrained by Architecture(05); consumed by Engineering(07)
> **responsibilities:** Show the derivation path from Vision through Feature and Design to Feature Design; assert the one-to-one relationship with Feature; show downstream consumers
> **generation_rules:** Use the tier diagram; list which standards derive from or reference Feature Design; state the one-to-one mapping constraint; reference non-contradiction rule
> **enhancement_rules:** Update the diagram when new standards are added; ensure derivation paths remain accurate; keep the one-to-one mapping explicit
> **validation_rules:** Derivation paths are complete; one-to-one mapping is stated; no orphaned standards; non-contradiction rule is documented
> **audit_rules:** Must exist; must include tier diagram; must list downstream standards; must state the one-to-one mapping constraint; must state non-contradiction rule

### Template

> **minimum_content:** derivation diagram plus relationship list
> **length_guidance:** concise
> **diagram_requirements:** custom

```markdown
### Traceability

> **semantic_type:** `traceability`
> **scope:** ...
> **out_of_scope:** ...
> **contributes:** ...
> **relationships:** ...
> **responsibilities:** ...
> **generation_rules:** ...
> **enhancement_rules:** ...
> **validation_rules:** ...
> **audit_rules:** ...

[Derivation chain diagram from Vision through Feature and Design to Feature Design]

[One-to-one mapping assertion with Feature Specification]

[Table of downstream consumers]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** custom (derivation chain diagram)
**Required cross-references:** Feature(04), Design(06), Feature Technical Design(10), Engineering(07)

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

## Relationships

| Document                 | Relationship                             |
| ------------------------ | ---------------------------------------- |
| Feature                  | One-to-one mapping                       |
| Design                   | Applies shared design principles         |
| External Context         | Applies user-facing external constraints |
| Architecture             | Provides structural realization          |
| Feature Technical Design | Provides technical realization           |
| Engineering              | Explains implementation decisions        |

---

## Required Characteristics

Feature Design should be:

* User-centered
* Feature-specific
* Cohesive
* Traceable to exactly one Feature
* Technology-independent
* Accessible and localizable by default

---

## Audit Rules

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

## Validation Rules

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

## Generation Rules

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

## Enhancement Rules

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

## Summary

Feature Design Documentation is the design realization of a single Feature Specification.

Each document maintains a strict one-to-one relationship with its corresponding Feature, applying the shared principles defined by the Design Documentation together with any relevant External Context to describe how users should experience that feature.

Its purpose is to bridge functional requirements and system architecture while preserving consistency, usability, accessibility, localization, platform conventions, and technology independence across the product ecosystem.

---

## Common Mistakes

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

## Documentation Folder

Feature Design documents live under:

```text
docs/raw/feature-design/
```

---

## Usage

Written one-to-one with a Feature, by whoever owns UX for that feature, after the Feature spec exists and before Feature Technical Design starts. Use `samgraha audit --domain feature-design` to confirm every Feature has a matching Feature Design and that shared Design principles were applied rather than redefined.

## Related

- [Feature Standard](04-feature-standards.md) — one-to-one mapping
- [Design Standard](06-design-standards.md) — shared principles this standard applies
- [Feature Technical Standard](10-feature-technical-standards.md) — technical realization of this design
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## One-to-One Mapping

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

## Design Principle Application

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

## External Context Application

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

## Feature Design Principles

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

## Technology Independence

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

## Quality Requirements

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
