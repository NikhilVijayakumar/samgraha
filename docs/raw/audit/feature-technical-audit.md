# Feature Technical Audit

## Context

Validates feature technical design docs in `docs/raw/feature-technical/` against
the **feature-technical.md** standard. Feature Technical Design translates one
Feature Specification into its architectural realization.

## Authority

`docs/raw/standards/feature-technical.md` — Audit Rules section.

## Scope

All files under `docs/raw/feature-technical/`. Quality evaluated individually and
across the feature technical design collection.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### FT1. One-to-One Mapping
Every Feature Technical Design corresponds to exactly one Feature Specification.
No document describes multiple unrelated features.

**Audit Rule:** A one-to-one mapping exists between Feature and Feature Technical
Design.

### FT2. Architecture Documentation Applied
Shared architectural principles from Architecture Documentation have been applied.
Feature Technical Design references architecture rather than redefining it.

**Audit Rule:** Shared Architecture Documentation has been applied.

### FT3. Relevant External Context Identified
External systems, platforms, protocols, or APIs required to realize the feature
are identified and referenced. Only context relevant to the feature is included.

**Audit Rule:** Relevant External Context has been identified.

### FT4. Feature Design Considerations Respected
UX-driven architectural needs from Feature Design have been incorporated where
architectural decisions affect user experience. Feature Design is not redefined.

**Audit Rule:** Feature Design considerations have been respected.

### FT5. Component Responsibilities Clear
All components participating in the feature have clearly defined responsibilities.
Each component's role in realizing the feature is documented.

**Audit Rule:** Component responsibilities are clearly defined.

### FT6. Communication Paths Understandable
How components communicate — data flow, events, API calls, IPC — is documented
and understandable. Communication paths respect architectural boundaries.

**Audit Rule:** Communication paths are understandable.

### FT7. Runtime Boundaries Respected
Runtime boundaries, process boundaries, and lifecycle are respected. Feature
technical design does not violate documented runtime architecture.

**Audit Rule:** Runtime boundaries are respected.

### FT8. External Architectural Constraints Reflected
Constraints introduced by external systems (API limits, platform capabilities,
protocol requirements) are documented and accommodated.

**Audit Rule:** External architectural constraints are reflected.

### FT9. Technology References Remain Architectural
Technology references appear only when architecturally significant (e.g., runtime
platform, persistence engine, message bus). Implementation-level references
(frameworks, libraries, syntax) are absent.

**Audit Rule:** Technology references remain architectural.

### FT10. No Implementation Details
No source code, algorithms, function implementations, TypeScript interfaces, Rust
traits, SQL queries, or API implementation patterns appear.

**Audit Rule:** No implementation details appear.

### FT11. References Instead of Duplicates
Architecture Documentation and External Context are referenced rather than
rewritten. No duplication of shared architectural principles or external
documentation.

**Audit Rule:** Architecture and External Context are referenced instead of
duplicated.

## Success Criteria

All checks FT1–FT11 pass. One-to-one mapping exists. Architecture applied.
Component responsibilities clear. Communication paths understandable. Technology
references remain architectural. No implementation details. Shared docs
referenced not duplicated.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/feature-technical/`
3. For each file, run checks FT1–FT11
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/feature-technical/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
