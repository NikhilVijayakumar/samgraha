# Documentation Audit System

## Context

Saṃgraha is a Knowledge Engineering Platform for AI-assisted coding. Documentation standards
(`docs/raw/standards/`) define contracts each doc domain must satisfy. Audits verify
compliance against those contracts.

## Authority Chain

Every audit validates against specific Audit Rules in one or more standards.
Each audit check traces to its source Audit Rule.

| Audit | Source Standard(s) | Audit Rules Used |
|---|---|---|
| vision-audit | standards/vision.md | All 7 Audit Rules |
| architecture-audit | standards/architecture.md | All 9 Audit Rules |
| design-audit | standards/design.md | All 9 Audit Rules |
| feature-audit | standards/feature.md | All 8 Audit Rules |
| feature-design-validation | standards/feature-design.md | All 11 Audit Rules |
| feature-technical-audit | standards/feature-technical.md | All 11 Audit Rules |
| prototype-audit | standards/prototype.md | All 8 Audit Rules |
| ownership-audit | standards/external-context.md | All 7 Audit Rules |
| external-context-ownership-audit | standards/external-context.md | All 7 Audit Rules (cross-ref focus) |
| build-audit | standards/engineering.md | All 9 Audit Rules (build focus) |
| security-audit | standards/engineering.md | All 9 Audit Rules (security focus) |
| statelessness-audit | standards/architecture.md + standards/engineering.md | Communication paths; Principles; Rationale |
| implementation-audit | standards/architecture.md + standards/feature-technical.md + standards/engineering.md | Arch aligns w/ Features; Component responsibilities; Eng aligns w/ Arch; No impl detail; No source code |
| readme-audit | standards/readme.md | All 9 Audit Rules |

## Scope

All audits operate within `docs/raw/`. Source validation (implementation-audit) also
reads `src/`.

## Execution Order

Audits follow a dependency order: foundation before specifics, independent before
cross-cutting, documentation before verification.

1. vision-audit — product purpose and direction
2. architecture-audit — system organization
3. design-audit — product-wide design principles
4. feature-audit — capabilities
5. feature-design-validation — user experience per feature
6. feature-technical-audit — architectural realization per feature
7. prototype-audit — executable validation
8. ownership-audit — external dependency documentation
9. external-context-ownership-audit — cross-reference verification
10. build-audit — build and packaging standards
11. security-audit — security engineering standards
12. statelessness-audit — cross-cutting pipeline purity
13. implementation-audit — documentation vs source verification
14. readme-audit — public entry point

## Exit Criteria

Each audit produces a report. All checks must pass before the corresponding
documentation domain is accepted. An audit fails if any mandatory check is
not satisfied or if the document under audit references a non-existent source.

## Audit Reports

Reports go in `docs/raw/audit/reports/<domain>/latest/`. Previous reports rotate to `archive/`.

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
$reportDir = "docs/raw/audit/reports/$domain"
if (Test-Path "$reportDir/latest") {
    Move-Item -Path "$reportDir/latest/*" -Destination "$reportDir/archive/" -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Path "$reportDir/latest" -Force | Out-Null
```

This ensures every audit cycle has a before/after comparison to measure improvement.
