# Documentation Audit System

## Context

Documentation standards (`docs/raw/standards/`) define contracts each documentation domain must satisfy. Audits verify compliance against those contracts.

Documentation lives under `docs/raw/`. Each standard declares its documentation folder. The implementation folder is declared in Engineering Documentation.

## Authority Chain

Every audit validates against specific Audit Rules in one or more standards.
Each audit check traces to its source Audit Rule.

| Audit | Source Standard(s) | Checks |
|---|---|---|
| vision-audit | standards/vision.md | V1–V12 (12 checks) |
| architecture-audit | standards/architecture.md | A1–A13 (13 checks) |
| design-audit | standards/design.md | D1–D12 (12 checks) |
| feature-audit | standards/feature.md | F1–F14 (14 checks) |
| feature-design-validation | standards/feature-design.md | FD1–FD15 (15 checks) |
| feature-technical-audit | standards/feature-technical.md | FT1–FT15 (15 checks) |
| prototype-audit | standards/prototype.md | P1–P15 (15 checks) |
| external-context-audit | standards/external-context.md | EC1–EC12 (12 checks) — validates External Context docs in isolation (inside-out) |
| external-context-ownership-audit | standards/external-context.md | EC1–EC7 (7 checks) — validates cross-doc consistency (outside-in) |
| engineering-audit | standards/engineering.md | E1–E12 (12 checks) — overall engineering collection, repository structure declaration, domain coverage |
| build-audit | standards/engineering.md | B1–B12 (12 checks, build focus) |
| security-audit | standards/engineering.md | SEC1–SEC12 (12 checks, security focus) |
| deterministic-runtime-audit | standards/architecture.md + standards/engineering.md | S1–S12 (12 checks) — pipeline determinism, stateless stages, artifact lifecycle |
| implementation-audit | standards/architecture.md + standards/feature-technical.md + standards/engineering.md | I1–I15 (15 checks) |
| readme-audit | standards/readme.md | R1–R12 (12 checks) |

## Scope

All audits operate within `docs/raw/`. Each standard's `# Documentation Folder` section declares which subfolder applies. Source validation (implementation-audit) reads the implementation folder declared in Engineering Documentation.

## Execution Order

Audits follow a dependency order: foundation before specifics, independent before
cross-cutting, documentation before verification.

1. vision-audit — product purpose and direction (foundation)
2. architecture-audit — system organization and structural foundation
3. design-audit — product-wide design principles
4. feature-audit — product capabilities
5. feature-design-validation — user experience per feature
6. feature-technical-audit — architectural realization per feature
7. prototype-audit — executable validation
8. external-context-audit — External Context docs in isolation (inside-out quality)
9. external-context-ownership-audit — cross-doc external dependency consistency (outside-in)
10. engineering-audit — overall engineering documentation collection, repository structure declaration, domain coverage for dependent audits
11. build-audit — build and packaging standards
12. security-audit — security engineering standards
13. deterministic-runtime-audit — pipeline determinism and stateless execution model
14. implementation-audit — documentation vs source verification
15. readme-audit — public entry point

## Exit Criteria

Each audit produces a report. All checks must pass before the corresponding
documentation domain is accepted. An audit fails if any mandatory check is
not satisfied or if the document under audit references a non-existent source.

## Audit Reports

Reports go in `docs/raw/reports/<domain>/latest/`. Previous reports rotate to `archive/`.

## Standard Report Format

Every audit report MUST contain the following sections:

### 1. Executive Summary
Summarizes corpus health, resolved findings from prior audit, new findings, and overall trajectory.

```
- **Overall Assessment:** [Poor / Fair / Good / Excellent]
- **Audit Score:** [X.X/10]
- **Critical Findings (P0):** [N]
- **Major Findings (P1):** [N]
- **Minor Findings (P2):** [N]
- **Informational (P3):** [N]
- **Documents Audited:** [N]
```

### 2. Score Details
Breakdown of how each dimension contributed to the score. Each audit defines its own
dimensions based on its validation checklist items.

### 3. Findings by Severity
- **P0** — Critical: blocks correctness or safety. Must fix before next cycle.
- **P1** — Major: violates a mandatory Audit Rule. Must fix within 1 cycle.
- **P2** — Minor: violates a non-mandatory rule or quality concern. Should fix.
- **P3** — Informational: observations, suggestions, no rule violation.

### 4. Findings Detail
Each finding includes: ID, severity, file path + line, violated check, description.

### 5. Remediation Tracking
Findings from prior report listed with status: Resolved / Unresolved / New.

## Report Rotation

Before writing a new report, rotate the previous report:

```powershell
$domain = "<domain>"
$reportDir = "docs/raw/reports/$domain"
if (Test-Path "$reportDir/latest") {
    Move-Item -Path "$reportDir/latest/*" -Destination "$reportDir/archive/" -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Path "$reportDir/latest" -Force | Out-Null
```

This ensures every audit cycle has a before/after comparison to measure improvement.
