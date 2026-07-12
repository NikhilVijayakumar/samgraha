# Stage 2 — Audit

**Use case:** `repo_new/case_2_has_documention`
**Tier:** 8
**Domains:** readme, product-guide

## Input

Documents produced by stage 1 (`01-generation.md`).

## Procedure

Run the real audit files unmodified against each document.

### Per-Domain Audit Files

| Domain | Deterministic doc | Semantic doc |
|---|---|---|
| readme | `audit/deterministic/document/15-readme.yaml` | `audit/semantic/document/15-readme.md` |
| product-guide | `audit/deterministic/document/16-product-guide.yaml` | `audit/semantic/document/16-product-guide.md` |

Plus section-level audits for each. Score via `calculation/summary/final_score.yaml` — 4 equal buckets.

## Output

A report per domain. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
