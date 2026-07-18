# Capability Dispatch — Technical Design

## Architecture

```
MCP Tool Call (run_system_validate, etc.)
    │
    ▼
adapter.rs — tool handler, validates params, calls run_capability_for()
    │
    ▼
capability.rs — resolve_capability() finds script, execute_capability() runs it
    │
    ├── 4-tier discovery: system → standard → override → built-in
    ├── input JSON construction (target, options, phase_id)
    ├── prerequisite check (check_phase_prerequisites)
    ├── tempdir creation, --in/--out file writes
    ├── Command::new(script_path).args([...]).output()
    └── JSON output parsing, result return
    │
    ▼
env.rs — run_capability_script(), repo_fingerprint()
```

## Participating Components

### capability.rs (618 lines)

Core dispatch engine. Contains:

- **`Capability` enum** — `Validate`, `Calculate`, `Report`, `Scaffold`, `PlanGeneration`
- **`resolve_capability(cap, system_id, domain)`** — 4-tier script discovery
- **`execute_capability(cap, input_json)`** — full execution cycle (tempdir, write, run, parse)
- **`run_capability_for(cap, target, options, phase_id)`** — convenience wrapper with prerequisite gating
- **`check_phase_prerequisites(phase_id)`** — checks `depends_on` deps against `script_runs`
- **`evaluate_expiry(rule)`** — TTL and head_commit expiry evaluation
- **`InitPlan`, `PlanPhase`, `PlanUseCase`, `ExpiryRule`** — plan generation types

### env.rs (90 lines)

Environment helpers:

- **`run_capability_script(script_path, args, timeout)`** — Command execution with timeout
- **`repo_fingerprint(repo_root)`** — git HEAD-based repository fingerprint

### adapter.rs (288 lines)

MCP tool handlers:

- **`run_system_validate`** — validates, calls `run_capability_for(&Capability::Validate, ...)`
- **`run_system_calculate`** — same pattern for Calculate
- **`run_system_report`** — same for Report
- **`run_system_scaffold`** — same for Scaffold
- **`run_system_plan_generation`** — same for PlanGeneration
- **`run_system_script`** — generic catch-all, parses capability string
- **`copy_standard_scripts`** — copies bundled scripts to .samgraha/system-scripts/

## Data Structures

### Input JSON (sent to script via --in)

```json
{
  "target": {
    "domain": "feature",
    "document_path": "docs/raw/feature/audit-framework.md",
    "check_ids": ["F1", "F2"]
  },
  "options": {
    "severity_filter": "error",
    "custom_option": "value"
  },
  "phase_id": "phase-3"
}
```

### Output JSON (read from script via --out)

```json
{
  "status": "pass",
  "score": 85.0,
  "findings": [
    {
      "criterion_id": "F1",
      "severity": "error",
      "message": "Missing required section: Purpose",
      "location": {"line": 10}
    }
  ],
  "metadata": {
    "checks_run": 12,
    "checks_passed": 11
  }
}
```

### script_runs Table (§8.5)

```sql
CREATE TABLE script_runs (
    id INTEGER PRIMARY KEY,
    system_id TEXT NOT NULL,
    capability TEXT NOT NULL,
    domain TEXT,
    input_hash TEXT,
    output_hash TEXT,
    exit_code INTEGER,
    duration_ms INTEGER,
    head_commit TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
```

## Four-Tier Discovery

```
resolve_capability(Validate, system_id="kriti", domain="feature")
    │
    ├── 1. System: .samgraha/systems/kriti/scripts/validate.py
    │      (registered via system.yaml)
    │
    ├── 2. Standard: .samgraha/standards/samgraha-documentation/scripts/validate.py
    │      (bundled with the documentation standard)
    │
    ├── 3. Override: scripts/validate.py  (user-provided)
    │
    └── 4. Built-in: Rust pipeline module fallback
```

## Prerequisite Gating Flow

```
run_capability_for(Validate, target, options, phase_id)
    │
    ├── check_phase_prerequisites(phase_id)
    │     ├── Load phase from system_plans table
    │     ├── Check depends_on → each dep must have script_runs entry
    │     ├── Evaluate expiry rules (TTL, head_commit)
    │     └── Return blocked JSON if any prerequisite fails
    │
    └── execute_capability(cap, input_json)
```

## MCP Surface

| Tool | Params | Returns |
|------|--------|---------|
| `run_system_validate` | `{system_id, domain, target?, options?, phase_id?}` | `{status, score, findings}` |
| `run_system_calculate` | `{system_id, domain, target?, options?}` | `{status, results}` |
| `run_system_report` | `{system_id, domain, target?, options?}` | `{status, report_path}` |
| `run_system_scaffold` | `{system_id, domain, target?, options?}` | `{status, scaffolded_files}` |
| `run_system_plan_generation` | `{system_id, domain, options?}` | `{status, plan}` |
| `run_system_script` | `{system_id, capability, domain, target?, options?}` | `{status, ...}` |

## Performance Considerations

- Script discovery is cached per session (system.yaml parsed once).
- Tempdir creation and file I/O are the main overhead (~1-5ms per invocation).
- Script execution time depends on the script itself (async scripts return immediately).
- `repo_fingerprint()` is cheap (git rev-parse HEAD).

## Security Considerations

- Scripts receive `--repo-root` and are expected to stay within it.
- No secrets or credentials are passed to scripts via input JSON.
- Script output is schema-validated before persistence.
- `check_phase_prerequisites` prevents stale scripts from corrupting state.

## Traceability

This document derives from:

- Feature: Capability Dispatch
- Architecture: Component Model
- Architecture: Extensibility Architecture

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
