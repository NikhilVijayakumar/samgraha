# Design Audit

## Purpose

Verifies that the Design Documentation defines a complete, reusable, technology-independent design system for the product.

This audit evaluates the **design documentation as an integrated collection**, ensuring that design philosophy, interaction principles, visual language, accessibility, localization, and usability standards establish a consistent foundation for Feature Design.

Design Documentation describes **shared design principles**, not feature implementations or UI specifications.

---

# Authority

`docs/raw/standards/design.md` — **Audit Rules** section.

---

# Scope

Applies to every document under:

```text
docs/raw/design/
```

The audit evaluates:

* Individual document quality
* Cross-document consistency
* Design system completeness
* Responsibility separation
* Collection-wide coherence
* Feature Design readiness

The Design collection is evaluated as **one reusable design system**.

---

# Validation Checklist

---

# Design System

## D1. Reusable Design Principles

Design principles are reusable across multiple applications, workflows, and features.

Principles should define long-lived design guidance rather than individual solutions.

---

## D2. Design Philosophy Defined

The overall design philosophy is clearly documented.

The philosophy explains:

* design goals
* user experience principles
* communication principles
* interaction philosophy
* visual philosophy

Design decisions should derive from the documented philosophy.

---

## D3. Design System Completeness

The Design collection completely describes the shared design language.

Examples include:

* visual language
* interaction principles
* navigation principles
* accessibility
* localization
* responsiveness
* information hierarchy
* feedback principles

Missing design concerns should be reported.

---

## D4. Technology Independence

Design remains independent of implementation technologies.

Design documentation should not describe:

* UI frameworks
* CSS
* HTML
* JavaScript
* rendering technologies
* component libraries
* implementation patterns

Implementation belongs to Engineering.

---

# Documentation Quality

## D5. Modular Documentation

Design documentation is organized into focused documents.

Each document owns one design concern.

Large concerns should be decomposed.

---

## D6. Responsibility Separation

Design documents should not overlap.

Each principle is defined once.

Related concerns should be referenced rather than duplicated.

---

## D7. Feature Independence

Design Documentation should not describe:

* feature workflows
* application screens
* feature behavior
* business rules
* user stories

Feature-specific design belongs exclusively to Feature Design.

---

## D8. Cross-Repository Reuse

Shared design principles should be reusable across repositories.

External design systems should be referenced rather than duplicated.

---

# Design Quality

## D9. Accessibility Guidance

Accessibility principles are documented where applicable.

Examples include:

* inclusive interaction
* keyboard accessibility
* readability
* visual hierarchy
* assistive technology support

Accessibility guidance should define principles rather than implementation.

---

## D10. Localization Guidance

Localization principles are documented where appropriate.

Examples include:

* language independence
* text expansion
* RTL support
* cultural neutrality
* formatting considerations

Localization guidance should remain reusable.

---

## D11. Consistency

Terminology, principles, visual language, and design philosophy remain consistent throughout the Design collection.

Conflicting design guidance is not permitted.

---

## D12. Future Maintainability

The Design Documentation should remain maintainable.

New features should inherit the Design system without requiring changes to the Design Documentation.

The Design collection should evolve slowly compared to Feature Design.

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

Scores evaluate design quality rather than pass/fail compliance.

---

# Category Weights

| Category              | Weight |
| --------------------- | -----: |
| Design System         |    35% |
| Documentation Quality |    30% |
| Design Quality        |    35% |

Weighted scores produce an overall **Design Score (0–100)**.

---

# Success Criteria

The Design Documentation should:

* define a reusable design system
* document a clear design philosophy
* remain technology independent
* contain no feature-specific behavior
* provide accessibility and localization guidance
* maintain modular responsibilities
* remain internally consistent
* support Feature Design without modification

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores
5. Validation Scores (D1–D12)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Feature Design Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                       | Status            |
| -------------------------- | ----------------- |
| Documentation Quality      | PASS / FAIL       |
| Design Quality             | PASS / FAIL       |
| Feature Design Readiness   | READY / NOT READY |
| Engineering Design Support | READY / NOT READY |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all Design documents.
3. Verify required design concerns are present.
4. Execute validation checks D1–D12.
5. Score every validation.
6. Score every Design document.
7. Calculate weighted category scores.
8. Calculate the overall Design Score.
9. Compare against the previous report when available.
10. Identify findings and prioritized recommendations.
11. Assess Feature Design Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/design/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
