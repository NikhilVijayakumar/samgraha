# Project Planner — Technical Design

## Architecture

The Project Planner follows the same layered architecture as the rest of Samgraha:

```
ProjectPlanner (trait)
  ├── NewProjectPlanner    — full lifecycle (8 phases)
  ├── DocAuditPlanner      — docs only (audit + fix + verify)
  ├── ImplTestAuditPlanner — impl + security + runtime (audit + fix + verify)
  └── BuildAuditPlanner    — build only (audit + fix + verify)

ProjectPlan (persisted)
  └── ProjectPhase[] (ordered, dependency-aware)

PhaseExecutor
  ├── GeneratePhaseExecutor   — calls compile()
  ├── AuditPhaseExecutor      — calls run_pipeline() per domain
  ├── FixPhaseExecutor        — creates fix sessions via audit-fix pipeline
  └── VerifyPhaseExecutor     — re-runs pipelines, checks score threshold

PlanOrchestrator — loads plan, finds pending phase, executes, updates status
```

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

Two new tables in `knowledge.db`:

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

## Phase Generation Logic

### NewProjectPlanner
1. Detect what docs exist (vs blank project)
2. If blank: Phase 1 = Generate (compile all domains)
3. Phase 2 = Audit (all doc domains)
4. Phase 3 = Fix (doc findings, one phase per domain if many)
5. Phase 4 = Audit (implementation, deterministic-runtime, security)
6. Phase 5 = Fix (impl+security findings)
7. Phase 6 = Audit (build)
8. Phase 7 = Fix (build findings)
9. Phase 8 = Verify (re-run critical pipelines + check gates)

### DocAuditPlanner
1. Run all doc-domain pipelines → collect findings
2. Group findings by domain
3. Generate one fix phase per domain with findings
4. Order by dependency (architecture before feature)
5. Add verify phase at end

### ImplTestAuditPlanner
1. Find all implementation, security, deterministic-runtime findings
2. Generate fix phases per domain
3. Add verify phase

### BuildAuditPlanner
1. Run build pipeline
2. Generate fix phase for build findings
3. Add verify phase

## Domain Ordering

Doc domains ordered to minimize cascading rework:

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
| `Audit` | `KnowledgeRuntime::run_pipeline()` per domain |
| `Fix` | `apply_finding_fix()` / `generate_fix_plan()` per finding |
| `Verify` | `run_pipeline()`, score compared against threshold (>= 70) |

## Prerequisite Fixes

Three changes required in existing code before Phase 3 (execution) works:

1. **`run_pipeline()` must return `report_id`** — currently discards it after storing. Change return type to `(PipelineReport, i64)`.

2. **Finding→target_path resolver** — needed for FixPhaseExecutor to derive file paths from findings automatically. Resolve `finding.document_id` via `registry.get_document(id)?.path`, fallback to `finding.location`.

3. **Verify threshold** — Verify phase compares `PipelineReport.score` against >= 70.0 (matches the "Acceptable" rating band in `reporting.rs`).

## V1 Scope

| Capability | v1 |
|-----------|-----|
| Plan generation for all 4 cases | Yes |
| Phasewise execution with dependency tracking | Yes |
| Progress tracking + status queries | Yes |
| Auto-execute audit phase (run pipelines) | Yes |
| Auto-generate fix sessions from findings | Yes |
| Verify phase (re-audit + gate check) | Yes |

## V1 Non-Goals

- No parallel phase execution
- No auto-retry on phase failure
- No `samgraha plan` CLI command — MCP-only
- No phase reordering
- No incremental re-generation
