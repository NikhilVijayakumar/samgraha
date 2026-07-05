# Audit Overview

## Purpose

What audit checks, how it helps maintain documentation quality, and how it fits into the workflow.

## Content

### What Audit Does

Audit checks compiled documentation against standard-defined rules and produces a quality report with actionable findings.

### Audit Pipeline

```
Compiled docs → Deterministic checks → Semantic checks → Scores → Report
```

1. **Deterministic** — Rule-based checks from each standard's `audit_rules` (section presence, title, prohibited-content leakage)
2. **Semantic** — Heuristic checks (short-document warnings, placeholder text, technology-independence, missing rationale); no AI model is called even when configured
3. **Scoring** — Quality score per document, domain, and overall
4. **Reporting** — Human-readable or machine-readable report output

### When to Audit

- **During development** — `samgraha audit` to check work in progress
- **In CI/CD** — `samgraha audit --gate 80` to enforce quality before merge
- **Periodically** — Track quality trends over time

### Finding Severity

| Severity | Meaning | Action |
|----------|---------|--------|
| error | Must fix for compliance | Fix before merge |
| warning | Should fix for quality | Fix when possible |
| suggestion | Consider fixing | Nice to have |

## Related

- [Deterministic Audit](deterministic.md)
- [Semantic Audit](semantic.md)
- [Scores](scores.md)
- [Command: audit](../commands/audit.md)
