# External Context Ownership Audit

## Context

Validates cross-references between `docs/raw/external-context/` and all other
documentation domains against the **external-context.md** standard. Ensures external
dependencies referenced across architecture, feature, feature-design,
feature-technical, and engineering docs are reflected in External Context.

## Authority

`docs/raw/standards/external-context.md` — Audit Rules section.

## Scope

- All files under `docs/raw/external-context/`
- Cross-references from `docs/raw/architecture/`, `docs/raw/feature/`,
  `docs/raw/feature-design/`, `docs/raw/feature-technical/`, `docs/raw/engineering/`

## Validation Checklist

Each check maps to one item in the standard's Audit Rules, applied across
documentation boundaries.

### EC1. External Dependencies Complete
Every external system, platform, or library referenced in architecture, feature,
feature-technical, or engineering docs has a corresponding entry in
`docs/raw/external-context/`. No undocumented external dependency exists.

**Audit Rule:** External dependencies are documented only when necessary.
(Applied bidirectionally — unnecessary deps have docs, necessary deps are missing.)

### EC2. One Document Per Dependency
External dependency references across all documentation point to clearly
identified, atomic External Context documents. No external-context doc covers
multiple unrelated dependencies.

**Audit Rule:** One document describes one dependency.

### EC3. Purpose Consistent Across References
The purpose of each external dependency is consistently described across all
referencing documents. No contradictory purpose statements.

**Audit Rule:** Dependency purpose is clearly explained.

### EC4. Constraints Applied Downstream
Constraints documented in External Context are reflected in the downstream docs
that reference it. If an external API has rate limits, feature-technical docs
account for them.

**Audit Rule:** Constraints are documented. (Verifies downstream application.)

### EC5. Referenced Not Duplicated Across Docs
All documentation domains reference External Context rather than duplicating
external knowledge. No architecture or feature doc rewrites external
documentation.

**Audit Rule:** External documentation is referenced rather than duplicated.
(Applied across all referencing docs.)

### EC6. Relevance Maintained
When other docs reference an external dependency, the relevance to Saṃgraha is
clear. Generic references ("uses SQLite") without explaining context indicate
missing External Context.

**Audit Rule:** Repository relevance is obvious. (Verified at reference sites.)

### EC7. No Architecture Leak in References
When other docs reference External Context, they reference only the external
dependency — not internal architecture details that belong in External Context.

**Audit Rule:** No internal architecture has leaked into External Context.
(Verified at reference sites.)

## Success Criteria

All checks EC1–EC7 pass. Every external dependency referenced in the
documentation ecosystem has an External Context entry. Cross-references are
consistent and do not duplicate external knowledge. No architecture leak at
reference sites.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/external-context/`
3. Search other doc domains for external dependency references
4. Cross-reference each against External Context entries
5. Run checks EC1–EC7
6. Collect failures — each must specify violated check and exact location
7. Write report to `docs/raw/audit/reports/external-context/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
