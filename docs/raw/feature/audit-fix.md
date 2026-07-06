# Audit-Fix Pipeline

This section details the Audit-Fix Pipeline.

## Purpose

The Audit-Fix Pipeline closes the remediation loop between audit findings and verified fixes. Each finding carries a three-layer context chain (Audit Spec → Audit Standard → Document Standard) that defines what "fixed" means. The pipeline consumes this chain to produce structured fix plans and, for documentation and configuration domains, auto-apply fixes with verification.

Fix is verified by re-audit, not by assumption.

---

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Finding Context Resolution

The pipeline shall resolve the full context chain for any audit finding: the originating audit spec, its audit standard, and the target document standard.

Context resolution shall use an explicit domain-to-file-path lookup table rather than deriving paths by string formatting, because two domains break the naming convention (`feature-design-validation.md` vs `*-audit.md`, `external-context-ownership-audit.md` distinct from `external-context-audit.md`).

---

## FR2. Plan Generation per Domain Type

The pipeline shall produce structured fix plans. Plan type is determined by domain:

| Plan Type | Domains | Auto-apply? |
|-----------|---------|-------------|
| Documentation | Architecture, Vision, Engineering, Feature, Feature Technical, Design, Prototype, External Context, External Context Ownership, Feature Design, README, Coverage, Consistency | Yes |
| Configuration | Dependency (excluded — stub pipeline) | N/A — rejected before planning |
| Implementation | Implementation, Deterministic Runtime | No — phasewise plan |
| Build | Build | No — phasewise plan |
| Security | Security | No — phasewise plan |
| Test | — | No — phasewise plan |

---

## FR3. Fix Plan Structure

Every fix plan shall contain: title, summary, prerequisites, ordered steps with verification gates, and rollback instructions. Each step specifies an action, target file, rationale, detail, verification criterion, and rollback.

---

## FR4. Auto-Apply for Documentation and Configuration

For Documentation and Configuration plan types, the pipeline shall automatically apply the fix to the target file. The fix must be bounded and structured such that verification is deterministic (re-run the same check).

---

## FR5. Phasewise Plan for Code Changes

For Implementation, Build, Security, and Test plan types, the pipeline shall produce a phasewise plan stored in SQLite, rendered via a per-type template. The user executes each step manually; the verifier checks each step's verification gate before allowing the next.

---

## FR6. Verification Loop

After a fix attempt, the pipeline shall re-run the failed check plus any dependent checks. If score ≥ 9/10, mark Fixed. If score < 9 and attempts < 3, feed verification details back to the planner and retry. If score < 9 and attempts ≥ 3, mark for human review.

---

## FR7. Dependent Check Re-Run

The pipeline shall re-run checks that depend on the fixed check. Dependency edges must be keyed by `(domain, check_id)` pairs because check-ID prefixes collide across domains (e.g., `EC1–EC12` exists in both External Context and External Context Ownership; `D1–D12` in Design collides with `D1–D8` in Dependency).

---

## FR8. Cached Context with Invalidation

The context chain shall be cacheable across findings in the same session. After any auto-write to a target file, the cached parse for that path must be invalidated to prevent stale planning.

---

## FR9. MCP Interface

All fix operations shall be accessible through the MCP interface, not through a CLI command.

---

## Business Rules

- Fix is verified by re-audit, not by assumption.
- The context chain defines what "fixed" means: Audit Spec → Audit Standard → Document Standard.
- Auto-apply is limited to Documentation and Configuration plan types.
- Code changes (Implementation, Build, Security, Test) produce phasewise plans — never auto-writes.
- Each finding fix attempts independently (no cross-domain fix transactions).
- Human review is required when verification fails after 3 attempts.
- Dependency domain is excluded from v1 — its pipeline is currently a stub.
- MCP-only in v1; no `samgraha fix` CLI command.

---

## Fix Lifecycle

