# Knowledge Repository Abstraction: Architecture Proposal

## 1. The Core Concept

Instead of thinking about a generic "Repository" and a floating "Knowledge System", Saṃgraha will recognize two distinct first-class repository types:

1. **Repository** (Normal software, infrastructure, or docs project)
2. **Knowledge Repository** (A repository whose sole purpose is authoring and publishing Knowledge Systems)

This formalizes the mental model of having a repository registry and a knowledge repository. It keeps the "local-first, explicit, deterministic" philosophy while drastically simplifying the CLI and configuration surface.

---

## 2. Configuration Model

Using `kind` instead of `type` reduces terminology overload, since "type" is already heavily used for documents, sections, and templates.

### Repository
A normal repository consumes exactly one active Knowledge System. This remains immutable per repository—no inheritance, no overlays, no composition.

```toml
[repository]
kind = "repository" # Default behavior if omitted

[documentation]
system = "dev" # Specifies the active Knowledge System
```

### Knowledge Repository
A knowledge repository produces and publishes one or more Knowledge Systems for other repositories to consume.

```toml
[repository]
kind = "knowledge"

[knowledge]
root = "system"
```

The `root` directory (e.g., `system/`) will contain multiple directories representing individual systems. The loader will automatically discover these without a central manifest like `registry.toml`.

---

## 3. Knowledge Repository Structure

To formalize the discovery, each subdirectory is a complete, independent Knowledge System. 

```text
knowledge-repo/
    samgraha.toml
    system/
        dev/
            system.toml
            standards/
            audit/
            templates/
            calculation/
            plan/
            script/
        academic/
            system.toml
            standards/
            ...
```

### The System Identity Manifest
While we don't need a central repository manifest for discovery, each individual Knowledge System directory should contain a small identity manifest (`system.toml`) for versioning, publishing, display, and validation:

```toml
# system/dev/system.toml
id = "dev"
name = "Software Development"
version = "1.0"
description = "General software engineering documentation."
```

---

## 4. Lifecycle & Workflow Differences

The separation creates two distinct lifecycle workflows and cleanly aligns the CLI verbs:
- `registry` manages the local registry of dependency repositories (unchanged from before this proposal — not renamed to `repository`, see §6.2).
- `knowledge` manages knowledge systems.

**Repository:**
`clone` → `samgraha registry sync` → `samgraha compile` → `samgraha audit`

**Knowledge Repository:**
`clone` → `samgraha knowledge publish` → `done`
*(Note: `knowledge publish` will internally invoke `compile` to build the Knowledge Packages before publishing to the Registry).*

---

## 5. Repository Matrix

This table summarizes the capabilities of each repository kind:

| Capability        | Repository | Knowledge Repository |
| ----------------- | ---------- | -------------------- |
| Compile           | ✅          | ✅                    |
| Audit             | ✅          | ✅                    |
| Search            | ✅          | ❌                    |
| MCP Runtime       | ✅          | ❌                    |
| Publish Knowledge | ❌          | ✅                    |
| Sync Knowledge    | ✅          | ✅                    |

---

## 6. Current Implementation Gaps & Optimization Analysis

Before implementing this proposal, we must address the following gaps in the current `samgraha` codebase:

### 6.1. Configuration Schema (`crates/common/src/config.rs`)
- **Gap**: `RepositoryConfig` currently lacks a `kind` field.
- **Optimization**: Introduce an enum `RepositoryKind` (`Repository`, `Knowledge`) defaulting to `Repository`.
- **Optimization**: Introduce a `knowledge.root` configuration field. Keep the fallback `default = "dev"` mechanism so a Repository without an explicit `system` specified knows what to pick.

