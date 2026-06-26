# Prototype Audit

## Purpose

Verifies that the Prototype Runtime completely validates the documented product before production implementation begins.

This audit evaluates the **Prototype Documentation and Prototype Runtime as an integrated validation environment**, ensuring that documented Features, Feature Designs, Feature Technical Designs, Engineering assumptions, and user workflows are validated through an executable prototype.

The Prototype Runtime exists to **validate documentation**, not to become production software.

---

# Authority

Audit rules are defined by the validation checks in this document (P1–P15).

Prototype coverage is validated against the repository's own documentation:

* Feature Documentation (`docs/raw/feature/`)
* Feature Design Documentation (`docs/raw/feature-design/`)
* Feature Technical Design Documentation (`docs/raw/feature-technical/`)
* Engineering Documentation (`docs/raw/engineering/`)
* External Context Documentation (`docs/raw/external-context/`)

---

# Scope

Applies to:

```text
docs/raw/prototype/
```

and validates against:

```text
docs/raw/feature/
docs/raw/feature-design/
docs/raw/feature-technical/
docs/raw/engineering/
docs/raw/external-context/
```

The audit evaluates the complete prototype environment.

---

# Validation Checklist

---

# Product Validation

## P1. Feature Coverage Complete

Every documented Feature is represented within the prototype.

Missing feature validation is reported.

---

## P2. Feature Design Validated

Feature Design workflows are implemented within the prototype.

The prototype accurately reflects documented user experience.

---

## P3. Feature Technical Design Supported

Prototype behavior supports the documented Feature Technical Design.

Mock APIs, navigation, data flow, and simulated behavior remain consistent with technical specifications.

---

## P4. User Workflows Complete

Every documented user workflow is executable.

Examples include:

* primary workflow
* alternative workflows
* failure scenarios
* recovery paths
* edge cases

---

# Runtime Validation

## P5. Navigation Complete

Navigation correctly represents the documented Feature Design.

Routes, transitions, dialogs, and navigation states behave consistently.

---

## P6. Mock API Contracts

Mock APIs satisfy documented Feature Technical Design contracts.

Requests, responses, validation, and simulated failures remain consistent.

---

## P7. Mock Persistence Consistency

Mock persistence behaves deterministically.

Supported operations include:

* create
* read
* update
* delete
* search
* filtering
* pagination (when applicable)

Behavior should remain predictable.

---

## P8. External Context Simulated

Relevant External Context is simulated where required.

Examples include:

* external services
* third-party APIs
* authentication providers
* operating system behavior
* platform capabilities

External systems should be simulated rather than connected.

---

# Engineering Validation

## P9. Engineering Assumptions Validated

The prototype validates documented Engineering assumptions.

Examples include:

* navigation model
* state management
* mock runtime
* repository organization
* prototype architecture

Assumptions should be verified before implementation.

---

## P10. Prototype Isolation

Prototype artifacts remain isolated from production implementation.

Prototype resources should be clearly identifiable.

Examples include:

* mock data
* mock services
* mock persistence
* prototype runtime

Prototype assets should never be mistaken for production code.

---

## P11. No Production Dependencies

Prototype Runtime should not depend upon:

* production APIs
* production databases
* production authentication
* production infrastructure
* production cloud services

Production systems should never be modified by the prototype.

---

## P12. Disposable Artifacts

Prototype artifacts remain disposable.

Deleting the prototype should never affect production implementation.

Prototype assets should always be reproducible.

---

# Validation Quality

## P13. Documentation Consistency

Prototype behavior remains consistent with:

* Feature Documentation
* Feature Design
* Feature Technical Design
* Engineering
* External Context

Contradictory behavior should be reported.

---

## P14. User Experience Fidelity

The prototype accurately communicates the intended product experience.

Users should be able to validate:

* usability
* workflows
* navigation
* interactions
* visual flow

without requiring production implementation.

---

## P15. Future Maintainability

Prototype Documentation and Runtime remain modular.

Feature additions should require localized prototype changes rather than broad rewrites.

Prototype Runtime should scale with the documentation.

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

Scores evaluate validation quality rather than prototype completeness alone.

---

# Category Weights

| Category               | Weight |
| ---------------------- | -----: |
| Product Validation     |    30% |
| Runtime Validation     |    30% |
| Engineering Validation |    20% |
| Validation Quality     |    20% |

Weighted scores produce an overall **Prototype Validation Score (0–100)**.

---

# Success Criteria

The Prototype Runtime should:

* validate every documented Feature
* validate Feature Design
* support Feature Technical Design
* execute complete user workflows
* simulate required External Context
* satisfy documented mock API contracts
* provide deterministic mock persistence
* remain isolated from production systems
* remain disposable
* provide sufficient confidence before implementation

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Prototype Module Scores
5. Validation Scores (P1–P15)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                     | Status              |
| ------------------------ | ------------------- |
| Prototype Quality        | PASS / FAIL         |
| Product Validation       | PASS / FAIL         |
| UX Validation            | PASS / FAIL         |
| Engineering Validation   | PASS / FAIL         |
| Implementation Readiness | READY / NOT READY   |
| Production Risk          | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory Prototype Documentation and Runtime artifacts.
3. Verify Feature, Feature Design, Feature Technical Design, Engineering, and External Context coverage.
4. Execute validation checks P1–P15.
5. Score every validation.
6. Score every prototype module.
7. Calculate weighted category scores.
8. Calculate the overall Prototype Validation Score.
9. Compare against the previous report when available.
10. Identify validation gaps, prototype inconsistencies, and production risks.
11. Assess Implementation Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/prototype/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
