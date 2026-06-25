# Implementation Audit

## Context

Cross-domain audit that validates documentation consistency with source code.
Ensures documentation promises match implementation reality. Validates against
Audit Rules from **architecture.md**, **feature-technical.md**, and **engineering.md**.

## Authority

- `docs/raw/standards/architecture.md` — Audit Rule: Architecture aligns with
  Features; Architecture avoids implementation detail.
- `docs/raw/standards/feature-technical.md` — Audit Rules: Component responsibilities
  are clearly defined; No implementation details appear.
- `docs/raw/standards/engineering.md` — Audit Rules: Engineering standards align
  with Architecture; Source code is not documented.

## Scope

- Source code under `src/`
- Architecture docs, feature-technical docs, engineering docs that describe
  implementation-facing concerns

## Validation Checklist

Each check maps to a specific Audit Rule from a specific standard.

### I1. Architecture Aligns with Features (architecture.md)
The architecture described in docs is actually realized in source code. Features
described in architecture have corresponding implementation paths.

**Source:** architecture.md Audit Rule — Architecture aligns with Features.

### I2. No Implementation Detail in Architecture (architecture.md)
Architecture docs do not contain source code, algorithms, function signatures,
or implementation patterns. Architecture describes organization, not code.

**Source:** architecture.md Audit Rule — Architecture avoids implementation detail.

### I3. Engineering Aligns with Architecture (engineering.md)
Engineering standards documented (build, test, dependency practices) match how
source code is actually organized and built. Engineering does not contradict
architecture.

**Source:** engineering.md Audit Rule — Engineering standards align with
Architecture.

### I4. No Source Code in Engineering Docs (engineering.md)
Engineering documents describe rationale and standards — not source code.
No code snippets, file contents, or implementation details.

**Source:** engineering.md Audit Rule — Source code is not documented.

### I5. Component Responsibilities Clear in Source (feature-technical.md)
Components described in feature-technical docs correspond to actual modules in
source code. Module boundaries and responsibilities match documentation.

**Source:** feature-technical.md Audit Rule — Component responsibilities are
clearly defined.

### I6. No Implementation Details in Tech Docs (feature-technical.md)
Feature Technical Design docs describe architectural realization — not source
code, implementations, algorithms, or function bodies.

**Source:** feature-technical.md Audit Rule — No implementation details appear.

## Success Criteria

All checks I1–I6 pass. Architecture in docs matches source structure. No
implementation detail leaks into architecture or engineering docs. Engineering
practices match documented standards. Component responsibilities in docs match
source modules.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. Review architecture, engineering, and feature-technical docs for
   implementation-facing claims
3. Verify each claim against `src/` source code
4. Search for source code fragments in documentation files
5. Run checks I1–I6
6. Collect failures — each must specify violated check, source standard, exact location
7. Write report to `docs/raw/audit/reports/implementation/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
