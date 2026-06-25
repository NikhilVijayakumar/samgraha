# Feature Audit

## Purpose

Verifies that the Feature Documentation completely, accurately, and independently defines the product capabilities of the repository.

This audit evaluates the **Feature Documentation as an integrated product specification**, ensuring that every feature is atomic, business-focused, implementation-independent, traceable to the Vision, and sufficiently complete to support Feature Design without assumptions.

Feature Documentation defines **what the product must do**.

It does not define **how the product is designed or implemented**.

---

# Authority

`docs/raw/standards/feature.md` — **Audit Rules** section.

---

# Scope

Applies to every document under:

```text
docs/raw/feature/
```

The audit evaluates:

* Individual feature quality
* Collection-wide consistency
* Product completeness
* Vision alignment
* Product readiness

The Feature collection is evaluated as **one complete product specification**.

---

# Validation Checklist

---

# Feature Definition

## F1. Atomic Features

Each document defines exactly one feature.

A feature should represent one independently understandable capability.

Multiple unrelated capabilities should not be combined.

---

## F2. Responsibilities Defined

Every feature clearly defines:

* purpose
* responsibilities
* boundaries
* expected outcomes

The feature should be understandable without implementation knowledge.

---

## F3. Product Scope Complete

The Feature collection completely describes the product capabilities.

Missing major product capabilities should be identified.

The collection should represent the complete product.

---

## F4. Technology Independence

Features remain independent of implementation technologies.

Feature documentation should not describe:

* programming languages
* frameworks
* databases
* APIs
* architecture
* implementation strategies
* source code

Technology belongs to Engineering.

---

# Product Definition

## F5. Business Rules Complete

Business rules are complete, consistent, and unambiguous.

Product behavior should not require engineering assumptions.

---

## F6. Acceptance Criteria Complete

Every feature defines measurable acceptance criteria.

Acceptance criteria should:

* be testable
* be objective
* define completion
* avoid ambiguity

---

## F7. Product Constraints Documented

Product constraints affecting feature behavior are documented.

Examples include:

* permissions
* limits
* validation
* workflows
* dependencies
* product policies

Constraints should remain product-focused.

---

## F8. User Value Clear

Every feature clearly explains the value it provides.

The documentation should answer:

* Who benefits?
* Why does the feature exist?
* What problem does it solve?

---

# Documentation Quality

## F9. Vision Traceability

Every feature traces to one or more Vision objectives.

The relationship should be explicit.

Features without Vision support should be reported.

---

## F10. Independent Understanding

A feature should be understandable without reading unrelated feature documents.

Cross-feature dependencies should be minimized.

When dependencies exist, they should be referenced rather than duplicated.

---

## F11. No Design or Engineering Leakage

Feature Documentation should not contain:

* UI design
* UX behavior
* architecture
* implementation
* engineering rationale

Those concerns belong to:

* Design
* Feature Design
* Feature Technical Design
* Engineering

---

## F12. Terminology Consistency

Feature terminology remains consistent across the Feature collection.

Business concepts should have one definition.

Contradictory product language is not permitted.

---

# Product Readiness

## F13. Downstream Readiness

Feature Documentation provides sufficient information for Feature Design.

Designers should not need to invent product behavior.

---

## F14. Future Maintainability

Feature Documentation remains modular.

Adding new capabilities should require new feature documents rather than modification of unrelated features.

The Feature collection should scale without duplication.

---

# Scoring Model

Each validation is scored.

| Score | Meaning           |
| ----- | ----------------- |
| 10    | Excellent         |
| 8–9   | Very Good         |
| 6–7   | Good              |
| 4–5   | Needs Improvement |
| 1–3   | Poor              |
| 0     | Missing           |

Scores evaluate product specification quality rather than pass/fail compliance.

---

# Category Weights

| Category              | Weight |
| --------------------- | -----: |
| Feature Definition    |    30% |
| Product Definition    |    35% |
| Documentation Quality |    20% |
| Product Readiness     |    15% |

Weighted scores produce an overall **Feature Score (0–100)**.

---

# Success Criteria

The Feature Documentation should:

* define one capability per document
* completely describe product behavior
* remain implementation independent
* define complete business rules
* define measurable acceptance criteria
* clearly communicate user value
* remain traceable to the Vision
* contain no Design or Engineering concerns
* provide a complete foundation for Feature Design

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores
5. Validation Scores (F1–F14)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                        | Status              |
| --------------------------- | ------------------- |
| Documentation Quality       | PASS / FAIL         |
| Product Specification       | PASS / FAIL         |
| Feature Design Readiness    | READY / NOT READY   |
| Engineering Assumption Risk | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all Feature documents.
3. Verify required product capabilities are documented.
4. Execute validation checks F1–F14.
5. Score every validation.
6. Score every Feature document.
7. Calculate weighted category scores.
8. Calculate the overall Feature Score.
9. Compare against the previous report when available.
10. Identify findings and prioritized recommendations.
11. Assess Feature Design Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/feature/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
