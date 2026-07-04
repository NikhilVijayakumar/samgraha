# Knowledge Resolution — Feature Technical Design

This section details the Knowledge Resolution — Feature Technical Design.

## Purpose

This document describes the architectural realization of the Knowledge Resolution feature.

Knowledge Resolution assembles the engineering knowledge required by a consumer into a coherent Knowledge Package. It identifies relevant repositories, documentation domains, dependencies, and generated artifacts, composing only the knowledge necessary for the requested context.

This document applies the architectural principles defined in Component Model, Workspace Architecture, Knowledge Flow, and Communication Architecture.

---

## Feature Specification

- **Feature:** docs/raw/feature/knowledge-resolution.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/workspace.md, docs/raw/architecture/knowledge-flow.md, docs/raw/architecture/runtime-boundary.md, docs/raw/architecture/communication.md

---

## Participating Components

This section details the Participating Components.

### Knowledge Planner

The Knowledge Planner produces the Knowledge Plan — the ordered list of `knowledge.db` paths to open. The Planner is deterministic: it reads `samgraha.toml [knowledge]` config + `.meta` files + the current repository manifest. The Planner takes no query context. The Planner is invoked before the Resolver opens any databases.

### Knowledge Runtime

The Knowledge Runtime owns resolution execution. Resolution is a Knowledge Service invoked during Knowledge Context creation (once per context lifetime, reused across reconnects).

### Knowledge Registry

The Knowledge Registry provides compiled knowledge for resolution. The resolver reads documents, metadata, and dependency information from the registry.

### Metadata Cache

Metadata Cache provides per-dependency repository metadata during resolution. The resolver reads `.meta.json` files from `.samgraha/dependencies/` to locate `knowledge.db` files and validate freshness. The cache is written during sync operations, never at runtime.

### Workspace Management

Workspace Management provides the workspace context — which repositories participate, their relationships, and shared configuration.

### Knowledge Package

The Knowledge Package component receives the resolved content and produces the deployable package artifact.

### Incremental Build

Incremental Build ensures the Knowledge Registry reflects current documentation. Resolution operates on up-to-date compiled knowledge.

### Repository Registry

The Repository Registry manages repository lifecycle and synchronization. It is never contacted during resolution — the Metadata Cache bridges the gap between compile-time registry updates and runtime resolution.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Knowledge Planner | Produce deterministic Knowledge Plan from config + .meta + manifest | Knowledge Plan (ordered Vec<PathBuf>) |
| Knowledge Runtime | Coordinate resolution, invoke resolver, enforce repository boundaries |
| Knowledge Registry | Provide compiled knowledge, metadata, and dependency information |
| Metadata Cache | Provide per-dependency repository metadata, enforce TTL |
| Workspace Management | Define workspace scope, repository membership, dependency declarations |
| Knowledge Resolution Service | Open planned stores, resolve dependencies, compose package contents |
| Knowledge Package | Package resolved content into deployable format |
| Repository Registry | Synchronize repository metadata (compile-time only, never at runtime) |

---

## Component Interactions

```text
Consumer Request
        │
        ▼
Knowledge Runtime
        │
        ▼
Knowledge Planner (read config + .meta + manifest → Knowledge Plan)
        │
        ▼
Workspace Management (resolve workspace context)
        │
        ▼
Metadata Cache (lookup dependency metadata, enforce TTL)
        │
        ├── hit  → Knowledge Registry (read compiled knowledge)
        └── miss → Degrade gracefully, report missing dependency
        │
        ▼
Knowledge Resolution Service (open only planned knowledge.db files)
        │
        ├── Dependency Resolution (with cycle detection via DFS)
        ├── Repository Location (path resolution from cached metadata)
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

1. ContextManager creates a Knowledge Context (triggered at startup or on reconnect after rebuild).
2. Runtime identifies the active workspace and target repository.
3. Runtime invokes the Knowledge Planner: reads `samgraha.toml [knowledge]` + `.meta` files + current manifest → produces ordered Knowledge Plan (which repos to open in which priority order). No query context used.
3.5. Runtime invokes the Knowledge Resolution service with the Knowledge Plan.
4. Resolver opens only the planned `knowledge.db` files from the Knowledge Plan. Resolver reads workspace configuration to determine participating repositories and their dependencies.
5. Resolver queries the Metadata Cache for each declared dependency by ID:
   - Cache hit with valid TTL: read `knowledge_db` path, revision, exports.
   - Cache hit with expired TTL: use cached metadata (graceful degradation), report stale status.
   - Cache miss: report missing dependency, continue with available repositories.
6. Resolver runs dependency resolution with DFS cycle detection:
   - Track visited set and recursion stack per dependency.
   - If cycle detected, abort immediately with full cycle path error.
   - No partial package produced on cycle detection.
7. Resolver opens each resolved dependency's `knowledge.db` at the path from cached metadata.
8. Resolver reads compiled knowledge from the Knowledge Registry for each available repository.
9. Resolver identifies required documentation domains and artifacts.
10. Resolver excludes unrelated repositories and unnecessary artifacts (context reduction).
11. Resolver composes the resolved knowledge set.
12. Resolver validates package completeness and dependency consistency.
13. Resolver passes the composed content to Knowledge Package for packaging.
14. Resolver returns package metadata to the Knowledge Runtime.

---

## Runtime Behavior

This section details the Runtime Behavior.

### Resolution Lifecycle

```
Initialize Resolution
        │
        ▼
