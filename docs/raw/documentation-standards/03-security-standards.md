# Security Standard

## Table of Contents
- [Purpose](#purpose)
- [Threat Model](#threat-model)
- [Data Classification](#data-classification)
- [Security Principles](#security-principles)
- [Compliance Requirements](#compliance-requirements)
- [Incident Response](#incident-response)
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
- [Revision History](#revision-history)
- [Relationship to Per-Domain Security Sections](#relationship-to-per-domain-security-sections)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Security exists — its role as the project-wide authority on what is being defended and why
> **out_of_scope:** Per-domain enforcement, specific controls, implementation details, technology choices
> **contributes:** Establishes Security's authority so every other domain derives its posture from this document instead of re-deriving independently
> **relationships:** Derives from Vision(01) and Philosophy(02); feeds Architecture(05), Engineering(07), and Feature Technical(10)
> **responsibilities:** Define Security's role as the single source of threat model, data classification, and security principles
> **generation_rules:** State what Security defines, what it does not define, and where enforcement lives; mirror the Vision/Philosophy pattern
> **enhancement_rules:** Keep the scope boundary between Security and per-domain sections sharp; remove any control-level language
> **validation_rules:** Purpose is technology-independent; scope is limited to project-wide posture; enforcement boundary is explicit
> **audit_rules:** Must exist; must not name specific controls or libraries; must define the boundary between project-wide and per-domain

### Template

> **minimum_content:** 1–2 paragraphs
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
<!-- One paragraph: what Security defines and its role in the documentation hierarchy -->
<!-- One paragraph: what Security does not define, and where enforcement lives -->
```

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Vision(01), Philosophy(02)

---

This document defines the standard for Security documentation within the engineering documentation ecosystem.

A Security document establishes the project-wide threat model, data classification scheme, security principles, and compliance posture that every other domain must operate within.

It defines **what the project is defending against and why**, once, at the project level — the same way Vision defines why the product exists and Philosophy defines the values guiding decisions.

It does not define how any single component enforces that posture. That belongs to each domain's own Security Considerations or Security Standards section.

---

## Threat Model

> **semantic_type:** `threat_model`
> **scope:** The project-wide threat model — adversaries, threats, attack surfaces, and failure modes the project defends against
> **out_of_scope:** Per-component trust boundaries, per-feature attack surface enumeration, specific mitigation tooling
> **contributes:** Gives every downstream domain the authoritative list of threats to address in their own Security sections
> **relationships:** Drives Data Classification and Security Principles; consumed by Architecture(05), Engineering(07), and Feature Technical(10) Security sections
> **responsibilities:** Enumerate the threat landscape using a named, structured methodology (STRIDE, PASTA, or OWASP Threat Modeling)
> **generation_rules:** Use a recognized threat-modeling methodology; enumerate adversaries, threats, and attack surfaces; link each threat to at least one downstream mitigation expectation
> **enhancement_rules:** Update when the attack surface changes; add new adversary profiles as the product evolves; preserve the methodology structure
> **validation_rules:** Threat model uses a named methodology; every threat has a severity or priority; every threat maps to at least one downstream expectation
> **audit_rules:** Must exist; must use a named methodology; must not list specific tooling or library mitigations; must not duplicate per-component trust boundaries

### Template

> **minimum_content:** 3–5 subsections
> **length_guidance:** extensive
> **diagram_requirements:** flowchart

```markdown
<!-- Methodology: state which threat-modeling methodology is used (STRIDE, PASTA, or OWASP) -->
<!-- Adversary Profiles: enumerate likely adversaries with motivation and capability -->
<!-- Threat Enumeration: list threats with category, severity, and affected assets -->
<!-- Attack Surfaces: identify external and internal surfaces -->
<!-- Downstream Expectations: map each threat to at least one domain that must address it -->
```

**Required subsections:** methodology, adversary profiles, threat enumeration, attack surfaces, downstream expectations
**Optional subsections:** trust assumptions, risk scoring matrix
**Required diagrams:** threat landscape overview (flowchart or table)
**Required cross-references:** Data Classification, Security Principles

---

*(To be written by the domain expert. This section defines the project-wide threat model.)*

---

## Data Classification

> **semantic_type:** `data_classification`
> **scope:** The project's data classification scheme — sensitivity levels, data types, and handling expectations for each class
> **out_of_scope:** Storage implementation details, encryption library selection, database schema, per-component data flow diagrams
> **contributes:** Gives every downstream domain a shared vocabulary for data sensitivity so controls are applied proportionally
> **relationships:** Derived from Threat Model; consumed by Architecture(05), Engineering(07), and Feature Technical(10) Security sections
> **responsibilities:** Define sensitivity levels with at least one example per level; specify handling expectations that apply across the project
> **generation_rules:** Classify data by sensitivity, not by storage location; define at least two distinct levels; provide concrete examples for each level
> **enhancement_rules:** Add new data types as they are introduced; refine sensitivity definitions when new compliance obligations apply; preserve the classification structure
> **validation_rules:** At least one sensitivity level is defined with an example; handling expectations are stated per level; classification is technology-independent
> **audit_rules:** Must exist; must define at least one sensitivity level; must not reference specific storage technologies; must not omit handling expectations

### Template

> **minimum_content:** 2–3 subsections
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
<!-- Sensitivity Levels: define each level (e.g. Public, Internal, Confidential, Restricted) -->
<!--   For each level: description, one concrete example, handling expectations -->
<!-- Handling Expectations: cross-cutting rules (access control, encryption at rest/transit, retention) -->
<!-- Application: how downstream domains map their data to these levels -->
```

**Required subsections:** sensitivity levels, handling expectations
**Optional subsections:** data inventory examples, classification decision tree
**Required diagrams:** none
**Required cross-references:** Threat Model, Security Principles

---

*(To be written by the domain expert. This section defines the data classification scheme.)*

---

## Security Principles

> **semantic_type:** `security_principles`
> **scope:** Project-wide security principles — the enduring values that constrain every downstream design and implementation decision
> **out_of_scope:** Specific control implementations, library choices, per-domain enforcement patterns, coding standards
> **contributes:** Provides the judgment framework that shapes Architecture, Engineering, and Feature Technical Security decisions
> **relationships:** Derived from Philosophy(02); consumed by Architecture(05), Engineering(07), and Feature Technical(10) Security sections
> **responsibilities:** State principles specific enough to guide a real design decision, not generic platitudes like "be secure"
> **generation_rules:** Specialize Philosophy's guiding principles for security; use memorable, stable phrasing; ensure each principle can constrain a concrete decision
> **enhancement_rules:** Add principles when the threat landscape demands new constraints; remove principles that have become obsolete; preserve core intent
> **validation_rules:** Principles are specific enough to evaluate a design against; no principle is generic enough to be meaningless; principles are technology-independent
> **audit_rules:** Must exist; must contain at least one principle; principles must not be generic ("be secure"); must not reference specific technologies

### Template

> **minimum_content:** 1–2 subsections
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
<!-- State each security principle with: -->
<!--   - Name: memorable, stable phrase -->
<!--   - Rationale: why this principle constrains design decisions -->
<!--   - Example decision it would constrain (optional but recommended) -->
<!-- Link principles back to Philosophy(02) guiding principles they specialize -->
```

**Required subsections:** none
**Optional subsections:** principle-to-decision examples, derivation from Philosophy
**Required diagrams:** none
**Required cross-references:** Philosophy(02), Threat Model

---

*(To be written by the domain expert. This section defines the project-wide security principles.)*

---

## Compliance Requirements

> **semantic_type:** `compliance`
> **scope:** Regulatory, contractual, or self-imposed compliance obligations the project must satisfy
> **out_of_scope:** Per-domain control implementation, audit tooling configuration, certification processes
> **contributes:** Makes compliance obligations traceable to specific downstream controls; prevents obligations from being forgotten
> **relationships:** Derived from Threat Model and Vision; consumed by Architecture(05), Engineering(07), and Feature Technical(10) Security sections
> **responsibilities:** List compliance regimes by name and scope; state the obligation each imposes; trace each to downstream control expectations
> **generation_rules:** Reference compliance requirements by name (GDPR, HIPAA, PCI-DSS, SOC 2); state scope and applicability; avoid prescribing specific controls
> **enhancement_rules:** Add new requirements when regulations change; update scope when the product enters new markets; preserve the obligation-to-control trace chain
> **validation_rules:** Each requirement is named; scope is defined; each maps to at least one downstream control expectation
> **audit_rules:** Must exist if the project has compliance obligations; must name each regime; must not prescribe specific implementation controls

### Template

> **minimum_content:** 2–3 subsections
> **length_guidance:** moderate
> **diagram_requirements:** none

```markdown
<!-- For each compliance regime: -->
<!--   - Name: regulatory or contractual body (e.g. GDPR, HIPAA, PCI-DSS, SOC 2) -->
<!--   - Scope: what parts of the project it applies to -->
<!--   - Key obligations: what the project must do or avoid -->
<!--   - Downstream expectations: which domain(s) must implement controls for this obligation -->
<!-- Traceability matrix: obligation → downstream control expectation -->
```

**Required subsections:** per-regime breakdown, obligation-to-control traceability
**Optional subsections:** compliance timeline, audit cadence
**Required diagrams:** none
**Required cross-references:** Threat Model, Vision(01)

---

*(To be written by the domain expert. This section defines compliance obligations.)*

---

## Incident Response

> **semantic_type:** `incident_response`
> **scope:** Project-level incident response expectations — how the project prepares for, detects, and responds to security incidents
> **out_of_scope:** Per-domain runbooks, specific SIEM tooling, on-call rotation details, post-mortem templates
> **contributes:** Establishes the incident response posture that downstream domains operationalize
> **relationships:** Derived from Threat Model; consumed by Engineering(07) and Feature Technical(10) for operational implementation
> **responsibilities:** Define detection expectations, response escalation, communication requirements, and recovery objectives at the project level
> **generation_rules:** State detection and response expectations as outcomes, not tooling; define escalation paths; specify communication and recovery objectives
> **enhancement_rules:** Update when the threat landscape changes; refine escalation paths as the team grows; preserve the outcome-level framing
> **validation_rules:** Detection and response expectations are stated; escalation paths are defined; no specific SIEM or tooling is mandated
> **audit_rules:** Must exist if the project handles sensitive data; must not mandate specific tooling; must define escalation and communication expectations

### Template

> **minimum_content:** 2–3 subsections
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

```markdown
<!-- Detection Expectations: what signals indicate a security incident -->
<!-- Response Escalation: who is notified, in what order, within what timeframe -->
<!-- Communication Requirements: internal and external notification expectations -->
<!-- Recovery Objectives: expected time-to-contain, time-to-restore -->
```

**Required subsections:** detection expectations, response escalation, recovery objectives
**Optional subsections:** communication templates, post-incident review expectations
**Required diagrams:** escalation flowchart
**Required cross-references:** Threat Model

---

*(To be written by the domain expert. This section defines incident response expectations.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Hard limitations the project operates under — regulatory, contractual, infrastructure, or organizational constraints that shape security decisions
> **out_of_scope:** Soft preferences, technology choices, implementation patterns, architectural decisions
> **contributes:** Prevents downstream domains from proposing designs that violate hard constraints; saves time by making limitations visible upfront
> **relationships:** Derived from Vision(01) and External Context; consumed by Architecture(05), Engineering(07), and Feature Technical(10)
> **responsibilities:** Enumerate non-negotiable limitations with enough specificity to evaluate a proposed design against them
> **generation_rules:** Identify constraints from regulatory, contractual, and organizational sources; state each as a hard boundary, not a preference; avoid prescribing solutions
> **enhancement_rules:** Add constraints when new obligations are discovered; remove constraints that no longer apply; preserve the hard-boundary framing
> **validation_rules:** Each constraint is specific enough to evaluate a design against; constraints are not disguised preferences; no implementation solutions are embedded
> **audit_rules:** Must exist if hard constraints apply; must not embed implementation solutions; must be evaluable as pass/fail against a proposed design

### Template

> **minimum_content:** 1–2 subsections
> **length_guidance:** concise
> **diagram_requirements:** none

```markdown
<!-- For each constraint: -->
<!--   - Source: regulatory, contractual, infrastructure, or organizational -->
<!--   - Statement: hard boundary expressed as pass/fail evaluable condition -->
<!--   - Impact: which downstream domains are affected -->
<!-- Avoid embedding solutions — state the boundary, not how to satisfy it -->
```

**Required subsections:** none
**Optional subsections:** constraint-to-domain impact matrix
**Required diagrams:** none
**Required cross-references:** Vision(01), External Context

---

*(To be written by the domain expert. This section defines hard security constraints.)*

---

## Required Sections

Every Security document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases | Content Requirements |
|---------|--------------|----------|---------|---------------------|
| Threat Model | `threat_model` | ✓ | Threats, Threat Modeling, Risk Assessment | Named methodology (STRIDE, PASTA, or OWASP); adversary profiles; threat enumeration with severity; attack surfaces; at least one downstream mitigation expectation per threat |
| Data Classification | `data_classification` | ✓ | Data Sensitivity, Data Classes | At least two distinct sensitivity levels; one concrete example per level; handling expectations per level; technology-independent classification |
| Security Principles | `security_principles` | ✓ | Security Posture, Guiding Security Principles | At least one principle; specific enough to constrain a concrete design decision; no generic platitudes; technology-independent |
| Compliance Requirements | `compliance` | | Regulatory Requirements, Compliance Obligations | Named compliance regimes (e.g. GDPR, HIPAA); scope and applicability per regime; obligation-to-downstream-control traceability |
| Incident Response | `incident_response` | | Incident Handling, Breach Response | Detection expectations; escalation paths; communication requirements; recovery objectives; outcome-level framing (no tooling mandates) |
| Purpose | `purpose` | | Overview, Summary | Security's role as project-wide authority; scope boundary with per-domain sections; derivation from Vision/Philosophy |
| Constraints | `constraints` | | Limitations | Non-negotiable limitations from regulatory, contractual, or organizational sources; each evaluable as pass/fail against a proposed design |
| Traceability | `traceability` | | Traces To, Derived From | Tier diagram showing derivation chain; list of downstream consumers; non-duplication rule stated |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Security aims to:

* Establish one authoritative threat model and data classification the whole project references.
* Prevent every domain from re-deriving its own security posture independently.
* Make compliance obligations traceable to specific downstream controls.
* Keep security a first-class constraint on Architecture and Engineering, not an afterthought.

---

## Non-Goals

Security(03) does not define:

* Specific authentication/authorization implementation
* Cryptographic algorithm or library selection
* CI/CD security tooling configuration
* Per-component trust boundaries
* Per-feature attack surface enumeration
* Source code

These responsibilities belong to Architecture, Engineering, and Feature Technical's own Security sections.

---

## Success Criteria

A Security document is successful when:

* Architecture, Engineering, and Feature Technical can each write their own Security Considerations/Standards sections by referencing this document instead of re-deriving a threat model from scratch.
* Data sensitivity is classified once and referenced consistently across the project.
* Compliance obligations are traceable to specific controls in downstream documents.
* A new contributor can read this document and understand what the project defends against before reading any implementation detail.

---

## Responsibilities

A Security document is responsible for defining:

* The project's threat model — what adversaries, threats, and failure modes are in scope
* Data classification — what data exists and how sensitive each class is
* Security principles that constrain every downstream design and implementation decision
* Compliance obligations the project must satisfy (regulatory, contractual, or self-imposed)
* Incident response expectations at a project level

Security provides the constraint every structural and implementation decision must satisfy — the same role Philosophy plays for values, applied specifically to threat and risk.

---

## Scope

A Security document should describe:

* Threats and adversaries the project defends against
* Data classification and sensitivity levels
* Project-wide security principles (e.g. least privilege, defense in depth, zero trust)
* Compliance regimes that apply (e.g. GDPR, HIPAA, PCI-DSS, SOC 2)
* Incident response expectations
* Security review or audit cadence

The Security document should remain stable, revisited when the threat landscape or compliance obligations change — not on every feature or release.

---

## Out of Scope

A Security document must not describe:

* Specific authentication or authorization implementations
* Cryptographic library or algorithm choices
* CI/CD security tooling (SAST, dependency scanning)
* Per-component trust boundaries
* Per-feature attack surface
* Source code

These belong to Architecture's Security Considerations, Engineering's Security Standards, and Feature Technical's Security Considerations respectively — see Relationship to Per-Domain Security Sections below.

---

## Inputs

Security derives from:

* Vision — what's being protected follows from what the product is
* Philosophy — security principles are a specialization of guiding principles

Security should not derive from implementation, architecture, or engineering decisions — it constrains those, not the other way around.

---

## Outputs

Security provides direction for:

* Architecture — Security Considerations
* Engineering — Security Standards
* Feature Technical Design — Security Considerations

Every per-domain Security section should be traceable to a threat, data class, or principle documented here.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Security connects to the documentation hierarchy — its derivation from Vision/Philosophy and its downstream consumers
> **out_of_scope:** Implementation-level traceability, audit event lineage, per-feature trace chains
> **contributes:** Makes Security's influence visible and verifiable; proves every downstream Security section is derived, not invented
> **relationships:** Derives from Vision(01) and Philosophy(02); feeds Architecture(05) Security Considerations, Engineering(07) Security Standards, Feature Technical(10) Security Considerations
> **responsibilities:** Show the derivation path from Vision/Philosophy to Security and from Security to every downstream Security section
> **generation_rules:** Use a text diagram showing the tier chain; list which downstream domains consume Security; state the non-duplication rule
> **enhancement_rules:** Update the diagram when new domains add Security sections; ensure derivation paths remain accurate
> **validation_rules:** Derivation paths are complete; no orphaned downstream Security sections; non-duplication rule is stated
> **audit_rules:** Must exist; must include tier diagram; must list downstream consumers; must state non-duplication constraint

### Template

> **minimum_content:** 1–2 subsections
> **length_guidance:** concise
> **diagram_requirements:** flowchart

```markdown
<!-- Tier diagram: Vision/Philosophy → Security → downstream domains -->
<!-- Downstream consumers: list each domain that consumes Security and how -->
<!-- Non-duplication rule: downstream sections reference this document, do not restate -->
```

**Required subsections:** tier diagram, downstream consumers, non-duplication rule
**Optional subsections:** none
**Required diagrams:** tier derivation chain (flowchart or text diagram)
**Required cross-references:** Vision(01), Philosophy(02), Architecture(05), Engineering(07), Feature Technical(10)

---

```text
Vision, Philosophy
       │
       ↓
   Security (03)
       │
       ├── guides ──> Architecture (05) ── Security Considerations realizes it structurally
       │
       └── guides ──> Engineering (07) ── Security Standards realizes it at the tooling/code level
                            │
                            ↓
                  Feature Technical (10) ── Security Considerations realizes it per feature
```

Every downstream Security Considerations or Security Standards section should trace back to this document's threat model, data classification, or principles — not restate them independently.

---

## Relationships

| Document | Relationship |
|---|---|
| Vision | Security derives from Vision (what's being protected follows from what the product is) |
| Philosophy | Security derives from Philosophy (security principles are a specialization of guiding principles) |
| Architecture | Security guides Architecture; Architecture's Security Considerations realizes it structurally |
| Engineering | Security guides Engineering; Engineering's Security Standards realizes it at the tooling/code level |
| Feature Technical | Security's threat model and data classification apply per feature via Feature Technical's own Security Considerations |

---

## Required Characteristics

A Security document should be:

* Specific about data classification, not just principles
* Traceable from every downstream Security section
* Stable across releases
* Technology-independent
* Compliance-aware
* Threat-focused rather than control-focused

---

## Generation Rules

When generating Security documentation:

* Define the threat model before listing controls.
* Classify data by sensitivity, not by storage location.
* Document security principles that apply across all domains.
* Reference compliance requirements by name and scope.
* Keep the project-wide security posture separate from per-domain enforcement.

---

## Enhancement Rules

When enhancing Security documentation:

* Verify the threat model reflects current attack surfaces.
* Update data classification when new data types are introduced.
* Add compliance requirements when regulations change.
* Ensure per-domain Security sections remain consistent with this standard.
* Preserve existing threat model structure while refining detail.

---

## Audit Rules

An audit should verify:

* Threat Model section exists and uses a named, structured methodology (error if missing).
* Data Classification section exists with at least one sensitivity level defined (error if missing).
* Security Principles are specific enough to be traceable to at least one downstream Security Considerations/Standards section (warning if too generic).
* No content duplicating what belongs in Architecture's, Engineering's, or Feature Technical's own Security sections (warning if found — see Relationship to Per-Domain Security Sections).

---

## Validation Rules

A Security document is considered valid when:

* A threat model is documented using a structured methodology (STRIDE, PASTA, or OWASP Threat Modeling).
* Data classification levels are defined with at least one example per level.
* Security principles are stated and are specific enough to guide a design decision, not generic ("be secure").
* No cryptographic algorithm, library, or CI tooling choice appears (that belongs to Engineering).
* No per-component trust boundary diagram appears (that belongs to Architecture).
* No per-feature attack surface appears (that belongs to Feature Technical).

---

## Summary

Security is the project-wide statement of what is being defended, against what, and why — the threat model, data classification, and security principles every structural and implementation decision must satisfy.

It sits beside Feature in Tier 2, guided by Vision and Philosophy, and guides Architecture and Engineering downstream the same way Philosophy does — without replacing the Security Considerations or Security Standards sections those domains, and Feature Technical, already own.

---

## Common Mistakes

Examples include:

* Restating the same threat model separately inside Architecture, Engineering, and Feature Technical instead of referencing this document.
* Listing cryptographic algorithms or SAST tools here instead of in Engineering's Security Standards.
* Treating "security will be handled later" as acceptable deferral.
* Defining principles too generically to constrain any actual decision (e.g. "we take security seriously").
* Omitting data classification and jumping straight to controls.

These should be reported during audits.

---

## Documentation Folder

Security documents live under:

```text
docs/raw/security/
```

---

## Usage

Written once per project, alongside or shortly after Vision and Philosophy, by whoever owns security/compliance for the project. Revisit on threat-landscape or compliance changes, not on every feature. Once registered in code, use `samgraha audit --domain security` to confirm Architecture, Engineering, and Feature Technical reference this document's threat model rather than duplicating it.

## Related

- [Vision Standard](01-vision-standards.md) — Security derives from Vision
- [Philosophy Standard](02-philosophy-standards.md) — Security derives from Philosophy
- [Architecture Standard](05-architecture-standards.md) — has its own Security Considerations section that realizes this one structurally
- [Engineering Standard](07-engineering-standards.md) — has its own Security Standards section that realizes this one at the tooling/code level
- [Feature Technical Standard](10-feature-technical-standards.md) — has its own Security Considerations section that realizes this one per feature

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| Draft | — | — | Initial proposal. No `StandardDefinition` for `security` exists in `crates/standards/src/builtin.rs` yet. Required Sections, audit rules, and relationships below are proposed. |

## Relationship to Per-Domain Security Sections

Architecture, Engineering, and Feature Technical each already have their own Security Considerations or Security Standards section. This document does not replace them — it is what they derive from, the same way Architecture and Design derive from Philosophy without Philosophy restating either.

| Layer | Owns | Derives From Security(03) |
|---|---|---|
| Security (03) | Threat model, data classification, security principles, compliance posture — once, project-wide | — |
| Architecture — Security Considerations | Trust boundaries between components, where threat-model mitigations map onto system structure | The threat model and principles this document defines |
| Engineering — Security Standards | Secrets management tooling, cryptographic library/algorithm choices, SAST/dependency-scanning enforcement in CI | The security principles and compliance obligations this document defines |
| Feature Technical — Security Considerations | Per-feature authentication/authorization flows, per-feature input validation, per-feature audit events | The threat model and data classification this document defines, scoped to one feature |

An audit finding a duplicated threat model or data classification scheme inside Architecture, Engineering, or Feature Technical should flag it as belonging here instead — those sections should reference Security(03), not restate it.

---

## Quality Requirements

A Security document should be:

* Written once per project, revised only when threat landscape or compliance obligations change
* Specific about data classification, not just principles
* Traceable — every downstream Security Considerations/Standards section should be able to cite which threat or principle it addresses
* Technology independent — it says what must be defended against, not which library does it
* Reviewed on a defined cadence, not left stale

---
