# Architecture Audit

## Purpose

Verifies that the Architecture Documentation collectively defines a complete, coherent, implementation-independent architectural model of the system.

This audit evaluates the **architecture as an integrated collection**, not as independent documents. Individual documents may pass independently while the architecture as a whole may still fail if responsibilities overlap, terminology diverges, architectural relationships become inconsistent, or the documentation is not sufficiently mature for Engineering.

Architecture documents describe **system organization, responsibilities, ownership, boundaries, communication, knowledge flow, and architectural principles**. They must not describe implementation.

---

# Authority

`docs/raw/standards/architecture.md` — **Audit Rules** section.

---

# Scope

Applies to every document under:

```text
docs/raw/architecture/
```

The audit evaluates:

* Individual document quality
* Cross-document consistency
* Architectural completeness
* Responsibility separation
* Collection-wide coherence
* Engineering readiness

The Architecture collection is evaluated as **one architectural system**.

---

# Validation Checklist

Each validation maps to one or more rules defined in the Architecture Standard.

---

# Collection Integrity

## A1. Modular Architecture

Architecture is organized as a collection of focused documents.

Each document describes one architectural concern.

Large concerns are decomposed into multiple documents rather than accumulating unrelated responsibilities.

**Audit Rule:** Architecture is modular.

---

## A2. Architectural Completeness

The architecture collection completely describes the platform.

Required architectural concerns should be documented.

Examples include:

* System Overview
* Component Model
* Knowledge Flow
* Runtime Boundary
* Communication
* Persistence
* Security
* Extension Model
* Deployment
* Workspace (when applicable)

Missing architectural concerns are reported.

**Audit Rule:** Architecture completely describes the system.

---

## A3. Responsibility Separation

Every architecture document owns exactly one architectural responsibility.

Responsibilities must not overlap.

Documents should reference related concerns rather than redefine them.

**Audit Rule:** Responsibilities are clearly separated.

---

## A4. No Duplication

Architectural concepts are defined once.

Duplicate explanations, duplicated diagrams, duplicated responsibilities, or repeated architectural definitions are not permitted.

Shared concepts should be referenced instead.

**Audit Rule:** Documents do not duplicate one another.

---

# Structural Integrity

## A5. Ownership Explicit

Every architectural component clearly defines:

* ownership
* responsibilities
* boundaries

Ownership must remain unambiguous across the entire architecture.

**Audit Rule:** Ownership is explicit.

---

## A6. Boundaries Explicit

Architectural boundaries are documented.

Examples include:

* component boundaries
* runtime boundaries
* repository boundaries
* persistence boundaries
* communication boundaries
* deployment boundaries

Implicit boundaries are not permitted.

**Audit Rule:** Boundaries are documented.

---

## A7. Architectural Relationships

Relationships between architectural components are documented.

Examples include:

* dependencies
* interactions
* ownership
* delegation
* architectural layering

Relationships remain consistent throughout the architecture.

**Audit Rule:** Architectural relationships are explicit.

---

## A8. Communication & Knowledge Flow

Architecture documents collectively describe:

* communication paths
* responsibility flow
* ownership transfer
* knowledge flow

Knowledge movement should be understandable without implementation knowledge.

**Audit Rule:** Communication and knowledge flow are understandable.

---

# Consistency

## A9. Architectural Consistency

Component names, terminology, responsibilities, diagrams, and architectural concepts remain consistent across every architecture document.

Conflicting architectural definitions are not permitted.

**Audit Rule:** Architecture is internally consistent.

---

## A10. Traceability Complete

Every architecture document participates in the documentation traceability chain.

Each document:

* derives from higher-level documentation
* provides context for lower-level documentation

No orphan architecture documents exist.

**Audit Rule:** Architecture is fully traceable.

---

## A11. Technology Independence

Architecture remains independent of implementation technology.

Architecture must not describe:

* programming languages
* frameworks
* source code
* algorithms
* APIs
* database schemas
* protocols
* implementation patterns

Technology references are acceptable only when architecturally significant.

Implementation belongs to Engineering Documentation.

**Audit Rule:** Architecture avoids implementation detail.

---

## A12. Feature Independence

Architecture defines the structural foundation of the platform.

Architecture must not describe individual feature implementations or derive architectural decisions from Feature Documentation.

Feature Technical Design maps Features onto Architecture—not the reverse.

**Audit Rule:** Architecture supports Feature Technical Design without depending on Feature Documentation.

---

## A13. Cross-Repository References

When architectural concepts originate from external repositories, Architecture references the external documentation rather than duplicating it.

Repository relationships remain explicit.

**Audit Rule:** Cross-repository references are used instead of duplication.

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

Scores should reflect documentation quality rather than simple pass/fail.

---

## Category Weights

