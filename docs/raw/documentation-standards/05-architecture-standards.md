# Architecture Standard

## Table of Contents
- [Purpose](#purpose)
- [System Overview](#system-overview)
- [Component Model](#component-model)
- [Communication](#communication)
- [Data Flow](#data-flow)
- [Security](#security)
- [Rationale](#rationale)
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
- [Architecture as a Documentation Collection](#architecture-as-a-documentation-collection)
- [Single Responsibility](#single-responsibility)
- [Architectural Boundaries](#architectural-boundaries)
- [Architectural Principles](#architectural-principles)
- [Technology Independence](#technology-independence)
- [Cross-Repository Architecture](#cross-repository-architecture)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Architecture Documentation exists — its reason for being within the documentation ecosystem
> **out_of_scope:** Feature lists, implementation details, technology choices, component specifics
> **contributes:** Establishes the root intent that all Architecture sections and downstream decisions derive from
> **relationships:** Architecture(05) is Tier 2; derived from Vision(01) and Philosophy(02); constrains Feature Technical Design(10)
> **responsibilities:** Define why Architecture Documentation is needed and what it achieves for the project
> **generation_rules:** Start from the project's structural needs; describe the purpose without referencing specific components or technologies
> **enhancement_rules:** Strengthen clarity without adding scope; remove ambiguity; ensure purpose survives component and technology changes
> **validation_rules:** Purpose is clearly defined; no implementation details present; understandable without code knowledge; stable over time
> **audit_rules:** Must exist; must not contain feature lists; must not reference specific technologies; must be technology-independent

### Template

> **minimum_content:** 2 paragraphs
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
This document defines the standard for [Documentation Type] within the [ecosystem name] documentation ecosystem.

[Documentation Type] describes [what it describes].

Unlike [related type], [distinctive characteristic].

[Core purpose statement.]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** none

This document defines the standard for Architecture Documentation within the engineering documentation ecosystem.

Architecture Documentation describes the structural organization of a system.

Unlike other documentation types, Architecture is not expected to be represented by a single document.

Instead, it is a structured collection of related documents that collectively describe the responsibilities, boundaries, interactions, and organization of the system.

Architecture explains **how responsibilities are organized**.

It does not explain implementation details.

---

## System Overview

> **semantic_type:** `system_overview`
> **scope:** High-level description of the system — what it is, what it does, and how it is organized at the top level
> **out_of_scope:** Component internals, implementation details, class hierarchies, API contracts, code organization
> **contributes:** Provides the entry point for understanding the system; grounds all subsequent architectural sections in a shared context
> **relationships:** Derived from Vision(01); referenced by Component Model and all downstream architecture sections; consumed by Feature Technical Design(10)
> **responsibilities:** Describe the system's purpose, primary capabilities, and structural approach in terms a new contributor can grasp quickly
> **generation_rules:** Start from the Vision and project scope; describe the system at the highest abstraction level; use diagrams where helpful; avoid technology specifics
> **enhancement_rules:** Clarify scope boundaries; remove implementation leakage; ensure the overview remains accurate as the system evolves
> **validation_rules:** Overview is concise and accurate; no component internals described; no technology specifics; serves as a reliable entry point
> **audit_rules:** Must exist; must not describe component internals; must not reference specific libraries or frameworks; must be understandable without code knowledge

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** moderate
> **diagram_requirements:** component

```markdown
## System Overview

> [metadata block]

### Overview
[1-2 paragraphs: system purpose, primary capabilities, high-level approach]

### Structural Approach
[1 paragraph: how the system is organized at the top level]

### Diagram
[High-level component or system context diagram]
```

**Required subsections:** Overview, Diagram
**Optional subsections:** Structural Approach, Key Capabilities
**Required diagrams:** system context or component overview diagram
**Required cross-references:** Vision(01)

*(To be written by the domain expert. This section defines the high-level system description and structural approach.)*

---

## Component Model

> **semantic_type:** `component_model`
> **scope:** The system's components — what they are, what each owns, and how they relate to one another
> **out_of_scope:** Internal class design, function implementations, algorithm details, code structure
> **contributes:** Makes component responsibilities and ownership explicit; provides the structural foundation for Feature Technical Design
> **relationships:** Derived from System Overview; referenced by Communication Paths and Data Flow; constrains Feature Technical Design(10)
> **responsibilities:** Define each component's responsibility, ownership boundaries, and relationship to other components
> **generation_rules:** Identify major system responsibilities; assign each to a component; define boundaries explicitly; avoid implementation detail
> **enhancement_rules:** Split components when responsibilities grow; merge overlapping components; strengthen ownership definitions
> **validation_rules:** Each component has a clear responsibility; responsibilities do not overlap; boundaries are explicit; ownership is assigned
> **audit_rules:** Must exist; must not describe class hierarchies or function signatures; must define responsibility and ownership for each component; must not duplicate Communication or Data Flow content

### Template

> **minimum_content:** 1 subsection per component
> **length_guidance:** extensive
> **diagram_requirements:** component

```markdown
## Component Model

> [metadata block]

### Components

#### [Component Name]
- **Responsibility:** [what this component owns]
- **Ownership:** [data/processes owned]
- **Interfaces:** [how other components interact with it]

#### [Component Name]
- **Responsibility:** [what this component owns]
- **Ownership:** [data/processes owned]
- **Interfaces:** [how other components interact with it]

### Component Diagram
[Diagram showing all components and their relationships]
```

**Required subsections:** Components (with one entry per component), Component Diagram
**Optional subsections:** Component Relationships, Boundary Definitions
**Required diagrams:** component relationship diagram
**Required cross-references:** System Overview, Communication, Data Flow

*(To be written by the domain expert. This section defines the system's components, their responsibilities, and how they relate.)*

---

## Communication

> **semantic_type:** `communication_paths`
> **scope:** How components communicate — the paths, patterns, and contracts that govern inter-component interaction
> **out_of_scope:** Network protocols, API implementations, message serialization formats, transport layer details
> **contributes:** Makes inter-component interaction explicit and predictable; prevents hidden dependencies and coupling
> **relationships:** Derived from Component Model; referenced by Data Flow; constrains Feature Technical Design(10) integration decisions
> **responsibilities:** Define communication paths, interaction patterns, and the contracts that govern component communication
> **generation_rules:** Map component interactions; define patterns (sync, async, event-driven); specify contracts without implementation detail
> **enhancement_rules:** Clarify interaction contracts; remove hidden communication paths; strengthen predictability of component interaction
> **validation_rules:** All component interactions are documented; communication contracts are explicit; no hidden coupling; patterns are consistent
> **audit_rules:** Must exist; must not describe network protocols or serialization; must define contracts for all inter-component communication; must not duplicate Data Flow content

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** moderate
> **diagram_requirements:** sequence

```markdown
## Communication

> [metadata block]

### Communication Paths
[For each communication path: source, destination, pattern (sync/async/event), contract]

### Interaction Patterns
[Description of communication patterns used across the system]

### Communication Diagram
[Sequence or flow diagram showing inter-component communication]
```

**Required subsections:** Communication Paths, Communication Diagram
**Optional subsections:** Interaction Patterns, Contract Definitions
**Required diagrams:** sequence diagram of primary communication paths
**Required cross-references:** Component Model, Data Flow

*(To be written by the domain expert. This section defines how system components communicate and interact.)*

---

## Data Flow

> **semantic_type:** `data_flow`
> **scope:** How data moves through the system — the paths, transformations, and ownership boundaries for data
> **out_of_scope:** Database schemas, query implementations, serialization formats, data structure internals
> **contributes:** Makes data movement transparent; ensures data ownership and lifecycle are explicit across component boundaries
> **relationships:** Derived from Component Model and Communication; referenced by Security; constrains Feature Technical Design(10) data decisions
> **responsibilities:** Describe how data enters, moves through, and exits the system; identify data ownership boundaries and transformations
> **generation_rules:** Trace data paths through components; identify ownership boundaries; describe transformations at the architectural level
> **enhancement_rules:** Clarify data ownership; remove undocumented data paths; strengthen lifecycle documentation
> **validation_rules:** Data paths are complete; ownership is explicit; transformations are documented; no orphaned data flows
> **audit_rules:** Must exist; must not describe database schemas or query implementations; must identify data ownership; must cover all major data paths

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

```markdown
## Data Flow

> [metadata block]

### Data Paths
[For each major data path: entry point, transformations, ownership boundaries, exit point]

### Data Ownership
[Table or list mapping data entities to owning components]

### Data Flow Diagram
[Flowchart showing data movement through the system]
```

**Required subsections:** Data Paths, Data Flow Diagram
**Optional subsections:** Data Ownership, Data Transformations
**Required diagrams:** data flow diagram covering all major paths
**Required cross-references:** Component Model, Communication, Security

*(To be written by the domain expert. This section defines how data moves through the system and who owns it.)*

---

## Security

> **semantic_type:** `security_considerations`
> **scope:** Architectural security — the boundaries, controls, and threat model that govern system security posture
> **out_of_scope:** Implementation-level security (auth libraries, encryption APIs, specific CVE mitigations), coding practices
> **contributes:** Ensures security is a first-class architectural concern; provides the threat model that Engineering(07) implements
> **relationships:** Guided by Philosophy(02); informed by External Context threat models; constrains Engineering(07) and Feature Technical Design(10)
> **responsibilities:** Define security boundaries, threat model, access control model, and data protection requirements at the architectural level
> **generation_rules:** Identify trust boundaries; define the threat model; specify security controls architecturally; avoid implementation specifics
> **enhancement_rules:** Update threat model as the system evolves; strengthen security boundaries; remove implementation leakage
> **validation_rules:** Trust boundaries are defined; threat model is documented; security controls are architecturally specified; no implementation details
> **audit_rules:** Must exist; must define trust boundaries; must reference threat model; must not describe specific security libraries or implementations

### Template

> **minimum_content:** 2 subsections
> **length_guidance:** moderate
> **diagram_requirements:** component

```markdown
## Security

> [metadata block]

### Trust Boundaries
[Description of where trust changes — external/internal, component-to-component]

### Threat Model
[Key threats, attack vectors, and mitigations at the architectural level]

### Security Controls
[Architectural security measures — access control model, data protection requirements]
```

**Required subsections:** Trust Boundaries, Threat Model
**Optional subsections:** Security Controls, Access Control Model
**Required diagrams:** trust boundary diagram
**Required cross-references:** Component Model, Data Flow, Philosophy(02)

*(To be written by the domain expert. This section defines the architectural security posture, boundaries, and threat model.)*

---

## Rationale

> **semantic_type:** `rationale`
> **scope:** Why architectural decisions were made — the reasoning, trade-offs, and alternatives considered
> **out_of_scope:** Implementation trade-offs, code-level performance analysis, library comparison details
> **contributes:** Preserves the intent behind architectural choices; prevents regression through uninformed re-architecture
> **relationships:** Derived from Vision(01) and Philosophy(02); referenced by Architecture sections to justify structural choices
> **responsibilities:** Document the reasoning behind significant architectural decisions, including alternatives considered and rejected
> **generation_rules:** Record decisions at the point they are made; capture alternatives and rejection reasons; keep rationale tied to architectural concerns
> **enhancement_rules:** Update rationale when decisions change; remove outdated reasoning; keep rationale current with architectural state
> **validation_rules:** Key decisions have documented rationale; alternatives are recorded; reasoning is tied to architectural goals
> **audit_rules:** Must exist for significant decisions; must not describe implementation trade-offs; must reference architectural goals; must capture alternatives

### Template

> **minimum_content:** 1 entry per significant decision
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
## Rationale

> [metadata block]

### [Decision Name]
- **Context:** [what prompted this decision]
- **Decision:** [what was decided]
- **Alternatives Considered:** [what else was evaluated]
- **Rejection Reason:** [why alternatives were rejected]
- **Architectural Goal:** [which goal this serves]
```

**Required subsections:** one entry per significant architectural decision
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Vision(01), Philosophy(02)

*(To be written by the domain expert. This section defines the reasoning behind architectural decisions.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Architectural constraints — the non-functional requirements, platform limitations, and organizational rules that bound the architecture
> **out_of_scope:** Implementation constraints, coding standards, library version requirements, build system limitations
> **contributes:** Makes architectural boundaries explicit; prevents designs that violate fundamental system constraints
> **relationships:** Derived from External Context and Platform Pillars(01); constrains Component Model and all downstream architecture decisions
> **responsibilities:** Document non-functional requirements, platform constraints, and organizational rules that shape architectural decisions
> **generation_rules:** Identify hard constraints first; distinguish constraints from preferences; document the source and reason for each constraint
> **enhancement_rules:** Remove obsolete constraints; clarify constraint severity; ensure constraints remain aligned with External Context
> **validation_rules:** Constraints are explicit and sourced; hard constraints are distinguished from soft preferences; no implementation constraints included
> **audit_rules:** Must exist; must not describe implementation limitations; must distinguish hard from soft constraints; must reference their source

### Template

> **minimum_content:** 1 list of constraints
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
## Constraints

> [metadata block]

### Hard Constraints
[Constraints that cannot be violated — with source and reason for each]

### Soft Constraints
[Preferences and guidelines that should be followed unless justified]

### Platform Constraints
[Hardware, OS, or runtime constraints that shape architecture]
```

**Required subsections:** Hard Constraints
**Optional subsections:** Soft Constraints, Platform Constraints
**Required diagrams:** none
**Required cross-references:** External Context, Platform Pillars(01)

*(To be written by the domain expert. This section defines the non-functional requirements and constraints that bound the architecture.)*

---

## Required Sections

Every Architecture document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases | Content Requirements |
|---------|--------------|----------|---------|---------------------|
| System Overview | `system_overview` | ✓ | Overview, Architecture Overview | System purpose, primary capabilities, structural approach; entry point for new contributors |
| Component Model | `component_model` | ✓ | Components, Component Architecture | Component names, responsibilities, ownership boundaries, relationships |
| Communication | `communication_paths` | ✓ | Communication Paths, Component Communication | Communication paths, interaction patterns, contracts between components |
| Data Flow | `data_flow` | ✓ | Data Movement, Information Flow | Data entry/exit points, movement paths, ownership boundaries, transformations |
| Security | `security_considerations` | ✓ | Security Architecture, Security Model | Trust boundaries, threat model, access control model, data protection requirements |
| Purpose | `purpose` | | Overview, Summary | Root intent, why Architecture Documentation exists, scope boundaries |
| Rationale | `rationale` | | Decision Rationale, Architectural Decisions, Why | Decision reasoning, alternatives considered, trade-offs, rejection criteria |
| Constraints | `constraints` | | Limitations, Non-Functional Requirements | Non-functional requirements, platform limitations, organizational rules with source attribution |
| Traceability | `traceability` | | Traces To, Derived From | Tier model, derivation chain, downstream standards, non-contradiction rule |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Architecture Documentation aims to:

* Give the system a single authoritative structural description.
* Make component boundaries and responsibilities explicit.
* Let Feature Technical Design constrain itself against one source of truth.
* Keep structure stable while individual features change.

---

## Non-Goals

Architecture Documentation does not attempt to define:

* Product purpose
* Feature specifications
* Engineering decisions
* Technology selection
* Source code
* Algorithms
* Build configuration
* Library usage

These responsibilities belong to other documentation standards.

---

## Success Criteria

Architecture Documentation is successful when:

* Engineers understand how the system is organized.
* Responsibilities are unambiguous.
* New contributors can locate architectural information easily.
* Components can evolve independently.
* Implementation decisions remain consistent with architectural intent.
* AI systems can understand the system organization without reading source code.

---

## Responsibilities

Architecture Documentation is responsible for describing:

* System structure
* Components
* Responsibilities
* Ownership boundaries
* Communication
* Dependencies
* Data flow
* Runtime organization
* Deployment organization
* Architectural constraints
* System invariants

Architecture defines how the system is organized.

It does not define how it is implemented.

---

## Scope

Architecture Documentation may include:

* System Overview
* Component Model
* Module Boundaries
* Runtime Boundaries
* Communication Architecture
* Data Flow
* Security Architecture
* Deployment Architecture
* Persistence Architecture
* Plugin Architecture
* Integration Architecture
* Lifecycle Documentation
* Architectural Invariants

Projects should include only the architectural topics relevant to the repository.

Architecture documentation is intentionally modular.

---

## Out of Scope

Architecture Documentation must not describe:

* Product vision
* Feature requirements
* User experience
* UI design
* Source code
* Algorithms
* Class implementations
* Function implementations
* Programming syntax
* Library APIs
* Configuration files
* Build scripts

These belong to downstream documentation.

---

## Inputs

Architecture Documentation derives from:

* Vision
* Technology Decisions
* Platform Constraints
* Security Threat Model
* Engineering Principles

Architecture is independent of Feature Documentation and Feature Design.

Architecture should not derive from implementation.

---

## Outputs

Architecture provides direction for:

* Feature Technical Design
* Engineering Decisions
* Implementation
* Testing Standards
* Validation
* Documentation Audits

Implementation should conform to Architecture.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Architecture connects to the documentation hierarchy — the derivation chain from Vision to Implementation
> **out_of_scope:** Code-level traceability, test traceability, bug tracking, version history
> **contributes:** Makes Architecture's influence visible and verifiable across the documentation ecosystem
> **relationships:** Architecture(05) is Tier 2; derived from Vision(01); constrains Feature Technical Design(10); consumed by Engineering(07)
> **responsibilities:** Show the derivation path from Vision through Architecture to Implementation; assert that no downstream document contradicts Architecture
> **generation_rules:** Use the tier model diagram; list which documents Architecture feeds; state the non-contradiction rule
> **enhancement_rules:** Update the diagram when new standards are added; ensure derivation paths remain accurate
> **validation_rules:** Derivation paths are complete; no orphaned standards; non-contradiction rule is stated
> **audit_rules:** Must exist; must include tier diagram; must list downstream standards; must state non-contradiction constraint

### Template

> **minimum_content:** 1 diagram + derivation list
> **length_guidance:** concise
> **diagram_requirements:** flowchart

```markdown
## Traceability

> [metadata block]

### Derivation Chain
[Diagram showing Architecture's position in the documentation tier model]

### Downstream Impact
[List of standards that Architecture constrains or feeds into]

### Non-Contradiction Rule
[Statement that no downstream document may contradict Architecture]
```

**Required subsections:** Derivation Chain, Non-Contradiction Rule
**Optional subsections:** Downstream Impact
**Required diagrams:** tier model diagram
**Required cross-references:** Vision(01), Feature Technical Design(10), Engineering(07)

Architecture should remain traceable.

```text
Vision
    │
    ├──────────────────────────┐
    ↓                          ↓
Feature chain            Architecture Documentation
(Feature → Feature Design)    (technology decisions,
    │                          platform constraints,
    │                          engineering principles)
    └──────────────────────────┘
                ↓
      Feature Technical Design
      (Feature + Architecture + optional External Context)
                ↓
          Engineering
                ↓
          Implementation
```

Architecture provides the structural foundation that Feature Technical Design applies to specific features.

---

## Relationships

| Document | Relationship |
|---|---|
| Philosophy | Architecture is guided by Philosophy |
| Security | Architecture is guided by Security's threat model (once registered) |
| Engineering | Soft, non-mandatory alignment — most frameworks expect an architecture, none require one first |
| Feature Technical Design | Architecture constrains it |
| External Context | May be referenced when integration constraints affect structure |

---

## Required Characteristics

Architecture Documentation should be:

* Consistent across components
* Traceable to Philosophy and Security
* Technology-independent at the system level
* Boundary-respecting
* Stable
* Reviewable independent of any single feature

---

## Audit Rules

An audit should verify:

* Architecture is modular.
* Responsibilities are clearly separated.
* Ownership is explicit.
* Boundaries are documented.
* Communication paths are understandable.
* Documents do not duplicate one another.
* Architecture supports Feature Technical Design without depending on Feature Documentation.
* Architecture avoids implementation detail.
* Cross-repository references are used instead of duplication.

Architecture quality is evaluated across the complete documentation collection.

---

## Validation Rules

Architecture Documentation is considered valid when:

* Responsibilities are clearly defined.
* Every architectural concern has a documented owner.
* Boundaries are explicit.
* Communication paths are documented.
* Component responsibilities do not overlap.
* Architectural documents remain modular.
* Architecture remains traceable to Features.
* No implementation details dominate architectural descriptions.

Validation applies to the architecture collection rather than individual files.

---

## Generation Rules

When generating Architecture Documentation:

* Decompose by architectural responsibility.
* Prefer multiple focused documents over one large document.
* Describe responsibilities before technologies.
* Define boundaries explicitly.
* Reference external architectures rather than duplicating them.
* Maintain traceability to Features.

Architecture generation should optimize for maintainability rather than document count.

---

## Enhancement Rules

When enhancing Architecture Documentation:

* Improve separation of concerns.
* Split oversized documents.
* Remove duplicated responsibilities.
* Strengthen ownership definitions.
* Clarify communication.
* Improve traceability.
* Remove implementation leakage.
* Preserve architectural intent.

Architecture should become clearer through refinement.

---

## Summary

Architecture Documentation is the structural specification of the system.

It is a modular collection of focused documents that collectively describe system organization, responsibilities, ownership, boundaries, and communication.

The objective is not to document every implementation detail, but to provide a clear, maintainable, and traceable architectural model that guides engineering and implementation throughout the lifecycle of the project.

---

## Common Mistakes

Examples include:

* Mixing architecture with implementation.
* Large monolithic architecture documents.
* Undefined ownership.
* Hidden communication paths.
* Missing boundaries.
* Technology-driven architecture.
* Duplicated architectural responsibilities.
* Documenting source code instead of architecture.

These should be reported during audits.

---

## Documentation Folder

Architecture documents live under:

```text
docs/raw/architecture/
```

---

## Usage

Written by architects/senior engineers when a system's structure changes; read by anyone writing Feature Technical Design (which constrains itself to what Architecture permits). Use `samgraha compile --domain architecture` after adding a new architecture document, and `samgraha audit --domain architecture` to catch missing System Overview / Component Model / Security sections before review.

## Related

- [Feature Technical Standard](10-feature-technical-standards.md) — constrained by Architecture
- [Engineering Standard](07-engineering-standards.md) — guided by Architecture
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Architecture as a Documentation Collection

Architecture is a collection of related documents.

Example:

```text
architecture/

    system-overview.md

    component-model.md

    runtime-boundary.md

    communication.md

    persistence.md

    deployment.md
```

Each document should describe one architectural concern.

Responsibilities should not overlap.

---

## Single Responsibility

Every architecture document should have one primary responsibility.

Examples:

* Runtime Boundary
* Plugin Lifecycle
* Persistence
* Security
* Communication

Large architecture documents should be decomposed into smaller focused documents.

---

## Architectural Boundaries

Architecture should define:

* Responsibility ownership
* Component boundaries
* Communication boundaries
* Data ownership
* Runtime ownership
* Security boundaries
* Extension boundaries

Boundaries should be explicit.

Implicit architecture should be avoided.

---

## Architectural Principles

Architecture should promote:

* Separation of concerns
* High cohesion
* Low coupling
* Explicit ownership
* Stable interfaces
* Predictable communication
* Clear dependencies
* Replaceable components

Projects may define additional architectural principles.

---

## Technology Independence

Architecture should remain implementation independent whenever practical.

Architecture should describe:

* responsibilities

instead of

* frameworks

Architecture may reference technologies only when they are architecturally significant.

Example:

Acceptable

* Electron Main Process
* Browser Process
* Plugin Runtime

Not acceptable

* React Hooks
* Axios
* SQLite API
* Rust syntax

Technology selection belongs in Engineering Documentation.

---

## Cross-Repository Architecture

If a repository depends on another repository:

Architecture should define:

* ownership boundaries
* interaction contracts
* communication model

Architecture should not duplicate another repository's architecture.

Instead, reference the relevant documentation.

---

## Quality Requirements

Architecture Documentation should be:

* Modular
* Cohesive
* Traceable
* Technology appropriate
* Responsibility driven
* Consistent
* Maintainable
* Scalable
* Reviewable

Architecture should evolve through decomposition rather than document growth.

---
