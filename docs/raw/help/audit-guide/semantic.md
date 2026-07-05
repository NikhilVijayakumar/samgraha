# Semantic Audit Provider

## Purpose

Heuristic quality checks that go beyond the deterministic rule checks — no AI model is involved today.

## Content

### Overview

The semantic provider (`SemanticAuditProvider`) runs a fixed set of heuristics over each document's body text. Unlike the deterministic provider, it is not driven by each standard's `audit_rules` list — it always runs the same five checks against every document, scoped by domain where noted.

### Heuristic Checks

| Check ID | Severity | What It Evaluates |
|----------|----------|--------------------|
| sem-001 | warning | Document body is under 50 words |
| sem-002 | suggestion | Body contains placeholder text (`TBD`, `TODO`, `FIXME`, `placeholder`, `coming soon`, `to be determined`) |
| sem-003 | suggestion | `vision`, `feature`, or `design` docs reference implementation technology (e.g. React, PostgreSQL, Kubernetes, Rust) |
| sem-004 | suggestion | `vision` docs contain requirement-level language (`SHALL`, `MUST`, `REQUIRED`, `FR1`, `API`, `endpoint`) |
| sem-005 | suggestion | `engineering` or `architecture` docs over 50 words lack any rationale/decision language |

### Not AI-Powered

Despite the `[ai]` config section (`lms`, `ollama`, `openai` provider endpoints for enrichment), the semantic audit provider does not call out to any model — it is pure string/keyword matching. The `[audit].providers` config and the manifest's `semantic-audit` capability flag additionally require `ai.provider` to be set before advertising semantic audit as available, but the provider's own logic never uses it.

### Running Semantic Audit

```bash
samgraha audit --provider semantic

# Run both providers together
samgraha audit --provider deterministic --provider semantic
```

## Related

- [Deterministic Audit](deterministic.md)
- [Configuration: AI](../configuration/ai.md)
- [Stages](stages.md)
