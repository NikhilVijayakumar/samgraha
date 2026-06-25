# Build Audit

## Context

Validates build documentation against the **engineering.md** standard. Build
documentation describes repository-wide build standards, technology selection,
and engineering principles governing how the project is built and packaged.

## Authority

`docs/raw/standards/engineering.md` — Audit Rules section.

## Scope

Build-related documentation under `docs/raw/engineering/` (build-standards,
packaging, CI pipeline docs). Build configuration files (`pyproject.toml`,
`tsconfig.json`, lock files) may be referenced for verification.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### B1. Build Principles Documented
Build engineering principles are documented: reproducibility, determinism,
idempotent builds, CI/CD philosophy. Principles guide build decisions.

**Audit Rule:** Repository-wide engineering principles are documented.

### B2. Build Technology Selection Includes Rationale
Why specific build tools were chosen (hatch, tsc, pytest, etc.) is documented
with rationale. "Why not alternatives" is included for non-obvious choices.

**Audit Rule:** Technology selection includes rationale.

### B3. Build Standards Align with Architecture
Build decisions respect architectural constraints (target platforms, runtime
requirements, deployment model). Build does not contradict architecture.

**Audit Rule:** Engineering standards align with Architecture.

### B4. Relevant External Context Applied
Build-related external dependencies (CI platforms, package registries, build
toolchains) are referenced from External Context. External build documentation
is referenced rather than duplicated.

**Audit Rule:** Relevant External Context has been applied.

### B5. Build Documents Remain Modular
Build documentation is decomposed into focused documents (build-standards.md,
packaging.md, ci.md). A single monolithic build document should be split.

**Audit Rule:** Documents remain modular.

### B6. No Overlapping Responsibilities
Build documentation responsibilities do not overlap with other engineering docs.
Build standards do not duplicate deployment standards or dependency standards.

**Audit Rule:** Responsibilities do not overlap.

### B7. No Feature-Specific Build Engineering
Build documentation describes repository-wide standards, not feature-specific
build steps. Each feature should not require custom build configuration.

**Audit Rule:** Feature-specific engineering is absent.

### B8. No Source Code in Build Documentation
Build docs describe engineering rationale and standards — not source code,
makefile content, or CI script contents that belong in configuration files.

**Audit Rule:** Source code is not documented.

### B9. Build Rationale Explicit
Build decisions include explicit rationale. "Why" is documented alongside "what."
Implied reasons should be made explicit.

**Audit Rule:** Engineering rationale is explicit rather than implied.

## Success Criteria

All checks B1–B9 pass. Build principles documented. Technology rationale present.
Build aligns with architecture. Documents modular and non-overlapping. No source
code. Rationale explicit.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. Review build-related docs under `docs/raw/engineering/`
3. Run checks B1–B9
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/build/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
