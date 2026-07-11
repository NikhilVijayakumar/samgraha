# Audit-Fix Pipeline — Feature Technical Design

This section details the Audit-Fix Pipeline — Feature Technical Design.

## Purpose

This document describes the architectural realization of the Audit-Fix Pipeline feature.

The Audit-Fix Pipeline closes the remediation loop between audit findings and verified fixes. It consumes the three-layer context chain (Audit Spec → Audit Standard → Document Standard) that defines what "fixed" means, produces structured fix plans, and for documentation and configuration domains auto-applies fixes with verification.

This document applies the architectural principles defined in Component Model and Knowledge Flow.

---

## Feature Specification

- **Feature:** docs/raw/feature/audit-fix.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/knowledge-flow.md

---

## Participating Components

This section details the Participating Components.

### Fix Orchestrator

The Fix Orchestrator composes the four-stage pipeline: PlanningContextBuilder → FixPlanner → Executor → Verifier. It manages the ≤3 attempt verification loop and feeds verification feedback back to the planner.

### PlanningContextBuilder

The PlanningContextBuilder resolves the context chain for a finding: loads and parses the audit spec, audit standard, and document standard for the finding's domain. Maintains a domain-to-file-path lookup table for the two domains that break the naming convention (`feature-design-validation.md`, `external-context-ownership-audit.md`). Caches parsed contexts within a session, invalidating after any auto-write to the cached path.

### FixPlanner

FixPlanner implementations produce structured FixPlan objects. One implementation per plan type: DocPlanner, ConfigPlanner, ImplPlanner, BuildPlanner, SecPlanner, TestPlanner. Each receives a PlanningContext and an Intent and returns a FixPlan with ordered steps.

### Executor

Executor implementations apply FixPlan steps. DocExecutor and ConfigExecutor auto-write to the target file. PlanExecutor serializes the plan to SQLite and renders it via a per-type template for manual user execution.

### Verifier

The Verifier re-runs the failed check plus dependent checks after a fix attempt. Dependency edges are keyed by `(domain, check_id)` pairs to avoid cross-domain collisions. Returns a Verdict with per-check scores.

### Audit Framework

The Audit Framework produces the reports and findings that the Fix Pipeline consumes. The pipeline reads finding data from existing reports and does not re-audit the full domain.

### Knowledge Runtime

The Knowledge Runtime exposes `run_single_check()` — net-new functionality that extracts a single check from a pipeline and executes it independently, returning the check's score. Also exposes `apply_finding_fix()` and `generate_fix_plan()` as public methods.

### Template Engine

The Template Engine (reporting.rs) renders FixPlan data into readable markdown documents using the same Tera template syntax as report templates. Six plan templates at `docs/raw/fix-plan-templates/`.

### SQLite Registry

The SQLite Registry stores fix sessions, attempts, plans, and plan steps in four new tables (V28 migration).

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Fix Orchestrator | Compose pipeline stages, manage ≤3 attempt loop, route feedback |
| PlanningContextBuilder | Resolve context chain, domain→file lookup, cache with invalidation |
| FixPlanner | Produce FixPlan from PlanningContext + Intent |
| Executor | Apply FixPlan (auto-write doc/config, serialize plan for others) |
| Verifier | Re-run failed + dependent checks, return Verdict |
| Audit Framework | Source of reports and findings |
| Knowledge Runtime | Expose `run_single_check()`, `apply_finding_fix()`, `generate_fix_plan()` |
| Template Engine | Render FixPlan to markdown via per-type template |
| SQLite Registry | Persist sessions, attempts, plans, steps |

---

## Component Interactions

```text
MCP Tool Call
    │
    ▼
Fix Orchestrator
    │
    ├── PlanningContextBuilder
    │       ├── Audit Spec (docs/raw/audit/<domain>-audit.md)
    │       ├── Audit Standard (docs/raw/audit-standards/<domain>/*.md)
    │       ├── Document Standard (docs/raw/documentation-standards/<domain>.md)
    │       └── Target file
    │
    ├── FixPlanner → produces FixPlan
    │
    ├── Executor
    │       ├── DocExecutor (auto-write .md)
    │       ├── ConfigExecutor (auto-write .toml/.yaml/.json)
    │       └── PlanExecutor (serialize to SQLite → render template)
    │
    └── Verifier
            ├── KnowledgeRuntime.run_single_check()
            └── Dependency graph (domain-qualified check IDs)
```

