# Knowledge Resolution — Feature Technical Design

## Purpose

This document describes the architectural realization of the Knowledge Resolution feature.

Knowledge Resolution assembles the engineering knowledge required by a consumer into a coherent Knowledge Package. It identifies relevant repositories, documentation domains, dependencies, and generated artifacts, composing only the knowledge necessary for the requested context.

This document applies the architectural principles defined in Component Model, Workspace Architecture, Knowledge Flow, and Communication Architecture.

---

## Feature Specification

- **Feature:** docs/raw/feature/knowledge-resolution.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/workspace.md, docs/raw/architecture/knowledge-flow.md, docs/raw/architecture/communication.md

---

## Participating Components

### Knowledge Runtime

The Knowledge Runtime owns resolution execution. Resolution is a Knowledge Service invoked during package generation or runtime context preparation.

### Knowledge Registry

The Knowledge Registry provides compiled knowledge for resolution. The resolver reads documents, metadata, and dependency information from the registry.

### Workspace Management

Workspace Management provides the workspace context — which repositories participate, their relationships, and shared configuration.

### Knowledge Package

The Knowledge Package component receives the resolved content and produces the deployable package artifact.

### Incremental Build

Incremental Build ensures the Knowledge Registry reflects current documentation. Resolution operates on up-to-date compiled knowledge.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Knowledge Runtime | Coordinate resolution, invoke resolver, enforce repository boundaries |
| Knowledge Registry | Provide compiled knowledge, metadata, and dependency information |
| Workspace Management | Define workspace scope, repository membership, dependency declarations |
| Knowledge Resolution Service | Identify relevant knowledge, resolve dependencies, compose package contents |
| Knowledge Package | Package resolved content into deployable format |

---

## Component Interactions

```text
Consumer Request
        │
        ▼
Knowledge Runtime
        │
        ▼
Workspace Management (resolve workspace context)
        │
        ▼
Knowledge Registry (read compiled knowledge)
        │
        ▼
Knowledge Resolution Service
        │
        ├── Repository Discovery
        ├── Dependency Resolution
        ├── Knowledge Composition
        └── Context Reduction
        │
        ▼
Knowledge Package (compose output)
        │
        ▼
Knowledge Runtime (return result)
```

### Resolution Request Flow

1. Consumer initiates resolution through the Knowledge Runtime.
2. Runtime identifies the active workspace and target repository.
3. Runtime invokes the Knowledge Resolution service.
4. Resolver reads workspace configuration to determine participating repositories.
5. Resolver reads repository dependency declarations.
6. Resolver queries the Knowledge Registry for compiled knowledge within scope.
7. Resolver identifies required documentation domains and artifacts.
8. Resolver excludes unrelated repositories and unnecessary artifacts (context reduction).
9. Resolver composes the resolved knowledge set.
10. Resolver validates package completeness and dependency consistency.
11. Resolver passes the composed content to Knowledge Package for packaging.
12. Resolver returns package metadata to the Knowledge Runtime.

---

## Runtime Behavior

### Resolution Lifecycle

```
Initialize Resolution
        │
        ▼
Resolve Workspace
        │
        ▼
Discover Repositories
        │
        ▼
Resolve Dependencies
        │
        ▼
Compose Knowledge
        │
        ▼
Reduce Context
        │
        ▼
Validate Package
        │
        ▼
Return Resolved Content
```

### Determinism

Identical workspace configuration and registry content produce identical resolution output. Resolution depends only on declared configuration and compiled knowledge.

---

## Communication Paths

### Knowledge Runtime → Workspace Management

The runtime queries workspace membership, repository configuration, and dependency declarations.

### Knowledge Runtime → Knowledge Registry

The runtime queries compiled knowledge within the resolved scope. Registry queries are filtered by repository and domain.

### Knowledge Resolution → Knowledge Package

The resolver passes composed knowledge to the packaging service. The package service produces the final deployable artifact.

---

## Data Ownership

| Data | Owner | Resolver Access |
|---|---|---|
| Workspace Configuration | Workspace Management | Read |
| Repository Configuration | Repository | Read |
| Compiled Knowledge | Knowledge Registry | Read |
| Dependency Declarations | Repository | Read |
| Resolution Context | Knowledge Runtime | Transient |
| Composed Knowledge | Knowledge Package | Transient |

---

## Integration Points

### Workspace Management

Resolution depends on workspace configuration to determine repository scope and dependencies.

### Knowledge Registry

Resolution reads all compiled knowledge from the registry. Registry integrity directly affects resolution correctness.

### Knowledge Package

Resolution produces composed knowledge for the packaging layer. The Knowledge Package component handles serialization and distribution.

### CLI Interface

The CLI may trigger resolution through compile or info commands. Resolution context is derived from the current workspace.

---

## External Dependency Integration

Resolution operates entirely offline. No external services participate in knowledge resolution.

Optional: Remote repository registries may be supported in the future for distributed resolution. Core resolution remains local-first.

---

## Runtime Constraints

- Resolution must complete without network access.
- Resolution must support large workspaces (100+ repositories).
- Resolution must exclude unrelated repositories and artifacts.
- Resolution must preserve repository isolation.
- Resolution must resolve transitive dependencies correctly.
- Resolution must detect dependency cycles and report them.

---

## Architectural Constraints

- Resolution must not modify the Knowledge Registry.
- Resolution must not access repositories outside the declared workspace.
- Resolution must preserve repository ownership in composed packages.
- Resolution must not introduce knowledge not present in the registry.

---

## Security Considerations

- Repository boundaries are enforced — resolution never accesses unrelated repositories.
- Dependency visibility is controlled by declared configuration.
- Workspace membership does not imply unrestricted repository access.
- Package composition preserves audit metadata for integrity verification.

---

## Performance Considerations

- Workspace resolution must complete within 1 second for moderate workspaces (10 repositories).
- Dependency resolution must detect cycles efficiently.
- Context reduction minimizes package size by excluding unnecessary artifacts.
- Registry queries are batched to minimize read operations.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Missing dependency | Report unresolved dependency, abort resolution |
| Dependency cycle | Report cycle with participating repositories, abort resolution |
| Registry unavailable | Report error, retry after registry initialization |
| Configuration error | Report error with configuration path, abort resolution |
| Workspace not found | Report error, suggest initialization |

---

## Extension Points

### Resolution Profiles

Custom resolution profiles may define different composition strategies (minimal, development, production, full knowledge).

### Dependency Resolvers

Alternative dependency resolution strategies may be registered for specific workspace types or repository relationships.

### Context Reducers

Custom context reduction logic may optimize package composition for specific consumer profiles.

---

## Traceability

This document derives from:

- Feature: Knowledge Resolution
- Architecture: Component Model
- Architecture: Workspace Architecture
- Architecture: Knowledge Flow
- Architecture: Communication Architecture

This document provides technical context for:

- Engineering Resolution Strategy
- Knowledge Package Technical Design
- MCP Runtime Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
