# Stage 2 — Audit

**Use case:** `repo_new/case_1_no_documentation`
**Tier:** 6
**Domains:** qa

## Input

Document produced by stage 1 (`01-generation.md`): `qa.md`.

## Procedure

Run the real audit files unmodified against the generated document.

### Audit Files

| Audit type | File |
|---|---|
| Deterministic document | `audit/deterministic/document/12-qa.yaml` |
| Deterministic section | `audit/deterministic/section/12-qa/*.yaml` |
| Semantic document | `audit/semantic/document/12-qa.md` |
| Semantic section | `audit/semantic/section/12-qa/*.md` |

Score via `calculation/summary/final_score.yaml` — 4 equal buckets (25% each).

## Output

A report. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
