# Stage 2 — Audit

**Use case:** `repo_existing/case_1_no_documentation`
**Tier:** 2
**Domains:** security, feature, architecture, design, engineering, external-context

## Input

Documents produced by stage 1 (`01-generation.md`): one document per domain.

## Procedure

For each domain, run the real audit files unmodified against the generated document. Produce a report per domain.

### Per-Domain Audit Files

| Domain | Deterministic doc | Deterministic section | Semantic doc | Semantic section |
|---|---|---|---|---|
| security | `audit/deterministic/document/06-security.yaml` | `audit/deterministic/section/security/*.yaml` | `audit/semantic/document/06-security.md` | `audit/semantic/section/security/*.md` |
| feature | `audit/deterministic/document/04-feature.yaml` | `audit/deterministic/section/feature/*.yaml` | `audit/semantic/document/04-feature.md` | `audit/semantic/section/feature/*.md` |
| architecture | `audit/deterministic/document/05-architecture.yaml` | `audit/deterministic/section/architecture/*.yaml` | `audit/semantic/document/05-architecture.md` | `audit/semantic/section/architecture/*.md` |
| design | `audit/deterministic/document/07-design.yaml` | `audit/deterministic/section/design/*.yaml` | `audit/semantic/document/07-design.md` | `audit/semantic/section/design/*.md` |
| engineering | `audit/deterministic/document/08-engineering.yaml` | `audit/deterministic/section/engineering/*.yaml` | `audit/semantic/document/08-engineering.md` | `audit/semantic/section/engineering/*.md` |
| external-context | `audit/deterministic/document/15-external-context.yaml` | `audit/deterministic/section/external-context/*.yaml` | `audit/semantic/document/15-external-context.md` | `audit/semantic/section/external-context/*.md` |

Score via `calculation/summary/final_score.yaml` — 4 equal buckets (25% each).

## Output

A report per domain. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
