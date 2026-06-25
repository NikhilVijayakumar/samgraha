# Feature Design Validation

## Context

Validates feature design docs in `docs/raw/feature-design/` against the
**feature-design.md** standard. Feature Design translates one Feature Specification
into a user-centered design by applying shared Design principles.

## Authority

`docs/raw/standards/feature-design.md` — Audit Rules section.

## Scope

All files under `docs/raw/feature-design/`. Quality evaluated individually and
across the feature design collection.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### FD1. One-to-One Mapping
Every Feature Design document corresponds to exactly one Feature Specification.
No Feature Design describes multiple unrelated features. Filename or metadata
indicates the mapped feature.

**Audit Rule:** A one-to-one mapping exists between Feature and Feature Design.

### FD2. Design Documentation Applied
Shared design principles from Design Documentation have been applied. Feature
Design references design principles rather than redefining them.

**Audit Rule:** Shared Design Documentation has been applied.

### FD3. Relevant External Context Identified
External systems, platform conventions, or standards that affect user experience
are identified and referenced. Only context relevant to the feature is included.

**Audit Rule:** Relevant External Context has been identified.

### FD4. User Workflows Complete
User workflows, interactions, and journeys within the feature are fully documented.
All user paths (happy path, alternatives, edge cases) covered.

**Audit Rule:** User workflows are complete.

### FD5. Accessibility Considered
Accessibility considerations are documented where applicable to the feature.
Feature Design does not ignore accessibility.

**Audit Rule:** Accessibility has been considered.

### FD6. Localization Considered
Localization considerations are documented where applicable. Language, region,
or cultural adaptation requirements identified.

**Audit Rule:** Localization has been considered where applicable.

### FD7. External Constraints Reflected
User-facing constraints introduced by external systems are documented in the
design (e.g., platform UX guidelines, regulatory requirements).

**Audit Rule:** User-facing external constraints are reflected.

### FD8. Technology Independent
Design describes user interaction, workflow, and behavior — not frontend
frameworks, component libraries, APIs, or implementation patterns.

**Audit Rule:** Design remains technology independent.

### FD9. No Implementation Details
No source code, algorithms, function signatures, or implementation patterns
appear in Feature Design.

**Audit Rule:** No implementation details appear.

### FD10. No Architectural Decisions
No component responsibilities, runtime behavior, communication paths, or other
architectural decisions appear in Feature Design.

**Audit Rule:** No architectural decisions appear.

### FD11. References Instead of Duplicates
Design Documentation and External Context are referenced rather than rewritten.
No duplication of shared design principles or external documentation.

**Audit Rule:** Design Documentation and External Context are referenced instead
of duplicated.

## Success Criteria

All checks FD1–FD11 pass. One-to-one mapping exists for every feature. Design
principles applied. User workflows complete. Technology independent. No
implementation or architecture present. Shared docs referenced not duplicated.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/feature-design/`
3. For each file, run checks FD1–FD11
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/feature-design/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
