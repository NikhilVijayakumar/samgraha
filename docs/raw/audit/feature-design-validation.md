# Feature Design Audit

## Purpose

Verifies that the Feature Design Documentation completely translates Feature Specifications into user-centered experiences while consistently applying the shared Design system.

This audit evaluates the **Feature Design collection as an integrated UX specification**, ensuring that every feature is mapped to one Feature Specification, applies shared Design principles, incorporates relevant External Context, and remains independent of Architecture and Engineering.

Feature Design defines **how users experience a feature**.

It does not define **how the feature is implemented**.

---

# Authority

`docs/raw/standards/feature-design.md` — **Audit Rules** section.

---

# Scope

Applies to every document under:

```text
docs/raw/feature-design/
```

The audit evaluates:

* Individual Feature Design quality
* Collection-wide consistency
* Design system application
* User experience completeness
* Readiness for Feature Technical Design

The Feature Design collection is evaluated as **one complete UX specification**.

---

# Validation Checklist

---

# Feature Mapping

## FD1. One-to-One Mapping

Every Feature Design document maps to exactly one Feature Specification.

The mapping should be explicit.

Missing or duplicate mappings are reported.

---

## FD2. Feature Coverage Complete

Every Feature Specification has a corresponding Feature Design document.

Missing Feature Designs should be identified.

The collection should completely cover the product.

---

## FD3. Design System Applied

Shared Design Documentation has been correctly applied.

Feature Design should reference Design Documentation rather than redefine it.

Only feature-specific UX decisions belong here.

---

## FD4. Relevant External Context Applied

Only relevant External Context influencing user experience is referenced.

External knowledge should not be duplicated.

---

# User Experience

## FD5. User Workflows Complete

User workflows completely describe the feature.

Examples include:

* primary workflow
* alternative workflows
* error scenarios
* edge cases
* recovery paths

User behavior should be understandable without implementation knowledge.

---

## FD6. User Experience Complete

The user experience completely describes:

* interactions
* feedback
* navigation
* information presentation
* user expectations

Design decisions should support the documented Feature Specification.

---

## FD7. Accessibility Considered

Accessibility considerations are documented where applicable.

Feature Design should explain user-facing accessibility requirements rather than implementation techniques.

---

## FD8. Localization Considered

Localization and internationalization considerations are documented where applicable.

User-facing language, formatting, and cultural considerations should be addressed.

---

## FD9. External Constraints Reflected

User-facing constraints originating from External Context are reflected in the design.

Examples include:

* platform conventions
* operating system guidelines
* accessibility regulations
* industry standards

---

# Documentation Quality

## FD10. Technology Independence

Feature Design remains independent of implementation technologies.

Feature Design should not describe:

* UI frameworks
* component libraries
* HTML
* CSS
* APIs
* databases
* implementation patterns

Technology belongs to Engineering.

---

## FD11. No Architecture or Engineering Leakage

Feature Design should not contain:

* architecture
* runtime behavior
* communication patterns
* persistence
* engineering rationale
* implementation decisions

These concerns belong to Feature Technical Design and Engineering.

---

## FD12. References Rather Than Duplication

Shared Design Documentation and External Context should be referenced rather than rewritten.

Duplicated documentation should be reported.

---

## FD13. UX Consistency

Interaction terminology, navigation patterns, visual language, and user expectations remain consistent throughout the Feature Design collection.

Conflicting UX guidance is not permitted.

---

# Design Readiness

## FD14. Feature Technical Design Readiness

Feature Design provides sufficient UX information for Feature Technical Design.

Technical designers should not need to invent user interactions or workflows.

---

## FD15. Future Maintainability

Feature Design remains modular.

Changes to one feature should not require unrelated Feature Design documents to change.

The collection should scale without duplication.

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

Scores evaluate UX specification quality rather than pass/fail compliance.

---

# Category Weights

| Category              | Weight |
| --------------------- | -----: |
| Feature Mapping       |    25% |
| User Experience       |    40% |
| Documentation Quality |    20% |
| Design Readiness      |    15% |

Weighted scores produce an overall **Feature Design Score (0–100)**.

---

# Success Criteria

The Feature Design Documentation should:

* provide a one-to-one mapping with Feature Specifications
* correctly apply the shared Design system
* document complete user workflows
* reflect relevant External Context
* consider accessibility and localization
* remain technology independent
* contain no Architecture or Engineering concerns
* provide a complete foundation for Feature Technical Design

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores
5. Validation Scores (FD1–FD15)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                               | Status              |
| ---------------------------------- | ------------------- |
| Documentation Quality              | PASS / FAIL         |
| UX Specification                   | PASS / FAIL         |
| Design System Application          | PASS / FAIL         |
| Feature Technical Design Readiness | READY / NOT READY   |
| Engineering Assumption Risk        | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all Feature Design documents.
3. Verify every Feature Specification has a corresponding Feature Design document.
4. Execute validation checks FD1–FD15.
5. Score every validation.
6. Score every Feature Design document.
7. Calculate weighted category scores.
8. Calculate the overall Feature Design Score.
9. Compare against the previous report when available.
10. Identify findings and prioritized recommendations.
11. Assess Feature Technical Design Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/feature-design/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
