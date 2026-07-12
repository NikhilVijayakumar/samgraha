# Stage 2 — Audit

**Use case:** `repo_existing/case_2_has_documention`
**Tier:** 7
**Domains:** build

## Input

Document produced by stage 1 (`01-generation.md`).

## Procedure

Run the real audit files unmodified.

### Per-Domain Audit Files

| Domain | Scripts (check-name) | Deterministic doc | Semantic doc |
|---|---|---|---|
| build | `build-succeeds`, `artifact-exists` | `audit/deterministic/document/14-build.yaml` | `audit/semantic/document/14-build.md` |

Plus section-level audits. Score via `calculation/summary/final_score.yaml` — 4 equal buckets.

## Output

A report. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
