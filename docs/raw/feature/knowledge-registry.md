# Knowledge Registry

## Purpose

The Knowledge Registry is the compiled representation of engineering knowledge produced from repository documentation.

It serves as the authoritative source of generated knowledge artifacts for search, retrieval, audit metadata, enrichment, and MCP delivery.

The Knowledge Registry stores compiled knowledge. It does not manage repository identity, registration, or metadata synchronization. Those responsibilities belong to the Repository Registry.

The Knowledge Registry is a generated artifact.

It is never manually edited and can always be regenerated from repository documentation.

---

## Functional Requirements

## FR1. Knowledge Artifact Storage

The Knowledge Registry shall store generated knowledge artifacts.

Examples include:

* compiled documents
* semantic sections
* section semantic types
* compiled chunks
* search indexes
* cross-reference indexes
* dependency graphs
* audit metadata
* enrichment artifacts
* package manifests
* future generated artifacts

The registry stores compiled knowledge rather than source documentation.

Semantic sections shall be stored as first-class artifacts alongside their parent document. Each section record shall preserve:

* semantic type (e.g. `functional_requirements`, `business_rules`, `constraints`)
* canonical section name
* content
* required flag
* source document reference
* section order within document

---

## FR2. Knowledge Retrieval

The Knowledge Registry shall support retrieval of compiled knowledge.

Knowledge may be retrieved by:

* repository
* workspace
* documentation domain
* document
* feature
* artifact
* metadata
* search query
* semantic section type
* section type across documents

Examples of section-type retrieval:

* all `functional_requirements` sections across the repository
* all `constraints` sections for a given domain
* the `purpose` section of a specific document
* all `dependencies` sections across the workspace

Retrieval shall operate independently of source documentation.

---

## FR3. Artifact Lifecycle Management

The Knowledge Registry shall manage generated artifact lifecycles.

Supported lifecycle operations include:

* creation
* update
* replacement
* invalidation
* regeneration
* disposal

Generated artifacts shall remain independently manageable.

---

## FR4. Registry Integrity

The Knowledge Registry shall maintain integrity information.

Integrity metadata may include:

* build status
* registry version
* artifact versions
* dependency consistency
* validation state
* corruption detection

Consumers shall be able to determine whether the registry is suitable for use.

---

## FR5. Version Awareness

The Knowledge Registry shall track build metadata.

Examples include:

* document hashes
* artifact hashes
* compiler version
* build version
* build timestamp
* enrichment version
* audit version

Version metadata supports incremental builds and reproducibility.

---

## FR6. Workspace Support

The Knowledge Registry shall support both repository and workspace knowledge.

Workspace registries shall preserve repository boundaries while enabling unified knowledge retrieval.

---

## FR7. Concurrent Access

The Knowledge Registry shall support:

* concurrent readers
* serialized updates
* safe incremental rebuilds
* consistent query behavior

Consumers shall never observe partially committed registry updates.

---

## Business Rules

* Documentation is the authoritative source of knowledge.
* The Knowledge Registry is a generated artifact.
* Generated artifacts are disposable.
* Registry contents shall be reproducible.
* Registry updates shall preserve consistency.
* Source documentation shall never be modified by the registry.
* Consumers interact with compiled knowledge rather than documentation.

---

## Registry Lifecycle

```text
Documentation
        │
        ▼
Knowledge Compiler
        │
        ▼
Knowledge Registry
        │
        ├── Compiled Documents
        ├── Search Indexes
        ├── Audit Metadata
        ├── Enrichment Artifacts
        ├── Package Manifests
        └── Future Artifacts
        │
        ▼
Knowledge Consumers
        │
        ├── Resolver
        ├── MCP Runtime
        ├── Engineering CLI
        └── Future Consumers
```

This is the knowledge track. Repository metadata follows a separate track through the Repository Registry.

---

## Inputs

The Knowledge Registry consumes:

* compiled documentation
* audit metadata
* enrichment artifacts
* build metadata

---

## Outputs

The Knowledge Registry provides:

* compiled knowledge
* searchable artifacts
* audit metadata
* enrichment metadata
* package metadata

Outputs are consumed by platform services.

---

## Constraints

The Knowledge Registry shall:

* support large repositories
* support large workspaces
* operate offline
* remain deterministic
* support incremental updates
* recover through regeneration
* preserve repository isolation
* scale with documentation volume

Physical persistence mechanisms are implementation concerns.

---

## Dependencies

The Knowledge Registry depends upon:

* Knowledge Compiler
* Incremental Build
* Audit Framework
* Knowledge Enrichment

The Knowledge Registry is distinct from the Repository Registry. The Knowledge Registry stores compiled engineering knowledge. The Repository Registry stores repository metadata. The two registries never intersect.

The Registry provides knowledge to:

* Knowledge Resolver
* MCP Runtime
* Engineering CLI

---

## Non-Goals

The Knowledge Registry does not:

* modify documentation
* own compilation
* execute audits
* perform enrichment
* resolve dependencies
* manage repository identity or registration
* synchronize metadata between repositories
* deliver knowledge to AI agents

Those responsibilities belong to their respective platform components. Repository identity, registration, and metadata synchronization belong to the Repository Registry.

---

## Future Extensions

The Knowledge Registry should support future capabilities, including:

* knowledge package caching
* semantic indexes
* graph indexes
* artifact replication
* workspace aggregation
* repository snapshots
* schema migration

Future extensions should integrate without changing the logical registry model.

---

## Acceptance Criteria

The feature is successful when:

* compiled knowledge is consistently available
* generated artifacts remain reproducible
* repository and workspace knowledge can be retrieved efficiently
* registry integrity is maintained
* incremental updates preserve correctness
* regeneration fully restores the registry
* platform components consume the registry without requiring source documentation

---

## Traceability

This feature derives from the following Vision commitments:

* **Documentation is the source of truth.**
* **Knowledge is compiled before delivery.**
* **Generated artifacts are disposable.**
* **Knowledge artifacts are reproducible.**

**Note:** Repository registration (previously FR1) has been transferred to the Repository Registry feature. The Knowledge Registry now owns compiled engineering knowledge only. Repository identity, metadata, and synchronization are managed by the Repository Registry.

**Traceability**

Vision → Feature: Knowledge Registry
Architecture: Component Model → Feature: Knowledge Registry
Architecture: Repository Registry Architecture → Feature: Repository Registry (separate track)
