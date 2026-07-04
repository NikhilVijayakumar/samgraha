# Context Manager — Feature Technical Design

This section details the Context Manager — Feature Technical Design.

## Purpose

This document describes the architectural realization of the Context Manager feature.

The Context Manager owns the Knowledge Context lifecycle within the Saṃgraha runtime. It decouples context lifetime from MCP connection lifetime, enabling context reuse across reconnects, TTL-based expiry, and transparent rebuild on revision change. The Context Manager is the single point of authority for when a Knowledge Context is created, reused, rebuilt, or disposed.

This document applies the architectural principles defined in Component Model and Runtime Boundary.

---

## Feature Specification

- **Feature:** docs/raw/feature/context-manager.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/runtime-boundary.md

---

## Participating Components

This section details the Participating Components.

### Context Manager

ContextManager owns the Knowledge Context lifecycle, tracks connection count, manages TTL-based expiry, and performs revision-based invalidation. It is the sole authority for context transitions (Active ↔ Inactive ↔ Disposed). McpAdapter holds a ContextManager instance and delegates all context operations to it.

### Knowledge Context

The managed object. Holds the assembled Knowledge Package, the Knowledge Plan, assembly time, and TTL configuration. Was formerly named KnowledgeSession. Its lifetime is independent of MCP connections — it is created once and survives client disconnects until TTL expiry or revision change triggers a rebuild.

### MCP Adapter

Consumer of ContextManager. Calls `context_manager.active()` to obtain the current Knowledge Context for every request. Calls `on_connect()` on client connect and `on_disconnect()` on client disconnect. McpAdapter carries no context lifecycle logic of its own.

### Knowledge Planner

Invoked by ContextManager during context creation and rebuild. Reads `samgraha.toml [knowledge]` and `.meta` files to produce a deterministic Knowledge Plan. The Planner is called once per context lifetime, not per-request.

### Runtime Package

Assembled by ContextManager via the Resolver during context creation and rebuild. Contains a `Vec<(KnowledgePlanEntry, Arc<RegistryStore>)>` — the materialized mapping from plan entries to their backing registry stores. RuntimePackage is owned by KnowledgeContext.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Context Manager | Own context lifecycle; track connection count; manage TTL; trigger revision checks; invoke Planner + Resolver on create/rebuild |
| Knowledge Context | Hold assembled Knowledge Package and Knowledge Plan; report validity via `is_valid()`; carry assembly_time and ttl_secs |
| MCP Adapter | Call on_connect() / on_disconnect(); call active() to obtain context for request serving; carry no lifecycle logic |
| Knowledge Planner | Produce deterministic Knowledge Plan from config + .meta files; called once per context creation or rebuild |
| Runtime Package | Materialize plan entries to RegistryStore handles; owned by KnowledgeContext; read-only during serving |

---

## Component Interactions

```text
McpAdapter
    │
    ├── on client connect  → ContextManager.on_connect()
    ├── on client disconnect → ContextManager.on_disconnect()
    ├── on each request    → ContextManager.active() → &KnowledgeContext
    └── on background tick → ContextManager.dispose_if_expired()

ContextManager
    │
    ├── create/rebuild → Planner::plan(root, config) → KnowledgePlan
    │                  → RuntimePackage::from_plan(&plan) → RuntimePackage
    │                  → KnowledgeContext { package, plan, assembly_time, ttl_secs }
    │
    └── active() → Option<&KnowledgeContext>
```

### Request Flow

1. MCP client connects — McpAdapter calls `context_manager.on_connect()`.
2. ContextManager checks connection count and context state:
   - If first connection and context is Inactive: perform revision check via `is_valid()`.
     - Valid (revision unchanged, within TTL): transition to Active, reuse context.
     - Invalid: run Planner + Resolver, rebuild context, transition to Active.
   - If connection count > 0 (already Active): increment count, no rebuild needed.
3. McpAdapter calls `context_manager.active()` — receives `&KnowledgeContext`.
4. McpAdapter serves request from context's Knowledge Package.
5. MCP client disconnects — McpAdapter calls `context_manager.on_disconnect()`.
6. ContextManager decrements connection count.
   - If count reaches 0: transition to Inactive, record `inactive_since`.
   - TTL countdown begins.
7. Background or next-connect call to `dispose_if_expired()`: if TTL elapsed while Inactive, drop context.

---

## Runtime Behavior

### Runtime Lifecycle

```
ContextManager created (process start)
        │
        ▼
Create KnowledgeContext
  Planner::plan(root, config) → KnowledgePlan
  RuntimePackage::from_plan(&plan) → RuntimePackage
  KnowledgeContext { package, plan, assembly_time, ttl_secs }
  connection_count = 0, inactive_since = None
        │
        ▼
State: Inactive (no clients yet)
        │
MCP/CLI connects → on_connect()
        │
        ├── connection_count == 0 and inactive:
        │     run is_valid(plan) check
        │     ├── valid (revision unchanged, within TTL) → reuse → Active
        │     └── invalid → rebuild → Active
        │
        └── connection_count > 0 → Active (increment count)
        │
MCP/CLI disconnects → on_disconnect()
        │
        ├── connection_count > 0 → Active (decrement count)
        └── connection_count == 0 → Inactive
              inactive_since = Instant::now()
              TTL countdown begins
        │
        ├── Next connect (within TTL) → on_connect() → revision check → reuse or rebuild
        └── TTL expires (dispose_if_expired call) → context dropped, handles closed
```

### Phase 8 Struct

```rust
pub struct ContextManager {
    context: Option<KnowledgeContext>,
    connection_count: usize,
    inactive_since: Option<Instant>,
    knowledge_ttl: Duration,
    root: PathBuf,
    config: SamgrahaConfig,
}
```

