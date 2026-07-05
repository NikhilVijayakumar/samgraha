# Repository Registry — Feature Technical Design

This section details the Repository Registry — Feature Technical Design.

## Purpose

This document describes the architectural realization of the Repository Registry feature.

The Repository Registry is the authoritative catalog of repositories participating in the Sagraha platform. It owns repository lifecycle — registration, discovery, metadata synchronization, and dependency resolution — while never storing or accessing engineering knowledge.

The Repository Registry operates on the metadata track, separate from the knowledge track. It reads only Repository Manifests, never opens `knowledge.db`, and is never in the runtime query path.

This document applies the architectural principles defined in Component Model, Knowledge Flow, Runtime Boundary, Workspace Architecture, and Persistence Architecture.

---

## Feature Specification

- **Feature:** docs/raw/feature/repository-registry.md
- **Architecture:** docs/raw/architecture/repository-registry.md, docs/raw/architecture/component-model.md, docs/raw/architecture/knowledge-flow.md, docs/raw/architecture/runtime-boundary.md, docs/raw/architecture/workspace.md, docs/raw/architecture/persistence.md

---

## Participating Components

This section details the Participating Components.

### Repository Registry

The Repository Registry owns repository lifecycle management. It maintains an authoritative catalog of registered repositories, their manifests, dependency graphs, and synchronization history.

### Registry Client

Registry Client is the programmatic interface through which all components interact with the Repository Registry. The `RegistryClient` trait defines the contract — register, unregister, sync, discover, query. `FileRegistryClient` is the default SQLite-backed implementation.

### Metadata Cache

Metadata Cache stores per-dependency repository metadata in a local SQLite database (`.samgraha/registry.db`, `repository_cache` table). The cache is written during compilation and sync operations, read during knowledge resolution. TTL enforcement prevents silent staleness.

### Knowledge Resolver

The Knowledge Resolver reads from the Metadata Cache to locate repositories and resolve dependencies during knowledge composition. It never contacts the Repository Registry at runtime.

### Knowledge Compiler

The Knowledge Compiler produces the Repository Manifest as a second explicit output alongside `knowledge.db`. The manifest is written only on successful compilation and contains repository metadata for synchronization.

### CLI

The CLI exposes repository management through `samgraha registry *` subcommands — register, unregister, sync, refresh, status, list, resolve.

### MCP Runtime

The MCP Runtime exposes registry operations through MCP tools alongside knowledge operations. Single adapter, dual routing.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Repository Registry | Maintain authoritative catalog; own repository lifecycle, dependency graph, sync history |
| Registry Client | Provide programmatic interface; abstract storage backend via trait |
| FileRegistryClient | SQLite-backed implementation; CRUD for manifests, UUID lookup, dependency queries |
| Metadata Cache | Store per-dependency metadata in SQLite `repository_cache` table; enforce TTL |
| Knowledge Compiler | Produce manifest.json as second compiler output |
| Knowledge Resolver | Read metadata cache for dependency resolution; never contact Registry at runtime |
| CLI | Surface registry operations as CLI subcommands |
| MCP Runtime | Surface registry operations as MCP tools |

---

## Component Interactions

```text
                          Knowledge Compiler
                                │
                          (compilation success)
                                │
                    ┌───────────┴───────────┐
                    │                       │
                    ▼                       ▼
          knowledge.db              manifest.json
          (Knowledge Track)         (Metadata Track)
                                        │
                              (explicit sync)
                                        │
                                        ▼
                              Registry Client
                                │       │
                            ┌────┘       └────┐
                            ▼                  ▼
                  FileRegistryClient    (future HttpRegistryClient)
                  (.samgraha/registry.db)
                             │
                             ▼
                    Knowledge Resolver
                            │
                            ▼
                  Knowledge Package
```

### Registration Flow

