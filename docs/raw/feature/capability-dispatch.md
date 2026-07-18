# Capability Dispatch

## Purpose

Capability Dispatch is the mechanism by which samgraha delegates audit, calculation, reporting, scaffolding, and plan generation to external system scripts. Instead of hardcoding domain-specific logic in Rust, samgraha discovers and invokes scripts provided by registered knowledge systems.

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Capability Abstraction

The dispatch system shall define six canonical capabilities:

| Capability | Purpose |
|---|---|
| `validate` | Run audit checks against documents |
| `calculate` | Compute derived metrics or scores |
| `report` | Generate human-readable or machine-readable reports |
| `scaffold` | Generate boilerplate documentation from templates |
| `plan_generation` | Generate phased project plans |
| `init` | Initialize a knowledge system's local state |

Each capability has a defined input/output contract. Scripts implement one or more capabilities.

---

## FR2. Four-Tier Script Discovery

The dispatch system shall discover scripts through four tiers, in priority order:

1. **System scripts** — registered via `system.yaml` in a knowledge system's directory
2. **Standard scripts** — bundled with the documentation standard
3. **Override scripts** — user-provided scripts in the repository
4. **Built-in fallback** — Rust-compiled logic (existing pipeline modules)

The first tier that provides a matching script wins. If no script is found, the built-in fallback executes.

---

## FR3. Script Input/Output Contract

All scripts shall receive standardized command-line arguments:

- `--repo-root <path>` — repository root directory
- `--in <path>` — JSON input file (target, options, context)
- `--out <path>` — path where script writes JSON output

Scripts shall write a JSON status envelope to `--out`:

```json
{
  "status": "pass|fail|error",
  "score": 85.0,
  "findings": [...],
  "metadata": {...}
}
```

---

## FR4. MCP Tool Interface

Five dedicated MCP tools plus one generic catch-all shall expose capability dispatch:

| Tool | Capability |
|---|---|
| `run_system_validate` | validate |
| `run_system_calculate` | calculate |
| `run_system_report` | report |
| `run_system_scaffold` | scaffold |
| `run_system_plan_generation` | plan_generation |
| `run_system_script` | any (generic catch-all) |

All tools accept `system_id`, `domain`, and optional `target`, `options`, and `phase_id` parameters.

---

## FR5. Prerequisite Gating

Before executing a script, the dispatch system shall check phase prerequisites:

- Verify `depends_on` dependencies are satisfied (script_runs table)
- Evaluate TTL and head_commit expiry rules
- Return blocked-precondition JSON if prerequisites are not met

This prevents stale scripts from running when upstream dependencies have changed.

---

## FR6. Script Run Tracking

Every script invocation shall be recorded in the `script_runs` table with:

- system_id, capability, domain
- input hash, output hash
- exit_code, duration_ms
- git head_commit, created_at

This provides an audit trail and enables cache invalidation.

---

## Business Rules

- Scripts are sandboxed to the repository root — they cannot access paths outside `--repo-root`.
- Script failures are non-fatal to the platform — the built-in fallback executes.
- Script output is validated against the expected schema before persistence.
- Scripts may be async (background) or sync (blocking), declared in `system.yaml`.
- Multiple systems may provide scripts for the same domain — the highest-priority system wins.

---

## Non-Goals

- Capability Dispatch does not:
  - replace the built-in Rust pipeline modules (they serve as fallback)
  - provide a scripting language or DSL
  - manage script installation or dependency resolution
  - execute untrusted code without user consent

---

## Acceptance Criteria

The feature is successful when:

- a registered system's `validate` script is invoked instead of the Rust pipeline
- script discovery correctly prioritizes system > standard > override > built-in
- script failures gracefully fall back to built-in logic
- all script runs are tracked in `script_runs`
- prerequisite gating prevents stale script execution
- MCP tools expose all five capabilities plus the generic catch-all

---

## Traceability

This feature derives from the following Vision commitments:

- **Extensibility without modification.**
- **Systems own their validation logic.**
- **Samgraha orchestrates; systems implement.**

**Traceability**

Vision → Feature: Capability Dispatch
