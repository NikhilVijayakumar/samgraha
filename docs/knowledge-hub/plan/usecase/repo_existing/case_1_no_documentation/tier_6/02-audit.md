# Stage 2 — Audit

**Use case:** `repo_existing/case_1_no_documentation`
**Tier:** 6
**Domains:** qa

## Input

Document produced by stage 1 (`01-generation.md`): `qa.md`.

## Procedure

Run the real audit files unmodified against the generated document.

### Per-Domain Audit Files

| Domain | Scripts (check-name) | Deterministic doc | Semantic doc |
|---|---|---|---|
| qa | `unit-test-coverage` | `audit/deterministic/document/12-qa.yaml` | `audit/semantic/document/12-qa.md` |

Plus section-level audits. Score via `calculation/summary/final_score.yaml` — 4 equal buckets.

## Output

A report. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
