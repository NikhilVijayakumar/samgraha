# Capability Dispatch

## Purpose

Capability Dispatch is the mechanism by which samgraha delegates audit, calculation, reporting, scaffolding, and plan generation to external system scripts. Instead of hardcoding domain-specific logic in Rust, samgraha discovers and invokes scripts provided by registered knowledge systems.

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Capability Abstraction

The dispatch system shall define six canonical capabilities:

| Capability | Serialized as | Purpose |
|---|---|---|
| `validate` | `validate` | Run audit checks against documents |
| `calculate` | `calculate` | Compute derived metrics or scores |
| `report` | `report` | Generate human-readable or machine-readable reports |
| `scaffold` | `scaffold` | Generate boilerplate documentation from templates |
| `plan_generation` | `plan-generation` (hyphenated) | Generate phased project plans |
| `init` | `init` | Return a system's phase-wise plan (§8.4) |

Each capability has a defined input/output contract (§8.1/§8.2 of
`generic-script-architecture-proposal.md`). A system's script for a given
capability is discovered by name (see FR2), never declared inside
`system.yaml`.

---

## FR2. Four-Tier Script Discovery

The dispatch system shall discover scripts through four tiers, in priority order:

1. **Override** — `check_overrides` in `samgraha.toml`, keyed by capability name
2. **RepoScript** — `{repo}/scripts/<capability>.<ext>`
3. **LocalScript** — `.samgraha/scripts/<capability>.<ext>` (synced copy)
4. **GlobalScript** — scripts shipped next to the MCP binary

The first tier with a matching script wins. **There is no fifth,
built-in-fallback tier.** If no script is found at any of the four tiers,
the capability fails with a clear error — it does not silently run
built-in Rust logic. (The 22 hardcoded Rust pipeline modules that used to
fill this role are deleted — `codebase-refactoring-proposal.md` §10
Phase 4.)

---

## FR3. Script Input/Output Contract

All scripts shall receive standardized command-line arguments:

- `--repo-root <path>` — repository root directory
- `--in <path>` — JSON input file
- `--out <path>` — path where script writes JSON output

Scripts shall write a JSON status envelope to `--out`:

```json
{
  "status": "ok",
  "message": null,
  "written": []
}
```
Value-returning capabilities (`calculate`, `scaffold`) add their own
top-level fields alongside this envelope (e.g. `final_score`/`band` for
`calculate`, `created`/`skipped` for `scaffold`).

---

## FR4. MCP Tool Interface

Five dedicated MCP tools plus one generic catch-all shall expose capability dispatch, plus four tools for reading/writing init plans and plan-generation inputs:

| Tool | Capability / purpose |
|---|---|
| `run_system_validate` | validate |
| `run_system_calculate` | calculate |
| `run_system_report` | report |
| `run_system_scaffold` | scaffold |
| `run_system_plan_generation` | plan-generation |
| `run_system_script` | any (generic catch-all, `capability` param) |
| `store_system_plan` / `get_system_plan` | write/read a system's init plan |
| `store_plan_generation_input` / `get_plan_generation_input` | write/read a semantic plan-generation input |

Real params are `system_name`, `repo_root`, `input_json`, `phase_id`,
`timeout_secs`, `repo_path` (and `target` on `report`/`plan-generation`) —
not `system_id`, `domain`, or `options`.

---

## FR5. Prerequisite Gating

Before executing a script for a named `phase_id`, the dispatch system shall check phase prerequisites:

- Verify every dependency phase (`workflow_phase_dependencies`) has a
  matching, valid entry in `script_runs`
- Evaluate TTL and head_commit expiry rules
- Return a blocked-precondition response if any dependency is missing or expired

This prevents a phase from running before its declared prerequisites are
satisfied. Gating is opt-in per call — it only activates when a `phase_id`
is passed.

---

## FR6. Script Run Tracking

Every successful script invocation for a given `phase_id` shall be
recorded in the `script_runs` table with:

- `standard_id`, `repo_fingerprint`, `capability`, `phase_or_check_key`
- `ran_at`
- `expiry_rule_json`/`expires_at`/`head_commit_at_run` (all nullable — no
  expiry rule means the run never goes stale)

This provides the data `check_phase_prerequisites` reads to gate later calls.

---

## Business Rules

- Scripts are expected to stay within `--repo-root` — samgraha does not sandbox this itself.
- **Script failures are not silently recovered — there is no fallback.** A capability with no working script fails with a clear, actionable error.
- Script output is parsed as the standard envelope (`status`/`message`/`written`); anything else in the JSON is stored opaquely, never interpreted.
- A phase's dependency check only runs when `phase_id` is supplied; capability calls without one skip gating entirely.

---

## Non-Goals

- Capability Dispatch does not:
  - provide a built-in fallback for any capability — a missing script is a hard error, by design
  - provide a scripting language or DSL
  - manage script installation or dependency resolution
  - execute untrusted code without user consent

---

## Acceptance Criteria

The feature is successful when:

- a registered system's `validate` script is invoked, and a system with no script gets a clear error naming the missing capability
- script discovery correctly checks Override → RepoScript → LocalScript → GlobalScript, in that order, with no fifth tier
- all script runs for a named phase are tracked in `script_runs`
- prerequisite gating prevents a phase from running before its dependencies have a valid `script_runs` entry
- MCP tools expose all five capabilities plus the generic catch-all, plus init-plan and plan-generation-input read/write

---

## Traceability

This feature derives from the following Vision commitments:

- **Extensibility without modification.**
- **Systems own their validation logic.**
- **Samgraha orchestrates; systems implement.**

**Traceability**

Vision → Feature: Capability Dispatch
