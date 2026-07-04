# Context Manager

This section details the Context Manager.

## Purpose

The ContextManager owns the Knowledge Context lifecycle, independent of any transport adapter connection.

It decouples context lifetime from MCP connection lifetime, enabling context reuse across reconnects, sharing across tools such as CLI, MCP, and Inspector, and graceful handling of disconnect and reconnect without expensive replanning.

Transport adapters access contexts through the ContextManager. They do not own, create, or dispose contexts.

---

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Context Ownership

The ContextManager shall create, own, and dispose Knowledge Contexts.

Transport adapters access contexts but never own them.

Context creation is always initiated by the ContextManager. No external component constructs a KnowledgeContext directly.

---

## FR2. Lifecycle Management

The ContextManager shall manage the full Knowledge Context lifecycle.

The lifecycle states are:

* **Active** — one or more clients are connected; the context is serving requests.
* **Inactive** — no clients are connected; the TTL countdown is running.
* **Reuse** — a client reconnects within the TTL window and the repo revision is unchanged; the existing context is returned without rebuilding.
* **Rebuild** — a client reconnects after TTL expiry or a repo revision change; the ContextManager rebuilds the context transparently.
* **Dispose** — the TTL expires while Inactive, or an explicit close is requested; the context is released.

Rebuild creates a new context. The old context continues serving until the rebuild completes.

---

## FR3. Connection Tracking

The ContextManager shall track the active connection count.

On each client connect, the ContextManager increments the connection count and transitions the context to Active if it was Inactive.

On each client disconnect, the ContextManager decrements the connection count. When the count reaches zero, the context transitions to Inactive and the TTL countdown begins.

---

## FR4. Revision-Based Invalidation

On reconnect, the ContextManager shall check the revision of each repository in the Knowledge Plan against the cached revision.

If any repository revision has changed, the ContextManager rebuilds the context regardless of remaining TTL.

If all repository revisions are unchanged and the TTL has not expired, the ContextManager reuses the existing context.

---

## FR5. Rebuild on Expiry

When the context is invalid due to TTL expiry or revision change, the ContextManager shall rebuild transparently.

Callers receive a valid context. The rebuild is not visible to the caller beyond latency.

---

## FR6. Tool Sharing

CLI, MCP, and future tools shall all access the same ContextManager.

The cost of Knowledge Context assembly is paid once per process, not once per tool or once per connection.

---

## FR7. Single to Multi Context Path

Phase 8 provides one context slot. Phase 9 extends to named contexts identified by a context key.

The ContextManager interface is stable across both phases. Phase 9 adds a key parameter without breaking existing callers.

---

## Business Rules

* Only the ContextManager creates KnowledgeContext objects.
* Transport adapters call `context_manager.active()` and never hold or dispose contexts directly.
* One ContextManager per process.
* Context is rebuilt, not modified. Rebuild creates a new context; the old one serves until the rebuild completes.
* TTL is measured from the moment the last client disconnects, not from context creation.

---

## Context Lifecycle

```text
                    ┌─────────────┐
                    │   Create    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
          ┌────────▶│   Active    │◀────────┐
          │         └──────┬──────┘         │
          │                │ last client    │
          │                │ disconnects    │
          │                ▼                │
          │         ┌─────────────┐         │
          │         │  Inactive   │         │
          │         └──────┬──────┘         │
          │                │                │
          │       ┌────────┴────────┐       │
          │       │                 │       │
          │  reconnect          reconnect   │
          │  within TTL,        after TTL   │
          │  revision ok        or revision │
          │       │             changed     │
          │       ▼                 │       │
          │   ┌───────┐        ┌────▼────┐  │
          └───│ Reuse │        │ Rebuild │──┘
              └───────┘        └─────────┘
                                    │
                           TTL expires while
                           Inactive, or explicit
                           close requested
                                    │
                                    ▼
                             ┌─────────────┐
                             │   Dispose   │
                             └─────────────┘
```

---

## Inputs

The ContextManager consumes:

* connect and disconnect signals from transport adapters
* Knowledge Plan produced by the Knowledge Planner
* repository revision metadata from each registered repository
* TTL configuration from `samgraha.toml`

---

## Outputs

The ContextManager produces:

* a valid KnowledgeContext on each `active()` call
* lifecycle state transitions (Active, Inactive, Rebuild, Dispose)

---

## Constraints

The ContextManager shall:

* hold at most one context slot in Phase 8
* guarantee callers always receive a valid context or a clear error
* rebuild without blocking callers longer than the rebuild duration
* operate within a single process; no cross-process context sharing
* be the sole owner of KnowledgeContext objects

---

## Dependencies

The ContextManager depends upon:

* Knowledge Runtime (session.rs — KnowledgeContext)
* Knowledge Planner (produces the Knowledge Plan consumed during context assembly)
* Knowledge Resolution (assembles the Knowledge Package into the context)

The ContextManager is used by:

* McpAdapter
* Engineering CLI
* Future transport adapters

---

## Non-Goals

The ContextManager does not:

* implement knowledge search, audit, or compilation
* manage transport adapter lifecycle
* persist context state to disk
* enforce audit policy on knowledge delivery

Those responsibilities belong to other platform components.

---

## Future Extensions

* Named contexts (Phase 9): `context_manager.active(key: &str)` with a HashMap of context slots.
* Context persistence to disk: survive process restart without full rebuild.
* Distributed context sharing: share a context across multiple processes or machines.

---

## Acceptance Criteria

The feature is successful when:

* a context survives an MCP disconnect and is reused on reconnect within TTL without rebuilding
* CLI and MCP share the same context without incurring a second assembly cost
* a repository revision change triggers a rebuild and no stale data is served
* TTL expiry while Inactive disposes the context cleanly with no resource leak
* callers always receive a valid context or a clear error, never a partially assembled context

---

## Traceability

This feature derives from the following Vision commitments:

* **Knowledge assembly cost is paid once per workspace, not per consumer.**
* **Transport adapters are delivery mechanisms, not knowledge owners.**
* **Context lifetime is independent of connection lifetime.**

**Traceability**

Vision → Feature: Context Manager
