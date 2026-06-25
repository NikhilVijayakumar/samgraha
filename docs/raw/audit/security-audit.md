# Security Audit

## Context

Validates security documentation against the **engineering.md** standard. Security
documentation describes repository-wide security principles, technology choices,
and engineering standards governing secure implementation.

## Authority

`docs/raw/standards/engineering.md` — Audit Rules section.

## Scope

Security-related documentation under `docs/raw/engineering/` (security-standards,
implementation-standards). Source code patterns may be referenced for verification.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### SEC1. Security Principles Documented
Security engineering principles are documented: least privilege, defense in depth,
secure defaults, input validation. Principles guide security decisions.

**Audit Rule:** Repository-wide engineering principles are documented.

### SEC2. Security Technology Selection Includes Rationale
Why specific security technologies, libraries, or approaches were chosen is
documented with rationale. Includes reasoning for decisions like key management,
encryption, authentication.

**Audit Rule:** Technology selection includes rationale.

### SEC3. Security Standards Align with Architecture
Security decisions respect architectural constraints (trust boundaries, data
flow, component isolation). Security does not contradict architecture.

**Audit Rule:** Engineering standards align with Architecture.

### SEC4. Relevant External Context Applied
Security-related external dependencies (auth providers, secret management,
vulnerability databases) are referenced from External Context.

**Audit Rule:** Relevant External Context has been applied.

### SEC5. Security Documents Remain Modular
Security documentation is decomposed into focused documents. A single monolithic
security document is split by concern (authentication, data protection, API
security, supply chain).

**Audit Rule:** Documents remain modular.

### SEC6. No Overlapping Responsibilities
Security documentation responsibilities do not overlap with other engineering
domains. Security standards do not duplicate architecture or deployment standards.

**Audit Rule:** Responsibilities do not overlap.

### SEC7. No Feature-Specific Security
Security documentation describes repository-wide standards, not feature-specific
security behavior. Feature-specific security belongs in Feature Technical Design.

**Audit Rule:** Feature-specific engineering is absent.

### SEC8. No Source Code in Security Documentation
Security docs describe engineering rationale and standards — not source code,
security configuration files, or implementation details.

**Audit Rule:** Source code is not documented.

### SEC9. Security Rationale Explicit
Security decisions include explicit rationale. "Why" is documented alongside
"what." Security trade-offs are acknowledged.

**Audit Rule:** Engineering rationale is explicit rather than implied.

## Success Criteria

All checks SEC1–SEC9 pass. Security principles documented. Technology rationale
present. Security aligns with architecture. Documents modular and non-overlapping.
No source code. Rationale explicit.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. Review security-related docs under `docs/raw/engineering/`
3. Run checks SEC1–SEC9
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/security/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
