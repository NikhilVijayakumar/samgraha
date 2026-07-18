# Audit Overview

## Purpose

What audit checks, how it helps maintain quality, and how it fits into the workflow.

## Content

### What Audit Does

Audit checks artifacts against declared contracts and produces a quality or conformance report with actionable findings.

### Dual Pipeline Model

**Documentation Audit** (default): checks compiled documentation against standard-defined rules.

```
Compiled docs → Deterministic checks → Semantic checks → Scores → Report
```

1. **Deterministic** — Rule-based checks (section presence, title, prohibited content)
2. **Semantic** — Heuristic checks (short docs, placeholder text, technology independence)
3. **Scoring** — Quality score per document, domain, and overall
4. **Reporting** — Human-readable or machine-readable report output

**Custom Pipelines** (Build, Security, Consistency, Coverage, Dependency): collect evidence from config, source code, artifacts, and runtime, then verify against declared contracts.

```
Evidence Collection → Verification → Findings → Report
```

### Audit Types

| Audit | What it verifies |
|---|---|
| Documentation Audit | Docs against standards (dynamic domain count) |
| Implementation Audit | Docs against source code |
| Build Audit | Build docs vs config vs artifacts |
| Security Audit | Security docs vs config vs code vs runtime |
| Consistency Audit | Adjacent layer alignment + terminology |
| Coverage Audit | Bidirectional doc↔code + orphan detection |
| Dependency Governance | Dependency justification, policy, health (spec) |

### When to Audit

- **During development** — `samgraha audit` to check work in progress
- **In CI/CD** — `samgraha audit --gate 80` to enforce quality before merge
- **Periodically** — Track quality trends over time
- **Per pipeline** — `samgraha audit --pipeline <name>` for targeted checks

### Finding Severity

| Severity | Meaning | Action |
|----------|---------|--------|
| error | Must fix for compliance | Fix before merge |
| warning | Should fix for quality | Fix when possible |
| suggestion | Consider fixing | Nice to have |

Orphan findings (Coverage Audit) are always warning, never error.

## Related

- [Deterministic Audit](deterministic.md)
- [Semantic Audit](semantic.md)
- [Scores](scores.md)
- [Command: audit](../commands/audit.md)
- [Build Audit](../concepts/build-audit.md)
- [Coverage Audit](../concepts/coverage.md)
