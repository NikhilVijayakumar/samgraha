# Stage 2 — Audit

**Use case:** `repo_existing/case_1_no_documentation`
**Tier:** 1
**Domains:** vision, philosophy

## Input

Documents produced by stage 1 (`01-generation.md`): `vision.md` and `philosophy.md`.

## Procedure

For each domain, run the real audit files unmodified against the generated document. Produce a report per domain.

### Per-Domain Audit Steps

1. **Deterministic document audit:** Run `audit/deterministic/document/{domain}.yaml` against the document.
2. **Deterministic section audit:** Run `audit/deterministic/section/{domain}/*.yaml` against each section.
3. **Semantic document audit:** Run `audit/semantic/document/{domain}.md` against the whole document.
4. **Semantic section audit:** Run `audit/semantic/section/{domain}/*.md` against each section.
5. **Score:** Compute final score via `calculation/summary/final_score.yaml` — 4 equal buckets (25% each).

### Per-Domain Audit Files

| Audit type | Vision | Philosophy |
|---|---|---|
| Deterministic document | `audit/deterministic/document/01-vision.yaml` | `audit/deterministic/document/02-philosophy.yaml` |
| Deterministic section | `audit/deterministic/section/vision/*.yaml` | `audit/deterministic/section/philosophy/*.yaml` |
| Semantic document | `audit/semantic/document/01-vision.md` | `audit/semantic/document/02-philosophy.md` |
| Semantic section | `audit/semantic/section/vision/*.md` | `audit/semantic/section/philosophy/*.md` |

## Output

A report per domain. This stage never fixes anything.

## Differs From Other Use Cases

No difference — same audit files, same procedure across all use cases.
