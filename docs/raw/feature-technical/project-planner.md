# Project Planner — Technical Design

## Architecture

The Project Planner follows a single-planner architecture:

```
StandardWorkflowPlanner (all ProjectCase variants)
  reads plan_scenarios from registered documentation standard
  produces ProjectPlan with tier-ordered, dependency-aware phases

ProjectPlan (persisted in project_plans/project_phases — registry.db,
             see the "SQLite Schema" section below; not to be confused
             with a system's init plan, which lives in
             workflow_use_cases/workflow_phases/workflow_phase_dependencies
             in standards.db — two different mechanisms, same-sounding names)
  └── ProjectPhase[] (ordered, dependency-aware)

PhaseExecutor
  ├── GeneratePhaseExecutor   — calls compile()
  ├── AuditPhaseExecutor      — calls validate scripts via capability dispatch — no fallback; a domain with no working script fails clearly
  ├── FixPhaseExecutor        — creates fix sessions via audit-fix pipeline
  └── VerifyPhaseExecutor     — re-runs validation, checks score threshold

PlanOrchestrator — loads plan, finds pending phase, executes, updates status
```

The legacy hardcoded planners (`NewProjectPlanner`, `DocAuditPlanner`, `ImplTestAuditPlanner`, `BuildAuditPlanner`) have been removed. All four `ProjectCase` variants now route to `StandardWorkflowPlanner` via `resolve_planner()`.

## Data Structures

Defined in `crates/schemas/src/planner.rs`:

```rust
pub struct ProjectPlan {
    pub id: String,
    pub title: String,
    pub case: ProjectCase,
    pub phases: Vec<ProjectPhase>,
    pub status: PlanStatus,
    pub created_at: String,
    pub updated_at: String,
}

pub struct ProjectPhase {
    pub id: String,
    pub plan_id: String,
    pub phase_number: u32,
    pub name: String,
    pub phase_type: PhaseType,
    pub domains: Vec<String>,
    pub pipeline_ids: Vec<String>,
    pub dependencies: Vec<String>,
    pub status: PhaseStatus,
}

pub enum ProjectCase {
    NewProject,
    DocAudit,
    ImplTestAudit,
    BuildAudit,
}

pub enum PhaseType {
    Generate,
    Audit,
    Fix,
    Verify,
}

pub enum PlanStatus { Active, Completed, Failed }

pub enum PhaseStatus { Pending, InProgress, Completed, Failed, Blocked }
```

## SQLite Schema — V29

Two tables in `knowledge.db`:

```sql
CREATE TABLE project_plans (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    case_type TEXT NOT NULL,        -- new_project | docs_audit | impl_test_audit | build_audit
    status TEXT NOT NULL DEFAULT 'active',
    current_phase TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE project_phases (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES project_plans(id),
    phase_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    phase_type TEXT NOT NULL,
    domains TEXT NOT NULL,           -- JSON array
    pipeline_ids TEXT NOT NULL,      -- JSON array
    dependencies TEXT NOT NULL,      -- JSON array of phase IDs
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TEXT,
    completed_at TEXT,
    result_json TEXT,
    UNIQUE(plan_id, phase_number)
);
```

## Phase Generation Logic — StandardWorkflowPlanner

`StandardWorkflowPlanner` reads `plan_scenarios` from the registered documentation standard. Each scenario specifies tier ordering, domain grouping, dependency edges, and per-step content (audit/fix/verify directives).

1. Load `plan_scenarios` for the requested `ProjectCase`
2. Determine tier ordering from standard's `domains_by_tier`
3. Generate one phase per tier × step combination
4. Apply `depends_on` edges between phases
5. Apply `enforce_order` within a tier to split into sequential sub-phases
6. Set `pipeline_ids` to empty (Audit/Verify phases dispatch via capability scripts, not hardcoded pipeline lists)

## Domain Ordering

Derived from the standard's tier definitions, not hardcoded. Typical ordering:

1. Vision
2. Architecture
3. Engineering
4. Readme
5. External Context
6. External Context Ownership
7. Design
8. Feature
9. Feature Design
10. Feature Technical
11. Prototype
12. Consistency
13. Coverage

## MCP Surface

| Method | Params | Returns |
|--------|--------|---------|
| `project_plan` | `{ case, title? }` | `{ plan_id, phases }` |
| `project_plan_get` | `{ plan_id }` | Full plan with phase statuses |
| `project_plan_list` | `{}` | All plans + status |
| `project_plan_execute` | `{ plan_id, phase_number? }` | Execute phase (or next pending) |
| `project_plan_status` | `{ plan_id }` | Progress (e.g. "3/8 phases complete") |
| `project_plan_abort` | `{ plan_id, reason }` | Mark plan failed |

## Integration With Existing Systems

| Phase Type | What It Calls |
|-----------|---------------|
| `Generate` | `KnowledgeRuntime::compile()` |
| `Audit` | System `validate` scripts via `capability::resolve_capability()` — no fallback; a domain with no working script fails clearly |
| `Fix` | `apply_finding_fix()` / `generate_fix_plan()` per finding |
| `Verify` | Re-runs validation via capability scripts, score compared against threshold (>= 70) |

## V1 Scope

| Capability | v1 |
|-----------|-----|
| Plan generation for all 4 cases via StandardWorkflowPlanner | Yes |
| Phasewise execution with dependency tracking | Yes |
| Progress tracking + status queries | Yes |
| Auto-execute audit phase (capability dispatch, no fallback) | Yes |
| Auto-generate fix sessions from findings | Yes |
| Verify phase (re-audit + gate check) | Yes |

## V1 Non-Goals

- No parallel phase execution
- No auto-retry on phase failure
- No `samgraha plan` CLI command — MCP-only
- No phase reordering
- No incremental re-generation
