# Prototype Audit

## Context

Validates prototype docs in `docs/raw/prototype/` against the **prototype.md**
standard. Prototype Documentation provides an executable simulation of the
application before production implementation begins.

## Authority

`docs/raw/standards/prototype.md` — Audit Rules section.

## Scope

All files under `docs/raw/prototype/`. Quality evaluated across the complete
prototype.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### P1. Validates All Documented Features
The prototype validates every feature documented in `docs/raw/feature/`. No
feature is missing from the prototype simulation.

**Audit Rule:** The prototype validates all documented features.

### P2. User Workflows Complete
All user workflows from Feature Design are functional in the prototype. Users
can navigate through complete workflows.

**Audit Rule:** User workflows are complete.

### P3. Navigation Functional
The prototype is fully navigable. All routes, screens, and transitions function
as specified in Feature Design.

**Audit Rule:** Navigation is functional.

### P4. Mock APIs Satisfy Contracts
Mock APIs implement the contracts documented in Feature Technical Design.
API requests return appropriate responses matching the documented schema.

**Audit Rule:** Mock APIs satisfy documented contracts.

### P5. Mock Persistence Consistent
Mock persistence behaves consistently across the prototype. Create, read,
update, delete, and search operations work as expected. Deterministic behavior.

**Audit Rule:** Mock persistence behaves consistently.

### P6. Relevant External Context Simulated
External systems that affect user experience or behavior are simulated where
necessary for validation. External Context is referenced, not duplicated.

**Audit Rule:** Relevant External Context is simulated.

### P7. Prototype Artifacts Disposable
Prototype artifacts are clearly separable from production code. No prototype
code can be mistaken for production implementation. Artifacts are disposable.

**Audit Rule:** Prototype artifacts remain disposable.

### P8. No Production Implementation
No production implementation has leaked into the prototype. No production
databases, APIs, authentication, or infrastructure are used.

**Audit Rule:** No production implementation has leaked into the prototype.

## Success Criteria

All checks P1–P8 pass. All features validated. Workflows complete. Mock APIs
satisfy contracts. Persistence consistent. External context simulated. Artifacts
disposable. No production leak.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/prototype/`
3. Run prototype and verify behavior against feature and feature-design docs
4. Run checks P1–P8
5. Collect failures — each must specify violated check and exact location
6. Write report to `docs/raw/audit/reports/prototype/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
