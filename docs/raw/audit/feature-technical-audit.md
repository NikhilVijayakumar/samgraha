# Feature Technical Design Audit

## Purpose

Verifies that the Feature Technical Design Documentation completely translates Feature Specifications into architectural realization while consistently applying the shared Architecture and respecting the Feature Design.

This audit evaluates the **Feature Technical Design collection as an integrated technical specification**, ensuring that every feature is mapped to one Feature Specification, applies shared Architecture, incorporates relevant External Context, respects Feature Design, and is sufficiently complete for implementation.

Feature Technical Design defines **how the architecture realizes a feature**.

It does not define **how the feature is implemented in code**.

---

# Authority

`docs/raw/standards/feature-technical.md` — **Audit Rules** section.

---

# Scope

Applies to every document under:

```text
docs/raw/feature-technical/
```

The audit evaluates:

* Individual Feature Technical Design quality
* Collection-wide consistency
* Architecture application
* Technical completeness
* Implementation readiness

The Feature Technical Design collection is evaluated as **one complete architectural realization of the product**.

---

# Validation Checklist

---

# Feature Mapping

## FT1. One-to-One Mapping

Every Feature Technical Design document maps to exactly one Feature Specification.

Mappings should be explicit.

Missing or duplicate mappings are reported.

---

## FT2. Feature Coverage Complete

Every Feature Specification has a corresponding Feature Technical Design document.

Missing technical specifications should be identified.

The collection should completely cover the product.

---

## FT3. Architecture Applied

Shared Architecture Documentation has been correctly applied.

Feature Technical Design should reference Architecture rather than redefine it.

Only feature-specific architectural realization belongs here.

---

## FT4. Feature Design Consulted Where Applicable

Feature Design is not a required input to Feature Technical Design.

Where user experience decisions directly influence architectural realization, Feature Design should be referenced.

Examples where Feature Design is relevant:

* navigation requiring routing architecture
* accessibility requiring structural support
* offline behavior requiring synchronization architecture

Feature Design should not be duplicated.

---

## FT5. Relevant External Context Applied

Relevant External Context is referenced.

Only dependencies affecting the feature's realization should be included.

External documentation should be referenced rather than duplicated.

---

# Technical Realization

## FT6. Component Responsibilities Defined

Every participating architectural component clearly defines:

* responsibilities
* ownership
* interactions
* boundaries

Responsibilities should remain consistent with the Architecture.

---

## FT7. Communication Flow Complete

Communication between participating components is documented.

Examples include:

* request flow
* event flow
* messaging
* persistence interactions
* dependency interactions

Communication should remain understandable without implementation knowledge.

---

## FT8. Runtime and Architectural Boundaries Respected

Feature realization respects:

* runtime boundaries
* process boundaries
* persistence boundaries
* communication boundaries
* repository boundaries

Feature Technical Design must not violate the documented Architecture.

---

## FT9. External Constraints Reflected

Constraints originating from External Context are reflected.

Examples include:

* platform limitations
* protocol requirements
* API constraints
* performance limitations
* compatibility requirements

Constraints should influence architectural realization.

---

# Documentation Quality

## FT10. Technology References Remain Architectural

Technology references are permitted only when architecturally significant.

Examples include:

* runtime platform
* persistence engine
* transport mechanism
* operating environment

Implementation technologies should not appear.

---

## FT11. No Implementation Leakage

Feature Technical Design should not contain:

* source code
* algorithms
* framework APIs
* programming language constructs
* interfaces
* database schemas
* SQL
* implementation patterns

Implementation belongs to Engineering.

---

## FT12. References Rather Than Duplication

Architecture Documentation, Feature Design, and External Context should be referenced rather than rewritten.

Duplicated documentation should be reported.

---

## FT13. Architectural Consistency

Component names, responsibilities, boundaries, communication, and terminology remain consistent throughout the Feature Technical Design collection.

Contradictory architectural guidance is not permitted.

---

# Implementation Readiness

## FT14. Implementation Readiness

Feature Technical Design provides sufficient architectural information for implementation.

Engineers should not need to invent:

* component responsibilities
* architectural interactions
* runtime behavior
* dependency usage

Implementation decisions should remain within Engineering.

---

## FT15. Future Maintainability

Feature Technical Design remains modular.

Changes to one feature should not require unrelated Feature Technical Design documents to change.

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

Scores evaluate technical specification quality rather than pass/fail compliance.

---

# Category Weights

| Category                 | Weight |
| ------------------------ | -----: |
| Feature Mapping          |    20% |
| Technical Realization    |    40% |
| Documentation Quality    |    20% |
| Implementation Readiness |    20% |

Weighted scores produce an overall **Feature Technical Design Score (0–100)**.

---

# Success Criteria

The Feature Technical Design Documentation should:

* provide a one-to-one mapping with Feature Specifications
* correctly apply the shared Architecture
* respect Feature Design
* incorporate relevant External Context
* define complete component responsibilities
* document communication and runtime behavior
* remain architecture-focused
* contain no implementation details
* provide a complete foundation for Engineering and implementation

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores
5. Validation Scores (FT1–FT15)
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
| Architecture Application       | PASS / FAIL         |
| Technical Specification        | PASS / FAIL         |
| Engineering Readiness          | READY / NOT READY   |
| Implementation Readiness       | READY / NOT READY   |
| Implementation Assumption Risk | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all Feature Technical Design documents.
3. Verify every Feature Specification has a corresponding Feature Technical Design document.
4. Execute validation checks FT1–FT15.
5. Score every validation.
6. Score every Feature Technical Design document.
7. Calculate weighted category scores.
8. Calculate the overall Feature Technical Design Score.
9. Compare against the previous report when available.
10. Identify findings and prioritized recommendations.
11. Assess Engineering and Implementation Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/feature-technical/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
