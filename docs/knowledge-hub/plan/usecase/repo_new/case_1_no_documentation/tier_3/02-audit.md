# Stage 2 — Audit

**Use case:** `repo_new/case_1_no_documentation`
**Tier:** 3
**Domains:** feature-design, feature-technical

## Input

Documents produced by stage 1 (`01-generation.md`): one document per domain.

## Procedure

For each domain, run the real audit files unmodified. Produce a report per domain.

### Per-Domain Audit Files

| Domain | Deterministic doc | Deterministic section | Semantic doc | Semantic section |
|---|---|---|---|---|
| feature-design | `audit/deterministic/document/09-feature-design.yaml` | `audit/deterministic/section/feature-design/*.yaml` | `audit/semantic/document/09-feature-design.md` | `audit/semantic/section/feature-design/*.md` |
| feature-technical | `audit/deterministic/document/10-feature-technical.yaml` | `audit/deterministic/section/feature-technical/*.yaml` | `audit/semantic/document/10-feature-technical.md` | `audit/semantic/section/feature-technical/*.md` |

Score via `calculation/summary/final_score.yaml` — 4 equal buckets (25% each).

## Output

A report per domain. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure across all use cases.
