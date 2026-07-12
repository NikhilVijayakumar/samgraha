# Stage 2 — Audit

**Use case:** `repo_existing/case_1_no_documentation`
**Tier:** 8
**Domains:** readme, product-guide

## Input

Documents produced by stage 1 (`01-generation.md`): `readme.md` and `product-guide.md`.

## Procedure

For each domain, run the real audit files unmodified against the generated document.

### Per-Domain Audit Files

| Domain | Scripts (check-name) | Deterministic doc | Semantic doc |
|---|---|---|---|
| readme |  | `audit/deterministic/document/15-readme.yaml` | `audit/semantic/document/15-readme.md` |
| product-guide | `public-contract-diff` | `audit/deterministic/document/16-product-guide.yaml` | `audit/semantic/document/16-product-guide.md` |

Plus section-level audits for each. Score via `calculation/summary/final_score.yaml` — 4 equal buckets.

## Output

A report per domain. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
