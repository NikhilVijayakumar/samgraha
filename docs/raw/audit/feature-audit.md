# Feature Audit

## Context

Validates feature docs in `docs/raw/feature/` against the **feature.md** standard.
Feature documents describe product capabilities, not implementation.

## Authority

`docs/raw/standards/feature.md` — Audit Rules section.

## Scope

All files under `docs/raw/feature/`. Quality evaluated per document and across the
feature collection.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### F1. One Feature Per Document
Each document describes exactly one feature. No document combines multiple
unrelated capabilities. Features are atomic.

**Audit Rule:** Each document describes one feature.

### F2. Feature Responsibilities Clear
Feature purpose, boundaries, and responsibilities are clearly defined. The reader
understands what the feature does without reading implementation.

**Audit Rule:** Feature responsibilities are clear.

### F3. Technology Independent
Features describe capabilities, expected behavior, and business rules — not
frameworks, programming languages, databases, APIs, or implementation patterns.

**Audit Rule:** Features remain technology independent.

### F4. Business Rules Complete
All business rules for the feature are documented. Functional requirements are
complete enough for downstream documentation to proceed without guessing.

**Audit Rule:** Business rules are complete.

### F5. Acceptance Criteria Exist
Each feature has documented acceptance criteria. Criteria are testable and
unambiguous (pass/fail determinable).

**Audit Rule:** Acceptance criteria exist.

### F6. Traceable to Vision
Every feature traces to the Vision document. Feature purpose supports one or more
Vision objectives.

**Audit Rule:** Features remain traceable to Vision.

### F7. No Implementation or Architecture
Features do not contain implementation decisions, architectural decisions,
programming language choices, framework references, or source code.

**Audit Rule:** No implementation or architectural decisions appear.

### F8. Independently Understandable and Implementable
A feature document can be understood and implemented without reading unrelated
feature documents. Cross-feature coupling is minimized.

**Audit Rule:** Features are independently understandable and implementable.

## Success Criteria

All checks F1–F8 pass for all feature docs. Features are atomic, technology
independent, traceable to Vision, independently implementable. No implementation
or architecture present.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/feature/`
3. For each file, run checks F1–F8
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/feature/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
