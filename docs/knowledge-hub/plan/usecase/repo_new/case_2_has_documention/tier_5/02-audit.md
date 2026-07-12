# Stage 2 — Audit

**Use case:** `repo_new/case_2_has_documention`
**Tier:** 5
**Domains:** implementation

## Input

Document produced by stage 1 (`01-generation.md`).

## Procedure

Run the real audit files unmodified.

### Audit Files

| Audit type | File |
|---|---|
| Deterministic document | `audit/deterministic/document/13-implementation.yaml` |
| Semantic document | `audit/semantic/document/13-implementation.md` |

Plus section-level audits. Score via `calculation/summary/final_score.yaml` — 4 equal buckets.

## Output

A report. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
