# Audit Standards Meta-Framework

## Version
1.0.0

## Purpose
Defines how the 90+ individual audit standards interact, which are mandatory vs optional, how conflicts are resolved, and how the standards themselves are versioned and updated.

## Standard Categories and Scope Matrix

| Category | Applies To | Mandatory | Notes |
|---|---|---|---|
| `feature/` | Every feature doc | Yes | Core product artifact |
| `feature-technical/` | Production features only | Yes | Skip for prototypes |
| `architecture/` | System-level architecture docs | Yes | Not per-feature |
| `engineering/` | All code artifacts | Yes | |
| `design/` | UI/UX-bearing features | Conditional | Skip for backend-only |
| `vision/` | Vision or strategy docs | Yes | Once per project |
| `philosophy/` | Philosophy or principles docs | Conditional | Once per project |
| `prototype/` | Prototype artifacts only | Yes | Replaces feature-technical |
| `external-context/` | Integration or dependency docs | Conditional | Apply when external deps present |
| `feature-design/` | Feature design documents | Conditional | Apply when UX flows defined |
| `readme/` | README files | Yes | |

## Standard Application Order

When a doc triggers multiple categories, apply in this sequence:
1. `feature/` or `vision/` or `philosophy/` (intent layer)
2. `feature-technical/` or `architecture/` (design layer)
3. `engineering/` (implementation layer)
4. `design/` (presentation layer)
5. `external-context/` (integration layer)

## Conflict Resolution Rules

When two standards produce opposite verdicts on the same property, apply in priority order:

1. **Security** beats everything — if a security standard requires X and a performance standard allows skipping X, security wins.
2. **Constraints** beat recommendations — a mandatory criterion outranks a recommended criterion.
3. **Specificity** beats generality — the more specific standard (feature-technical) overrides the generic one (feature) for the same property.
4. **Explicit document of exception** — a doc may override a recommended criterion by naming the exception and justification inline; this must be flagged as a WARNING, not an ERROR.

## Mandatory vs Optional Criteria

Within each standard:
- `mandatory` criteria: FAIL = ERROR verdict. Cannot be waived.
- `recommended` criteria: FAIL = WARNING verdict. Can be waived with documented justification.

## Standards Versioning Policy

Each standard file carries a `## Version` field using semver:
- **Patch** (1.0.x): typo fixes, clarification of existing criteria only.
- **Minor** (1.x.0): new recommended criteria, new edge cases, updated examples.
- **Major** (x.0.0): new mandatory criteria, scoring weight changes, removed criteria.

Major version bumps require review against existing audited docs. A breaking major version must include a migration note.

## Standards Review Cadence

Standards are reviewed:
- Triggered: when a new gap class is discovered across 3+ audits.
- Scheduled: annually, against industry reference (OWASP, NIST, WCAG, ISO 25010).
- On request: any contributor may file a gap report.

## Adding New Standards

1. File belongs in the appropriate category directory.
2. Must follow the standard template: Version, Engineering Intent, Audit Objectives, Expected Quality, Red Flags, Edge Cases, Scoring Criteria, Output Schema.
3. All new mandatory criteria start as `recommended` for one release cycle before promotion.
4. Must be registered in this meta-framework's Scope Matrix before activation.

## Known Cross-Cutting Standards

These topics must be covered in every applicable category (not just one):

| Topic | Required In |
|---|---|
| Observability | `feature/`, `feature-technical/`, `architecture/` |
| Security | `feature-technical/`, `architecture/`, `engineering/` |
| Traceability | All categories (each has own traceability file) |
| Data Governance | `feature-technical/`, `architecture/` |
| Versioning/Change | `feature-technical/`, `external-context/` |
