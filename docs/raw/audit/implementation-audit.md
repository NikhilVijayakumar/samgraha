# Implementation Audit

## Purpose

Verifies that the source code faithfully implements the documented system.

This audit evaluates the **implementation against the documentation**, ensuring that Architecture, Feature Technical Design, Engineering, and External Context are consistently realized in source code.

Implementation is **never the source of truth**.

Documentation defines the contract.

Source code must conform to that contract.

---

# Authority

Audit rules are defined by the validation checks in this document (I1–I15).

Implementation is validated against the repository's own documentation:

* Architecture Documentation (`docs/raw/architecture/`)
* Feature Technical Design Documentation (`docs/raw/feature-technical/`)
* Engineering Documentation (`docs/raw/engineering/`)
* External Context Documentation (`docs/raw/external-context/`)

---

# Scope

Applies to the implementation folder declared in Engineering Documentation.

Read the repository structure document under `docs/raw/engineering/` to determine the declared implementation folder before auditing. The implementation folder and the rationale for its structure must be documented there.

Compares implementation against:

```text
docs/raw/architecture/
docs/raw/feature-technical/
docs/raw/engineering/
docs/raw/external-context/
```

The audit evaluates the complete implementation.

---

# Validation Checklist

---

# Architectural Conformance

## I1. Architecture Realized

The documented Architecture is reflected in the implementation.

Examples include:

* component structure
* ownership
* layering
* boundaries
* responsibilities

Implementation should realize the documented Architecture.

---

## I2. Runtime Boundaries Preserved

Implementation respects documented runtime boundaries.

Examples include:

* process boundaries
* lifecycle boundaries
* communication boundaries
* repository boundaries

Runtime responsibilities should not leak across boundaries.

---

## I3. Communication Conformance

Communication paths implemented in code match documented communication.

Examples include:

* APIs
* events
* messaging
* persistence interactions

Undocumented communication paths should be reported.

---

## I4. Dependency Conformance

Implementation uses only documented architectural dependencies.

Unexpected architectural dependencies should be identified.

Circular dependencies should be reported.

---

# Feature Conformance

## I5. Feature Technical Design Realized

Feature implementations match the documented Feature Technical Design.

Documented responsibilities should exist in implementation.

Implementation should not invent undocumented architectural behavior.

---

## I6. Component Responsibilities Preserved

Component responsibilities in source code match the documented ownership.

Responsibilities should remain cohesive.

Responsibility leakage should be reported.

---

## I7. External Context Applied

Implementation correctly applies documented External Context.

Examples include:

* platform constraints
* protocol constraints
* dependency usage
* compatibility requirements

Implementation should respect documented constraints.

---

# Engineering Conformance

## I8. Engineering Standards Applied

Implementation follows documented Engineering standards.

Examples include:

* repository organization
* build conventions
* dependency strategy
* testing conventions
* coding standards

Engineering documentation should accurately describe implementation.

---

## I9. Repository Organization Conforms

Repository organization matches Engineering documentation.

Examples include:

* workspace layout
* module organization
* package structure
* ownership boundaries

Unexpected structural deviations should be reported.

---

## I10. Build and Dependency Conformance

Implementation follows the documented build strategy.

Dependency usage should remain consistent with Engineering documentation.

Undocumented dependencies should be reported.

---

# Documentation Integrity

## I11. Documentation Remains Implementation Independent

Documentation should not contain implementation details.

Examples include:

* source code
* algorithms
* framework APIs
* language syntax
* implementation patterns

Implementation belongs exclusively in source code.

---

## I12. No Architectural Drift

Implementation should not introduce undocumented architecture.

New architectural concepts discovered in source code should be reported.

Documentation should remain authoritative.

---

## I13. Traceability Complete

Every major implementation module should trace to documented Architecture or Feature Technical Design.

Orphan implementations should be identified.

---

# Implementation Quality

## I14. Naming Consistency

Implementation terminology remains consistent with documentation.

Examples include:

* component names
* module names
* feature names
* architectural terminology

Naming inconsistencies should be reported.

---

## I15. Future Maintainability

Implementation structure supports the documented architecture.

Implementation should remain maintainable.

Architecture should continue to guide future development.

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

Scores evaluate implementation conformance rather than code quality.

---

# Category Weights

| Category                  | Weight |
| ------------------------- | -----: |
| Architectural Conformance |    30% |
| Feature Conformance       |    25% |
| Engineering Conformance   |    20% |
| Documentation Integrity   |    15% |
| Implementation Quality    |    10% |

Weighted scores produce an overall **Implementation Conformance Score (0–100)**.

---

# Success Criteria

The implementation should:

* faithfully realize the documented Architecture
* correctly implement Feature Technical Design
* follow Engineering standards
* apply documented External Context
* preserve architectural boundaries
* introduce no undocumented architecture
* maintain complete documentation traceability
* remain consistent with documented terminology

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Module Scores
5. Validation Scores (I1–I15)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                      | Status              |
| ------------------------- | ------------------- |
| Architecture Conformance  | PASS / FAIL         |
| Feature Conformance       | PASS / FAIL         |
| Engineering Conformance   | PASS / FAIL         |
| Documentation Conformance | PASS / FAIL         |
| Production Readiness      | READY / NOT READY   |
| Documentation Drift Risk  | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Read the repository structure document under `docs/raw/engineering/` to determine the declared implementation folder and understand the rationale for its structure.
3. Inventory implementation modules under the declared implementation folder.
4. Load Architecture, Feature Technical Design, Engineering, and External Context documentation.
5. Execute validation checks I1–I15.
6. Score every validation.
7. Score every implementation module.
8. Calculate weighted category scores.
9. Calculate the overall Implementation Conformance Score.
10. Compare against the previous report when available.
11. Identify undocumented implementation, architectural drift, and documentation inconsistencies.
12. Assess Production Readiness and Documentation Drift Risk.
13. Generate the audit report using the Standard Audit Report format.
14. Write the report to:

```text
docs/raw/reports/implementation/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
