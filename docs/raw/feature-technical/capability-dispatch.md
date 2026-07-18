# Capability Dispatch — Technical Design

## Architecture

```
MCP Tool Call (run_system_validate, etc.)
    │
    ▼
adapter.rs — tool handler, resolves standard, calls run_capability_for()
    │
    ▼
capability.rs — resolve_capability() finds script, execute_capability() runs it
    │
    ├── 4-tier discovery: Override → RepoScript → LocalScript → GlobalScript
    ├── input JSON construction (target/write to a temp file)
    ├── prerequisite check (check_phase_prerequisites, only when phase_id given)
    ├── tempdir creation, --in/--out file writes
    ├── Command::new(script_path).args([...]).output()
    └── JSON output parsing, result return — no fallback of any kind if no script exists
    │
    ▼
env.rs — run_capability_script(), repo_fingerprint(), current_head_sha()
```

There is no 5th "built-in fallback" tier. The 22 hardcoded Rust pipeline
modules that used to serve this role are deleted
(`codebase-refactoring-proposal.md` §10 Phase 4). A capability with no
matching script at any of the 4 tiers fails with a clear error —
`"No validate script found for pipeline kind '{kind}' ... the hardcoded
Rust pipeline was removed."` — not a silent fallback.

## Participating Components

### capability.rs (~660 lines)

Core dispatch engine. Contains:

- **`Capability` enum** — `Validate`, `Calculate`, `Report`, `Scaffold`,
  `PlanGeneration`, `Init` (six total; serializes as `snake_case` except
  `PlanGeneration` → `"plan-generation"`, hyphenated — see `Display` impl)
- **`resolve_capability(capability, repo_root, config)`** — 4-tier script
  discovery, returns `None` if nothing found at any tier
- **`execute_capability(source, capability, repo_root, input_json_path,
  timeout_secs)`** — full execution cycle (subprocess spawn, `--in`/`--out`
  file I/O, JSON envelope parsing)
- **`check_phase_prerequisites(conn, standard_id, repo_fingerprint,
  phase_id, current_head)`** — queries `workflow_phases`/
  `workflow_phase_dependencies` for the phase's deps, then `script_runs`
  for each dep's last run + validity
- **`record_script_run(...)`** — upserts a successful run into
  `script_runs`, computing `expires_at`/`head_commit_at_run` from the
  phase's `ExpiryRule`
- **`evaluate_expiry(...)`** — TTL and head_commit validity evaluation
- **`InitPlan`, `PlanPhase`, `PlanUseCase`, `ExpiryRule`** — the wire
  format an `init` script returns and `store_system_plan` accepts; **not**
  the storage format (see Data Model below)

### env.rs

Environment helpers:

- **`run_capability_script(script_path, repo_root, input_json_path,
  timeout_secs)`** — subprocess execution with the `--repo-root`/`--in`/
  `--out` contract
- **`repo_fingerprint(repo_root)`** — currently just the repo root path,
  displayed
- **`current_head_sha(repo_root)`** — `git rev-parse HEAD`, `None` if
  unavailable

### adapter.rs

MCP tool handlers (`run_capability_for` is the shared implementation
behind all six):

- **`run_system_validate`** / **`run_system_calculate`** /
  **`run_system_report`** / **`run_system_scaffold`** /
  **`run_system_plan_generation`** — dedicated tools, capability baked in
  by tool name
- **`run_system_script`** — generic catch-all, `capability` passed as a
  string param
- **`store_system_plan`** / **`get_system_plan`** — write/read an `init`
  plan, split into/reconstructed from the normalized tables
- **`store_plan_generation_input`** / **`get_plan_generation_input`** —
  write/read the semantic plan-generation input (§8.3)

## Data Model

### Storage — normalized, not a JSON blob

An `init` plan is **not** stored as one JSON column anymore. `store_system_plan`
splits the incoming `InitPlan` JSON into three tables
(`schema-redesign-proposal.md` §2.1):

```sql
CREATE TABLE workflow_use_cases (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    use_case_id  TEXT    NOT NULL,
    label        TEXT    NOT NULL,
    UNIQUE(standard_id, use_case_id)
);

CREATE TABLE workflow_phases (
    id                INTEGER PRIMARY KEY,
    use_case_id       INTEGER NOT NULL REFERENCES workflow_use_cases(id) ON DELETE CASCADE,
    phase_id          TEXT    NOT NULL,
    sort_order        INTEGER NOT NULL DEFAULT 0,
    kind              TEXT    NOT NULL CHECK (kind IN ('semantic','script')),
    description       TEXT,
    script_path       TEXT,
    pre_script        TEXT,
    post_script       TEXT,
    expiry_rule_json  TEXT,
    UNIQUE(use_case_id, phase_id)
);

CREATE TABLE workflow_phase_dependencies (
    id                   INTEGER PRIMARY KEY,
    phase_id             INTEGER NOT NULL REFERENCES workflow_phases(id) ON DELETE CASCADE,
    depends_on_phase_id  INTEGER NOT NULL REFERENCES workflow_phases(id) ON DELETE CASCADE,
    UNIQUE(phase_id, depends_on_phase_id)
);
```