1. User invokes `samgraha registry register` (CLI) or `register_repository` (MCP), or `samgraha init` completes initialization.
2. Caller supplies the repository's `manifest.json` (CLI reads it from `.samgraha/`; MCP callers pass it as a JSON string parameter).
3. Caller calls `RegistryClient::register(&manifest)`.
4. `FileRegistryClient` validates: UUID is unique, `knowledge_db` path is inside declared `root_path`.
5. Registry writes repository entry to `repository_cache` table.
6. **MCP only:** the adapter checks whether `knowledge.db` exists at `repository_root`/`knowledge.location`. If missing, it immediately compiles that repository (`CompilationService::execute` against the target repo's own `.samgraha/knowledge.db`) before returning. This is the one case where compilation is triggered implicitly rather than by an explicit `compile` call — it exists so a manifest can be registered ahead of a first compile without a separate round trip. The CLI path does not do this; `samgraha registry register` still assumes the manifest's `knowledge.db` already exists.
7. Registry returns success with repository identity (MCP response includes `auto_compiled: bool`).

### Synchronization Flow

1. User invokes `samgraha registry sync` (explicit, always available), or a successful `compile` (CLI or MCP) triggers it automatically when `[resolver].auto_refresh` is `true` (default) — see docs/raw/feature-technical/cli-interface.md § Dependency/Interest Auto-Registration.
2. Caller reads `manifest.json` from each dependency declared in `[[repository.dependencies]]` (this backs both `knowledge.dependencies` and `knowledge.interests` name resolution).
3. Caller calls `RegistryClient::sync(&config)`.
4. `FileRegistryClient` validates UUID consistency: stored UUID must match manifest UUID. Mismatch is an error (manual `sync`) or a logged warning, non-fatal (automatic post-compile sync).
5. Registry updates repository entry — revision, audit status, exports, capabilities, dependencies.
6. Registry updates synchronization timestamp.
7. Metadata Cache is updated — upserts `repository_cache` table entries for each dependency with new revision and TTL.

### Resolution Flow (Runtime)

1. Knowledge Resolver receives a resolution request.
2. Resolver reads workspace configuration to determine the current repository and its declared dependencies.
3. For each dependency, Resolver queries the Metadata Cache by dependency ID.
4. Cache hit: verify TTL (`expires` field against current time). If expired, report `STALE_METADATA` status but use cached data (graceful degradation).
5. Cache miss: report missing dependency, continue with available repositories (no Registry contact).
6. Cache hit (valid): read `knowledge_db` path from `repository_cache` table.
7. Resolver opens `knowledge.db` at the resolved path for each available dependency.
8. Resolver composes the Knowledge Package.
9. Registry is never contacted during this flow.

---

## Runtime Behavior

This section details the Runtime Behavior.

### Registry Lifecycle

```
Initialize Registry Client
        │
        ▼
Open or Create SQLite Store
        │
        ▼
Run Registry Migrations (REG_V1: repository_cache table)
        │
        ▼
Ready for Operations
        │
        ├── Register Repository
        ├── Unregister Repository
        ├── Synchronize Manifest
        ├── Discover Repositories
        ├── Query Metadata
        └── List Registered
```

### Repository Status Computation

Status is computed on demand, never persisted.

```
RepositoryStatus::compute(metadata, now)
        │
        ├── path exists?        No  → Missing
        ├── path accessible?    No  → Unavailable
        ├── expires elapsed?    Yes → StaleMetadata
        ├── audit FAIL/ERROR?   Yes → AuditFailed
        ├── revision mismatch?  Yes → StaleKnowledge
        │                            (cached vs manifest.json at repo root)
        └── all clear?               → Registered
```

`StaleKnowledge` requires reading `manifest.json` from the dependency's `repository_root` and comparing its revision against the cached revision. This is a filesystem operation, not a Registry query.

### Cycle Detection

Dependency cycles are detected using depth-first search with a visited set and recursion stack:

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

When a cycle is detected, resolution aborts immediately. No partial package is produced. The error reports the full cycle path:

```
error: Dependency cycle detected
  astra → prana → tantra → astra

  Suggestion: Remove one of the following dependencies to break the cycle:
    - tantra.dependencies in samgraha.toml: remove "astra"
    - astra.dependencies in samgraha.toml: remove "prana"
```

---

## Communication Paths

This section details the Communication Paths.

### Knowledge Compiler → Repository Registry (via Registry Client)

The compiler writes manifest.json. Post-compile, `auto_refresh` triggers a local Registry update. The compiler does not call `RegistryClient` directly — routing is through a post-compile hook in the compilation pipeline.

### CLI → Repository Registry (via Registry Client)

The `samgraha registry *` commands call `RegistryClient` methods directly. Each subcommand maps to one or two trait method calls.

### MCP → Repository Registry (via Registry Client)

The `McpAdapter` holds `Arc<dyn RegistryClient>`. Registry MCP tools call trait methods through this reference. Registry operations are routed to the client; knowledge operations are routed to the `KnowledgeRuntime`.

### Knowledge Resolver → Metadata Cache

The Resolver reads `repository_cache` entries from `.samgraha/registry.db`. It never calls `RegistryClient` at runtime. Cache writes happen only during sync operations (compile or explicit).

---

## Data Ownership

| Data | Owner | Registry Access |
|---|---|---|
| Repository Identity (UUID, id, name) | Repository | Read/write via sync |
| Repository Manifest | Knowledge Compiler | Read during sync |
| Repository Revision | Knowledge Compiler | Cached from manifest |
| Dependency Graph | Repository Registry | Authoritative |
| Sync History | Repository Registry | Write |
| Metadata Cache | Metadata Cache | Read by Resolver |
| Compiled Knowledge (`knowledge.db`) | Repository | Never accessed |
| Documentation | Repository | Never accessed |

---

## Integration Points

This section details the Integration Points.

### Knowledge Compiler

The compiler produces the manifest as a second output. Post-compile, if `auto_refresh = true`, the manifest is synced to the local Registry. The compiler never reads Registry data.

### Knowledge Resolver

The Resolver is the primary consumer of cached metadata. It reads `repository_cache` entries from `.samgraha/registry.db` to locate dependency `knowledge.db` files. The Resolver never contacts the Registry at runtime.

### CLI

The CLI provides the user-facing interface for registry operations. `samgraha registry *` commands map to `RegistryClient` methods.

### MCP Runtime

The MCP Runtime provides dual routing — knowledge operations through `KnowledgeRuntime`, registry operations through `RegistryClient`. Single adapter pattern. `register_repository` additionally calls into the Knowledge Compiler directly (via the same `compile_external` path used by the `compile` tool) when the manifest's declared `knowledge.db` is absent — the only registry-triggered compilation in the system.

### Metadata Cache

The cache bridges the Registry and the Resolver. It is written during sync, read during resolution. SQLite-backed for local operation, optional in-memory overlay for long-running MCP server.

---

## Key Algorithms

This section details the Key Algorithms.

### Dependency Resolution

1. Start with the current repository's declared dependencies (from `samgraha.toml`).
2. For each dependency, look up `id → CachedRepoMetadata` in the Metadata Cache.
3. Resolve each dependency's own dependencies recursively (transitive resolution).
4. Track resolved set to avoid duplicate resolution.
5. Detect cycles via DFS visited set + recursion stack.
6. Return resolved dependency list with metadata (root path, `knowledge.db` path, revision, exports).

### StaleKnowledge Detection

```
fn is_stale_knowledge(cached: &CachedRepoMetadata) -> bool {
    let manifest_path = Path::new(&cached.repository_root).join(".samgraha/manifest.json");
    let manifest_content = std::fs::read_to_string(manifest_path).ok()?;
    let manifest: RepositoryManifest = serde_json::from_str(&manifest_content).ok()?;
    Some(manifest.revision != cached.revision)
}
```

This requires filesystem access to the dependency's repository root. It is not a Registry query.

---

## External Dependency Integration

The Repository Registry operates on local SQLite storage for Phase 1-5. No external services, databases, or network calls are required.

The `RegistryClient` trait defines the extension point for future remote implementations. `FileRegistryClient` is the only Phase 1-5 implementation.

Future `HttpRegistryClient` (Phase 7+) would live in `crates/providers/` per dependency isolation requirements.

---

## Runtime Constraints

- Registry operations must complete within 100ms for single-repository operations.
- Discovery queries across 50+ repositories must complete within 500ms.
- Synchronization must not block compilation.
- Metadata Cache reads must complete within 10ms per dependency.
- Resolution must never contact the Registry — cache hit provides metadata.
- Cache miss must degrade gracefully (report missing dependency, continue).
- Registry must support concurrent readers.
- Registry must support transactional writes.

---

## Architectural Constraints

- Registry reads only Repository Manifests — never opens `knowledge.db`.
- Registry is a compile-time and synchronization resource only.
- Registry is never in the runtime query path.
- Repository metadata is disposable — always refreshable from manifests.
- Registry preserves two-track separation (metadata track never intersects knowledge track).
- Registry maintains stable repository identity via UUID.

---

## Security Considerations

- UUID consistency is validated on every `sync()` and `register()` — stored UUID must match manifest UUID.
- Path validation prevents traversal attacks — `validate_path()` ensures `knowledge_db` paths are within declared `repository_root`.
- Registry never accesses engineering knowledge — no knowledge exposure risk.
- Remote registry implementations (future) require explicit configuration and `registry_url` setting.
- Repository metadata is non-sensitive (identity, revision, capabilities, exports).

---

## Performance Considerations

- SQLite indexes on `uuid` and `id` columns ensure fast lookups.
- Metadata Cache reads are file-based — no database overhead for resolution-time queries.
- Repository status is computed on demand — no persisted state to maintain.
- Cycle detection runs in O(V + E) time.
- Metadata Cache TTL enforcement is a single field comparison.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Manifest not found | Report error, suggest compilation first |
| UUID mismatch during sync | Report error with stored vs manifest UUID, reject sync |
| SQLite store corruption | Report integrity failure, suggest reinitialization |
| Metadata Cache miss at resolution | Report missing dependency, continue with available |
| Expired metadata | Use stale metadata (graceful degradation), report stale status |
| Path validation failure | Report invalid path, reject registration or sync |
| Concurrent write conflict | SQLite serializes, retry is caller responsibility |
| `knowledge.db` missing at MCP `register_repository` | Auto-compile the target repository before returning; registration fails if that compile fails |

---

## Extension Points

This section details the Extension Points.

### Registry Backends

The `RegistryClient` trait is the extension point for storage backends. `FileRegistryClient` is the default. Future backends implement the same trait.

### Repository Locations

Future location types (Git, HTTP, Cloud, Package Registry) integrate through the Knowledge Resolver, not the Registry. The `RegistryClient` contract remains unchanged.

### Metadata Cache Overlay

Phase 7+ may add an optional in-memory overlay for long-running MCP server processes. The file remains the source of truth; the overlay caches parsed structs with TTL invalidation.

---

## Traceability

This document derives from:

- Feature: Repository Registry
- Architecture: Repository Registry Architecture
- Architecture: Component Model
- Architecture: Knowledge Flow
- Architecture: Runtime Boundary
- Architecture: Workspace Architecture
- Architecture: Persistence Architecture

This document provides technical context for:

- Engineering Repository Registry Strategy
- Knowledge Resolution Technical Design
- MCP Runtime Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
