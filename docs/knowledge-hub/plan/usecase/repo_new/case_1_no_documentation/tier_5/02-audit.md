# Stage 2 — Audit

**Use case:** `repo_new/case_1_no_documentation`
**Tier:** 5
**Domains:** implementation

## Input

Document produced by stage 1 (`01-generation.md`): `implementation.md`.

## Procedure

Run the real audit files unmodified against the generated document.

### Per-Domain Audit Files

| Domain | Scripts (check-name) | Deterministic doc | Deterministic section | Semantic doc | Semantic section |
|---|---|---|---|---|---|
| implementation | `folder-structure`, `dependency-manifest`, `lint-pass` | `audit/deterministic/document/13-implementation.yaml` | `audit/deterministic/section/13-implementation/*.yaml` | `audit/semantic/document/13-implementation.md` | `audit/semantic/section/13-implementation/*.md` |

Score via `calculation/summary/final_score.yaml` — 4 equal buckets (25% each).

## Output

A report. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