`get_system_plan` reconstructs the original `InitPlan` JSON shape by
querying these three tables back (`ORDER BY sort_order` preserves the
original phase array order). Real edges, not a `depends_on` array — same
pattern `domain_relationships` already used for the domain-level
dependency graph, applied at phase granularity.

### `script_runs` — run history + expiry (§8.5)

```sql
CREATE TABLE script_runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id         INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint    TEXT    NOT NULL,
    capability          TEXT    NOT NULL,
    phase_or_check_key  TEXT    NOT NULL,
    ran_at              TEXT    NOT NULL DEFAULT (datetime('now')),
    expiry_rule_json    TEXT,
    expires_at          TEXT,
    head_commit_at_run  TEXT,
    UNIQUE(standard_id, repo_fingerprint, capability, phase_or_check_key)
);
```
`expiry_rule_json` `NULL` means never expires. `type: "ttl"` is the
recommended default (checked against `expires_at`); `type: "head_commit"`
is an opt-in for checks that specifically need commit-granularity
freshness (checked against `head_commit_at_run`) — it's coarser than it
sounds, since a rebase/amend changes HEAD without changing content.

## Envelope Shapes

### Value-returning capabilities (`calculate`, `scaffold`)

```json
{
  "status": "ok",
  "message": null,
  "written": [],
  "final_score": 82,
  "band": "Good",
  "breakdown": { "deterministic_document": 90, "semantic_section": 74 }
}
```
`scaffold` additionally reports `created`/`skipped` file lists for
idempotency.

### Document-rendering capabilities (`report`, `plan-generation`)

Same envelope shape as above — `status`/`message`/`written` — but the
script also writes the actual rendered document directly to whatever
target path it was given; `written` names those paths.

## Prerequisite Gating Flow

```
run_capability_for(capability, ..., phase_id: Some("tier2-audit"))
    │
    ├── resolve_standard_id() — best-effort; a repo with nothing
    │     registered can still run a capability off pure file-based
    │     discovery, it just skips gating (backward-compat with validate's
    │     original, pre-capability-dispatch behavior)
    │
    ├── check_phase_prerequisites(standard_id, repo_fingerprint, phase_id)
    │     ├── Look up the phase in workflow_phases (join through
    │     │     workflow_use_cases by standard_id)
    │     ├── Query workflow_phase_dependencies for its deps
    │     ├── For each dep, check script_runs: no row = missing_precondition,
    │     │     row but expired = expired_precondition
    │     └── Return blocked JSON if any dependency fails
    │
    └── execute_capability(source, capability, repo_root, input_path, timeout)
          — on success, record_script_run() upserts into script_runs
```

Blocked response shape (§8.6):
```json
{
  "blocked": true,
  "reason": "missing_precondition",
  "phase_id": "tier1-audit",
  "phase_kind": "script",
  "message": "Phase 'tier1-audit' has not been run yet for this repo.",
  "how_to_run": {
    "tool": "run_system_script",
    "args": { "capability": "validate", "phase_id": "tier1-audit" }
  }
}
```

## MCP Surface

| Tool | Key params | Returns |
|------|--------|---------|
| `run_system_validate` / `run_system_calculate` / `run_system_scaffold` | `system_name?`, `repo_root?`, `input_json?`, `phase_id?`, `timeout_secs?`, `repo_path?` | Capability envelope (above), or a blocked-precondition response |
| `run_system_report` / `run_system_plan_generation` | Same, plus `target?` (report render target) | Same |
| `run_system_script` | `capability` (required), plus all of the above | Same |
| `store_system_plan` | `system_name`, `plan_json` | `{stored, system_name, use_cases, total_phases}` |
| `get_system_plan` | `system_name`, `repo_path?` | `{system_name, plan}` (reconstructed `InitPlan` JSON, or `plan: null`) |
| `store_plan_generation_input` | `system_name`, `workflow_id`, `domain_key?`, `instance_key?`, `input_json` | `{stored, system_name, workflow_id}` |
| `get_plan_generation_input` | `system_name`, `workflow_id`, `domain_key?`, `instance_key?` | `{input_json, previous_input_json}` or `{input: null}` |

There is no `system_id`, bare `domain`, or `options` param anywhere in the
real schema — `input_json` is a path to a file the caller assembles, not
an inline object.

## Performance Considerations

- Script discovery walks the 4 tiers on every call — no caching across
  calls today.
- Tempdir creation and file I/O are the main overhead per invocation.
- Script execution time depends entirely on the script itself.
- `repo_fingerprint()`/`current_head_sha()` are cheap (path display / one
  `git rev-parse`).

## Security Considerations

- Scripts receive `--repo-root` and are expected to stay within it —
  samgraha does not sandbox this itself.
- No secrets or credentials are passed to scripts via the input JSON.
- `check_phase_prerequisites` prevents a phase from running before its
  declared dependencies have a valid `script_runs` entry.

## Traceability

This document derives from:

- Feature: Capability Dispatch
- Architecture: Component Model
- Architecture: Extensibility Architecture

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
