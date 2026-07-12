# Final Score — Vision

**Document:** pilot/01-vision.md
**Domain:** vision
**Audit Date:** 2026-07-12
**Revision:** 1

---

## Score Breakdown

| Component | Weight | Score | Contribution |
|---|---:|---:|---:|
| Deterministic Document | 50% | 100 / 100 | 50.0 |
| Semantic Document | 50% | 100 / 100 | 50.0 |
| **Final Score** | | | **100 / 100** |

```
final_score = Σ (bucket_score / 100 × bucket_weight)
            = (100/100 × 50) + (100/100 × 50)
            = 50.0 + 50.0
            = 100.0
```

## Band

**Excellent** (95+)

## Gate

**PASS** — Score 100 exceeds threshold of 70 (Acceptable).

---

## Component Details

### Deterministic Document (100/100)

All 7 rules pass:
- vis-doc-001: Required sections present ✓ (weight 1.5)
- vis-doc-002: No empty required sections ✓ (weight 1.0)
- vis-doc-003: Single product vision ✓ (weight 0.5)
- vis-doc-004: No technology references ✓ (weight 1.0)
- vis-doc-005: Vision derives from no other domain ✓ (weight 1.0)
- vis-doc-006: References downstream ✓ (weight 0.5)
- vis-doc-007: No duplicate content ✓ (weight 0.5)

### Semantic Document (100/100)

All 3 criteria pass:
- C1: Cross-section coherence ✓ (weight 35)
- C2: Technology independence ✓ (weight 35)
- C3: Collection coherence ✓ (weight 30)

---

## Metadata

| Field | Value |
|---|---|
| Domain | vision |
| Session | pilot-tier1 |
| Calculation | `calculation/summary/final_score.yaml` |
| Bands | `calculation/summary/score_bands.yaml` |
