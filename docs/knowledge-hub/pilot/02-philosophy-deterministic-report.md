# Deterministic Whole-Document Report — Philosophy

**Document:** pilot/02-philosophy.md
**Standard:** `documentation-standards/02-philosophy-standards.md`
**Rule File:** `audit/deterministic/document/02-philosophy.yaml`
**Auditor:** System (deterministic engine)
**Audit Date:** 2026-07-12
**Revision:** 1

---

## Score

**Deterministic Whole Score: 100 / 100**
(baseline — first audit of this document)

```
score = 100 × (Σ weight of passed rules) / (Σ weight of all rules)
# calculation: deterministic_document_v1
      = 100 × 6.0 / 6.0
```

Total possible weight across all 7 document-level rules is fixed at **6.0** (phil-doc-001 1.5, 002 1.0, 003 0.5, 004 1.0, 005 0.5, 006 0.5, 007 1.0).

### Category Scores

| Category | Score | Rules |
|---|---:|---|
| Collection Completeness | 100 / 100 | phil-doc-001, 002 |
| Modularity | 100 / 100 | phil-doc-003 |
| Technology Independence | 100 / 100 | phil-doc-004 |
| Cross-References | 100 / 100 | phil-doc-005 |
| Duplicate Content | 100 / 100 | phil-doc-006 |
| Prescriptive Integrity | 100 / 100 | phil-doc-007 |

---

## 1. Collection Completeness — weight 2.5 of 6.0

| Rule | Check | Severity | Weight | Current | Evidence |
|---|---|---|---:|---|---|
| phil-doc-001 | Required sections present (Guiding Principles, Values, Trade-offs) | error (mandatory) | 1.5 | PASS | All 3 required sections present: Principles (line 9), Values (line 53), Trade-offs (line 73) |
| phil-doc-002 | No empty required sections | error (mandatory) | 1.0 | PASS | All required sections contain substantive content |

## 2. Modularity — weight 0.5 of 6.0

| Rule | Check | Severity | Weight | Current | Evidence |
|---|---|---|---:|---|---|
| phil-doc-003 | Document covers one philosophical concern | warning (recommended) | 0.5 | PASS | Single focus: Lattice's decision-making philosophy |

## 3. Technology Independence — weight 1.0 of 6.0

| Rule | Check | Severity | Weight | Current | Evidence |
|---|---|---|---:|---|---|
| phil-doc-004 | No implementation technology references | error (mandatory) | 1.0 | PASS | No programming languages, frameworks, libraries, APIs, database schemas, or protocols named |

## 4. Cross-References — weight 0.5 of 6.0

| Rule | Check | Severity | Weight | Current | Evidence |
|---|---|---|---:|---|---|
| phil-doc-005 | Required cross-references present | warning (recommended) | 0.5 | PASS | Purpose section explicitly references Vision as upstream |

## 5. Duplicate Content — weight 0.5 of 6.0

| Rule | Check | Severity | Weight | Current | Evidence |
|---|---|---|---:|---|---|
| phil-doc-006 | No duplicate content within document | warning (recommended) | 0.5 | PASS | All sections cover distinct concerns |

## 6. Prescriptive Integrity — weight 1.0 of 6.0

| Rule | Check | Severity | Weight | Current | Evidence |
|---|---|---|---:|---|---|
| phil-doc-007 | Guiding principles are prescriptive, not descriptive | error (mandatory) | 1.0 | PASS | All 4 principles state actionable directives: "When X, do Y" pattern with concrete examples |

---

## Failures Requiring Attention

No failures — all 7 document-level rules pass.

---

## Metadata

| Field | Value |
|---|---|
| Domain | philosophy |
| Standard | documentation-standards |
| Rule File | `audit/deterministic/document/02-philosophy.yaml` |
| Auditor | System (deterministic engine) |
| Audit Date | 2026-07-12 |
| Revision | 1 |
| Session | pilot-tier1 |