```
Finding
    │
    ▼
PlanningContextBuilder → loads audit spec + audit standard + doc standard + target
    │
    ▼
PlanningContext + Intent
    │
    ▼
FixPlanner → produces FixPlan
    │
    ▼
Executor → applies FixPlan (doc/config auto-write; others render plan)
    │
    ▼
Verifier → re-runs failed check + dependents
    ├─ score ≥ 9 → mark Fixed
    ├─ score < 9, attempts < 3 → feedback → FixPlanner refines
    └─ score < 9, attempts ≥ 3 → needs_human_review
```

---

## Inputs

The Audit-Fix Pipeline consumes:

- Audit finding (report_id, criterion_id, domain, score)
- Audit Spec (`docs/raw/audit/<domain>-audit.md` via lookup table)
- Audit Standard (`docs/raw/audit-standards/<domain>/*.md`)
- Document Standard (`docs/raw/standards/<domain>.md`)
- Target file (document / config / source)
- Verification feedback from prior attempts

---

## Outputs

The pipeline produces:

- `FixPlan` with title, summary, prerequisites, ordered steps, rollback
- Auto-applied edits (doc/config only)
- Rendered plan markdown (non-auto-apply types)
- `Verdict` with per-check scores
- `FixSession` with attempt history

Outputs are stored in SQLite (`fix_sessions`, `fix_attempts`, `fix_plans`, `fix_plan_steps`).

---

## Constraints

The Audit-Fix Pipeline shall:

- operate entirely offline
- require no AI models
- never modify source code, build files, test files, or security config automatically
- enforce a maximum of 3 fix attempts per finding session
- scope each session to a single finding within a single domain
- validate that all target paths reside under the repository root
- be cacheable within a session (context reuse across findings)
- invalidate cache after any auto-write to a cached path

---

## Dependencies

The Audit-Fix Pipeline depends upon:

- Audit Framework (reports, findings, scoring)
- Documentation Standards (defines the compliance target)
- Audit Specs (defines checks and procedures)
- KnowledgeRuntime (`run_single_check()` — net-new)
- SQLite Registry (`fix_sessions`, `fix_attempts`, `fix_plans`, `fix_plan_steps` tables)
- MCP Runtime (tool registration and dispatch)
- Template engine (reporting.rs — plan rendering)
- `toml_edit` (round-trip safe TOML manipulation)
- `serde_yaml` (YAML manipulation)

---

## Non-Goals

The Audit-Fix Pipeline does not:

- auto-commit changes (user commits manually)
- auto-write code, build, test, or security changes
- run parallel fix sessions
- provide a `samgraha fix` CLI command
- support cross-domain fix transactions
- include the Dependency domain (currently a stub pipeline)
- execute rollback automatically (rollback is documented text)
- require AI providers
- require internet connectivity

---

## Future Extensions

The pipeline should support:

- Migration plans (`Intent::Migrate`)
- Upgrade plans (`Intent::Upgrade`)
- Refactoring plans (`Intent::Refactor`)
- Auto-rollback for doc/config fixes (snapshot pre-write content)
- Concurrency guard on auto-write executors (mtime check before write)
- Philosophy/guiding-principles context for DocPlanner

---

## Acceptance Criteria

The feature is successful when:

- a finding's context chain is resolved correctly, including the two edge-case domains that break the naming convention
- documentation fixes auto-apply and pass re-audit at score ≥ 9/10 within 3 attempts
- configuration fixes auto-apply and pass re-audit at score ≥ 9/10 within 3 attempts
- implementation/build/security/test findings produce a phasewise plan in SQLite, rendered to markdown, with per-step verification gates
- dependent checks re-run after a fix and their results are reflected in the verdict
- verification loop terminates (pass ≤ 3 attempts, or needs_human_review)
- all operations succeed via MCP tools only
- Dependency domain is excluded from fix operations (no attempt to fix stub findings)

---

## Traceability

This feature derives from the following Vision commitments:

- **Audit before trust.**
- **Standards define contracts; audits verify compliance.**
- **Knowledge should be verified before delivery.**
- **Audit is a first-class engineering capability.**

**Traceability**

Vision → Feature: Audit Framework → Feature: Audit-Fix Pipeline
