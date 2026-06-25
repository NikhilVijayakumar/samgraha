# Statelessness Audit

## Context

Validates that pipeline stage interfaces and engineering principles support
deterministic, stateless operation. Statelessness means same input produces same
output regardless of execution context. Validates against Audit Rules from
**architecture.md** (communication paths) and **engineering.md** (principles and
rationale).

## Authority

- `docs/raw/standards/architecture.md` — Audit Rule: Communication paths are
  understandable.
- `docs/raw/standards/engineering.md` — Audit Rules: Engineering principles are
  documented; Engineering rationale is explicit rather than implied.

## Scope

Pipeline stage interface descriptions in architecture docs. Engineering principle
documentation related to determinism and statelessness.

## Validation Checklist

Each check maps to a specific Audit Rule from a specific standard.

### S1. Communication Paths Understandable (architecture.md)
Pipeline stage interfaces are well-defined and stateless. Each stage's input and
output contracts are documented. No hidden shared state between stages. Communication
between stages follows documented, predictable paths.

**Source:** architecture.md Audit Rule — Communication paths are understandable.

### S2. Determinism as Engineering Principle (engineering.md)
Deterministic output and stateless operation are documented as engineering
principles. The principle explains why determinism matters and how it is
maintained across pipeline stages.

**Source:** engineering.md Audit Rule — Repository-wide engineering principles are
documented.

### S3. Statelessness Rationale Explicit (engineering.md)
Engineering rationale for stateless design choices is explicit — not implied. Why
certain stages were designed as pure functions, why caches are invalidated, why
external calls are idempotent. Trade-offs are acknowledged.

**Source:** engineering.md Audit Rule — Engineering rationale is explicit rather
than implied.

## Success Criteria

All checks S1–S3 pass. Pipeline communication paths are stateless and documented.
Determinism is an explicit engineering principle. Rationale for stateless design
is documented.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. Review architecture docs for pipeline stage interface descriptions
3. Review engineering docs for determinism and statelessness principles
4. Run checks S1–S3
5. Collect failures — each must specify violated check, source standard, exact location
6. Write report to `docs/raw/audit/reports/statelessness/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
