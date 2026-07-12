# Stage 2 — Audit

**Use case:** `repo_existing/case_2_has_documention`
**Tier:** 1
**Domains:** vision, philosophy

## Input

Documents produced by stage 1 (`01-generation.md`): migrated Vision and Philosophy docs.

## Procedure

Run the real audit files unmodified against each document.

### Per-Domain Audit Files

| Domain | Scripts (check-name) | Deterministic doc | Deterministic section | Semantic doc | Semantic section |
|---|---|---|---|---|---|
| vision |  | `audit/deterministic/document/01-vision.yaml` | `audit/deterministic/section/01-vision/*.yaml` | `audit/semantic/document/01-vision.md` | `audit/semantic/section/01-vision/*.md` |
| philosophy |  | `audit/deterministic/document/02-philosophy.yaml` | `audit/deterministic/section/02-philosophy/*.yaml` | `audit/semantic/document/02-philosophy.md` | `audit/semantic/section/02-philosophy/*.md` |

Score via `calculation/summary/final_score.yaml` — 4 equal buckets.

## Output

A report per domain. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure.
