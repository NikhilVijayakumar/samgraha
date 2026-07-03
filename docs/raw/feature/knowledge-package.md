# Knowledge Package

This section details the Knowledge Package.

## Purpose

A Knowledge Package is the deployable representation of compiled engineering knowledge.

It encapsulates all engineering knowledge required by a consumer into a single, portable, deterministic package. A Knowledge Package is produced by the Knowledge Resolution process and consumed by the Knowledge Runtime.

Rather than exposing repositories directly, Saṃgraha delivers Knowledge Packages. This allows consumers to receive only the engineering knowledge relevant to their context while preserving repository boundaries, ownership, and audit integrity.

Knowledge Packages are generated artifacts and can always be regenerated from source documentation.

---

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Package Composition

The platform shall compose Knowledge Packages from resolved engineering knowledge.

A Knowledge Package may contain:

* compiled documentation
* semantic sections
* section semantic types
* documentation metadata
* repository metadata
* workspace metadata
* audit metadata
* enrichment artifacts
* search indexes
* dependency metadata
* package manifest

Only required knowledge shall be included.

Packages may be composed at section granularity. A package may include selected semantic section types from each document rather than complete documents, reducing package size while preserving the specific engineering knowledge the consumer requires.

---

## FR2. Package Manifest

Every Knowledge Package shall include a manifest describing its contents.

The manifest shall identify:

* package identity
* package version
* included repositories
* included documentation domains
* included artifacts
* dependency information
* generation timestamp
* build metadata

The manifest enables package validation and inspection.

---

## FR3. Package Integrity

Knowledge Packages shall maintain integrity metadata.

Integrity information may include:

* package hash
* artifact hashes
* dependency consistency
* audit status
* validation results

Consumers shall be able to determine whether a package is valid before use.

---

## FR4. Audit Preservation

Knowledge Packages shall preserve audit information.

Audit metadata shall remain associated with every included knowledge artifact.

Consumers shall be able to distinguish:

* verified knowledge
* advisory audit results
* audit history
* validation status

Knowledge Packages shall not remove audit information.

---

## FR5. Repository Isolation

Knowledge Packages shall preserve repository ownership.

Each artifact shall remain traceable to:

* originating repository
* documentation domain
* source document
* owning workspace

Package composition shall not obscure repository boundaries.

---

## FR6. Consumer Profiles

Knowledge Packages shall support multiple delivery profiles.

Examples include:

* Development
* Documentation
* Engineering
* AI Assistant
* Minimal
* Full Knowledge

Profiles determine the scope of packaged knowledge.

Profiles may specify included semantic section types in addition to document scope. This allows fine-grained control over package contents without packaging full documents.

Examples of section-aware profile composition:

```
AI Assistant profile:
  include section types: [purpose, functional_requirements, business_rules, constraints]
  exclude section types: [future_extensions, traceability]

Minimal profile:
  include section types: [purpose]

Engineering profile:
  include section types: [purpose, functional_requirements, business_rules, constraints, dependencies]
  include section types: [architecture_decisions, rationale]
```

Section-aware profiles reduce AI context consumption while preserving the engineering knowledge most relevant to each consumer type.

---

## FR7. Package Validation

Knowledge Packages shall be validated before publication.

Validation shall verify:

* manifest completeness
* dependency consistency
* artifact integrity
* audit availability
* package completeness

Invalid packages shall not be published.

---

## FR8. Package Lifecycle

The platform shall support the complete lifecycle of Knowledge Packages.

Lifecycle operations include:

* generation
* validation
* publication
* loading
* replacement
* regeneration
* disposal

Knowledge Packages remain reproducible throughout their lifecycle.

---

## Package Layouts

Knowledge Packages are assembled in one of two layouts.

| Layout | Portability | Use Case |
|---|---|---|
| Physical | Portable across environments | Default; copies `knowledge.db` and `docs/` into output directory |
| Virtual | Workspace-local only | Performance optimization; references source `knowledge.db` by absolute path |

Physical is the default and the only distributable format.

Virtual packages break if any source repository moves or the package is used on a different machine. Virtual packages are not suitable for distribution or archiving.

---

## Business Rules

* Knowledge Packages are generated artifacts.
* Documentation remains the authoritative source.
* Knowledge Packages are deterministic.
* Packages preserve repository ownership.
* Physical packages remain portable across environments. Virtual packages are workspace-local only and not portable.
* Packages are disposable.
* Packages never modify repository documentation.

---

## Package Lifecycle

```text
Knowledge Registry
        │
        ▼
Knowledge Resolution
        │
        ▼
Package Composition
        │
        ▼
Package Validation
        │
        ▼
Knowledge Package
        │
        ▼
Knowledge Runtime
        │
        ▼
Knowledge Consumers
```

---

## Inputs

Knowledge Package generation consumes:

* resolved engineering knowledge
* repository metadata
* workspace metadata
* dependency metadata
* audit metadata
* enrichment artifacts
* package configuration

---

## Outputs

Knowledge Package generation produces:

* Knowledge Package
* package manifest
* package metadata
* validation metadata
* integrity metadata

Outputs are consumed by runtime services.

---

## Constraints

Knowledge Packages shall:

* remain deterministic
* support large engineering workspaces
* preserve repository isolation
* support incremental regeneration
* remain portable
* support offline usage
* scale with engineering knowledge

Physical package formats are implementation concerns.

---

## Dependencies

Knowledge Package depends upon:

* Knowledge Resolution
* Knowledge Registry
* Audit Framework
* Knowledge Enrichment
* Workspace Management

Knowledge Package provides deployable engineering knowledge to:

* Knowledge Runtime
* Engineering CLI
* Documentation Services
* Future platform consumers

---

## Non-Goals

Knowledge Package does not:

* compile documentation
* execute audits
* perform enrichment
* discover repositories
* deliver knowledge directly

Those responsibilities belong to their respective platform capabilities.

---

## Future Extensions

The Knowledge Package framework should support future capabilities, including:

* package signing
* package compression
* package encryption
* package federation
* incremental package updates
* streaming packages
* package caching
* package replication
* consumer-specific package optimization

Future capabilities should integrate without changing the logical package model.

---

## Acceptance Criteria

The feature is successful when:

* engineering knowledge is packaged deterministically
* package integrity is verifiable
* repository ownership is preserved
* packages remain portable across environments
* runtime services consume packages consistently
* packages are fully reproducible from documentation
* package regeneration restores the complete engineering knowledge state

---

## Traceability

This feature derives from the following Vision commitments:

* **Knowledge is compiled before delivery.**
* **Only relevant engineering knowledge is delivered to consumers.**
* **Generated artifacts are disposable.**
* **Engineering knowledge is deterministic and reproducible.**
* **Repository ownership and traceability are preserved throughout the knowledge lifecycle.**

**Traceability**

Vision → Feature: Knowledge Package
