# External Context Audit

## Purpose

Verifies that each External Context document correctly describes one external dependency in a way that is atomic, relevant, constraint-complete, and free of internal architecture leakage.

This audit evaluates the **External Context collection as individual knowledge dependency documents**, ensuring that every external system, library, platform, or protocol that materially influences the repository is documented with clear purpose, constraints, repository relevance, and authoritative references.

External Context defines **what external knowledge contributors must understand**.

It does not replace the documentation owned by external projects.

---

# Authority

Audit rules are defined by the validation checks in this document (EC1–EC12).

---

# Scope

Applies to every document under:

```text
docs/raw/external-context/
```

Each document is evaluated individually.

The collection is also evaluated for completeness and consistency.

---

# Validation Checklist

---

# Document Quality

## EC1. One Document Per Dependency

Each External Context document describes exactly one external dependency.

Documents covering multiple unrelated dependencies should be split.

**Audit Rule:** One document describes one dependency.

---

## EC2. Dependency Necessity Justified

The dependency is documented because it materially influences the repository.

Widely understood libraries without project-specific conventions should not have External Context documents.

**Audit Rule:** External dependencies are documented only when necessary.

---

## EC3. Dependency Purpose Clearly Explained

Why the dependency exists in this repository is documented.

The explanation is specific to this repository — not a generic description of the external project.

**Audit Rule:** Dependency purpose is clearly explained.

---

## EC4. Repository Relevance Explicit

Why this dependency matters to this repository is obvious from the document.

The relevance to features, architecture, or engineering is explicit.

**Audit Rule:** Repository relevance is obvious.

---

# Content Completeness

## EC5. Constraints Documented

Constraints the dependency introduces are documented.

Examples include:

* API limitations
* platform requirements
* version compatibility
* behavioral constraints
* performance characteristics
* security implications
* licensing constraints

Constraints should influence how downstream documentation applies the dependency.

**Audit Rule:** Constraints are documented.

---

## EC6. Usage Context Explained

How the dependency is used within this repository is explained.

Examples include:

* which capabilities are used
* which capabilities are excluded
* repository-specific conventions
* integration approach

Generic "what is X" descriptions are insufficient without repository-specific context.

---

## EC7. External Documentation Referenced

Authoritative external documentation is referenced.

The External Context document summarizes relevant knowledge — it does not reproduce the external project's documentation.

**Audit Rule:** External documentation is referenced rather than duplicated.

---

# Documentation Integrity

## EC8. No Internal Architecture Leakage

External Context does not describe:

* internal repository architecture
* internal feature specifications
* source code
* build configuration
* implementation patterns

These belong in Architecture, Engineering, or Feature Technical Design.

**Audit Rule:** No internal architecture has leaked into External Context.

---

## EC9. No Duplication of External Documentation

External Context summarizes and references.

It does not copy or reproduce sections of external documentation.

**Audit Rule:** External documentation is referenced rather than duplicated.

---

## EC10. Implementation Independence

External Context does not describe how the dependency is implemented in source code.

It describes the knowledge dependency — not the implementation of that dependency.

---

# Collection Quality

## EC11. Collection Completeness

All external dependencies that materially influence the repository have External Context documents.

Missing material dependencies should be identified.

Dependencies that are no longer relevant should be flagged for removal.

---

## EC12. Consistent Terminology

Dependency names, capability descriptions, and constraint terminology remain consistent throughout the External Context collection.

Contradictory dependency descriptions are not permitted.

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

Scores evaluate knowledge dependency quality rather than pass/fail compliance.

---

# Category Weights

| Category                  | Weight |
| ------------------------- | -----: |
| Document Quality          |    30% |
| Content Completeness      |    30% |
| Documentation Integrity   |    25% |
| Collection Quality        |    15% |

Weighted scores produce an overall **External Context Score (0–100)**.

---

# Success Criteria

The External Context Documentation should:

* contain one document per material dependency
* justify each dependency's existence
* explain repository-specific relevance
* document constraints that influence downstream documentation
* reference authoritative external sources
* remain free of internal architecture leakage
* avoid duplicating external documentation
* provide complete coverage of material dependencies
* maintain consistent terminology

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores (per dependency)
5. Validation Scores (EC1–EC12)
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
| Documentation Quality          | PASS / FAIL         |
| Content Completeness           | PASS / FAIL         |
| Documentation Integrity        | PASS / FAIL         |
| Downstream Reference Readiness | READY / NOT READY   |
| Architecture Leakage Risk      | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all External Context documents.
3. Verify every material dependency has a corresponding document.
4. Execute validation checks EC1–EC12.
5. Score every validation.
6. Score every External Context document.
7. Calculate weighted category scores.
8. Calculate the overall External Context Score.
9. Compare against the previous report when available.
10. Identify missing dependencies, architecture leakage, and documentation gaps.
11. Assess Downstream Reference Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/external-context/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