### Phase 9 Struct (future — multi-context)

```rust
pub struct ContextManager {
    contexts: HashMap<String, KnowledgeContext>,
    active_name: Option<String>,
    // connection_count per context, TTL per context
}
```

Phase 9 extends ContextManager to hold one context per named workspace. The interface (`active()`, `on_connect()`, `on_disconnect()`, `dispose_if_expired()`) remains stable across the transition; callers need no changes beyond passing a workspace name.

---

## Communication Paths

### McpAdapter → ContextManager

McpAdapter is the sole caller of ContextManager's lifecycle interface. All calls are in-process, synchronous (Phase 8). No IPC or network boundary.

### ContextManager → Knowledge Planner

ContextManager invokes `Planner::plan(root, config)` during context creation and rebuild. The Planner reads `samgraha.toml` and `.meta` files from disk. This is the only I/O-bound step in the lifecycle.

### ContextManager → Resolver / RuntimePackage

ContextManager invokes `RuntimePackage::from_plan(&plan)` to open RegistryStore handles per plan entry. Each `Arc<RegistryStore>` is retained for the context lifetime.

### ContextManager → KnowledgeContext

ContextManager creates, owns, and drops KnowledgeContext. It calls `context.is_valid()` during reconnect to decide reuse vs. rebuild.

---

## Data Ownership

| Data | Owner | Access |
|---|---|---|
| KnowledgeContext | ContextManager | Create / drop |
| KnowledgePlan | KnowledgeContext (via ContextManager) | Read (serving), write (creation) |
| RuntimePackage | KnowledgeContext | Read (serving), write (creation) |
| connection_count | ContextManager | Read/Write |
| inactive_since | ContextManager | Read/Write |
| knowledge_ttl | ContextManager (from config) | Read |
| Root path + config | ContextManager (from McpAdapter init) | Read |

---

## Integration Points

### McpAdapter

McpAdapter is the primary integration point. It holds `ContextManager` as a field (replacing the former `session: Option<KnowledgeSession>`). All context lifecycle calls route through ContextManager.

### Knowledge Planner

ContextManager integrates with the Planner at context creation and rebuild. The Planner interface is unchanged; ContextManager is a new caller.

### Knowledge Context (was KnowledgeSession)

KnowledgeContext is the output of ContextManager's create/rebuild operations. Its `is_valid()` method is the contract ContextManager depends on for reuse decisions.

---

## External Dependency Integration

ContextManager has no external dependencies. All operations are in-process or disk I/O (via Planner reading `.meta` files). No network, no external services.

---

## Runtime Constraints

- ContextManager must be created before the first MCP client connects.
- ContextManager must hold at most one KnowledgeContext in Phase 8.
- ContextManager must not expose KnowledgeContext to callers when it is being rebuilt (rebuild is atomic from the caller's perspective).
- ContextManager must not drop a context while connection_count > 0.
- TTL must be configurable through `samgraha.toml`; default is sufficient for interactive use.

---

## Architectural Constraints

- ContextManager must not contain request-serving logic. It provides access to the context; services serve from it.
- ContextManager must not access the Knowledge Registry directly. Registry access belongs to RuntimePackage / RegistryStore.
- ContextManager must not be bypassed. All context access routes through `active()`.
- The lifecycle interface (`on_connect`, `on_disconnect`, `active`, `dispose_if_expired`) must remain stable across Phase 8 → Phase 9 transition.

---

## Security Considerations

- ContextManager does not authenticate or authorize callers. Trust boundaries are enforced at the transport adapter layer.
- KnowledgeContext is never exposed outside the process. All access is in-process via `active()`.
- Rebuild on revision change prevents stale knowledge from being served after a compile cycle.

---

## Performance Considerations

- `on_connect()` must complete within 10ms for the reuse path (revision check only — no Planner + Resolver run).
- Cold-start context creation (Planner + Resolver) must complete within 2 seconds for a typical workspace of 10 repositories.
- `active()` must complete within 1ms — it is called on every request.
- `dispose_if_expired()` is called on a background interval or lazily on connect; it must complete within 1ms (no I/O — just a TTL comparison and Option drop).
- Cold-start cost amortizes across the full context lifetime. For a 30-minute TTL at 100 requests/minute, the 2-second startup is ~0.1% of total serving time.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Context creation fails at startup | Log error; MCP adapter starts but `active()` returns None; callers receive a single-repo fallback from runtime.registry |
| Rebuild fails on reconnect | Log error; if existing context is still within TTL, retain it and serve from it; if TTL expired, return unavailable until next rebuild attempt |
| Planner returns empty plan | Log warning; create context with empty RuntimePackage; serve no-result responses (not an error condition) |
| RegistryStore open fails for one entry | Log warning; skip that entry; continue with remaining entries in RuntimePackage |
| dispose_if_expired called while rebuilding | No-op; rebuild lock prevents double disposal |
| connection_count underflow | Clamp to 0, log error; prevents premature context disposal |

---

## Extension Points

### Multi-Context (Phase 9)

The Phase 9 `HashMap<String, KnowledgeContext>` extension requires only struct changes to ContextManager. The `active()` interface gains a workspace name parameter; all other lifecycle methods follow the same pattern per context.

### Pluggable Rebuild Strategy

The rebuild trigger (revision change) is encapsulated in `is_valid()`. Future strategies (e.g., file-watch-triggered rebuild, scheduled rebuild) can replace the on-connect check without changing the context lifecycle state machine.

---

## Traceability

This document derives from:

- Feature: Context Manager
- Architecture: Component Model
- Architecture: Runtime Boundary

This document provides technical context for:

- MCP Adapter Technical Design
- Knowledge Runtime Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
