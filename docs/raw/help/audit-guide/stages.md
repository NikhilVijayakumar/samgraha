# Audit Stages

## Purpose

The four audit stages (`AuditStage`: Deterministic, Section, Document, CrossDomain) — what each one represents and who produces its findings.

## Content

### Not a Sequential Pipeline

These four stages are not run in sequence by `samgraha audit`, and stages 2–4 are not computed automatically by any built-in heuristic engine. They are a classification used to store and gate findings at different granularities, mainly for AI coding agents doing semantic review over MCP.

### Stage: Deterministic

Produced by `samgraha audit` (the `DeterministicAuditProvider`/`SemanticAuditProvider`). Findings land in the `audit_results` table, one row per document, keyed off each standard's `audit_rules`.

### Stages: Section / Document / Cross-Domain

These are populated by external callers — typically an AI agent connected over MCP — via the `store_section_report`, `store_document_report`, and `store_cross_domain_report` tools. Each submits a `SemanticReport` (a `score` 0–100 plus a findings list) that is stored in the `semantic_reports` table:

- **Section** — a report about one section of one document.
- **Document** — a report about a whole document.
- **Cross-Domain** — a report spanning multiple documents/domains (e.g. checking a Feature traces to its Vision).

An agent typically calls `get_audit_knowledge(domain, section_type)` first (reads `docs/raw/audit-standards/<domain>/<section_type>.md`) for guidance on what to check, then `get_section_changed(section_id)` to skip re-analysis of unchanged sections, before submitting a report.

### Gating by Stage

`check_gate(stage, document_id)` blocks if: the Deterministic stage has any error-severity row in `audit_results`, or any other stage has a stored report with `score < 100`. See [Gates](gates.md).

### Finding Lifecycle

Findings submitted via the Section/Document/Cross-Domain stages carry a `status` (`Open`, `Fixed`, `Accepted`, `Ignored`, `FalsePositive`) updatable via `update_finding_status`.

## Related

- [Deterministic Audit](deterministic.md)
- [Semantic Audit](semantic.md)
- [Scores](scores.md)