### 6.2. CLI Command Restructuring (`crates/cli/src/commands.rs`)
- **Gap**: The CLI currently uses `samgraha standards register` (overloading "register").
- **Optimization**: Move to `samgraha knowledge publish` and `samgraha knowledge pull`. The existing `samgraha registry register`/`sync`/`status` group (managing this repo's local registry of dependency repositories) is retained under its existing `registry` name rather than renamed to `repository` — that verb predates this proposal and names a different concept (dependency-registry membership, not repository *kind*), and renaming a shipped CLI verb is a breaking change with no functional upside here.

### 6.3. Pipeline Selection via Factory
Instead of bypassing logic with `if knowledge` scattered everywhere, `compile` and `audit` should use a `PipelineFactory` to select the right pipelines based on the repository `kind`.

**Compile:**
Same command, different compiler pipeline output.
- *Repository*: `Compile` → `Knowledge Database`
- *Knowledge Repository*: `Compile` → `Knowledge Package`

**Audit:**
Pipelines are selected automatically based on `kind`.
- *Repository*: Runs Documentation, Implementation, Build, and Security Audits.
- *Knowledge Repository*: Runs Documentation and Knowledge System Audits.

### 6.4. Conceptual Clarity: System vs. Package
We formalize the distinction between a **Knowledge System** (the raw directory of standards and templates) and a **Knowledge Package** (the compiled artifact produced by `samgraha compile`).

---

## 7. Architectural Extensibility

While out of scope for the immediate refactor, this architectural change makes the system capable of supporting future repository types trivially via the `RepositoryKind` enum:
- `Template Repository`
- `Plugin Repository`

By decoupling the *kind* of repository from the *action* (compile/audit), the platform remains clean, intuitive, and highly extensible without introducing complex multi-system overrides or inheritance hierarchies.

---

## 8. Final Architecture

The resulting architecture cleanly separates the responsibilities:

* **Repository** consumes.
* **Knowledge Repository** produces.
* **Global Registry** distributes.

```text
                   Saṃgraha
                        │
        ┌───────────────┴───────────────┐
        │                               │
        ▼                               ▼
   Repository                  Knowledge Repository
        │                               │
        ▼                               ▼
One Knowledge System        Many Knowledge Systems
        │                               │
        ▼                               ▼
 Knowledge Database        Knowledge Packages
        │                               │
        └───────────────┬───────────────┘
                        ▼
             Global Knowledge Registry
```

---

## 9. Detailed Implementation Plan

The rollout of this architecture will be executed in four primary phases, minimizing disruption and ensuring each structural concept is solid before wiring it into the CLI.

### Phase 1: Core Configuration and Identity Models
**Objective:** Teach the underlying config and schema layers about Repository Kinds and Knowledge System Identity without changing execution logic yet.
*   **Update Configuration Schema (`crates/common/src/config.rs`)**:
    *   Add `RepositoryKind` enum (`Repository`, `Knowledge`).
    *   Update `RepositoryConfig` to include `kind` (defaulting to `Repository`) and `knowledge: Option<KnowledgeConfig>` containing `root: String` (default `"system"`).
*   **Introduce System Identity (`crates/schemas`)**:
    *   Create a schema for `system.toml` (e.g., `KnowledgeSystemIdentity` struct with `id`, `name`, `version`, `description`).

### Phase 2: Knowledge System Loader
**Objective:** Enable the platform to discover, load, validate, and build the in-memory model of multiple systems.
*   **Update Discovery (`crates/compiler/src/discovery.rs` or new module)**:
    *   Add logic that triggers when `kind == Knowledge`.
    *   Traverse `knowledge.root` (e.g., `system/`) to find all subdirectories containing a valid `system.toml`.
    *   Load and validate the identity and structure (`standards/`, `audit/`, etc.) of each discovered system.

### Phase 3: Pipeline Factory & Compilation Separation
**Objective:** Teach the `compile` and `audit` operations to select pipelines cleanly using a factory pattern.
*   **Pipeline Selection (`crates/services/src/compilation.rs` & `crates/audit/src/lib.rs`)**:
    *   Implement a `PipelineFactory` that receives the `RepositoryKind` and yields the correct list of audit/compilation pipelines.
    *   *Repository*: Compile generates a `Knowledge Database`. Audit runs Docs, Code, Build, Security.
    *   *Knowledge*: Compile generates a `Knowledge Package`. Audit runs Docs and Knowledge System integrity checks.

### Phase 4: CLI Alignment and Workflow Finalization
**Objective:** Expose the new capabilities to the user with the refined verb semantics.
*   **CLI Overhaul (`crates/cli/src/commands.rs`)**:
    *   Add `samgraha knowledge` command group.
    *   Implement `samgraha knowledge publish` (invokes `compile`, validates, and pushes all discovered Knowledge Packages to the Global Registry).
    *   Implement `samgraha knowledge pull`.
    *   Keep `samgraha registry` verbs intact for Repositories (see §6.2 — not renamed to `repository`).
    *   Deprecate/Remove the old `samgraha standards register` command. — **Done**: removed; `StandardsAction` no longer has a `Register` variant.
    *   Gate `knowledge publish` behind `RepositoryKind::Knowledge` (Repository Matrix §5: Publish Knowledge is ❌ for a plain Repository). — **Done**, `crates/cli/src/commands.rs::execute_knowledge`.
*   **Documentation & Testing**:
    *   Update boilerplate (e.g., initial `samgraha.toml` template) to explicitly include `kind = "repository"`. — **Done**: `RepositoryConfig.kind` has no `skip_serializing_if`, so `samgraha init` always emits it.
    *   Add E2E tests for both standard project flows and knowledge publishing flows. — **Done**: multi-system discovery + missing-directory-warning tests in `tests/tests/e2e_compile.rs`; MCP Repository Matrix gate tests in `crates/mcp/src/adapter.rs`.
