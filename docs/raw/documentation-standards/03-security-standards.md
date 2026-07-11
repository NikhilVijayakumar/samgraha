# Security Standard

## Table of Contents
- [Purpose](#purpose)
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

This document defines the standard for Security documentation within the engineering documentation ecosystem.

A Security document establishes the project-wide threat model, data classification scheme, security principles, and compliance posture that every other domain must operate within.

It defines **what the project is defending against and why**, once, at the project level — the same way Vision defines why the product exists and Philosophy defines the values guiding decisions.

It does not define how any single component enforces that posture. That belongs to each domain's own Security Considerations or Security Standards section.

---

## Required Sections

Every Security document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Threat Model | `threat_model` | ✓ | Threats, Threat Modeling, Risk Assessment |
| Data Classification | `data_classification` | ✓ | Data Sensitivity, Data Classes |
| Security Principles | `security_principles` | ✓ | Security Posture, Guiding Security Principles |
| Compliance Requirements | `compliance` | | Regulatory Requirements, Compliance Obligations |
| Incident Response | `incident_response` | | Incident Handling, Breach Response |
| Purpose | `purpose` | | Overview, Summary |
| Constraints | `constraints` | | Limitations |
| Traceability | `traceability` | | Traces To, Derived From |

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
