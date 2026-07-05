# Build Separation & SamagraIgnore + Persistent Knowledge Context Feasibility Report

> **v3** — Code-reviewed against current implementation. Naming errors corrected, gaps from reality added. See [§ Code Review Findings](#code-review-findings) at end.

> **v2** — Revised to incorporate Persistent Knowledge Context, ownership model, and architectural reasoning across both proposals.

---

## Understanding

Two changes to current build+compile architecture:

**Build separation.** Stop bundling samgraha's own repo docs with release. Only three directory trees are universal standards any repo needs: `docs/raw/standards/`, `docs/raw/audit/`, `docs/raw/audit-standards/`. Samgraha itself registers as a peer repo like any other — no special status in the build artifact.

**SamagraIgnore.** Per-repo ignore file (`.samagraignore`) with `.gitignore`-style glob syntax. Users exclude drafts, private dirs, temp files from compilation. Replaces hardcoded excludes and dead `IgnoreConfig` struct.

---

## Current State (What Ships in Build Today)

```
release/samgraha/        ← EVERYTHING bundled
  bin/mcp.exe
  bin/cli.exe
  samgraha.toml
  docs/raw/
    standards/           ← COMMON — stay
    audit/               ← COMMON — stay
    audit-standards/     ← COMMON — stay
    architecture/        ← SAMGRAHA-ONLY — remove from build
    engineering/         ← SAMGRAHA-ONLY — remove
    feature/             ← SAMGRAHA-ONLY — remove
    feature-technical/   ← SAMGRAHA-ONLY — remove
    philosophy/          ← SAMGRAHA-ONLY — remove
    release/             ← SAMGRAHA-ONLY — remove
    reports/             ← SAMGRAHA-ONLY — remove
    vision/              ← SAMGRAHA-ONLY — remove
  .samgraha/
    knowledge.db         ← PRE-COMPILED — remove from build
    manifest.json        ← PRE-COMPILED — remove from build
  run-mcp.cmd
  run-mcp.sh
  SHA256SUMS
```

### Why `knowledge.db` Must Be Removed from the Build Artifact

Old model — shared single DB:

```
One knowledge.db ships with build
  ├── ownership: unclear (build owns? samgraha owns?)
  ├── mixing: external repos write into it
  ├── sync: every compile touches the same file
  └── duplication: every recipient ships the same compiled data
```

New model — per-repository DB:

```
samgraha/ → .samgraha/knowledge.db (samgraha's own)
astra/    → .samgraha/knowledge.db (astra's own)
prana/    → .samgraha/knowledge.db (prana's own)
  └── Resolver opens multiples → Knowledge Package
  └── every repo owns its data
  └── no mixing, no duplication
```

---

## Target State (What Should Ship)

```
release/samgraha/
  bin/mcp.exe
  bin/cli.exe
  samgraha.toml           ← common config, no samgraha-specific docs
  docs/raw/
    standards/            ← universal
    audit/                ← universal
    audit-standards/      ← universal
  .samgraha/              ← empty, populated on first compile/init
  run-mcp.cmd
  run-mcp.sh
  SHA256SUMS
```

Samgraha's own repo is compiled and registered externally like any other:

```
cli compile /path/to/samgraha    → .samgraha/knowledge.db (peer)
cli sync   /path/to/samgraha    → registered in local registry (peer)
```

### Why Samgraha Must Become a Peer

Samgraha should not be special. Samgraha becomes **another repository** — exactly like Prana, Astra, Tantra. This simplifies architecture:

- Registry treats all repos identically — no special-casing in resolve, search, or audit
- Compile path works the same way — `compile(path)` writes into target's `.samgraha/`
- Build artifact becomes a pure **standards engine** — the samgraha source repo is optional
- Recipients can compile their own repos without ever touching samgraha source docs

No pre-compile `knowledge.db` in the artifact. No bundled `manifest.json`.

---

## Architecture Evolution

The design evolved through five versions:

```
V0 — Single shared knowledge.db
      One database. Every compile writes to the same file.
      Problem: ownership, mixing, sync contention.

V1 — Per-repository knowledge.db
      Each repo compiles into its own `.samgraha/knowledge.db`.
      Problem: no discovery mechanism for finding other repos.

V2 — Registry
      `registry.db` introduced as a metadata index.
      Discovery: knows where repos are, their revisions, their exports.
      Still: no unified query surface across repos.

V3 — Planner → Resolution Plan
      Deterministic resolution plan from config + `.meta` files.
      Answer: "which repos should I load?"
      Still: no in-memory assembly.

V4 — Knowledge Package
      Concrete type: `RuntimePackage` with opened store handles.
      Assembly from plan. In-memory. Provenance-aware.
      Still: tied to MCP connection lifetime.

V5 — Knowledge Context
      Reusable runtime wrapper around Knowledge Package.
      Survives across MCP reconnect, CLI operations, search, audit.
      Contains: opened stores, search/audit caches, workspace preferences.
```

---

## Overall Architecture Diagram

```
Repository (any)
  │
  ├── docs/raw/             ← markdown source
  │
  ├── compile
  │     │
  │     ▼
  │   .samgraha/
  │     knowledge.db        ← compiled docs + sections + graph
  │     manifest.json       ← identity + revision + exports
  │
  ├── sync
  │     │
  │     ▼
  │   Global Registry (registry.db)
  │     metadata only: uuid, path, revision, exports, status
  │
  ├── cli compile /external/repo
  │     │
  │     ▼
  │   /external/repo/.samgraha/knowledge.db  ← target's own store
  │
  ├── cli sync /external/repo
  │     │
  │     ▼
  │   registry + .samgraha/dependencies/<name>.meta
  │
  └── MCP server starts
        │
        ├── Planner
        │     config + .meta files → Resolution Plan
        │
        ├── Resolver
        │     Resolution Plan + opened stores → Knowledge Package
        │
        ├── Knowledge Context
        │     Runtime wrapper: package + caches + preferences
        │
        ├── Search     ← Knowledge Context → ranked results
        ├── Audit      ← Knowledge Context → findings
        ├── MCP tools  ← Knowledge Context → JSON-RPC responses
        └── CLI        ← Knowledge Context → terminal output
```

---

## Ownership Boundaries

Explicit invariants. No component may violate another's ownership.

### Repository Owns

```
knowledge.db       ← compiled documents, sections, graph
manifest.json      ← identity, revision, exports, dependencies
audit history      ← reports, findings, gate status
.samgraha/         ← all local state
```

### Registry Owns

```
metadata only      ← uuid, path, revision, exports, status
discovery          ← "find all repos that export architecture standards"
identity           ← canonical mapping: name → uuid → path
```

Registry is NOT:

```
Search             ← that is the Resolver's job
Storage            ← that is the Repository's job
Compilation        ← that is the Compiler's job
```

### Resolver Owns

```
Knowledge Package  ← deterministic assembly from plan + stores
provenance         ← which doc came from which repo
priority ordering  ← primary > dependency > interest
```

Resolver never:

```
reads markdown     ← that is the Compiler's job
parses docs        ← that is the Compiler's job
queries registry   ← that is the Planner's job (via .meta cache)
```

### Knowledge Context Owns

```
runtime state only ← no persistent data
opened SQLite handles
runtime search cache
runtime audit cache
loaded repository graph
Planner output (Resolution Plan)
statistics
workspace preferences
```

### Planner Responsibility

Planner DOES:

```
dependency resolution    ← from config [knowledge].dependencies
interest resolution     ← from config [knowledge].interests
repository ordering     ← primary > dependency > interest
priority assignment     ← each entry gets a Priority enum
```

Planner DOES NOT:

```
search                  ← that is SearchService
audit                   ← that is AuditService
query interpretation    ← that is the caller's context
```

Planner builds a **Resolution Plan** — a deterministic resolution candidate list.
Same `samgraha.toml` + same `.meta` files → identical plan, always.

### Dependency Cache

Stored at `.samgraha/dependencies/<name>.meta`.

Contains ONLY:

```
repository UUID
repository path
revision
exports
dependencies
capabilities
last_sync timestamp
expiration time
```

Never contains:

```
documents
sections
chunks
search index
knowledge content
```

Metadata is fast (~1KB per repo). Full knowledge is expensive (~MB per repo).
The cache keeps metadata separate so Planner can run without loading any knowledge.

---

## Cache Flow

```
Planner
  │
  ├── config [knowledge].dependencies → names
  ├── config [knowledge].interests    → names
  │
  ├── for each name:
  │     ├── .samgraha/dependencies/<name>.meta  ← metadata cache
  │     │     └── path, revision, exports, expires
  │     │
  │     ├── if .meta expired:
  │     │     └── Global Registry → refresh .meta
  │     │
  │     └── if no .meta:
  │           └── config [repository].dependencies path
  │
  └── ResolutionPlan
        entries: [(name, path, priority, status, revision, available)]
```

Only metadata is cached. Documentation is never duplicated.

---

## What Exists vs What Changes

### Build Separation

> **Status: NOT started.** Both changes below still in current build script as of this review.

| Aspect | Current | Target | Effort |
|--------|---------|--------|--------|
| `build-release.ps1:69` | `Copy-Item -Recurse docs/raw/*` ← **still verbatim in script** | Copy only `standards/`, `audit/`, `audit-standards/` | **1 line** |
| `build-release.ps1:74` | `cli compile --force` at pkg root ← **still present** | Remove — no pre-compile | **1 line** |
| Post-install registration | none | `init` or first `compile` registers samgraha as peer | **~20 lines** new |
| Common dirs on disk | `docs/raw/standards/` etc | same — `StandardRegistry` already reads from disk | **0 lines** |
| External repos discover stds | from bundled `docs/raw/standards/` | same — unchanged | **0 lines** |

### SamagraIgnore

> **Config path correction:** `IgnoreConfig` lives under `[repository]` in TOML (field of `RepositoryConfig`), not `[compilation]`. TOML path is `[repository.ignore]`, not `[compilation.ignore]`.

> **Discovery/glob mismatch:** `IgnoreConfig::default()` uses full glob syntax (`**/node_modules/**`). `collect_markdown_files` in `discovery.rs` uses substring dir-name matching — incompatible. Wiring IgnoreConfig patterns verbatim won't work without also upgrading the matching logic.

> **Default patterns count:** `IgnoreConfig::default()` has **3** patterns (node_modules, target, .git). `audit-standards` is NOT in defaults — it's hardcoded at `pipeline.rs:39-44` AND guarded a second time in `compilation.rs:84` via `rel.contains("audit-standards")`. Both guards need updating if IgnoreConfig replaces hardcoded excludes.

| Aspect | Current | Target | Effort |
|--------|---------|--------|--------|
| `IgnoreConfig` at `config.rs:110-126` | defined, dead code, under `[repository.ignore]` | wired into compile pipeline | **~5 lines** in `pipeline.rs` |
| Hardcoded excludes `pipeline.rs:37-44` | `["node_modules","target",".git","audit-standards"]` — substring match | moved to config defaults | **~10 lines** |
| Secondary `audit-standards` guard `compilation.rs:84` | `rel.contains("audit-standards")` | move into IgnoreConfig default | **~3 lines** |
| `collect_markdown_files` `discovery.rs:90-122` | substring dir-name match | glob match on relative path | **medium** — needs glob crate or path-contains |
| `.samagraignore` file | none | read from repo root, parse like `.gitignore` | **~80 lines** new |
| `samgraha.toml` patterns | none | `[repository.ignore]` — already in schema, just wire | **~5 lines** |

---

## Architectural Decisions

Three decisions made before implementation begins. Each one prevents a class of problems.

### Decision 1 — Rename KnowledgeSession First

`KnowledgeSession` → `KnowledgeContext` before any new feature is added.

Not after. Building features on `KnowledgeSession` while documentation, diagrams, and discussions all say `KnowledgeContext` creates a naming split that compounds PR by PR:

```
❌  adapter.rs: session: Option<KnowledgeSession>   ← old name
    session.rs: pub struct KnowledgeContext          ← new name
    runtime/context.rs: pub struct RuntimeContext   ← third name

✅  adapter.rs: context: Option<KnowledgeContext>   ← consistent
    session.rs: pub struct KnowledgeContext          ← same
    runtime/context.rs: pub struct RuntimeContext   ← different thing, obvious
```

Rename is pure mechanical — no logic changes. Do it as Phase 3, after build/ignore work (which doesn't touch session code) and before any session feature work.

### Decision 2 — ContextManager from the Start

`McpAdapter` holds a `ContextManager`, not `Option<KnowledgeContext>`.

```
❌  McpAdapter { context: Option<KnowledgeContext> }
    
✅  McpAdapter { context_manager: ContextManager }
    ContextManager { context: Option<KnowledgeContext> }  ← Phase 8 shape
    ContextManager { contexts: HashMap<String, KnowledgeContext> }  ← Phase 9 shape
```

The adapter's interface to context doesn't change between Phase 8 and Phase 9 — it always calls `self.context_manager.active()`. Multi-context arrives as an internal change to `ContextManager` with no adapter refactoring needed.

Cost now: one extra struct, ~50 lines. Cost avoided later: refactoring every adapter handler that touches `self.context` when multi-context arrives.

### Decision 3 — Context Lifetime Separate from MCP Lifetime

Context and MCP connection are different things with different lifecycles.

```
❌  MCP connects  →  context created
    MCP disconnects  →  context disposed

✅  context exists  (process-level, ContextManager-owned)
    MCP connects  →  use context
    MCP disconnects  →  context inactive, TTL countdown
    MCP reconnects (within TTL)  →  reuse context, no rebuild
    MCP reconnects (after TTL)  →  rebuild
```

This enables:

- Claude reconnects after a tool restart → no expensive rebuild
- CLI `search` command → uses same context as running MCP server
- Inspector connects → same context, no new plan/resolve cycle
- Future: multiple tools connected simultaneously, all sharing one assembled package

The context is owned by `ContextManager`, not by `McpAdapter`. `McpAdapter` is a consumer, not the owner.

---

## Implementation Plan

Each phase builds on the previous. Do not skip or reorder.

### Phase 1 — Build Separation (1 hour)

Script-only. No Rust changes.

**`build-release.ps1:69`** — copy only common dirs:

```powershell
$commonDirs = @("standards", "audit", "audit-standards")
foreach ($dir in $commonDirs) {
    Copy-Item -Recurse "$root\docs\raw\$dir" "$pkgDir\docs\raw\$dir"
}
```

**`build-release.ps1:74`** — delete `cli compile --force`.

`StandardRegistry` reads from `docs/raw/standards/` on disk — same path, no runtime code change needed.

### Phase 2 — SamagraIgnore (2-4 hours)

Wire `IgnoreConfig`, remove hardcoded excludes, add `.samagraignore` file support.

**`pipeline.rs:37-44`** — replace hardcoded array with config patterns (normalise to plain name-contains, not glob, until glob crate added):

```rust
let ignore_patterns = config.repository.ignore.patterns.clone();
// normalise: strip **/.../** → plain dir name for now
```

**`config.rs:IgnoreConfig::default()`** — add `audit-standards` as fourth default pattern.

**`compilation.rs:84`** — remove inline `rel.contains("audit-standards")` guard; covered by IgnoreConfig default.

**`discovery.rs`** — if `root/.samagraignore` exists, parse non-comment non-empty lines, merge with config patterns.

### Phase 3 — Rename (pure mechanical, 30 min)

Do this before adding any more features. Every subsequent PR uses correct names.

- `KnowledgeSession` → `KnowledgeContext` in `session.rs` and all callers
- `KnowledgePlan` → keep (already correct); update doc references that said `ResolutionPlan`
- Update `session.rs` file-level doc comment
- Update `adapter.rs` field name: `session` → `context`
- Update `services/src/lib.rs` re-exports

No logic changes. Rename only.

### Phase 4 — Revision Validation (1 hour)

Implement revision-first validity before adding any features that depend on correctness.

**`session.rs:is_valid()`** — extend to compare per-entry revisions:

```rust
pub fn is_valid(&self, plan: &KnowledgePlan) -> bool {
    if self.assembly_time.elapsed().as_secs() >= self.ttl_secs { return false; }
    for entry in plan.available() {
        let cached = self.plan.entries.iter()
            .find(|e| e.name == entry.name)
            .map(|e| e.revision);
        if cached != Some(entry.revision) { return false; }
    }
    true
}
```

Re-run `Planner::plan()` on each `is_valid()` call to get the current plan for comparison. Plan is cheap (reads `.meta` files, no DB).

### Phase 5 — Automatic Rebuild on Expiry (1 hour)

Fix silent degradation. Expired context rebuilds, not silently falls back.

**`adapter.rs`** — replace per-handler fallback pattern with a single `get_context()` helper:

```rust
fn get_context(&mut self) -> Option<&KnowledgeContext> {
    if let Some(ref ctx) = self.context {
        let plan = Planner::plan(&self.runtime.context.repository_root, &self.runtime.context.config);
        if ctx.is_valid(&plan) {
            return self.context.as_ref();
        }
        // Expired or revision changed — rebuild.
        tracing::info!("Context expired or stale — rebuilding");
        self.context = KnowledgeContext::create(...).ok();
    }
    self.context.as_ref()
}
```

Surface rebuild events in `workspace_status` response.

### Phase 6 — Audit/Search/Sections Through Context (2-3 hours)

Make ALL request handlers use the Knowledge Context, not just `handle_search`.

**`handle_audit`** — replace `self.runtime.audit()` with context-aware audit:
```rust
// Instead of: self.runtime.audit(domain, &providers, None)
// Use: context.package.all_documents() → AuditService::execute
```

**`handle_get_sections`** — same: read from `context.package` stores, not `self.runtime.registry`.

**`handle_get_documents_by_domain`** — same.

After this phase, all knowledge queries are cross-repo by default.

### Phase 7 — Lazy Interest Loading (1-2 hours)

Performance: don't open interest stores until requested.

**`session.rs` / `RuntimePackage`** — split `from_plan` into two paths:

```rust
impl RuntimePackage {
    pub fn from_plan_mandatory(plan: &KnowledgePlan) -> Result<Self>  // deps only
    pub fn load_interest(&mut self, name: &str, plan: &KnowledgePlan) -> Result<()>
}
```

`KnowledgeContext::create()` calls `from_plan_mandatory`. Interest stores open on first search/audit reference.

### Phase 8 — ContextManager (2-3 hours)

Introduce `ContextManager` even though only one context exists initially. Separates context lifecycle from MCP lifecycle.

```rust
pub struct ContextManager {
    context: Option<KnowledgeContext>,
    connection_count: usize,
    inactive_since: Option<Instant>,
    knowledge_ttl: Duration,
}

impl ContextManager {
    pub fn active(&self) -> Option<&KnowledgeContext>
    pub fn on_connect(&mut self) -> Result<()>    // reuse or rebuild
    pub fn on_disconnect(&mut self)               // → Inactive
    pub fn dispose_if_expired(&mut self)          // TTL countdown
}
```

`McpAdapter` holds `ContextManager`, not `Option<KnowledgeContext>`.

MCP connect → `context_manager.on_connect()`. MCP disconnect → `context_manager.on_disconnect()`.
CLI search → `context_manager.active()` (reuses existing context if valid, no MCP connection needed).

### Phase 9 — Multi-Context (future, scope TBD)

Only after Phase 8 is stable.

`ContextManager.context: Option<KnowledgeContext>` → `contexts: HashMap<String, KnowledgeContext>`.

`McpAdapter` unchanged — still calls `context_manager.active()`.

### Samgraha Self-Registration (can be any phase ≥ 1)

**Post-install** — `cli compile /path/to/samgraha-repo; cli sync /path/to/samgraha-repo`. Independent of session architecture. Optionally: `samgraha init --register-samgraha`.

---

## What Doesn't Change

| Component | Reason |
|-----------|--------|
| `CompilationService::execute` | Already takes config + root + registry. Thread config ref through; one guard line moves to IgnoreConfig. |
| `MCP::handle_compile` with `path` | Already compiles into external repo's `.samgraha/knowledge.db`. |
| `MCP::handle_sync` | Already reads external manifest and registers. |
| `RegistryStore` / `RegistryDb` | Schema unchanged. Registry tracks all repos equally. |
| `StandardRegistry` | Reads `docs/raw/standards/` from disk — same path in artifact. |
| `KnowledgeResolver` / `resolution.rs` | Not affected — build separation and session changes don't touch resolve path. |
| `Planner` | Not affected by build separation or SamagraIgnore — already reads config + `.meta` files. |
| `samgraha.toml` schema | `IgnoreConfig` already exists under `[repository.ignore]`; just wire it. |

---

# Persistent Knowledge Context — Feasibility Analysis

## Overview

Introduce a reusable runtime object (`Knowledge Context`) containing an assembled `Knowledge Package` + open resources, surviving across MCP sessions until invalidated by revision change or TTL expiry.

The `Knowledge Context` decouples context lifetime from MCP connection lifetime. An engineer's context persists across:

- MCP reconnect (Claude Code restarts)
- CLI operations (`search`, `audit`, `info`)
- Multiple searches within the same workspace
- Cross-repository comparisons

Each rebuild is expensive (plan + open stores). Reusing a valid context makes repeated operations fast.

---

## Knowledge Package vs Knowledge Context

These are different concepts with different properties.

### Knowledge Package

```
Knowledge Package

     ↓

  immutable

     ↓

  produced by Resolver
```

| Property | Value |
|----------|-------|
| Deterministic | Same inputs → identical package |
| Stateless | Contains no runtime state |
| Reproducible | Can be discarded and rebuilt |
| Contents | Repository set, dependency graph, provenance, store handles |

### Knowledge Context

```
Knowledge Context

     ↓

  runtime wrapper

     ↓

  contains caches

     ↓

  contains opened databases
```

| Property | Value |
|----------|-------|
| Disposable | Can be recreated from its Knowledge Package |
| Stateful | Contains caches, statistics, preferences |
| Mutable | Caches grow, statistics accumulate |
| Contents | Knowledge Package, SQLite handles, search cache, audit cache, loaded repo graph, Planner output, workspace preferences |

**Key principle:** Knowledge Package is deterministic. Knowledge Context is disposable.

A Context can always be reconstructed from its Package. The Package can always be reconstructed from its Plan. The Plan can always be reconstructed from config + `.meta` files.

---

## Context Lifecycle

**Key principle:** Context lifetime is independent of MCP connection lifetime.

Don't think: MCP starts → context starts. Think: context exists → MCP connects → uses context → disconnects → context survives.

This enables Claude, CLI, Inspector, and other tools to share the same assembled Knowledge Context without rebuilding it per connection.

```
Create
  │
  ├── Planner runs (config + .meta → KnowledgePlan)
  ├── Resolver assembles (plan + stores → Knowledge Package)
  └── Knowledge Context wraps package + caches
  └── State: Active

MCP/tool connects
  │
  └── ContextManager hands reference to active context
  └── Context state: Active (connection count +1)

MCP/tool disconnects
  │
  └── Connection count -1
  └── If count == 0 → State: Inactive
  └── TTL countdown begins (knowledge_ttl from config)

While Inactive
  │
  ├── Revision check on next connection attempt
  │     ├── Unchanged → State: Active (reuse, no rebuild)
  │     └── Changed   → Rebuild (plan → resolve → wrap)
  │
  └── TTL expired while still inactive → Dispose

Dispose
  │
  └── Close SQLite handles
  └── Drop caches
  └── ContextManager removes entry
```

### State Machine

```
                    ┌─────────────────────────────────────┐
                    │                                     │
  Create            │  MCP connects        MCP connects   │
     │              │  revision OK         revision OK    │
     ▼              │                                     │
  ┌────────┐  all disconnect   ┌──────────┐              │
  │ Active │ ──────────────► │ Inactive  │              │
  │        │ ◄────────────── │           │              │
  └────────┘  reconnect +     └─────┬────┘              │
     │        revision OK           │                   │
     │                         TTL expires         revision Δ
     │                              │             on reconnect
     │                              ▼                   │
     │                         ┌──────────┐             │
     │                         │ Disposed │         ┌───────────┐
     │                         └──────────┘         │ Rebuilding│
     │                                              └─────┬─────┘
     └──────────────────────────────────────────────────┘
                                               rebuild complete → Active
```

| State | Meaning |
|-------|---------|
| **Active** | At least one client connected. Serving requests from cached stores. |
| **Inactive** | No clients connected. Context alive, TTL countdown running. Reusable. |
| **Rebuilding** | Revision change detected on reconnect. Old context disposed; new one assembling. Old package still serves reads if available. |
| **Disposed** | TTL expired while inactive, or explicit close. Handles closed, caches dropped. |

**Why this matters:** Rebuilding a Knowledge Context is expensive (plan + open stores). A CLI search after MCP disconnect should reuse the same context, not trigger a full rebuild.

---

## Validation Flow

Validation is per-request, not per-context-creation. It is lightweight: local files + in-memory comparison.

```
Knowledge Context
  │
  ├── Step 1: Revision Changed?
  │     │
  │     ├── Compare each repo's cached revision
  │     │   against current manifest.json or .meta revision
  │     │
  │     ├── Yes → Invalidate context. Go to Rebuild.
  │     └── No  → Continue to Step 2.
  │
  ├── Step 2: Metadata TTL Expired?
  │     │
  │     ├── Check .meta file expires timestamp
  │     │
  │     ├── Yes → Refresh .meta from Global Registry.
  │     │         Re-check revision.
  │     │         If changed → Invalidate.
  │     │         If unchanged → Continue to Step 3.
  │     └── No  → Continue to Step 3.
  │
  ├── Step 3: Knowledge TTL Expired?
  │     │
  │     ├── Check context.assembly_time.elapsed()
  │     │
  │     ├── Yes → Invalidate context. Go to Rebuild.
  │     └── No  → Context is valid. Reuse.
  │
  └── Valid? → Reuse. Else → Rebuild.
```

**Revision is authoritative.** TTL is fallback.

If revision changed, context invalidates immediately regardless of TTL.
If only TTL expired but revision is unchanged, a metadata refresh sufficies — no full rebuild needed.

---

## Dual TTL

| TTL | Governs | Default | Refresh Cost | Rationale |
|-----|---------|---------|--------------|-----------|
| **Metadata TTL** | `.meta` file validity | 24h | ~1KB read, cheap | Dependencies change frequently. Refresh metadata to detect revision changes without touching knowledge. |
| **Knowledge Context TTL** | Assembled context validity | 720h (30d) | Expensive (plan + resolve + open stores) | Knowledge content is stable. Rebuild only when revision changes or TTL expires. |

Metadata changes frequently — dependencies are added, removed, recompiled. Knowledge Context changes only after a revision change (a repo was recompiled). The two TTLs reflect these different change frequencies.

---

## Lazy Interest Loading

Dependencies and interests have different load semantics.

### Dependencies

```
Always loaded.
  ├── Required for correctness
  ├── Always in Knowledge Package
  └── High priority in search results
```

### Interests

```
Loaded only when requested.
  ├── Optional knowledge source
  ├── Not in Knowledge Package by default
  ├── Lower priority if loaded
  └── Loaded on first reference during search or audit
```

### Loading Order

```
Current Repository
  │
  ├── (always loaded, highest priority)

Dependencies (from [knowledge].dependencies)
  │
  ├── (always loaded, high priority)
  │
  └── Knowledge Package assembled

  ...

Interest requested (search or audit references it)
  │
  ├── Load interest store
  ├── Add to Knowledge Package (lower priority)
  └── Continue serving request
```

Interests keep normal operations fast (fewer stores open) while still enabling broader discovery when the user explicitly references an interest repo.

---

## Workspace Preferences

Knowledge Context stores workspace-level preferences that persist across requests within the same context.

```
Knowledge Context Preferences
  │
  ├── Preferred Search Level    (section / document / cross-domain)
  ├── Preferred Domains         (architecture, engineering, feature, ...)
  ├── Audit Profile             (minimal / standard / full)
  ├── Default Filters           (status, severity, stage)
  └── Current Repository        (primary context focus)
```

Preferences avoid repeating the same parameters across consecutive MCP calls. An LLM working through a document audit can set the domain once and all subsequent `search` / `get_sections` / `audit` calls respect the preference.

---

## ContextManager

`McpAdapter` does not hold `Option<KnowledgeContext>` directly. It holds a `ContextManager`.

```
McpAdapter
  │
  └── ContextManager
        │
        └── active: Option<KnowledgeContext>   ← Phase 8 initial shape
        
        (future)
        └── contexts: HashMap<String, KnowledgeContext>
```

`ContextManager` is the stable interface. `McpAdapter` calls `context_manager.active()` for every request. Internally, ContextManager starts with one slot. Expanding to named contexts requires no changes in `McpAdapter`.

### ContextManager Interface

```rust
impl ContextManager {
    fn active(&self) -> Option<&KnowledgeContext>
    fn create(&mut self, root: &Path, config: &SamgrahaConfig) -> Result<()>
    fn rebuild_if_stale(&mut self) -> Result<()>
    fn on_disconnect(&mut self)       // → Active → Inactive
    fn on_connect(&mut self)          // → Inactive → Active (reuse or rebuild)
    fn dispose_expired(&mut self)     // → Inactive + TTL expired → Disposed
}
```

### Phase 8 Internal Shape (single context)

```
ContextManager {
    context: Option<KnowledgeContext>,
    connection_count: usize,
    inactive_since: Option<Instant>,
}
```

### Phase 9 Shape (multi-context)

```
ContextManager {
    contexts: HashMap<String, KnowledgeContext>,
    active_name: Option<String>,
}
```

Switching contexts avoids rebuilding. Each context is independent — different repos, different preferences, different caches.

```
Available Contexts
  │
  ├── Prana Development
  │     Status: Active (1 connection)
  │     Repositories: 3 (prana, astra, samgraha)
  │
  ├── Astra Research
  │     Status: Inactive (TTL: 23h remaining)
  │     Repositories: 2 (astra, tantra)
  │
  └── Legacy Workspace
        Status: Disposed (TTL expired)
        Repositories: 7
```

### CLI / MCP Commands

```
context new <name> --root <path>    ← create new context
context list                        ← list all contexts + states
context use <name>                  ← switch active context (reuse or rebuild)
context refresh                     ← force revision check + rebuild if dirty
context close <name>                ← dispose context immediately
```

---

## Search Flow

Current search:

```
Search
  │
  └── knowledge.db (single store, single repo)
```

After Knowledge Context:

```
Search
  │
  ├── Knowledge Context
  │     ├── is_valid()? → yes
  │     └── no → rebuild
  │
  ├── Knowledge Package
  │     ├── primary store
  │     ├── dependency stores
  │     └── interest stores (if loaded)
  │
  └── SearchService.search(KnowledgePackage, query)
        → ranked results with provenance
```

---

## Audit Flow

After Knowledge Context:

```
Audit
  │
  ├── Knowledge Context
  │     ├── is_valid()? → yes
  │     └── no → rebuild
  │
  ├── Knowledge Package
  │     ├── current repo (always)
  │     ├── dependencies (always)
  │     └── interests (if loaded)
  │
  ├── AuditService.audit(KnowledgePackage, scope)
  │     → sections from current repo
  │     → cross-domain from all loaded repos
  │
  └── findings with provenance
```

---

## What Already Exists (80% Implemented)

> **Naming correction:** Document used `KnowledgeContext` and `ResolutionPlan` throughout. Actual Rust type names are `KnowledgeSession` (in `session.rs`) and `KnowledgePlan` / `KnowledgePlanEntry` (in `planner.rs`). The planned rename is `KnowledgeSession` → `KnowledgeContext`. A separate `RuntimeContext` struct exists in `runtime/context.rs` — it is a path+config holder inside `KnowledgeRuntime`, NOT the same as `KnowledgeSession`.

| Component | File | Status |
|-----------|------|--------|
| `KnowledgeSession` ← _doc called this `KnowledgeContext`_ | `services/src/session.rs:52` | **Exists** — wraps `RuntimePackage` + TTL validation |
| `RuntimePackage` | `services/src/session.rs:14` | **Exists** — opens stores per plan entry |
| `Planner` + `KnowledgePlan` + `KnowledgePlanEntry` ← _doc called these `ResolutionPlan/Entry`_ | `services/src/planner.rs:52-96` | **Exists** — deterministic, no query context |
| `Planner::plan_dep` | `planner.rs:98-152` | **Exists** — resolves deps from config + `.meta` |
| `read_meta_file` / `write_meta_file` | `planner.rs:182-197` | **Exists** — `.samgraha/dependencies/*.meta` |
| `DepStatus` enum (7 states) | `planner.rs:18-34` | **Exists** — `Primary/Loaded/Stale/Outdated/Missing/Unresolved/RequiredMissing` |
| `knowledge.dependencies` | `config.rs` | **Exists** — config section |
| `knowledge.interests` | `config.rs` | **Exists** — lower priority than deps |
| `resolver.metadata_ttl` | `config.rs:209-210` | **Exists** — "24h" default |
| `resolver.knowledge_ttl` | `config.rs:213-214` | **Exists** — "720h" default |
| `KnowledgeSession::is_valid()` | `session.rs:76-78` | **Exists** — TTL-only check (no revision check) |
| `KnowledgeSession::dispose()` | `session.rs:90-92` | **Exists** — drop handles |
| Session created at MCP startup | `adapter.rs:26-29` | **Exists** — `KnowledgeSession::create()` when MCP starts |
| Search uses session | `adapter.rs:291-294` | **Exists** — `handle_search` checks session validity, falls back to `runtime.search()` |
| `RuntimeContext` | `services/src/runtime/context.rs:5` | **Exists, undocumented** — path+config holder inside `KnowledgeRuntime`, distinct from `KnowledgeSession` |
| `KnowledgeRuntime` | `services/src/runtime/runtime.rs:25` | **Exists, undocumented** — major runtime wrapper; holds `RuntimeContext`, `RegistryStore`, `StandardRegistry`, `AuditFramework`, `RuntimePolicy` |

---

## What Is Missing (20%)

| Feature | Gap | Effort |
|---------|-----|--------|
| **Multi-context management** | `McpAdapter` holds one `Option<KnowledgeSession>` — no `context new/list/use/refresh/close` commands | **~80 lines** new in adapter |
| **Revision change detection** | `is_valid()` only checks TTL, not revision diffs. Needs per-request `.meta` revision comparison | **~15 lines** in `session.rs:is_valid()` |
| **Lazy interest loading** | `RuntimePackage::from_plan` loads ALL entries (deps + interests) eagerly via `plan.available()` | **~30 lines** — add `load_interest(name)` method |
| **Explicit context CLI** | No `cli context` subcommand | **~100 lines** in `commands.rs` |
| **Workspace preferences** | No stored preferences per context | **~40 lines** — preferences struct + MCP params |
| **Session rebuild on expiry** | `is_valid()` checked per-request but expired session is never rebuilt — session stays `None` after expiry, falls back to single-repo runtime | **~15 lines** — rebuild trigger in `handle_request` |
| **Audit/sections not session-aware** | `handle_audit` and `handle_get_sections` call `self.runtime.*()` — single-repo only. Only `handle_search` uses the multi-repo session | **Scope TBD** — wire audit + sections through `KnowledgeSession` |

### Implementation Gap Detail

**Revision detection** — currently `KnowledgeSession::is_valid()`:

```rust
pub fn is_valid(&self) -> bool {
    self.assembly_time.elapsed().as_secs() < self.ttl_secs
}
```

Needs: compare each entry's cached revision against current `.meta` revision.
Note: use correct type `KnowledgePlan` not `ResolutionPlan` (doc naming error corrected):

```rust
pub fn is_valid(&self, plan: &KnowledgePlan) -> bool {
    if self.assembly_time.elapsed().as_secs() >= self.ttl_secs { return false; }
    for entry in plan.available() {
        let cached = self.plan.entries.iter()
            .find(|e| e.name == entry.name)
            .map(|e| e.revision);
        if cached != Some(entry.revision) { return false; }
    }
    true
}
```

**Multi-context management** — new MCP tools:

```json
{
  "context_new":     { "name": "string", "root": "string" },
  "context_list":    {},
  "context_use":     { "name": "string" },
  "context_refresh": {},
  "context_close":   { "name": "string" }
}
```

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Too many open repositories | File handle exhaustion | Planner limits to declared deps + interests. Lazy loading prevents unnecessary opens. Context lifecycle closes handles on dispose. |
| Stale metadata causes incorrect planning | Wrong dependency graph | Revision validation detects mismatch during `is_valid()`. Planner falls back to manifest.json if `.meta` is missing or expired. |
| Too many SQLite handles across contexts | Resource pressure | Each context manages its own handles. `context close` releases them. Configurable max contexts limit. |
| Broken dependency (path removed, repo deleted) | Failed resolve | Planner produces `DepStatus::RequiredMissing` instead of crashing. Knowledge Context still serves remaining repos. Error surfaced in `workspace_status`. |
| Knowledge Context TTL too long | Stale knowledge served | Revision check catches changes regardless of TTL. Metadata TTL (24h) refreshes frequently. Knowledge TTL is safety net, not primary invalidation. |

---

## Assumptions

```
Repositories are local.              ← Network mounts may add latency but don't change correctness.
Repository revisions are authoritative. ← manifest.json revision is source of truth.
Registry metadata is trusted.        ← Written by sync, read by Planner. No cross-repo auth.
Knowledge Package is reproducible.   ← Same config + same .meta = same plan = same package.
.Meta files are disposable.          ← Can be deleted and regenerated from manifest.json.
Knowledge Context is disposable.     ← Can always be reconstructed from its Package.
One context per workspace.           ← Contexts map 1:1 to logical workspaces.
```

---

## Rejected Alternatives

### Alternative A — One Giant `knowledge.db`

All repos compile into a single shared database.

| | |
|---|----|
| **Rejected because** | Ownership is unclear. Mixing data from multiple repos makes provenance tracking fragile. Sync contention as more repos write to the same DB. No natural partitioning for parallel reads. |
| **Verdict** | V0 architecture. Replaced by per-repo DB in V1. |

### Alternative B — Registry Stores Documents

Registry (`registry.db`) holds compiled document content, not just metadata.

| | |
|---|----|
| **Rejected because** | Duplication: every repo's knowledge lives in both its own `knowledge.db` and the registry. Registry becomes a bottleneck for search. Registry lifecycle (sync/expiry) conflates with document lifecycle (compile/audit). |
| **Verdict** | Registry is metadata-only. Documents stay in per-repo stores. |

### Alternative C — Open All Registered Repositories

Knowledge Context opens every repo the registry knows about.

| | |
|---|----|
| **Rejected because** | Performance: as registry grows (10+ repos), every context loads every store. Most are irrelevant to the current workspace. Memory waste. File handle pressure. |
| **Verdict** | Planner limits to declared deps + interests. Registry is for discovery only. |

### Alternative D — TTL-Only Validation

Validate Knowledge Context by age alone, ignoring revision changes.

| | |
|---|----|
| **Rejected because** | TTL is probabilistic. A repo recompiled 5 minutes ago is stale even if TTL has 23 hours remaining. Revision is deterministic — if it changed, knowledge is stale regardless of age. |
| **Verdict** | Revision is authoritative. TTL is fallback for silent changes (e.g., disk corruption, un-tracked modifications). |

### Alternative E — `compile_and_register` Single Tool

One tool that compiles a repo and immediately registers it in the registry.

| | |
|---|----|
| **Rejected because** | Couples compile (produce artifacts) with sync (update registry). These are separate lifecycle operations with different invalidation semantics. Compile should succeed even when registry is unavailable. |
| **Verdict** | Two tools: `compile(path)` and `sync(path)`. Separate responsibilities. |

---

## Future Scope

The current design intentionally excludes these features. They are deferred to future work.

| Feature | Why Excluded Now |
|---------|------------------|
| **Cloud Registry** | HTTP registry backend exists in config (`RegistryType::Http`) but is unimplemented. Requires auth, TLS, conflict resolution. |
| **Git Sync** | Auto-sync when a repository is pulled. Requires git hook integration or filesystem watcher. |
| **Remote Repository** | Compile a repo accessed over SSH or HTTPS. Requires remote file access layer. |
| **HTTP Registry** | Replace local `registry.db` with a network service. Requires server-side implementation. |
| **Distributed Cache** | Share `.meta` caches across machines. Requires coordination protocol. |
| **Context Persistence to Disk** | Save/restore Knowledge Context across process restarts. Requires serialization of open stores (challenging). |
| **`context` CLI commands** | `context new`, `context use`, `context refresh`, `context close`, `context list` — deferred until multi-context requirement is confirmed by usage patterns. |

---

## Scoring

### Knowledge Context Scores (by phase)

| Feature | Current | After Phase 3 | After Phase 8 | After Phase 9 |
|---------|---------|--------------|--------------|--------------|
| Naming consistency | 40% (KnowledgeSession/KnowledgeContext split) | 100% | 100% | 100% |
| Revision validation | 30% (TTL-only, no revision check) | 30% | 100% (Phase 4) | 100% |
| Audit/sections cross-repo | 20% (search only) | 20% | 100% (Phase 6) | 100% |
| Context lifecycle separation | 0% (MCP-bound) | 0% | 100% (Phase 8) | 100% |
| ContextManager | 0% (direct Option) | 0% | 100% (Phase 8) | 100% |
| Multi-Context | 0% | 0% | 0% | 90% (Phase 9) |
| Lazy Interests | 40% (eager) | 40% | 100% (Phase 7) | 100% |
| Workspace Preferences | 0% | 0% | 90% | 90% |
| Build Separation | 0% (not started) | 0% (Phase 1 done) | 100% | 100% |

### Before/After Scores

| Dimension | Before (no context) | After (persistent context) | Delta |
|-----------|---------------------|----------------------------|-------|
| Cold-start latency | 5/10 (replan every request) | 10/10 (plan once, reuse) | +5 |
| Warm-request latency | 8/10 (some caching) | 10/10 (no replan) | +2 |
| Memory efficiency | 7/10 (ephemeral) | 7/10 (same — context lives, stores open) | 0 |
| Revision correctness | 8/10 (TTL only) | 10/10 (per-entry revision check) | +2 |
| Multi-repo ergonomics | 6/10 (single session) | 9/10 (switch between workspaces) | +3 |
| Implementation risk | 10/10 (already exists) | 8/10 (needs context mgmt commands) | -2 |

### Combined PC B Workflow Scores

| Dimension | Current | After All | Delta |
|-----------|---------|-----------|-------|
| Document Creation | 5 / 10 | 9 / 10 | +4 |
| Section-by-Section Audit | 2 / 10 | 9 / 10 | +7 |
| Whole Document Audit | 2 / 10 | 9 / 10 | +7 |
| Cross-Domain Consistency | 1 / 10 | 9 / 10 | +8 |
| Token Usage | 7 / 10 | 9 / 10 | +2 |
| Context Relevance | 7 / 10 | 7.5 / 10 | +0.5 |
| Document Standards | 8 / 10 | 8 / 10 | 0 |
| Token Saving | 3 / 10 | 9 / 10 | +6 |
| Performance | 6 / 10 | 8 / 10 | +2 |
| **Overall** | **4 / 10** | **9 / 10** | **+5** |

---

## Feasibility Verdict

**Build Separation + SamagraIgnore: Feasible. Low risk. High leverage.**

| Change | Lines Changed | Risk | Impact |
|--------|--------------|------|--------|
| Build: narrow copy targets | **1 line** (`build-release.ps1:69`) | Minimal | High |
| Build: remove pre-compile | **1 line** (`build-release.ps1:74`) | Minimal | Medium |
| Wire `IgnoreConfig` into pipeline | **~15 lines** (`pipeline.rs`) | Low | High |
| Glob matching in discovery | **~30 lines** (`discovery.rs`) | Low-medium | Medium |
| `.samagraignore` file parser | **~80 lines** (new module) | Medium | High |
| Samgraha self-registration | **~20 lines** (init flow) | Low | Medium |
| **Total** | **~150 lines** | **Low** | **High** |

No schema changes. No database migrations. No MCP tool signature changes. All changes in build script + compiler discovery + config wiring.

**Persistent Knowledge Context: Already ~80% shipped. Remaining gaps ~270 lines.**

| Change | Lines Changed | Risk | Impact |
|--------|--------------|------|--------|
| Rename `KnowledgeSession` → `KnowledgeContext` (type + all callers) | Code-only | Minimal | High |
| Revision validation in `is_valid()` | **~15 lines** (`session.rs`) | Low | High |
| Session rebuild trigger on expiry | **~15 lines** (`adapter.rs`) | Low | High |
| Lazy interest loading | **~30 lines** (`session.rs`) | Low | Medium |
| Wire audit + sections through session (cross-repo) | **~30 lines** (`adapter.rs`) | Medium | High |
| Multi-context management | **~80 lines** (`adapter.rs`) | Medium | High |
| Context CLI commands | **~100 lines** (`commands.rs`) | Medium | Medium |
| **Total** | **~270 lines** | **Low-medium** | **High** |

---

## Key Files to Change

| File | Change |
|------|--------|
| `scripts/build-release.ps1:69` | Copy only `standards/`, `audit/`, `audit-standards/` |
| `scripts/build-release.ps1:74` | Remove `cli compile --force` |
| `crates/compiler/src/pipeline.rs:37-44` | Read ignore patterns from config, not hardcoded |
| `crates/compiler/src/discovery.rs:90-122` | Apply ignore patterns against relative paths (upgrade to glob matching) |
| `crates/common/src/config.rs:116-126` | Add `audit-standards` to `IgnoreConfig::default()` patterns |
| `crates/services/src/compilation.rs:84` | Remove inline `rel.contains("audit-standards")` guard — move to IgnoreConfig |
| `crates/cli/src/commands.rs` | Thread config ref through to pipeline |
| `crates/services/src/session.rs:76-78` | Add revision check to `is_valid()` |
| `crates/services/src/session.rs` | Add lazy interest loading; add session rebuild on expiry |
| `crates/mcp/src/adapter.rs` | Multi-context management + workspace preferences; wire audit/sections through session |

No changes needed in: `registry/`, `schemas/`.

---

## Comparison vs Previous Report

The old `test.md` (v2, 1136 lines) analyzed compile-routing, resolver architecture, and 5-layer knowledge session design. This report (v2, revised) analyzes **build separation**, **samagraignore**, and **persistent knowledge context** as integrated proposals with explicit ownership model.

**Roadmap integration:**

| Old Report Phase | This Report | Relationship |
|-----------------|-------------|-------------|
| Phase 1: compile ownership | Already done (MCP `compile(path)`) | Foundation |
| Phase 2: resolver + session | Renamed to **Knowledge Context** | Evolutionary |
| Phase 3: planner | Unchanged | Still correct |
| Phase 4: optimizations | Unchanged | Still correct |
| — | **Build separation** | New — prerequisite for clean peer model |
| — | **SamagraIgnore** | New — independent, additive |
| — | **Ownership boundaries** | New — formalized invariants |
| — | **Rejected alternatives** | New — architectural record |

---

# Code Review Findings

> This section was added in v3 after reviewing the current implementation against the document claims. Grouped by severity.

---

## Naming Errors (Must Fix Before Implementation)

These are factual errors in the document — the names don't match the code. Any implementation following the doc's names would create new types instead of extending existing ones.

| Document Name | Actual Rust Type | File | Action |
|--------------|-----------------|------|--------|
| `KnowledgeContext` (in session) | `KnowledgeSession` | `services/src/session.rs:52` | Rename session type in doc; plan the rename in code |
| `ResolutionPlan` | `KnowledgePlan` | `services/src/planner.rs:52` | Fix doc name |
| `ResolutionPlanEntry` | `KnowledgePlanEntry` | `services/src/planner.rs:37` | Fix doc name |
| `KnowledgeContext::create()` | `KnowledgeSession::create()` | `session.rs:60` / `adapter.rs:26` | Fix doc name |

Additionally: a `RuntimeContext` struct exists in `services/src/runtime/context.rs`. This is NOT the Knowledge Context — it's a path+config holder (`repository_root`, `registry_path`, `config`, `workspace_id`) used inside `KnowledgeRuntime`. The doc never mentions it, which creates confusion when reading the code.

---

## Config Path Error (Must Fix)

Document says ignore patterns live at `[compilation.ignore]` in `samgraha.toml`. They don't.

```
❌ [compilation.ignore]     ← doc says this
✅ [repository.ignore]      ← actual TOML path (IgnoreConfig is a field of RepositoryConfig)
```

Relevant code: `config.rs:43-44` (`RepositoryConfig.ignore: IgnoreConfig`).

---

## Build Separation Not Started

Both Phase A changes remain unimplemented in the current build script:

- `build-release.ps1:69` still does `Copy-Item -Recurse "$root\docs\raw\*"` — copies all domains.
- `build-release.ps1:74` still does `& ".\bin\cli.exe" compile --force` — pre-compile still runs.
- The `.samgraha/` directory IS created empty (line 61) but immediately populated by the compile on line 74.

No code changes needed in Rust for Phase A — it's entirely a script change.

---

## IgnoreConfig / Discovery Incompatibility

Three independent issues that compound each other:

**1. Glob format vs substring match.** `IgnoreConfig::default()` patterns use full glob syntax:
```
**/node_modules/**
**/target/**
**/.git/**
```
`collect_markdown_files` in `discovery.rs:107` does:
```rust
exclude.iter().any(|p| name.contains(p.trim_matches('*')))
```
After stripping leading/trailing `*`, `**/node_modules/**` becomes `/node_modules/` — that substring won't match a dir named `node_modules`. The current hardcoded strings `"node_modules"` work because they're plain names. Wiring `IgnoreConfig` patterns verbatim without fixing the matching logic will silently break exclusions.

**2. `audit-standards` missing from `IgnoreConfig::default()`.** The hardcoded pipeline exclude list has 4 entries; `IgnoreConfig::default()` only has 3. If the hardcoded list is removed and IgnoreConfig is the source of truth, `audit-standards` stops being excluded UNLESS it's added to defaults.

**3. Secondary `audit-standards` guard in `compilation.rs:84`.** Even if discovery doesn't pick it up, `CompilationService::execute` has:
```rust
if !abs.exists() || rel.contains("audit-standards") {
    registry.delete_document(stored.id)?;
}
```
This guard is independent of discovery. It needs to be generalised into IgnoreConfig deletion logic, or it will be orphaned dead code.

**Recommended resolution order:**
1. Add `audit-standards` to `IgnoreConfig::default()` patterns.
2. Normalise pattern matching in `collect_markdown_files` to plain name-contains (strip glob syntax) OR add glob crate and upgrade to proper glob matching.
3. Generalise the `compilation.rs:84` deletion guard to use IgnoreConfig patterns rather than hardcoded string.

---

## write_meta_file / read_meta_file Filename Coupling

`write_meta_file` (planner.rs:194) names the file by `meta.repository.id`:
```rust
let path = dir.join(format!("{}.meta", meta.repository.id));
```

`read_meta_file` (planner.rs:183) reads by dep name:
```rust
let path = root.join(".samgraha").join("dependencies").join(format!("{}.meta", name));
```

These match only if `repository.id` equals the dependency name used in `samgraha.toml`. This is an implicit invariant that isn't enforced or documented. If a repo's `repository.id` in `manifest.json` differs from the name used in `[repository.dependencies]` config, `.meta` files are written but never read back.

**Risk:** Medium. Likely works in practice today because `repository.id` defaults to dir name, which is also the conventional dep name. But fragile as repos get more explicit IDs.

---

## Audit and Sections Are Single-Repo Only

The document implies the Knowledge Session enables cross-repo audit and search. In reality:

| Handler | Uses session? | Scope |
|---------|--------------|-------|
| `handle_search` | Yes (with fallback) | Multi-repo (via session) |
| `handle_get_sections` | No | Primary repo only (`self.runtime`) |
| `handle_audit` | No | Primary repo only (`self.runtime`) |
| `handle_get_document` | No | Primary repo only (`self.runtime`) |
| `handle_audit_knowledge` | No | Primary repo only (`self.runtime`) |

`handle_audit` calls `self.runtime.audit()` which calls `self.registry.get_all_documents()` — this is the primary repo's `RegistryStore` only. Cross-repo audit is not implemented. The doc's "Audit Flow" diagram showing `KnowledgePackage → AuditService` is aspirational, not current.

---

## Session Rebuild Never Triggered

`KnowledgeSession` is created once at `McpAdapter::new()`. If it fails, `self.session = None`. If it expires (`is_valid()` returns false), there is no code path to rebuild it. The current fallback:

```rust
let results = match &self.session {
    Some(s) if s.is_valid() => s.search(&search_query)?,
    _ => self.runtime.search(&search_query)?,
};
```

silently degrades to single-repo search when the session expires, with no logging, no rebuild attempt, and no user-visible signal. The doc describes a Rebuild state in the lifecycle but it's not implemented.

---

## Undocumented Architecture Components

These exist in the codebase but aren't mentioned in the document:

| Component | File | Purpose |
|-----------|------|---------|
| `KnowledgeRuntime` | `services/src/runtime/runtime.rs:25` | Major runtime type; wraps `RuntimeContext`, `RegistryStore`, `ServiceRegistry`, `StandardRegistry`, `AuditFramework`, `RuntimePolicy`. The MCP adapter holds `Arc<KnowledgeRuntime>`. |
| `RuntimeContext` | `services/src/runtime/context.rs:5` | Path+config holder inside `KnowledgeRuntime`. Fields: `repository_root`, `registry_path`, `config`, `workspace_id`. |
| `RuntimePolicy` | `services/src/runtime/policy.rs` | Unexamined — governs runtime behavior policies. |
| `WorkspaceService` | `services/src/workspace.rs` | Multi-repo workspace compilation and search. Distinct from the Knowledge Session model. |
| `PackageService` | `services/src/package.rs` | Generates distributable knowledge packages. Distinct from `RuntimePackage`. Name collision risk. |

The `KnowledgeRuntime` / `KnowledgeSession` split is the most important undocumented distinction:
- `KnowledgeRuntime` = persistent process-level runtime (one per MCP server process)
- `KnowledgeSession` = assembled multi-repo view (recreatable, TTL-based, per "workspace context")
- `RuntimeContext` = thin config/path struct inside `KnowledgeRuntime`

---

## Optimization Opportunities

Not blocking, but worth tracking for Phase 4:

| Area | Current | Improvement |
|------|---------|-------------|
| **Search document loading** | `all_documents()` loads ALL docs from ALL stores into memory per search call | Stream per-store or add SQL FTS |
| **`max_results: usize::MAX` in search** | Fetches unlimited, paginates in memory | Pass actual `offset + limit` to cap fetch |
| **`.meta` file reads** | `Planner::plan()` reads + parses JSON from disk for each dep on every request | Cache parsed metadata in-session; `.meta` files change only on sync |
| **Session expiry degradation** | Silently drops to single-repo on TTL expiry | Rebuild session + log; surface stale state in `workspace_status` |
| **`interest` vs `dependency` load path** | Both use `plan.available()` — no distinction | Separate `load_mandatory()` / `load_interest()` paths in `RuntimePackage` |
| **`DocumentationConfig::exclusions` vs `IgnoreConfig::patterns`** | Two parallel ignore mechanisms in config, neither wired | Consolidate into one authoritative ignore path |

---

## State Machine vs Implementation

The original state machine (Valid → Dirty → Rebuilding → Stale → Disposed) has been superseded by the decided lifecycle model (Active → Inactive → Reuse/Dispose — see [§ Architectural Decisions](#architectural-decisions)).

Current implementation has two effective states:

- **Active** — `is_valid()` returns true
- **Degraded** — `is_valid()` returns false OR session is `None`; silently falls back to single-repo runtime

The decided model introduces **Inactive** as a first-class state managed by `ContextManager`. Phases 4, 5, and 8 implement the full lifecycle. Until Phase 8, the minimum acceptable improvement is: log a warning and trigger rebuild when validity check fails (Phase 5), rather than silently degrading.

---

# `samgraha init` + Repository Guard — Feasibility Report

> **v1** — `samgraha init` creates `.samgraha/` marker directory (analogous to `.git/`). Guard rejects operations in non-init'd repos. Lazy compilation rebuilds knowledge on access if missing.

---

## Summary

| Change | Lines | Risk | Impact |
|--------|-------|------|--------|
| `samgraha init` creates `.samgraha/` marker + config | ~15 lines | Minimal | High |
| CLI repo guard on all commands | ~20 lines | Low | High |
| MCP repo guard on discover_root | ~5 lines | Low | High |
| Lazy compile when knowledge.db missing | ~10 lines | Low | Medium |
| **Total** | **~50 lines** | **Low** | **High** |

---

## Design

### Marker Convention

```
.samgraha/        ← directory = "this is a samgraha repo" (like .git/)
```

No file inside — just the directory. Existing code already creates `.samgraha/` at compile time (`RegistryStore::open` creates it). The `init` command creates it eagerly.

### Backward Compatibility

Three states a directory can be in:

| State | `.samgraha/` exists | `samgraha.toml` exists | Guard passes? |
|-------|---------------------|------------------------|---------------|
| **New init** | Yes (by init) | Yes (by init) | Yes |
| **Existing compiled** | Yes (by compile) | Yes (by user/setup) | Yes |
| **Existing config-only** | No | Yes | **Yes** (fallback) |
| **Plain dir** | No | No | **No** |

The guard checks `.samgraha/` first. Falls back to `samgraha.toml` for repos configured before this feature. Uninitialized directories get the error.

### CLI Flow

```
$ samgraha compile
fatal: not a samgraha repository (or any parent). Run 'samgraha init' first.

$ samgraha init
Initialized samgraha repository at /path
  created .samgraha/
  created samgraha.toml

$ samgraha compile
Compiling... ✓
```

### MCP Flow

`discover_root()` in `main.rs` currently walks up from CWD looking for `samgraha.toml` or `.git`. Update to also check for `.samgraha/`. If nothing found in parent chain and CWD also has none, return error.

### Lazy Compilation

When a guard check passes but `knowledge.db` doesn't exist (deleted or first init), auto-compile on first operation. Already partially implemented in `handle_register_repository` — extend to all entry points.

---

## Implementation

### Files Changed

| File | Change |
|------|--------|
| `crates/cli/src/commands.rs` | `execute_init`: create `.samgraha/` dir; add `check_samgraha_repo()` guard; skip guard for `init`, `version` |
| `crates/cli/src/config.rs` | `discover_repository_root()`: add `.samgraha/` check |
| `crates/mcp/src/main.rs` | `discover_root()`: add `.samgraha/` check |
| `crates/services/src/runtime/runtime.rs` | `KnowledgeRuntime::new()`: accept option to create `.samgraha/` lazily |
| `crates/cli/src/main.rs` | Add guard before command dispatch |

### Guard Function (CLI)

```rust
fn ensure_samgraha_repo() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mut current = Some(cwd.as_path());
    while let Some(dir) = current {
        if dir.join(".samgraha").is_dir() || dir.join("samgraha.toml").exists() {
            return Ok(());
        }
        current = dir.parent();
    }
    anyhow::bail!(
        "fatal: not a samgraha repository (or any parent). Run 'samgraha init' first."
    );
}
```

### Init Command Update

```rust
fn execute_init(...) -> Result<ExitCode> {
    let root = ...;
    let samgraha_dir = root.join(".samgraha");
    std::fs::create_dir_all(&samgraha_dir)?;
    // ... existing config creation ...
    println!("Initialized samgraha repository at {}", root.display());
}
```

---

## Lazy Compilation Scope

| Trigger | Behavior |
|---------|----------|
| `samgraha compile` | Always compiles (explicit) |
| `samgraha search` | Check `knowledge.db`; if missing, compile first |
| `samgraha audit` | Check `knowledge.db`; if missing, compile first |
| `samgraha info` | Check `knowledge.db`; if missing → report "not compiled" |
| `samgraha registry register` | Already auto-compiles if knowledge.db missing |
| MCP `search`/`audit`/`get_sections` | Same lazy compile on missing knowledge.db |

---

## What Doesn't Change

| Component | Reason |
|-----------|--------|
| `RegistryStore::open()` | Already creates `.samgraha/` dir + runs migrations |
| `samgraha.toml` schema | Unchanged |
| MCP tool signatures | Unchanged |
| `KnowledgeRuntime` | Unchanged — guard is at CLI/MCP entry boundary |
| DB schema | Unchanged |
| CLI output format | Unchanged — error messages use existing patterns |

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Existing repos without `.samgraha/` | Rejected by guard | Fallback to `samgraha.toml` check |
| `samgraha init` on already-init'd repo | Overwrites config | `--force` flag already exists |
| Knowledge db auto-compile too slow | Slow first search | Only compiles on missing DB, not every access |
| `.git` no longer treated as implicit marker | Workspace detection breaks | `.git` stays as fallback in discover functions |

---

## Verdict

**Feasible. ~50 lines. Low risk.**

Simple feature with high usability leverage. No schema changes, no DB migrations, no API changes. The `.samgraha/` directory is already the de facto state directory — this formalizes it as the canonical marker.
