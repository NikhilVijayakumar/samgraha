# Vision Standard

## Table of Contents
- [Purpose](#purpose)
- [Vision](#vision)
- [Problem](#problem)
- [Solution](#solution)
- [Target Audience](#target-audience)
- [Platform Pillars](#platform-pillars)
- [Philosophy](#philosophy)
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
- [Technology Independence](#technology-independence)
- [Product Philosophy](#product-philosophy)
- [Guiding Principles](#guiding-principles)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why the product exists — its reason for being, independent of what it does or how it works
> **out_of_scope:** Feature lists, implementation details, architecture, technology choices, user workflows
> **contributes:** Provides the root intent that all other sections and downstream standards derive from
> **relationships:** Vision(01) root of hierarchy; referenced by all downstream standards
> **responsibilities:** Establish the product's reason for existence in terms anyone can understand
> **generation_rules:** Start with the problem, then state the purpose; avoid technical language; write for a new contributor who has never seen the codebase
> **enhancement_rules:** Strengthen clarity without adding scope; remove ambiguity; ensure purpose survives feature and technology changes
> **validation_rules:** Purpose is clearly defined; no implementation details present; understandable without technical knowledge; stable over time
> **audit_rules:** Must exist; must not contain feature lists; must not reference specific technologies; must be technology-independent

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
[One sentence stating why the product exists and the problem it addresses]
[One sentence stating the intended value or outcome for users]
[One sentence reinforcing the core identity of the product]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** none

### Examples

**Correct:**
> DataSync exists to help teams move information between systems without manual intervention, eliminating hours of repetitive data entry each week. DataSync is the bridge that turns fragmented data into a single source of truth.

**Incorrect:**
> DataSync is a Python-based ETL pipeline using Apache Airflow that runs daily cron jobs to sync PostgreSQL databases via REST APIs.
> *Why wrong: Contains implementation details (technology stack, scheduling mechanism, protocol) that belong in downstream documentation, not in the Purpose section.*

### Writing Guidance

- **Tone:** inspirational
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** new contributor
- **Do:** Write from the user's world, not the engineer's; anchor the purpose in the problem space; keep the language stable enough to survive technology changes
- **Don't:** Name programming languages, frameworks, or infrastructure; describe what the product does or how it works; use jargon that requires domain expertise to understand

This document defines the standard for Vision documentation within the engineering documentation ecosystem.

A Vision document establishes the long-term purpose, direction, and identity of a product or repository.

It defines **why** the product exists.

It does not define implementation, architecture, or engineering decisions.

All downstream documentation ultimately derives from the Vision.

---

## Vision

> **semantic_type:** `vision_statement`
> **scope:** The long-term vision for the product — where it is going and what it aims to become
> **out_of_scope:** Current features, implementation status, technology roadmap, release plans
> **contributes:** Provides the aspirational target that guides all downstream documentation and decisions
> **relationships:** Derived from Purpose; inspires Goals and Non-Goals; referenced by Features(04) and Architecture(05)
> **responsibilities:** Articulate the desired future state of the product in inspiring, stable terms
> **generation_rules:** Write from the product perspective; describe the future state, not the current state; avoid technical language
> **enhancement_rules:** Strengthen the aspirational quality; remove implementation leakage; keep the vision stable
> **validation_rules:** Vision is aspirational and future-oriented; technology-independent; inspiring; stable over time
> **audit_rules:** Must exist; must not describe current features; must not reference specific technologies; must be technology-independent

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
[Aspirational statement describing the desired future state of the product]
[What the product will enable or become once fully realized]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Purpose

### Examples

**Correct:**
> CloudBridge will become the trusted backbone for cross-organization data exchange, where any team can connect to any data source within minutes and trust that the information is accurate and current.

**Incorrect:**
> CloudBridge will migrate from REST to GraphQL by Q3, reaching 10,000 API calls per second with sub-50ms latency on AWS.
> *Why wrong: Describes a technology roadmap with specific implementation targets (latency, throughput, cloud provider) rather than an aspirational future state.*

### Writing Guidance

- **Tone:** inspirational
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** product owner
- **Do:** Paint a vivid picture of the fully realized product state; write in the future tense with aspirational language; tie the vision back to the Purpose section's "why"
- **Don't:** Mention specific technologies, release timelines, or implementation milestones; describe current product state or features; use metrics or benchmarks that belong in Success Criteria

*(To be written by the product owner. This section defines where the product is going.)*

---

## Problem

> **semantic_type:** `problem`
> **scope:** The problem the product solves — the real-world pain or gap that justifies the product's existence
> **out_of_scope:** Solution descriptions, feature lists, implementation approaches, technical constraints
> **contributes:** Justifies the Vision and Purpose; gives downstream standards the "why" behind every decision
> **relationships:** Drives Purpose; referenced by Features(04) as the root problem each feature addresses
> **responsibilities:** Describe the problem in terms the reader can feel — real pain, real gap, real cost of the status quo
> **generation_rules:** Describe the problem before the solution; use concrete examples; quantify impact where possible
> **enhancement_rules:** Sharpen the problem statement; remove solution leakage; ensure the problem is still current
> **validation_rules:** Problem is clearly articulated; no solution details present; understandable without technical knowledge
> **audit_rules:** Must exist; must not describe solutions; must not reference specific technologies; must be understandable by non-engineers

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
[Description of the real-world pain or gap the product addresses]
[Concrete example illustrating the problem in context]
[Quantified impact where possible — cost, time, frequency]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Purpose

### Examples

**Correct:**
> Teams that need to consolidate data from multiple sources spend hours each week on manual copying and pasting between spreadsheets. A mid-size operations team reports losing 12 hours per week to data reconciliation tasks, leading to delayed reports and costly errors.

**Incorrect:**
> Teams struggle with data silos. DataSync solves this by using scheduled Python scripts and a Redis cache layer to automatically merge CSV files.
> *Why wrong: Mixes solution details (technology, mechanism) into the Problem section. The Problem section should describe pain, not how the product addresses it.*

### Writing Guidance

- **Tone:** concrete
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** product owner
- **Do:** Use specific, quantifiable examples of the pain; describe the problem from the user's perspective; include the cost of inaction
- **Don't:** Mention the product name or any solution approach; describe the problem in abstract or theoretical terms; include technical error messages or stack traces

*(To be written by the product owner. This section defines what problem the product exists to solve.)*

---

## Solution

> **semantic_type:** `solution`
> **scope:** The solution approach at the highest level — what the product does to solve the stated problem
> **out_of_scope:** Feature specifications, architecture details, implementation plans, technology choices
> **contributes:** Bridges the Problem to the Vision; gives downstream standards the high-level approach
> **relationships:** Derived from Problem and Vision; referenced by Features(04) as the approach each feature implements
> **responsibilities:** Describe the solution approach in terms of what the product does, not how it works
> **generation_rules:** Start from the problem; describe the approach at the product level; avoid technical detail
> **enhancement_rules:** Keep the solution description at the right abstraction level; remove implementation leakage
> **validation_rules:** Solution addresses the stated problem; no implementation details present; technology-independent
> **audit_rules:** Must exist; must not describe architecture; must not reference specific technologies; must be technology-independent

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
[High-level description of what the product does to solve the stated problem]
[How the product's approach delivers value to the target audience]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Problem

### Examples

**Correct:**
> DataSync automates the collection, transformation, and delivery of data across connected systems. It provides a single place to define data flows and ensures that information stays consistent wherever it is used.

**Incorrect:**
> DataSync uses Python with Celery workers and RabbitMQ to queue data jobs, storing results in a PostgreSQL database with a React dashboard for monitoring.
> *Why wrong: Describes architecture and implementation technology instead of the product-level approach to solving the problem.*

### Writing Guidance

- **Tone:** structural
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** product owner
- **Do:** Describe the approach at the product level using action verbs; connect the solution directly back to the Problem section; keep descriptions at the "what it does" level
- **Don't:** Name libraries, frameworks, or databases; describe data flows, APIs, or internal system boundaries; discuss trade-offs between technology options

*(To be written by the product owner. This section defines the high-level solution approach.)*

---

## Target Audience

> **semantic_type:** `target_audience`
> **scope:** Who the product is for — the users, consumers, or beneficiaries of the product
> **out_of_scope:** User stories, personas with implementation detail, behavioral analytics, feature preferences
> **contributes:** Gives every downstream standard the "who" — decisions should serve this audience
> **relationships:** Derived from Purpose; referenced by Features(04) and Feature Design(09) for user-centric decisions
> **responsibilities:** Define the intended users or consumers in terms of their needs, not their technical profile
> **generation_rules:** Describe the audience by their goals and needs; avoid jargon; include who benefits and who decides
> **enhancement_rules:** Refine audience definition as understanding deepens; add new audience segments if discovered
> **validation_rules:** Audience is clearly defined; described by needs, not technical profiles; stable over time
> **audit_rules:** Must exist; must not describe implementation details; must be understandable without code knowledge

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
[Description of the intended users or consumers by their goals and needs]
[Who benefits from the product and who makes adoption decisions]
[What the audience expects or requires from the product]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Purpose

### Examples

**Correct:**
> CloudBridge serves operations teams who need to consolidate data from multiple sources into a single, reliable view. These teams prioritize accuracy and speed over technical flexibility, and their managers make adoption decisions based on time savings and error reduction.

**Incorrect:**
> CloudBridge is used by Python developers with 5+ years of experience who write pandas scripts and prefer CLI tools with YAML configuration.
> *Why wrong: Describes the audience by technical profile and specific skill requirements instead of goals and needs. The audience section should be understandable without code knowledge.*

### Writing Guidance

- **Tone:** conversational
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** product owner
- **Do:** Describe audiences by their goals, pain points, and decision-making criteria; distinguish between end users and decision-makers; include what each audience expects from the product
- **Don't:** List programming skills, tool proficiencies, or job titles as the defining trait; write user stories or persona cards; conflate technical users with the primary audience

*(To be written by the product owner. This section defines who the product serves.)*

---

## Platform Pillars

> **semantic_type:** `pillars`
> **scope:** The foundational pillars or foundations the product stands on — core capabilities or principles that organize the product
> **out_of_scope:** Feature lists, architecture components, technology stack, implementation details
> **contributes:** Organizes the product's capabilities into stable, named pillars that guide feature and architecture decisions
> **relationships:** Derived from Vision; referenced by Architecture(05) for structural organization; guides Design(06)
> **responsibilities:** Define the core pillars that organize the product's capabilities and guide downstream decisions
> **generation_rules:** Extract from the Vision and Purpose; name 3-5 stable pillars; keep them at the right abstraction level
> **enhancement_rules:** Add pillars when new capability areas emerge; merge overlapping pillars; keep the count manageable
> **validation_rules:** Pillars are stable; cover the product's full scope; are at the right abstraction level; are memorable
> **audit_rules:** Must exist if the product has multiple capability areas; must not list features; must be technology-independent

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
## [Pillar Name 1]

[One-sentence description of this pillar and its role in the product]

## [Pillar Name 2]

[One-sentence description of this pillar and its role in the product]

## [Pillar Name 3]

[One-sentence description of this pillar and its role in the product]
```

**Required subsections:** 3-5 named pillars
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Vision

### Examples

**Correct:**
> **Reliable Connections** — Every connection to an external system is resilient, recoverable, and transparent in its status.
> **Data Integrity** — Information delivered through the product is always accurate and traceable to its source.
> **Simple Configuration** — Setting up a new data flow requires no coding and minimal manual steps.

**Incorrect:**
> **Microservices** — The product uses a microservices architecture for scalability.
> **Docker Containers** — All components run in Docker for consistent deployment.
> **CI/CD Pipeline** — Continuous integration ensures code quality.
> *Why wrong: Describes technology choices and implementation architecture instead of foundational capability pillars that organize the product.*

### Writing Guidance

- **Tone:** structural
- **Voice:** third person
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Name each pillar with a memorable, two-word phrase; write one sentence per pillar that explains its role in the product; ensure pillars cover the full product scope without overlap
- **Don't:** Name specific technologies or components; use abstract nouns without a clear product connection; list more than five pillars

*(To be written by the product owner. This section defines the foundational pillars of the product.)*

---

## Philosophy

> **semantic_type:** `philosophy`
> **scope:** The product's philosophy — the values and principles that guide how the product is built and used
> **out_of_scope:** Implementation rules, coding standards, architectural patterns, framework-specific guidelines
> **contributes:** Shapes the judgment that produces features, architecture, and engineering decisions
> **relationships:** Links to Philosophy(02); referenced by Design(06) and Engineering(07) for decision-making
> **responsibilities:** Communicate the philosophy that guides product decisions in memorable, stable terms
> **generation_rules:** Extract from the product's values; express as principles, not rules; use concrete examples where helpful
> **enhancement_rules:** Strengthen the philosophical clarity; remove implementation leakage; keep values stable
> **validation_rules:** Philosophy is technology-independent; memorable; actionable when a decision is ambiguous; stable
> **audit_rules:** Must exist; must not reference specific technologies; must be evaluable against real decisions; must be stable

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
[Statement of the product's core philosophy — the values that guide decisions]

## [Philosophy Value 1]

[One-sentence description of this value and why it matters]

## [Philosophy Value 2]

[One-sentence description of this value and why it matters]

## [Philosophy Value 3]

[One-sentence description of this value and why it matters]
```

**Required subsections:** 3-5 philosophy values
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** none

### Examples

**Correct:**
> **Clarity Over Cleverness** — Every feature should be immediately understandable to the person using it, even if that means a less elegant implementation.
> **Trust by Default** — Users should never have to wonder whether their data is correct; accuracy is assumed, not requested.

**Incorrect:**
> **Use FastAPI** — The product favors high-performance Python web frameworks.
> **PostgreSQL First** — All persistent data must use PostgreSQL for consistency.
> *Why wrong: States technology preferences rather than guiding values. Philosophy should influence decisions at any abstraction level, not prescribe specific tools.*

### Writing Guidance

- **Tone:** inspirational
- **Voice:** third person
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Express each value as a memorable, memorable value with a one-sentence rationale; tie each value back to the product's purpose; keep the language abstract enough to survive technology changes
- **Don't:** Name frameworks, databases, or deployment targets; write rules that require specific tools; list more than five values

*(To be written by the product owner. This section defines the philosophy guiding the product.)*

---

## Required Sections

Every Vision document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases | Content Requirements |
|---------|--------------|----------|---------|---------------------|
| Purpose | `purpose` | ✓ | Overview, Summary | Single paragraph stating why the product exists; no implementation details |
| Vision | `vision_statement` | ✓ | Long-Term Vision, The Vision | 1-2 paragraphs describing the aspirational future state of the product |
| Problem | `problem` | ✓ | Problem Statement, The Problem | 1-3 paragraphs with concrete examples and quantified impact where possible |
| Solution | `solution` | ✓ | The Solution, Our Solution | 1-2 paragraphs describing the product-level approach to solving the problem |
| Target Audience | `target_audience` | ✓ | Audience, Who Is This For | 1-2 paragraphs defining users by goals and needs, not technical profiles |
| Platform Pillars | `pillars` | | Pillars, Foundations, Core Pillars | 3-5 named pillars, each with a one-sentence description |
| Philosophy | `philosophy` | | Product Philosophy, Design Philosophy | 3-5 principles expressed as memorable values with brief rationale |
| Guiding Principles | `guiding_principles` | | Principles, Core Principles | 3-5 enduring principles with rationale; stable across feature changes |
| Success Criteria | `success_criteria` | | Acceptance Criteria, Definition of Done | 3-6 observable outcomes tied to the Vision; measurable or evaluable |
| Traceability | `traceability` | | Traces To, Derived From | Tier diagram, list of downstream standards, non-contradiction rule statement |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

A Vision document aims to:

* Give every downstream document a single, stable source of "why" to trace back to.
* Let a new contributor understand product intent in minutes, without reading code.
* Let engineers evaluate a proposed feature or architecture change against a stated purpose.
* Outlive individual features, technology choices, and implementation cycles.

---

## Non-Goals

The Vision does not attempt to define:

* Product requirements
* Feature specifications
* User stories
* Technical architecture
* Technology stack
* Implementation standards
* Source code organization

These responsibilities belong to other documentation standards.

---

## Success Criteria

> **semantic_type:** `success_criteria`
> **scope:** Observable outcomes that confirm the Vision is being fulfilled — measurable evidence of alignment
> **out_of_scope:** Feature acceptance criteria, test cases, implementation benchmarks, code metrics
> **contributes:** Gives downstream standards a target to aim for; validates that Vision drives real decisions
> **relationships:** References Vision(01) purpose; evaluated by Features(04) and Architecture(05)
> **responsibilities:** Define what "success" looks like for the product at the Vision level — not at the feature level
> **generation_rules:** Derive from the Vision statement and problem; express as observable outcomes, not implementation tasks
> **enhancement_rules:** Add criteria when the Vision clarifies; remove criteria that belong to features or architecture
> **validation_rules:** Criteria are observable, not implementation-specific; tied to the Vision purpose; measurable or evaluable
> **audit_rules:** Must exist; must not contain test cases; must not reference specific technologies; must be evaluable without code

### Template

> **minimum_content:** 3 items
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
* [Observable outcome tied to the Vision — what success looks like]
* [Observable outcome tied to the Vision — what success looks like]
* [Observable outcome tied to the Vision — what success looks like]
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Vision, Purpose

### Examples

**Correct:**
> * Teams report spending less than 2 hours per week on data reconciliation tasks.
> * At least 80% of new data flows are set up without engineering support.
> * Data delivered through the product is accurate 99.9% of the time as verified by audits.

**Incorrect:**
> * The API response time is under 200ms.
> * The test suite achieves 95% code coverage.
> * Deployment frequency increases to daily releases.
> *Why wrong: Describes implementation-level metrics (performance, test coverage, release cadence) rather than observable outcomes tied to the Vision and product purpose.*

### Writing Guidance

- **Tone:** prescriptive
- **Voice:** third person
- **Structure:** bullet lists
- **Audience:** product owner
- **Do:** Write each criterion as a measurable or evaluable outcome; tie every criterion directly to the Vision statement; use concrete units of measure where possible
- **Don't:** Describe technical benchmarks like latency or throughput; include test coverage or deployment frequency; list more than six criteria

A Vision document is successful when:

* Engineers understand the long-term purpose of the project.
* Product decisions can be evaluated against the Vision.
* Features naturally derive from the Vision.
* Architecture supports the Vision without redefining it.
* Engineering decisions remain aligned with product goals.
* AI systems can infer product intent without reading implementation documents.

---

## Responsibilities

A Vision document is responsible for defining:

* Product purpose
* Long-term direction
* Core objectives
* Intended value
* Target users or consumers
* Guiding principles
* Product identity

The Vision provides the strategic foundation for every subsequent engineering decision.

---

## Scope

A Vision document should describe:

* Why the product exists
* What problem it intends to solve
* The long-term purpose of the project
* The intended value delivered
* The desired future state
* Product philosophy
* Product principles
* Success vision

The Vision should remain stable throughout the product lifecycle.

---

## Out of Scope

A Vision document must not describe:

* Features
* User workflows
* UI layouts
* Architecture
* Components
* Technology choices
* Programming languages
* Frameworks
* Databases
* APIs
* Algorithms
* Build systems
* Source code
* Library selection
* Implementation details

These belong in downstream documentation.

---

## Inputs

A Vision document may consider:

* Business objectives
* Product goals
* Market needs
* User problems
* Organizational direction

The Vision should not depend on implementation documentation.

---

## Outputs

A Vision document provides direction for:

* Feature documentation
* Feature Design
* Architecture
* Engineering Decisions
* Product Roadmaps
* Documentation audits

Every Feature should be traceable to the Vision.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How the Vision connects to the documentation hierarchy — the derivation chain from purpose to code
> **out_of_scope:** Implementation traceability, test traceability, bug tracking, version history
> **contributes:** Makes the Vision's influence visible and verifiable across the entire documentation ecosystem
> **relationships:** Vision(01) is Tier 0; feeds Features(04), Philosophy(02), Architecture(05); consumed by Feature Technical Design(10)
> **responsibilities:** Show the derivation path from Vision to every downstream standard; assert that no downstream document contradicts the Vision
> **generation_rules:** Use the tier model diagram; list which standards derive from Vision; state the non-contradiction rule
> **enhancement_rules:** Update the diagram when new standards are added; ensure derivation paths remain accurate
> **validation_rules:** Derivation paths are complete; no orphaned standards; non-contradiction rule is stated
> **audit_rules:** Must exist; must include tier diagram; must list downstream standards; must state non-contradiction constraint

### Template

> **minimum_content:** 1 diagram + 1 rule statement
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

```markdown
Tier 0: Vision (Purpose)
    ├──→ Tier 1: [Downstream Standard 1]
    ├──→ Tier 1: [Downstream Standard 2]
    └──→ Tier 2: [Downstream Standard 3]
```

**Required subsections:** tier diagram
**Optional subsections:** derivation list
**Required diagrams:** tier hierarchy flowchart
**Required cross-references:** all downstream standards listed

### Examples

**Correct:**
> Tier 0: Vision (Purpose, Problem, Solution)
>     ├──→ Tier 1: Philosophy (Values, Principles)
>     ├──→ Tier 1: Features (Feature List, Feature Details)
>     └──→ Tier 2: Architecture (System Design, Technology Choices)
>
> **Non-contradiction rule:** No downstream document may state a goal, constraint, or priority that contradicts the Vision. When conflicts arise, the Vision takes precedence.

**Incorrect:**
> Vision traces to the README and the CI/CD pipeline configuration.
> *Why wrong: References an implementation artifact (CI/CD pipeline) instead of the documentation hierarchy. Traceability should connect to other documentation standards, not to code or infrastructure.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** diagrams
- **Audience:** architect
- **Do:** Include a tier diagram showing the derivation hierarchy; list every downstream standard that derives from Vision; state the non-contradiction rule explicitly
- **Don't:** Reference source code files, CI/CD pipelines, or infrastructure artifacts; omit standards from the diagram; use prose where a diagram would be clearer

No downstream document should contradict the Vision.

---

## Relationships

| Document         | Relationship                                    |
| ---------------- | ----------------------------------------------- |
| Features         | Derived from Vision                             |
| Feature Design   | Supports Vision through Features                |
| Architecture     | Realizes Vision through system organization     |
| Engineering      | Supports Vision through implementation standards |
| External Context | Independent                                     |
| README           | Summarizes the repository using the Vision      |

---

## Required Characteristics

A Vision document should be:

* Stable
* Technology independent
* Product focused
* Long-term oriented
* Implementation independent
* Easy to understand
* Concise
* Inspirational
* Actionable

---

## Audit Rules

An audit should verify:

* The Vision explains why the product exists.
* The document is technology independent.
* No implementation details appear.
* Product philosophy is present.
* Guiding principles are documented.
* Downstream documentation remains consistent with the Vision.
* The Vision remains stable and future-oriented.

Any implementation detail should be reported as a standards violation.

---

## Validation Rules

A Vision document is considered valid if:

* The purpose is clearly defined.
* The long-term objective is explicit.
* Product philosophy is documented.
* Guiding principles are identified.
* No implementation details are present.
* No architectural decisions are described.
* No feature specifications are included.
* The document can guide future feature development.

---

## Generation Rules

When generating a Vision document:

* Focus on purpose before capability.
* Explain the problem before the solution.
* Describe long-term value.
* Avoid implementation language.
* Write from the product perspective.
* Prefer stable concepts over temporary goals.
* Keep technology decisions separate.

---

## Enhancement Rules

When enhancing a Vision document:

* Improve clarity.
* Strengthen long-term direction.
* Remove implementation leakage.
* Remove architectural discussion.
* Eliminate duplicated feature descriptions.
* Improve consistency with product philosophy.
* Preserve existing intent.

Enhancements should refine—not redefine—the Vision.

---

## Summary

The Vision is the highest-level engineering artifact within the documentation ecosystem.

Its responsibility is to communicate **why** the product exists and the long-term direction it should follow.

Every downstream document should refine the Vision without redefining it, ensuring that engineering decisions remain aligned with enduring product intent rather than temporary implementation choices.

---

## Common Mistakes

Examples of incorrect Vision content include:

* Listing application features.
* Explaining UI behavior.
* Discussing databases.
* Selecting programming languages.
* Describing frameworks.
* Explaining APIs.
* Introducing architecture diagrams.
* Including implementation plans.

These belong in downstream documentation.

---

## Documentation Folder

Vision documents live under:

```text
docs/raw/vision/
```

---

## Usage

Vision is written once per product and revised rarely — product owners author it; everyone else reads it before writing Features, since every Feature must trace back to the Vision. Use `samgraha compile --domain vision` to validate structure, and `samgraha search --domain vision` (or the MCP `search` tool) to pull Vision context into an AI-assisted feature-writing session.

## Related

- [Feature Standard](04-feature-standards.md) — every Feature derives from Vision
- [Philosophy Standard](02-philosophy-standards.md) — inspires Vision's guiding principles
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## Technology Independence

The Vision should remain independent of implementation technologies.

The following should generally never appear:

* Programming languages
* Frameworks
* Libraries
* Databases
* Infrastructure
* Build systems
* Cloud providers

Technology decisions evolve.

Vision should not.

---

## Product Philosophy

A Vision should communicate the philosophy that guides product decisions.

Examples include:

* Documentation First
* Privacy First
* Local First
* Offline First
* Accessibility First
* Developer Experience
* Simplicity
* Reliability

These describe values rather than implementation.

---

## Guiding Principles

> **semantic_type:** `guiding_principles`
> **scope:** Enduring principles that influence every future decision — values that survive feature and technology changes
> **out_of_scope:** Implementation rules, coding standards, architectural patterns, framework-specific guidelines
> **contributes:** Shapes the judgment that produces features, architecture, and engineering decisions
> **relationships:** Inspired by Philosophy(02); referenced by Design(06) and Engineering(07)
> **responsibilities:** Define principles that remain true even as specific features and technologies change
> **generation_rules:** Extract from product philosophy; express as stable values, not temporary goals; use memorable phrasing
> **enhancement_rules:** Add principles when the product evolves; remove principles that have become obsolete; preserve core intent
> **validation_rules:** Principles are technology-independent; stable across features; memorable; actionable when a decision is ambiguous
> **audit_rules:** Must exist; must not reference specific technologies; must be evaluable against real decisions; must be stable

### Template

> **minimum_content:** 1 paragraph + 3 principles
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
[Introductory paragraph explaining that these principles guide all downstream decisions]

## [Principle 1]

[One-sentence statement of the principle and its rationale]

## [Principle 2]

[One-sentence statement of the principle and its rationale]

## [Principle 3]

[One-sentence statement of the principle and its rationale]
```

**Required subsections:** 3-5 principles
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Philosophy

### Examples

**Correct:**
> **Fail Safely** — When a connection to an external system fails, the product preserves existing data and retries automatically rather than losing work.
> **Show, Don't Assume** — Every automated action should be visible to the user so they can verify correctness.

**Incorrect:**
> **Use Kubernetes** — The product should always be deployed on Kubernetes for orchestration.
> **TypeScript Everywhere** — All frontend and backend code must use TypeScript.
> *Why wrong: States technology mandates rather than enduring principles. Principles should survive technology changes and guide decisions regardless of implementation stack.*

### Writing Guidance

- **Tone:** inspirational
- **Voice:** third person
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** Phrase each principle as a decision rule that applies across technologies; include a one-sentence rationale for each; ensure principles are testable against real decisions
- **Don't:** Name programming languages, frameworks, or cloud providers; write principles that are only true for one implementation; list more than five principles

Vision should define enduring principles that influence future decisions.

Examples:

* Human-centered design
* AI-assisted engineering
* Open standards
* Predictable behavior
* Long-term maintainability

Principles should remain stable even as features evolve.

---

## Quality Requirements

A Vision document should:

* Clearly explain why the product exists.
* Communicate long-term direction.
* Inspire engineering decisions.
* Remain understandable without technical knowledge.
* Avoid implementation discussion.
* Remain stable over time.
* Provide sufficient guidance for feature definition.

---
