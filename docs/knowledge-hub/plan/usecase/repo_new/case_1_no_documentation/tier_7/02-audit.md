# Stage 2 — Audit

**Use case:** `repo_new/case_1_no_documentation`
**Tier:** 7
**Domains:** build

## Input

Document produced by stage 1 (`01-generation.md`): `build.md`.

## Procedure

Run the real audit files unmodified against the generated document.

### Per-Domain Audit Files

| Domain | Scripts (check-name) | Deterministic doc | Deterministic section | Semantic doc | Semantic section |
|---|---|---|---|---|---|
| build | `build-succeeds`, `artifact-exists` | `audit/deterministic/document/14-build.yaml` | `audit/deterministic/section/14-build/*.yaml` | `audit/semantic/document/14-build.md` | `audit/semantic/section/14-build/*.md` |

Score via `calculation/summary/final_score.yaml` — 4 equal buckets (25% each).

## Output

A report. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