### Fix Execution Flow

1. MCP tool `audit_fix_plan` invoked with `{report_id, criterion_id}`.
2. FixOrchestrator receives the finding.
3. PlanningContextBuilder resolves target path and loads context chain.
4. Intent is derived from the finding (`RestoreCompliance`).
5. FixPlanner produces a FixPlan with ordered steps.
6. Executor applies the plan (auto-write or plan record + render).
7. Verifier re-runs the failed check + dependents via `run_single_check()`.
8. If score ≥ 9, session marked passed, finding marked Fixed.
9. If score < 9 and attempts < 3, feedback enriches PlanningContext, repeat from step 5.
10. If score < 9 and attempts ≥ 3, session marked needs_human_review.

---

## Runtime Behavior

This section details the Runtime Behavior.

### Fix Session Lifecycle

```
Finding Received
    │
    ▼
PlanningContext Built (cached, may invalidate on write)
    │
    ▼
Plan Generated → stored in fix_plans (status=draft)
    │
    ▼
Plan Rendered → returned to caller as markdown
    │
    ▼
[Auto-apply path]
    Executor writes target → Verifier runs checks
    │
    ▼
[Phasewise path]
    User executes steps one by one:
        audit_fix_plan_execute_step → Verifier gates each step
        All steps verified → plan complete
    │
    ▼
Verdict:
    ├─ score ≥ 9 → fix_sessions.status = passed → finding = Fixed
    ├─ score < 9, < 3 attempts → feedback → re-plan
    └─ score < 9, ≥ 3 attempts → fix_sessions.status = needs_human_review
```

### Caching Behavior

PlanningContextBuilder caches parsed context by domain within a session. When DocExecutor or ConfigExecutor writes to a file, the cache entry for that file's domain is invalidated. Subsequent findings in the same domain will re-parse from disk.

---

## Communication Paths

This section details the Communication Paths.

### MCP → Fix Orchestrator

MCP tools invoke the FixOrchestrator with finding parameters. The orchestrator returns session IDs, plan markdown, step verification results, and final verdicts.

### PlanningContextBuilder → Standards Files

The builder reads and parses three files per domain: audit spec, audit standard, document standard. Uses an explicit domain→file lookup table for path resolution.

### FixPlanner → PlanningContext

The planner reads the assembled context to understand the compliance target, the current state of the target file, and prior verification feedback.

### Executor → Target File

DocExecutor and ConfigExecutor read and write the target file directly (auto-apply). PlanExecutor writes to SQLite only (no file mutation).

### Verifier → KnowledgeRuntime

The verifier calls `KnowledgeRuntime.run_single_check(domain, check_id)` for the failed check and each dependent check. KnowledgeRuntime dispatches to the appropriate pipeline and returns the single check's score.

---

## Data Ownership

| Data | Owner | Fix Pipeline Access |
|---|---|---|
| Audit Report | Audit Framework | Read |
| Finding | Audit Framework | Read |
| Audit Spec | Documentation Standards | Read |
| Audit Standard | Documentation Standards | Read |
| Document Standard | Documentation Standards | Read |
| Target file | Repository | Read + Write (doc/config only) |
| Fix Session | Fix Pipeline | Write (to Registry) |
| Fix Plan | Fix Pipeline | Write (to Registry) |
| Fix Attempt | Fix Pipeline | Write (to Registry) |
| Plan Template | Documentation | Read |

---

## Integration Points

This section details the Integration Points.

### Audit Framework

The pipeline reads existing audit reports and findings. No modification to the audit framework itself is required.

### Documentation Standards

The pipeline reads audit specs, audit standards, and document standards to build planning context. Standards are the authoritative definition of what "fixed" means.

### Knowledge Runtime

The pipeline requires `run_single_check()` — a new method that extracts and runs a single check from a pipeline. Also adds `apply_finding_fix()` and `generate_fix_plan()` as public entry points.

### Template Engine

The template engine is extended to support PlanContext rendering alongside existing ReportContext rendering. Six new templates at `docs/raw/fix-plan-templates/`.

### SQLite Registry

Four new tables added via V28 migration. Existing CRUD patterns in store.rs are extended for the new tables.

---

## Domain-to-File Lookup Table

The PlanningContextBuilder uses this explicit mapping instead of deriving paths by string formatting:

