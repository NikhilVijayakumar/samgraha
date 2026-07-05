# Audit

## Purpose

What audit is, why it matters, and how it enforces documentation quality.

## Content

**Audit** is the process of checking documentation against standard-defined rules. It produces a quality report with findings classified by severity.

### Why Audit Matters

- **Consistency** — All documents follow the same structural rules.
- **Completeness** — Required sections are present.
- **Purity** — Prohibited content (e.g., implementation details in feature docs) is flagged.
- **Traceability** — Documents link to upstream and downstream references.
- **Quality gates** — CI/CD can enforce minimum quality scores before merging.

### Audit Providers

| Provider | What It Checks | Method |
|----------|---------------|--------|
| **Deterministic** | Required sections, prohibited content, title presence | Rule-based, fast |
| **Semantic** | Readability, scope correctness, technology independence | Heuristic + optional AI |

### Audit Stages

1. **Deterministic** — Section existence, title, prohibited content (instant)
2. **Section quality** — Per-section heuristics (readability, length, relevance)
3. **Document quality** — Cross-section consistency, document-level heuristics
4. **Cross-domain** — Relationship validation between standards

### Scores

Audit produces a quality score (0-100) per document and per domain. Scores below configurable thresholds fail gates.

### Running Audit

`samgraha audit [domain] [--provider <name>]... [--all] [--gate [<score>]] [--report]` — `domain` is optional (positional), `--provider` is repeatable and defaults to `deterministic`, `--gate` alone means "require 100.0" (or pass `--gate 85` for a specific minimum; failing it exits with a non-zero `AuditFailure` code), and `--report` writes a markdown report under `[report].dir/audit/{latest,archive}/`.

## Related

- [Audit Guide: Overview](../audit-guide/overview.md)
- [Command: audit](../commands/audit.md)
- [Configuration: audit](../configuration/audit.md)
