# Audit Stages

## Purpose

The four documentation audit stages (`AuditStage`: Deterministic, Section, Document, CrossDomain) and how custom pipelines differ.

## Content

### Documentation Audit Stages

These four stages are a classification used to store and gate findings at different granularities, mainly for AI coding agents doing semantic review over MCP.

#### Stage: Deterministic

Produced by `samgraha audit`. In the new architecture, this stage dispatches to system-provided `validate` scripts via capability dispatch. For domains with a working system script, the script performs the checks. A domain with no working `validate` script fails clearly — there is no built-in fallback. Findings land in the `audit_results` table, one row per document, keyed off each standard's `audit_rules`.

#### Stages: Section / Document / Cross-Domain

These are populated by external callers — typically an AI agent connected over MCP — via the `store_section_report`, `store_document_report`, and `store_cross_domain_report` tools.

- **Section** — a report about one section of one document
- **Document** — a report about a whole document
- **Cross-Domain** — a report spanning multiple documents/domains

An agent typically calls `get_audit_knowledge(domain, section_type)` first for guidance, then `get_section_changed(section_id)` to skip re-analysis of unchanged sections, before submitting a report.

### Custom Pipelines (Other Audits)

Build, Security, Consistency, Coverage, and Dependency Governance audits use **custom pipelines** with their own evidence collection and verification stages. These are not the 4-stage Documentation Audit pipeline. They define their own procedure in their spec file.

| Pipeline | Stages |
|---|---|
| Build | Evidence collection (config) → (artifact inspection) → verification → findings → report |
| Security | Evidence collection (static + config) → (runtime) → verification → findings → report |
| Consistency | Layer inventory → pairwise compare → trace check → contradiction scan → terminology check → report |
| Coverage | Doc inventory → code inventory → forward match → reverse match → orphan report → score → report |

### Gating by Stage

`check_gate(stage, document_id)` blocks if: the Deterministic stage has any error-severity row in `audit_results`, or any other stage has a stored report with `score < 100`. See [Gates](gates.md).

### Finding Lifecycle

Findings submitted via the Section/Document/Cross-Domain stages carry a `status` (`Open`, `Fixed`, `Accepted`, `Ignored`, `FalsePositive`) updatable via `update_finding_status`.

## Related

- [Deterministic Audit](deterministic.md)
- [Semantic Audit](semantic.md)
- [Scores](scores.md)
