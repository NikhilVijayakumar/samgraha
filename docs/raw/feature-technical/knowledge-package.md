# Knowledge Package — Feature Technical Design

This section details the Knowledge Package — Feature Technical Design.

## Purpose

This document describes the architectural realization of the Knowledge Package feature.

A Knowledge Package is the deployable representation of compiled engineering knowledge. It encapsulates all engineering knowledge required by a consumer into a single, portable, deterministic package produced by Knowledge Resolution and consumed by the Knowledge Runtime.

This document applies the architectural principles defined in Component Model, Workspace Architecture, Persistence Architecture, Deployment Architecture, and Knowledge Flow.

---

## Feature Specification

- **Feature:** docs/raw/feature/knowledge-package.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/workspace.md, docs/raw/architecture/persistence.md, docs/raw/architecture/deployment.md, docs/raw/architecture/knowledge-flow.md

---

## Participating Components

This section details the Participating Components.

### Knowledge Package

Knowledge Package owns package composition, manifest generation, integrity metadata, and lifecycle management. It transforms resolved knowledge into deployable artifacts. It supports two layout modes: Physical (portable) and Virtual (workspace-local).

### Knowledge Resolution

Knowledge Resolution provides composed knowledge content for packaging. It determines which repositories, domains, and artifacts enter the package.

### Knowledge Registry

The Knowledge Registry provides the compiled knowledge and metadata that the package encapsulates.

### Knowledge Runtime

The Knowledge Runtime loads and serves Knowledge Packages. The runtime consumes packages as the authoritative compiled knowledge source during operation. Virtual packages must be loaded in the same workspace where they were generated.

### Incremental Build

Incremental Build ensures package contents reflect current documentation. Stale packages are invalidated and regenerated.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Knowledge Package | Compose package artifacts, generate manifests, compute integrity metadata, manage lifecycle, select layout mode |
| Knowledge Resolution | Provide composed knowledge scope and content |
| Knowledge Registry | Provide compiled knowledge for packaging |
| Knowledge Runtime | Load and serve packages during operation |
| Incremental Build | Invalidate stale packages, trigger regeneration |

---

## Component Interactions

```text
Knowledge Resolution
        │
        ▼
Knowledge Registry (read compiled knowledge)
        │
        ▼
Knowledge Package
        │
        ├── Apply Consumer Profile (filter scope + section types)
        ├── Compose Artifacts
        ├── Select Package Layout
        │   ├── Physical → Copy knowledge.db + docs/ → portable
        │   └── Virtual  → Write JSON manifest with absolute paths → local-only
        ├── Generate Manifest
        ├── Compute Integrity
        └── Validate Package
        │
        ▼
Knowledge Runtime (load and serve)
```

### Package Composition Flow

1. Knowledge Resolution determines the knowledge scope for the package.
2. Resolution reads compiled knowledge from the Knowledge Registry.
3. Resolution passes resolved content to Knowledge Package.
4. Knowledge Package applies the consumer profile to filter artifacts:
   - Document scope filter: which domains and documents to include.
   - Section type filter: which semantic types to include from each document.
   - Profile examples: AI Assistant includes `purpose`, `functional_requirements`, `business_rules`, `constraints`; excludes `future_extensions`, `traceability`.
5. Package composes the filtered artifacts — sections rather than full documents where the profile specifies section types.
6. Package selects the output layout:
   - **Physical** (default): Copies `knowledge.db` and `docs/` into the output directory. Portable across environments. Can be shared between machines or archived independently.
   - **Virtual**: Writes a `VirtualPackageManifest` JSON file with absolute paths to source `knowledge.db` files. Workspace-local only. Requires all source repositories to remain at declared paths. Not portable.
7. Package generates the package manifest describing contents, included section types, version, and metadata.
8. Package computes integrity metadata (hashes, dependency consistency, audit status).
9. Package validates completeness — manifest, artifacts, dependencies, integrity.
10. Package writes the final Knowledge Package to the output location.
11. Knowledge Runtime loads the validated package for serving.

---

## Runtime Behavior

This section details the Runtime Behavior.

### Package Lifecycle

```
Receive Resolved Content
        │
        ▼
Apply Consumer Profile
        │
        ├── Filter by domain / document scope
        └── Filter by semantic section types
        │
        ▼
Compose Package Artifacts
        │
        ├── Semantic Sections (filtered by profile)
        ├── Document Metadata
        ├── Indexes
        └── Enrichment Artifacts
        │
        ▼
Select Package Layout
        │
        ├── Physical (default):
        │     ├── Copy knowledge.db → output/
        │     ├── Copy docs/ → output/
        │     └── Write manifest → output/samgraha-package.json
        │
        └── Virtual (--layout virtual):
              ├── Resolve absolute paths to source knowledge.db files
              ├── Write VirtualPackageManifest → output/samgraha-package.json
              └── No file copies (references only)
        │
        ▼
Generate Manifest (includes layout, included_section_types)
        │
        ▼
Compute Integrity Metadata
        │
        ▼
Validate Package
        │
        ▼
Write Package
        │
        ▼
Notify Consumers
```

