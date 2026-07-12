# Stage 2 — Audit

**Use case:** `repo_new/case_2_has_documention`
**Tier:** 1
**Domains:** vision, philosophy

## Input

Documents produced by stage 1 (`01-generation.md`): `vision.md` and `philosophy.md` — either migrated from existing docs or generated from scratch.

## Procedure

For each domain, run the real audit files unmodified against the document. Produce a report per domain.

### Per-Domain Audit Files

| Audit type | Vision | Philosophy |
|---|---|---|
| Deterministic document | `audit/deterministic/document/01-vision.yaml` | `audit/deterministic/document/02-philosophy.yaml` |
| Deterministic section | `audit/deterministic/section/01-vision/*.yaml` | `audit/deterministic/section/02-philosophy/*.yaml` |
| Semantic document | `audit/semantic/document/01-vision.md` | `audit/semantic/document/02-philosophy.md` |
| Semantic section | `audit/semantic/section/01-vision/*.md` | `audit/semantic/section/02-philosophy/*.md` |

Score via `calculation/summary/final_score.yaml` — 4 equal buckets (25% each).

## Output

A report per domain. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure across all use cases.
