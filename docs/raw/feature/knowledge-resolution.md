# Knowledge Resolution

This section details the Knowledge Resolution.

## Purpose

Knowledge Resolution assembles the engineering knowledge required by a consumer into a coherent Knowledge Package.

Rather than exposing entire repositories, the resolver identifies relevant repositories, documentation domains, dependencies, and generated artifacts, then composes only the knowledge necessary for the requested context.

Knowledge Resolution minimizes context while preserving completeness.

The resulting Knowledge Package becomes the foundation for MCP delivery, engineering tools, and future knowledge consumers.

---

## Functional Requirements

This section details the Functional Requirements.

## FR1. Repository Discovery

The resolver shall discover repositories participating in knowledge composition.

Repository discovery uses the local metadata cache as its primary source:

* workspace configuration
* repository configuration
* declared dependencies
* cached metadata (`.samgraha/registry.db` — `repository_cache` table)

The metadata cache (`.samgraha/registry.db`) is populated from the Repository Registry during sync. The resolver never contacts the Repository Registry directly during discovery.

Repository discovery determines the available knowledge sources.

---

## FR2. Dependency Resolution

The resolver shall resolve repository dependencies.

Resolution shall identify:

* direct dependencies
* transitive dependencies
* optional dependencies
* dependency cycles
* unresolved dependencies

Resolution determines repository relationships rather than implementation dependencies.

Cycle detection uses depth-first search. When a cycle is detected, resolution aborts immediately and reports the full cycle path. No partial package is produced.

Example:
```
error: Dependency cycle detected
  astra → prana → tantra → astra
```

---

## FR3. Knowledge Composition

The resolver shall compose a Knowledge Package from resolved knowledge sources.

Knowledge Packages may contain:

* compiled documentation
* documentation metadata
* dependency metadata
* audit metadata
* enrichment artifacts
* search indexes
* package metadata

Only required knowledge shall be included.

---

## FR4. Context Reduction

Knowledge Resolution shall minimize unnecessary knowledge.

The resolver shall exclude:

* unrelated repositories
* unrelated documentation domains
* unused artifacts
* unnecessary metadata

Knowledge Packages should remain as small as possible while preserving engineering completeness.

---

## FR5. Cross-Repository Navigation

The resolver shall preserve relationships between repositories.

Relationships may include:

* documentation references
* feature mappings
* architectural relationships
* engineering dependencies
* external context references

Consumers shall navigate resolved knowledge seamlessly.

---

## FR6. Package Validation

Knowledge Packages shall be validated before publication.

Validation shall verify:

* dependency completeness
* artifact consistency
* registry integrity
* audit availability
* package metadata

Invalid packages shall be reported.

---

## FR7. Resolution Profiles

Repositories shall support configurable resolution profiles.

Examples include:

* Local Repository
* Workspace
* Development
* Production
* Documentation Only
* Full Knowledge

Profiles determine the scope of composed knowledge.

---

## FR8. Incremental Resolution

Knowledge Resolution shall support incremental updates.

When repository knowledge changes, only affected Knowledge Packages shall be recomposed.

---

## Business Rules

* Documentation remains the authoritative source of knowledge.
* The Knowledge Registry provides compiled knowledge.
* Resolution never modifies source documentation, compiled knowledge databases, or repository configuration. Resolution may update the registry cache in `.samgraha/registry.db` as a performance optimization. The cache database is disposable and can be deleted without affecting correctness.
* Knowledge Packages are generated artifacts.
* Resolution shall be deterministic.
* Unrelated repositories shall not be included.
* Resolution shall preserve repository ownership and boundaries.
* Resolution uses metadata cache only. The Repository Registry is never contacted at runtime.
* Cache miss at runtime degrades gracefully — reports missing dependency, does not block.

---

## Resolution Lifecycle

```text
Metadata Cache
        │
   ┌────┴────┐
   ▼         ▼
  Hit       Miss
   │         │
   ▼         ▼
Repository  Degrade
Discovery   Gracefully
   │
   ▼
Dependency Resolution
   │
   ▼
Knowledge Registry
   │
   ▼
Knowledge Composition
   │
   ▼
Package Validation
   │
   ▼
Knowledge Package
   │
   ▼
Knowledge Consumers
   │
   ├── MCP Runtime
   ├── Engineering CLI
   ├── Documentation Tools
   └── Future Consumers
```

Resolution uses the metadata cache first. Cache miss at runtime degrades gracefully — reports missing dependency, never contacts the Repository Registry.

---

## Inputs

Knowledge Resolution consumes:

* repository metadata (from cache)
* workspace metadata
* dependency declarations
* compiled knowledge
* audit metadata
* enrichment artifacts
* resolution configuration
* metadata cache (`.samgraha/registry.db` — `repository_cache` table)

---

## Outputs

Knowledge Resolution produces:

* Knowledge Packages
* dependency metadata
* package manifests
* package metadata
* repository relationships
* validation metadata

---

## Constraints

Knowledge Resolution shall:

* support large workspaces
* operate deterministically
* support offline operation
* preserve repository isolation
* minimize package size
* support incremental recomposition
* tolerate optional dependencies
* degrade gracefully on metadata cache miss

---

## Dependencies

Knowledge Resolution depends upon:

* Knowledge Registry
* Repository Registry (indirectly through metadata cache)
* Incremental Build
* Audit Framework
* Knowledge Enrichment
* Workspace Management

Knowledge Resolution provides Knowledge Packages to:

* MCP Runtime
* Engineering CLI
* Documentation Services
* Future platform consumers

---

## Non-Goals

Knowledge Resolution does not:

* compile documentation
* execute audits
* generate enrichment
* modify repositories
* deliver knowledge directly to AI agents

Those responsibilities belong to their respective platform components.

---

## Future Extensions

The Knowledge Resolution framework should support future capabilities, including:

* policy-based resolution
* consumer-specific package generation
* role-based knowledge composition
* semantic package optimization
* distributed package composition
* cached package reuse
* streaming package generation
* knowledge federation

Future capabilities should integrate without changing the core resolution model.

---

## Acceptance Criteria

The feature is successful when:

* only relevant knowledge is included in Knowledge Packages
* repository boundaries remain preserved
* dependency relationships are correctly resolved
* packages remain deterministic and reproducible
* package composition scales to large workspaces
* consumers receive complete engineering context with minimal unnecessary information

---

## Traceability

This feature derives from the following Vision commitments:

* **Documentation is the source of truth.**
* **Knowledge is compiled before delivery.**
* **Only relevant knowledge should be delivered to consumers.**
* **Repository relationships are explicit and deterministic.**
* **Knowledge Packages are reproducible generated artifacts.**

**Traceability**

Vision → Feature: Knowledge Resolution