| Category                      | Weight |
| ----------------------------- | -----: |
| Collection Integrity          |    25% |
| Structural Integrity          |    30% |
| Consistency                   |    30% |
| Cross-Repository Architecture |    15% |

The weighted score produces an overall Architecture Score out of **100**.

---

# Success Criteria

The Architecture collection should:

* completely describe the platform
* have clearly separated responsibilities
* contain no duplicated architectural concepts
* maintain consistent terminology
* define explicit ownership and boundaries
* document communication and knowledge flow
* remain implementation independent
* provide complete traceability
* serve as a stable foundation for Engineering Documentation

---

# Audit Report Requirements

The generated report **must** follow the structure below.

---

# 1. Executive Summary

Provide a concise overview suitable for engineering leadership.

Include:

* Overall Score
* Previous Score (if available)
* Score Change
* Overall Status
* Engineering Readiness
* Summary of overall quality
* Major improvements
* Major regressions
* Estimated effort remaining

Example:

```text
Overall Score      : 93 / 100
Previous Score     : 88 / 100
Improvement        : +5

Status             : PASS
Engineering Ready  : YES

Summary

The architecture is internally consistent and implementation independent.
Knowledge Flow and Runtime Boundary significantly improved.
Minor terminology inconsistencies remain between Workspace and Deployment.

Estimated effort to reach Excellent: Low.
```

---

# 2. Overall Score

Provide:

* Overall Score (0–100)
* Quality Rating

| Score    | Rating            |
| -------- | ----------------- |
| 95–100   | Excellent         |
| 90–94    | Very Good         |
| 80–89    | Good              |
| 70–79    | Acceptable        |
| Below 70 | Needs Improvement |

---

# 3. Category Scores

Report weighted scores.

Example:

| Category             | Score | Weight |
| -------------------- | ----: | -----: |
| Collection Integrity |    96 |    25% |
| Structural Integrity |    92 |    30% |
| Consistency          |    90 |    30% |
| Cross-Repository     |    95 |    15% |

---

# 4. Document Scores

Every architecture document receives an independent score.

Example:

| Document         | Score |
| ---------------- | ----: |
| System Overview  |    95 |
| Component Model  |    93 |
| Knowledge Flow   |   100 |
| Runtime Boundary |    96 |
| Communication    |    91 |
| Persistence      |    90 |
| Security         |    94 |
| Extension Model  |    92 |
| Workspace        |    95 |
| Deployment       |    93 |

---

# 5. Validation Scores

Provide a score for every audit rule.

| Validation | Score |
| ---------- | ----: |
| A1         |    10 |
| A2         |     9 |
| A3         |    10 |
| ...        |   ... |

Every score must include a short justification.

---

# 6. Trend Analysis

If a previous report exists, compare against it.

Include:

* Overall improvement
* Category improvements
* Most improved document
* Most regressed document
* New issues introduced
* Previously resolved issues

If no previous report exists, state:

> Baseline audit established.

---

# 7. Findings

Group findings by severity.

## Critical

Must be corrected before Engineering.

---

## Major

Strongly recommended improvements.

---

## Minor

Quality improvements.

---

## Observations

Interesting findings that require no action.

Every finding must include:

* Validation Rule
* Document
* Location
* Explanation
* Recommendation

---

# 8. Recommendations

Rank recommendations.

## High Impact

Estimated Score Gain

Estimated Engineering Benefit

---

## Medium Impact

Estimated Score Gain

---

## Low Impact

Estimated Score Gain

Recommendations should be prioritized by engineering value rather than document order.

---

# 9. Readiness Assessment

Assess readiness for the next documentation stage.

Include:

| Area                             | Status            |
| -------------------------------- | ----------------- |
| Documentation Quality            | PASS / FAIL       |
| Architecture Quality             | PASS / FAIL       |
| Engineering Readiness            | READY / NOT READY |
| Feature Technical Design Support | READY / NOT READY |

Provide justification.

---

# 10. Audit Metadata

Include:

* Audit Type
* Repository
* Audit Date
* Architecture Documents Audited
* Validation Rules Executed
* Previous Report Used
* Audit Version
* Standard Version

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all documents under `docs/raw/architecture/`.
3. Verify required architectural concerns are present.
4. Execute validation checks A1–A13 across the complete collection.
5. Score every validation.
6. Score every architecture document.
7. Calculate category scores.
8. Calculate the weighted overall score.
9. Compare against the previous report if available.
10. Produce findings and prioritized recommendations.
11. Assess Engineering Readiness.
12. Generate the report following the required report structure.
13. Write the report to:

```text
docs/raw/reports/architecture/latest/
```

using the **Standard Report Format** defined in:

```text
docs/raw/audit/README.md#standard-report-format
```
