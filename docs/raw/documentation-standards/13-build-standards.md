# Build Standard

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
- [Relationship to Engineering's Build Standards](#relationship-to-engineerings-build-standards)
- [Quality Requirements](#quality-requirements)

---


## Purpose

This document defines the standard for Build documentation within the engineering documentation ecosystem.

Build establishes the project-wide release policy: versioning scheme, packaging format, distribution channels, and artifact provenance for turning what Implementation (12) produced into a shippable artifact.

It defines **how a release is versioned, packaged, and distributed**, once, at the project level — the same way Security (03) defines the project-wide threat model once rather than per component.

It does not define the CI/CD mechanics of how commits get built and tested. That belongs to Engineering's own Build Standards section.

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
