# Project Planner

## Purpose

The Project Planner orchestrates multi-phase project workflows — generating plans from user goals, executing phases with dependency awareness, and tracking progress through verification. It transforms Samgraha from a toolkit of individual capabilities into a guided workflow engine.

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Plan Generation from Goal

The planner shall generate a phasewise plan from a user-supplied project goal.

Four goals are supported:
- `new-project` — full lifecycle from doc generation through verification
- `docs-audit` — audit and fix documentation only
- `impl-test-audit` — audit and fix implementation, security, and runtime issues
- `build-audit` — audit and fix build configuration

The planner reads project state from persisted artifacts (existing docs, previous reports) to determine the phase sequence. No separate runtime state is maintained.

### Acceptance Criteria

- `project_plan({ case: "docs-audit" })` returns a plan with audit, fix, and verify phases
- `project_plan({ case: "new-project" })` returns an 8-phase plan (generate → audit doc → fix doc → audit impl → fix impl → audit build → fix build → verify)
- Plan includes dependency-aware phase ordering
- Plan is persisted and retrievable via `project_plan_get`

## FR2. Phasewise Execution

The planner shall execute a project plan one phase at a time.

Each phase calls existing infrastructure:
- `Generate` → `compile()`
- `Audit` → `run_pipeline()` per domain
- `Fix` → `apply_finding_fix()` / `generate_fix_plan()` per finding
- `Verify` → re-run pipeline, compare score against threshold (recommended: score >= 70)

Execution is sequential within a plan. No parallel phase execution in v1.

### Acceptance Criteria

- `project_plan_execute({ plan_id })` runs the next pending phase
- `project_plan_execute({ plan_id, phase_number: 3 })` runs a specific phase
- Phase transitions from Pending → InProgress → Completed/Failed
- Failed phases do not block downstream execution (status set to Blocked)

## FR3. Progress Tracking

The planner shall track and report progress for active plans.

Progress includes: which phases are complete, which is current, overall fraction (e.g., "3/8 phases complete"), and per-phase status.

### Acceptance Criteria

- `project_plan_status({ plan_id })` returns phase counts by status
- Plans persist across server restarts (stored in SQLite)

## FR4. Plan Lifecycle Management

The planner shall support plan creation, retrieval, listing, execution, and abort.

Plans are created by goal. Once created, they persist until explicitly aborted or completed. Aborted plans are marked Failed.

### Acceptance Criteria

- `project_plan_list` returns all plans with status
- `project_plan_abort({ plan_id, reason })` marks plan as Failed
- Completed plans are retrievable for audit/reference