Resolve Workspace Context
        │
        ▼
Read Declared Dependencies
        │
        ▼
Query Metadata Cache
        │
        ├── Hit (valid TTL) → Extract knowledge_db paths
        ├── Hit (expired)   → Use stale, report STALE_METADATA status
        └── Miss            → Report missing, continue with available
        │
        ▼
Resolve Dependencies (DFS, detect cycles)
        │
        ▼
Open knowledge.db Files (read-only)
        │
        ▼
Read Compiled Knowledge
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

Identical workspace configuration, registry content, and metadata cache state produce identical resolution output. Resolution depends only on declared configuration, compiled knowledge, and cached metadata.

### Cycle Detection

The resolver detects dependency cycles using depth-first search:

```
initialize visited: Set<String>
initialize stack: Vec<String>

for each dependency:
    if not visited:
        dfs(dependency)

fn dfs(node):
    if node in stack:
        extract cycle from stack → abort with full cycle path
    if node in visited:
        return
    add node to visited
    push node to stack
    for each neighbor of node:
        dfs(neighbor)
    pop node from stack
```

On cycle detection, resolution aborts immediately. No partial package is produced.

---

## Communication Paths

This section details the Communication Paths.

### Knowledge Planner → Knowledge Resolution Service

The Planner passes the Knowledge Plan (ordered list of `knowledge.db` paths with priorities) to the Resolver. The Resolver opens only the planned stores. The Resolver never queries the Registry.

### Knowledge Runtime → Workspace Management

The runtime queries workspace membership, repository configuration, and dependency declarations.

### Knowledge Runtime → Knowledge Registry

The runtime queries compiled knowledge within the resolved scope. Registry queries are filtered by repository and domain.

### Knowledge Runtime → Metadata Cache

The runtime queries the Metadata Cache for each declared dependency. Cache reads are file-based — read `.meta.json` from `.samgraha/dependencies/{id}.meta.json`. No database queries at resolution time.

### Knowledge Runtime → Repository Registry

The Knowledge Resolver never contacts the Repository Registry at runtime. All required metadata is served from the Metadata Cache.

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
| Dependency Metadata (cached) | Metadata Cache | Read |
| Repository Manifest | Knowledge Compiler | Read during cache refresh |
| Resolution Context | Knowledge Runtime | Transient |
| Composed Knowledge | Knowledge Package | Transient |

---

## Integration Points

This section details the Integration Points.

### Workspace Management

Resolution depends on workspace configuration to determine repository scope and dependencies.

### Knowledge Registry

Resolution reads all compiled knowledge from the registry. Registry integrity directly affects resolution correctness.

### Metadata Cache

Resolution reads dependency metadata from the cache. Cache freshness is enforced by TTL comparison during resolution. Cache writes happen during compilation sync (auto_refresh) or explicit `samgraha registry sync`.

### Repository Registry

The Repository Registry is not integrated at runtime. It is a compile-time and synchronization integration only. Resolution integration with the Registry is indirect — through the Metadata Cache.

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
| Metadata cache miss | Report missing dependency, degrade gracefully — resolution continues with available repositories |
| Metadata cache expired | Report `STALE_METADATA`, use stale metadata if auto_refresh fails — resolution continues |
| Dependency cycle | Report cycle with full path, abort resolution — no partial package produced |
| Configuration error | Report error with configuration path, abort resolution |
| Workspace not found | Report error, suggest initialization |
| `knowledge.db` inaccessible | Report error for that repository, degrade gracefully — continue with remaining repositories |

---

## Extension Points

This section details the Extension Points.

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
- Architecture: Runtime Boundary
- Architecture: Communication Architecture

This document provides technical context for:

- Engineering Resolution Strategy
- Knowledge Package Technical Design
- MCP Runtime Technical Design
- Repository Registry Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