### Determinism

Identical resolution input produces identical packages within the same layout mode. Package generation depends only on resolved knowledge and packaging configuration.

### Repository Isolation

Packages preserve repository boundaries. Every artifact within a package remains traceable to its originating repository. Package composition never obscures repository ownership.

### Layout Selection

Layout is selected at package generation time:

- **Default:** Physical. No CLI flag required.
- **Virtual:** Explicit `--layout virtual` CLI flag.

`PackageProfile` (what knowledge to include) and `PackageLayout` (how to package it) are orthogonal axes. Profile filtering runs first. Then the layout branch decides whether to copy files (Physical) or write a manifest with references (Virtual).

```rust
pub struct PackageRequest {
    pub output_path: PathBuf,
    pub profile: PackageProfile,   // what knowledge to include
    pub layout: PackageLayout,     // how to package it
    pub repository_name: String,
}

pub enum PackageLayout {
    Physical,  // copies knowledge.db + docs/ → portable
    Virtual,   // JSON manifest with absolute paths → workspace-local only
}
```

---

## Communication Paths

This section details the Communication Paths.

### Knowledge Resolution → Knowledge Package

Resolution passes composed knowledge content and metadata to the packaging component.

### Knowledge Package → Knowledge Registry

Packaging reads compiled knowledge from the registry. It does not modify registry content.

### Knowledge Package → Knowledge Runtime

The runtime loads packages from the output location. The package is the unit of knowledge delivery.

---

## Data Ownership

| Data | Owner | Package Access |
|---|---|---|
| Compiled Knowledge | Knowledge Registry | Read |
| Resolved Content | Knowledge Resolution | Transient |
| Package Artifacts | Knowledge Package | Write |
| Package Manifest | Knowledge Package | Write |
| Integrity Metadata | Knowledge Package | Write |
| Delivery Location | Deployment | Write |

---

## Integration Points

This section details the Integration Points.

### Knowledge Resolution

Resolution determines what goes into the package. The packaging component receives pre-composed content.

### Knowledge Registry

The registry provides the compiled knowledge store. Packages are snapshots of registry content at a point in time. Virtual packages reference registry files by absolute path rather than copying them.

### Knowledge Runtime

The runtime consumes packages as the compiled knowledge source. Packages are loaded at runtime initialization.

### Incremental Build

Incremental Build invalidates packages when source documentation changes. Invalidated packages are regenerated on the next build.

---

## External Dependency Integration

Package generation operates entirely offline. No external services participate in package composition.

Optional: Future package distribution (remote registries, package servers) may introduce network integration. Core packaging remains local.

---

## Runtime Constraints

- Package generation must complete within predictable time bounds.
- Packages must support large engineering workspaces.
- Packages must remain portable across environments.
- Packages must support offline usage.
- Package validation must detect corruption before loading.
- Package regeneration must restore identical content.

---

## Architectural Constraints

- Packages are generated artifacts — never manually edited.
- Packages are disposable — always regenerable from documentation.
- Packages must not modify repository documentation.
- Packages must preserve repository ownership.
- Packages must remain deterministic.

---

## Security Considerations

- Package contents are validated before use.
- Integrity metadata detects corruption or tampering.
- Repository boundaries are preserved within packages.
- Package loading never executes code from package contents.
- Packages contain only compiled knowledge — no source paths exposed.

---

## Performance Considerations

- Package generation must scale linearly with knowledge volume.
- Package validation must complete before delivery.
- Package loading must complete within 2 seconds for typical workspaces.
- Integrity computation must not significantly increase generation time.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Invalid resolved content | Report validation error, abort packaging |
| Integrity mismatch | Report corruption, trigger regeneration |
| Storage failure | Report error, preserve prior valid package |
| Manifest generation failure | Report error, abort packaging |
| Validation failure | Report errors, prevent publication |

---

## Extension Points

This section details the Extension Points.

### Package Formats

Alternative package serialization formats may be registered (compressed, encrypted, streaming) without changing the composition model.

### Package Profiles

Consumer-specific package profiles define which artifacts and semantic section types are included. Profiles specify both document scope and section type filters. This enables section-granularity packaging: an AI Assistant profile can deliver only `purpose`, `functional_requirements`, `business_rules`, and `constraints` sections without packaging full documents. Profiles are configurable through repository configuration.

### Distribution Channels

Future distribution mechanisms (package registries, remote replication, federated delivery) integrate through the package output interface.

---

## Traceability

This document derives from:

- Feature: Knowledge Package
- Architecture: Component Model
- Architecture: Workspace Architecture
- Architecture: Persistence Architecture
- Architecture: Deployment Architecture
- Architecture: Knowledge Flow
- Architecture: Repository Registry Architecture

This document provides technical context for:

- Engineering Packaging Strategy
- Knowledge Runtime Technical Design
- Knowledge Resolution Technical Design
- Repository Registry Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
