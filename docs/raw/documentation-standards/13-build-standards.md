# Build Standard

## Table of Contents
- [Purpose](#purpose)
- [Versioning Policy](#versioning-policy)
- [Packaging & Distribution](#packaging--distribution)
- [Release Process](#release-process)
- [Artifact Provenance](#artifact-provenance)
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
- [Relationship to Engineering's Build Standards](#relationship-to-engineerings-build-standards)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why the Build standard exists — its role as the project-wide release policy governing versioning, packaging, distribution, and artifact provenance
> **out_of_scope:** CI/CD pipeline mechanics, per-feature implementation details, source code, architecture decisions
> **contributes:** Establishes Build's identity as the single authoritative release policy that Implementation(12) output gets turned into shippable artifacts
> **relationships:** Derived from Implementation(12) and Security(03); consumed by Readme(14) for install/run instructions; distinct from Engineering(07) CI/CD mechanics
> **responsibilities:** Define what Build is, what it is not, and how it relates to Implementation, Security, Engineering, and Readme
> **generation_rules:** Start with the relationship to Implementation and Security; state the project-wide scope; distinguish from Engineering's CI/CD mechanics
> **enhancement_rules:** Keep the scope boundary between Build and Engineering sharp; remove any CI/CD pipeline language that leaked in
> **validation_rules:** Purpose is clearly defined; no CI/CD mechanics present; project-wide scope stated; relationship to Implementation and Security established
> **audit_rules:** Must exist; must not contain CI/CD pipeline configuration; must state the relationship to Implementation(12) and Security(03)

This document defines the standard for Build documentation within the engineering documentation ecosystem.

Build establishes the project-wide release policy: versioning scheme, packaging format, distribution channels, and artifact provenance for turning what Implementation (12) produced into a shippable artifact.

It defines **how a release is versioned, packaged, and distributed**, once, at the project level — the same way Security (03) defines the project-wide threat model once rather than per component.

It does not define the CI/CD mechanics of how commits get built and tested. That belongs to Engineering's own Build Standards section.

---

## Versioning Policy

> **semantic_type:** `versioning_policy`
> **scope:** The project's versioning scheme, bump triggers, and version numbering rules
> **out_of_scope:** CI/CD pipeline mechanics, per-feature release notes, changelog generation tooling
> **contributes:** Makes versioning predictable for consumers and contributors; eliminates ambiguity about what triggers a bump
> **relationships:** Derived from Implementation(12) release semantics; referenced by Release Process for tagging rules; consumed by Readme(14) for version references
> **responsibilities:** Define the versioning scheme (semver or equivalent), what constitutes a major/minor/patch bump, and pre-release conventions
> **generation_rules:** Choose a versioning scheme; define bump rules with concrete examples; document pre-release and build metadata conventions if applicable
> **enhancement_rules:** Update bump rules when the project's release semantics change; preserve examples that clarify edge cases
> **validation_rules:** Versioning scheme is named and specific; every bump type has a definition with examples; no ambiguous or implicit rules remain
> **audit_rules:** Must exist as a required section (error if missing); must define at least one bump rule; must not be vague or implicit

*(To be populated by the domain expert. This section defines the versioning scheme and bump rules.)*

---

## Packaging & Distribution

> **semantic_type:** `packaging`
> **scope:** How release artifacts are packaged, named, and distributed — formats, registries, channels, and naming conventions
> **out_of_scope:** Build tooling configuration, CI/CD pipeline steps, source compilation mechanics
> **contributes:** Tells consumers exactly how to obtain and install a release; makes distribution channels explicit and verifiable
> **relationships:** Derived from Implementation(12) build outputs; referenced by Release Process for packaging steps; consumed by Readme(14) for install instructions
> **responsibilities:** Name every packaging format, distribution channel, and artifact naming convention the project uses
> **generation_rules:** List each package format with its distribution channel; specify artifact naming conventions; document any platform-specific variants
> **enhancement_rules:** Add new distribution channels as they are introduced; remove deprecated channels; keep format specifications current with what is actually shipped
> **validation_rules:** Every mentioned artifact format has a concrete distribution channel; naming conventions are documented; no vague references to "packages" without specifics
> **audit_rules:** Must exist as a required section (error if missing); must name concrete formats and channels; must not be vague

*(To be populated by the domain expert. This section defines packaging formats and distribution channels.)*

---

## Release Process

> **semantic_type:** `release_process`
> **scope:** The repeatable sequence from tagged commit to published artifact — the release checklist
> **out_of_scope:** CI/CD pipeline configuration, per-feature release notes, changelog generation
> **contributes:** Makes releases repeatable and auditable; prevents tribal knowledge about how to ship
> **relationships:** References Versioning Policy for tagging rules and Packaging & Distribution for artifact steps; implemented by Engineering's CI/CD mechanics
> **responsibilities:** Define a step-by-step release process from tag to publication, including rollback and deprecation procedures
> **generation_rules:** Write the process as a numbered sequence, not prose; include rollback and deprecation steps; reference Versioning Policy and Packaging & Distribution
> **enhancement_rules:** Update steps when the release toolchain changes; preserve rollback procedures; keep the checklist current with actual practice
> **validation_rules:** Process is a numbered sequence, not narrative prose; each step is concrete and actionable; rollback and deprecation are included
> **audit_rules:** Must exist as a required section (error if missing); must be a numbered sequence; must not be prose narrative; must include rollback

*(To be populated by the domain expert. This section defines the release checklist.)*

---

## Artifact Provenance

> **semantic_type:** `provenance`
> **scope:** How release artifacts are signed, checksummed, and verified — supply chain integrity practices
> **out_of_scope:** Security threat model details, signing infrastructure implementation, key management procedures
> **contributes:** Makes artifact integrity verifiable for consumers; satisfies supply chain security requirements
> **relationships:** References Security(03) threat model for signing practices; referenced by Readme(14) for verification instructions; aligned with Release Process
> **responsibilities:** Define signing, checksum, SBOM, and verification practices for published artifacts
> **generation_rules:** Reference Security(03) for threat model context; define what is signed and how; document verification steps consumers can follow
> **enhancement_rules:** Update signing practices when Security(03) posture changes; add new provenance mechanisms as they are adopted; preserve consumer-facing verification steps
> **validation_rules:** Provenance practices reference Security(03) rather than re-deriving a threat model; verification steps are consumer-actionable; signing scope is explicit
> **audit_rules:** Must reference Security(03) rather than restating a threat model (warning if duplicated); must define verification steps; must not be absent if signing is practiced

*(To be populated by the domain expert. This section defines artifact provenance and signing practices.)*

---

## Constraints

> **semantic_type:** `constraints`
> **scope:** Hard limitations that shaped the build and release policy — infrastructure, regulatory, or organizational constraints
> **out_of_scope:** Soft preferences, technology choices, CI/CD pipeline details, architectural decisions
> **contributes:** Prevents downstream domains from proposing release practices that violate hard constraints; makes limitations visible upfront
> **relationships:** May reflect constraints from Security(03) compliance requirements; influenced by Engineering(07) infrastructure constraints
> **responsibilities:** Document non-negotiable limitations that materially affect versioning, packaging, or distribution decisions
> **generation_rules:** Identify constraints from regulatory, infrastructure, and organizational sources; state each as a hard boundary; avoid prescribing solutions
> **enhancement_rules:** Add constraints when new obligations are discovered; remove constraints that no longer apply; preserve the hard-boundary framing
> **validation_rules:** Each constraint is specific enough to evaluate a release practice against; constraints are not disguised preferences; no implementation solutions embedded
> **audit_rules:** Must not embed implementation solutions; must be evaluable as pass/fail against a proposed release practice

*(To be populated by the domain expert. This section documents hard constraints on the build and release policy.)*

---

## Required Sections

Every Build document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Versioning Policy | `versioning_policy` | ✓ | Versioning, Version Scheme |
| Packaging & Distribution | `packaging` | ✓ | Packaging, Distribution Channels |
| Release Process | `release_process` | ✓ | Release Checklist, Release Steps |
| Artifact Provenance | `provenance` | | Signing, SBOM, Supply Chain |
| Purpose | `purpose` | | Overview, Summary |
| Constraints | `constraints` | | Limitations |
| Traceability | `traceability` | | Traces To, Derived From |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Build aims to:

* Give the project one authoritative release policy.
* Make versioning and packaging predictable for consumers.
* Make artifact provenance verifiable.

---

## Non-Goals

Build does not define:

* Product Vision
* Security threat models
* Architecture
* Per-feature implementation details
* CI/CD pipeline configuration
* Source code

These responsibilities belong to other documentation standards.

---

## Success Criteria

Build is successful when:

* A consumer can determine how to install a release without reading source code.
* A contributor can determine what triggers a version bump without asking.
* Release provenance is verifiable and traceable to Security(03)'s posture.
* Readme(14) can write install/run instructions by referencing this document.

---

## Responsibilities

Build is responsible for defining:

* The versioning scheme (semver or equivalent) and what triggers each bump
* Packaging formats and distribution channels (registries, package managers, binaries)
* The release process from tagged commit to published artifact
* Artifact provenance — signing, checksums, SBOM, supply-chain verification
* Deprecation and rollback policy for published releases

Build turns Implementation's as-built record into something a consumer can install and trust.

---

## Scope

Build may describe:

* Versioning policy and semver bump rules
* Packaging formats (containers, binaries, language-native packages)
* Distribution channels (registries, package managers, release pages)
* Release process and checklist
* Artifact signing and provenance
* Deprecation and rollback policy

Every Build document should remain project-wide and stable — revised when the release process changes, not per release.

---

## Out of Scope

Build must not describe:

* Product Vision
* Security threat models (Build's own provenance/signing practices should reference Security(03), not restate its threat model)
* Architecture
* Per-feature implementation deviations
* CI/CD pipeline mechanics (how commits get built and tested)
* Source code

Per-feature realization belongs to Implementation. CI/CD mechanics belong to Engineering.

---

## Inputs

Build derives from:

* Implementation (12) — what's actually being packaged
* Security (03) — threat model referenced for provenance/signing practices

---

## Outputs

Build provides:

* A shippable, versioned artifact
* Install/run instructions Readme (14) can reference

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Build derives from upstream sources and feeds downstream consumers — the dependency chain
> **out_of_scope:** Per-feature traceability, internal implementation provenance, CI/CD pipeline lineage
> **contributes:** Makes the Build document's place in the documentation ecosystem explicit; ensures readers know where Build comes from and who consumes it
> **relationships:** Derived from Implementation(12); informed by Security(03); consumed by Readme(14)
> **responsibilities:** Show the traceability chain from Implementation through Build to Readme; reference Security(03) for provenance context
> **generation_rules:** Map the chain: Implementation → Build → Readme; note Security(03) as an informed-by relationship; use a diagram if the chain is complex
> **enhancement_rules:** Update the chain when new consumers or sources are added; preserve the diagram as a quick reference
> **validation_rules:** Every upstream source and downstream consumer is named; the chain is visually clear; Security(03) is referenced, not restated
> **audit_rules:** Must exist; must name Implementation(12) as upstream; must name Readme(14) as consumer; must reference Security(03) for provenance

```text
Implementation (12)
        │
        ↓
     Build (13) ── informed by ──> Security (03) (provenance/signing practices)
        │
        ↓
     Readme (14) ── requires Build for install/run instructions
```

Every Build document should trace to the Implementation(s) it packages, and reference — not restate — Security(03)'s threat model when describing provenance or signing.

---

## Relationships

| Document | Relationship |
|---|---|
| Implementation | Build packages what Implementation produced |
| Engineering | Build's release policy is distinct from Engineering's CI/CD mechanics — see split above |
| Security | Build's artifact provenance/signing practices reference Security's threat model rather than restating it |
| Readme | Readme requires Build to exist, for install/run instructions |

---

## Required Characteristics

Build should be:

* Project-wide and stable, not rewritten per release
* Specific about versioning triggers
* Verifiable — provenance/signing claims can be checked
* Free of CI/CD mechanics that belong in Engineering

---

## Generation Rules

When generating Build documentation:

* Document what the build produces, not how CI/CD works.
* List build targets, entry points, and artifact locations.
* Specify required tools and versions.
* Document build-time dependencies separately from runtime dependencies.
* Keep build documentation accurate to the current shippable state.

---

## Enhancement Rules

When enhancing Build documentation:

* Update when build targets or artifacts change.
* Verify tool and dependency versions match the current environment.
* Remove references to deprecated build targets.
* Ensure the documentation matches what is actually shippable.
* Preserve the rationale for build-time constraints.

---

## Audit Rules

An audit should verify:

* Versioning Policy is present and specific (error if missing).
* Packaging & Distribution is present and specific (error if missing).
* Release Process is present (error if missing).
* No CI/CD pipeline mechanics duplicated from Engineering (warning if found).
* Provenance content references Security(03) rather than restating a threat model (warning if duplicated).

---

## Validation Rules

Build is considered valid when:

* Versioning Policy defines what triggers each version bump.
* Packaging & Distribution names concrete formats and channels, not "we'll figure it out."
* Release Process is a repeatable sequence, not prose describing intent.
* No CI/CD pipeline configuration appears (that belongs to Engineering).
* Provenance practices reference Security(03) instead of re-deriving a threat model.

---

## Summary

Build is the project-wide release policy — versioning, packaging, distribution, and provenance for turning what Implementation produced into something a consumer can install and trust — distinct from Engineering's CI/CD mechanics and from Security's threat model, which it references rather than restates.

---

## Common Mistakes

Examples include:

* Describing CI pipeline YAML instead of release/versioning policy.
* Leaving versioning policy implicit ("we just bump it when it feels right").
* Re-deriving a threat model for artifact signing instead of referencing Security(03).
* Treating Build as a per-release document instead of a stable, project-wide policy.
* Omitting rollback/deprecation policy entirely.

These should be reported during audits.

---

## Documentation Folder

Build documents live under:

```text
docs/raw/build/
```

---

## Usage

Written once per project, alongside Engineering, and revised when the release process itself changes rather than per release. Once registered in code, use `samgraha audit --domain build` to confirm versioning/packaging/distribution policy is documented and CI/CD mechanics haven't leaked in from Engineering.

## Related

- [Implementation Standard](12-implementation-standards.md) — what this packages
- [Engineering Standard](07-engineering-standards.md) — CI/CD mechanics this is distinct from
- [Security Standard](03-security-standards.md) — threat model this references for provenance/signing
- [Readme Standard](14-readme-standards.md) — requires this document to exist for install/run instructions

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| Draft | — | — | Initial proposal. No `StandardDefinition` for `build` exists in `crates/standards/src/builtin.rs` yet. Required Sections, audit rules, and relationships below are proposed. |

## Relationship to Engineering's Build Standards

Engineering(07) already has a mandatory **Build Standards** section: how the project is built — CI/CD mechanics, build tooling, pipeline configuration. Build(13) does not replace it. They operate at different scopes, the same split Security(03) draws against Architecture's, Engineering's, and Feature Technical's own Security sections.

| Layer | Owns | Derives From |
|---|---|---|
| Engineering — Build Standards | CI/CD mechanics: how commits get built and tested, pipeline configuration | Engineering's own technology rationale |
| Build (13) | Release policy: versioning scheme, packaging format, distribution channels, artifact provenance | Implementation(12) — what's actually being packaged |

A Build document that describes CI pipeline configuration instead of release/versioning/distribution policy should be flagged during audit — that content belongs in Engineering.

---

## Quality Requirements

Build should be:

* Project-wide and stable, not rewritten per release
* Specific about versioning triggers (what counts as a breaking change)
* Free of CI/CD pipeline mechanics that belong in Engineering
* Traceable to what Implementation actually produced
* Explicit about provenance/signing, referencing Security(03) rather than restating it

---