| Domain | Audit Spec File |
|---|---|
| architecture | `docs/raw/audit/architecture-audit.md` |
| vision | `docs/raw/audit/vision-audit.md` |
| engineering | `docs/raw/audit/engineering-audit.md` |
| feature | `docs/raw/audit/feature-audit.md` |
| feature_technical | `docs/raw/audit/feature-technical-audit.md` |
| design | `docs/raw/audit/design-audit.md` |
| prototype | `docs/raw/audit/prototype-audit.md` |
| external_context | `docs/raw/audit/external-context-audit.md` |
| external_context_ownership | `docs/raw/audit/external-context-ownership-audit.md` |
| feature_design | `docs/raw/audit/feature-design-validation.md` | ⚠ not `*-audit.md` |
| readme | `docs/raw/audit/readme-audit.md` |
| build | `docs/raw/audit/build-audit.md` |
| security | `docs/raw/audit/security-audit.md` |
| implementation | `docs/raw/audit/implementation-audit.md` |
| coverage | `docs/raw/audit/coverage-audit.md` |
| consistency | `docs/raw/audit/consistency-audit.md` |
| deterministic_runtime | `docs/raw/audit/deterministic-runtime-audit.md` |

---

## External Dependency Integration

The Audit-Fix Pipeline operates entirely offline. No external services participate in planning, execution, or verification.

New dependencies (crate-level):
- `toml_edit` — round-trip safe TOML manipulation for ConfigExecutor
- `serde_yaml` — YAML manipulation for ConfigExecutor

---

## Runtime Constraints

- Fix sessions are single-finding, single-domain — no cross-domain transactions.
- Maximum 3 fix attempts per session.
- Auto-write is limited to `.md`, `.toml`, `.yaml`, `.json` files.
- Phasewise plans require manual step-by-step execution by the user.
- Verification re-runs the failed check + its transitive dependents.
- Dependency graph resolution is bounded by the audit spec's cross-reference section.

---

## Architectural Constraints

- The pipeline must not modify the Audit Framework's report or finding data structures.
- Auto-write executors must not bypass the PlanningContext's cache invalidation.
- The Verifier must use `(domain, check_id)` pairs — bare check IDs are ambiguous.
- MCP is the sole interface for v1 — no CLI command.
- `audit_fix_accept` and `audit_fix_reject` must delegate to the existing `update_finding_status` MCP tool, not duplicate the write path.

---

## Security Considerations

- All target paths must be canonicalized and validated to reside under the repository root before any write.
- Auto-write executors should hash-check or mtime-check the target immediately before writing to detect concurrent user edits (future optimization).
- No secrets or credentials are handled by the fix pipeline.
- The pipeline is read-only for non-doc/config domains — no code or config is auto-mutated.

---

## Performance Considerations

- Context caching avoids redundant file I/O across findings in the same session.
- `run_single_check()` is significantly cheaper than full pipeline re-execution.
- Cache invalidation is keyed by target file path, not domain — narrow scope avoids unnecessary re-parsing.
- Plan rendering through the template engine is sub-millisecond for typical plan sizes.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Context file missing (audit spec / audit standard / doc standard) | Report error per missing file, abort session |
| Target file not found | Report error, abort session |
| Auto-write target outside repo root | Reject with security error, abort |
| `run_single_check()` fails (pipeline not found / check not found) | Report error, abort current attempt |
| Verdict score < 9 after 3 attempts | Mark session `needs_human_review`, leave finding Open |
| Step verification fails (phasewise plan) | Mark step failed, pause plan, allow `audit_fix_plan_update` or retry |
| Registry write failure | Return error to caller, session state lost |

---

## Extension Points

This section details the Extension Points.

### FixPlanner Implementations

New plan types implement the `FixPlanner` trait. Registration happens through the FixOrchestrator's planner registry.

### Executor Implementations

New executor types implement the `Executor` trait. The orchestrator selects executor by plan type.

### Intent Variants

New intents (Migration, Upgrade, Refactor) extend the `Intent` enum without changing the FixPlanner trait signature. Each new intent needs a way to build the context and a FixPlanner implementation that understands it — the orchestrator, executor, and verifier remain unchanged.

---

## Traceability

This document derives from:

- Feature: Audit-Fix Pipeline
- Architecture: Component Model
- Architecture: Knowledge Flow
- Feature Technical: Audit Framework

This document provides technical context for:

- Implementation Phase 1–5

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
