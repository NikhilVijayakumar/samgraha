# Vision Audit

## Purpose

Verifies that the Vision Documentation clearly establishes the long-term purpose, direction, and identity of the product while remaining technology-independent and consistent with all downstream documentation.

This audit evaluates the **Vision as the root of the documentation hierarchy**, ensuring that product purpose, philosophy, and guiding principles are documented in a form that can stably guide feature development, architecture, and engineering without encoding implementation decisions.

Vision Documentation defines **why the product exists**.

It does not define how it is designed, built, or deployed.

---

# Authority

`docs/raw/standards/vision.md` — **Audit Rules** section.

---

# Scope

Applies to every document under:

```text
docs/raw/vision/
```

The audit evaluates:

* Individual document quality
* Technology independence
* Philosophy and principle completeness
* Downstream documentation consistency
* Long-term stability

The Vision collection is evaluated as **the authoritative foundation of the documentation hierarchy**.

---

# Validation Checklist

---

# Vision Content

## V1. Purpose and Problem Defined

The Vision clearly explains why the product exists.

The problem it solves is immediately understandable.

The long-term purpose is unambiguous without technical knowledge.

**Audit Rule:** The Vision explains why the product exists.

---

## V2. Long-term Direction Explicit

The Vision communicates the desired future state of the product.

Direction should remain stable over time.

Short-term goals should not dominate.

**Audit Rule:** The Vision remains stable and future-oriented.

---

## V3. Product Philosophy Documented

Product philosophy is explicitly documented.

The philosophy communicates values that guide product decisions.

Examples include:

* Documentation First
* Local First
* Offline First
* AI-Readable by Default
* Deterministic Engineering

Philosophy should describe values rather than implementation.

**Audit Rule:** Product philosophy is present.

---

## V4. Guiding Principles Documented

Enduring principles that influence future decisions are documented.

Principles should remain stable even as features evolve.

**Audit Rule:** Guiding principles are documented.

---

## V5. Target Audience Identified

The Vision identifies who benefits from the product.

Audience should be described in terms of roles or needs rather than implementation personas.

---

# Technology Independence

## V6. No Implementation Technologies

No programming languages, frameworks, libraries, databases, infrastructure, build systems, or cloud providers appear.

Vision describes purpose and philosophy, not technology.

**Audit Rule:** The document is technology independent.

---

## V7. No Implementation Details

No implementation details, architectural decisions, algorithms, source code, or configuration appear.

Implementation belongs to downstream documentation.

**Audit Rule:** No implementation details appear.

---

## V8. No Feature Specifications

The Vision does not describe individual features, user workflows, or UI behavior.

Feature documentation derives from the Vision — it does not appear within it.

---

# Traceability and Consistency

## V9. Downstream Documentation Consistent

Feature docs, architecture, and engineering decisions do not contradict the Vision.

Vision is the root of the documentation hierarchy.

Conflicts between Vision and downstream documentation should be reported.

**Audit Rule:** Downstream documentation remains consistent with the Vision.

---

## V10. Vision Guides Feature Development

The Vision provides sufficient direction for feature definition.

Features should be derivable from the Vision without requiring engineering assumptions.

---

## V11. Stable and Future-Oriented

The Vision describes long-term product direction rather than short-term milestones.

It should remain interpretable across multiple engineering cycles.

**Audit Rule:** The Vision remains stable and future-oriented.

---

# Documentation Quality

## V12. Terminology Consistent

Vision terminology remains consistent with Architecture, Engineering, and downstream documentation.

Contradictory product language is not permitted.

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

Scores evaluate vision quality rather than pass/fail compliance.

---

# Category Weights

| Category                    | Weight |
| --------------------------- | -----: |
| Vision Content              |    35% |
| Technology Independence     |    30% |
| Traceability and Consistency|    20% |
| Documentation Quality       |    15% |

Weighted scores produce an overall **Vision Score (0–100)**.

---

# Success Criteria

The Vision Documentation should:

* clearly explain why the product exists
* communicate long-term product direction
* document product philosophy and guiding principles
* identify the intended audience
* remain completely technology-independent
* contain no feature specifications or implementation details
* remain consistent with all downstream documentation
* provide a stable foundation for feature definition
* remain interpretable across multiple engineering cycles

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores
5. Validation Scores (V1–V12)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                           | Status              |
| ------------------------------ | ------------------- |
| Vision Content                 | PASS / FAIL         |
| Technology Independence        | PASS / FAIL         |
| Downstream Consistency         | PASS / FAIL         |
| Feature Development Readiness  | READY / NOT READY   |
| Architectural Drift Risk       | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all Vision documents.
3. Verify required Vision content is present.
4. Execute validation checks V1–V12.
5. Score every validation.
6. Score every Vision document.
7. Calculate weighted category scores.
8. Calculate the overall Vision Score.
9. Compare against the previous report when available.
10. Identify technology leakage, implementation details, and downstream inconsistencies.
11. Assess Feature Development Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/vision/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
