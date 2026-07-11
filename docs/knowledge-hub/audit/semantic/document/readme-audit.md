# README Audit

This section details the README Audit.

## Purpose

Verifies that the repository README serves as a concise, accurate, and maintainable entry point into the repository.

This audit evaluates the **README as the repository landing page**, ensuring that it introduces the project, explains its purpose, guides readers to the correct documentation, and accurately reflects the current state of the repository without duplicating detailed documentation.

The README is **not** a replacement for the documentation.

Its responsibility is to guide readers to it.

---

# Authority

Audit rules are defined by the validation checks in this document (R1–R12).

---

# Scope

Applies to:

```text
README.md
```

and, where applicable:

```text
docs/**/README.md
```

The audit evaluates the repository entry points.

---

# Validation Checklist

---

# Repository Introduction

This section details the Repository Introduction.

## R1. Purpose Immediately Understandable

A new reader should understand within the opening section:

* what the repository is
* why it exists
* who it is for
* its primary responsibilities

The project purpose should be immediately obvious.

---

## R2. Repository Responsibilities Defined

The README clearly explains:

* repository responsibilities
* repository boundaries
* major capabilities
* intended audience

Responsibilities should remain high level.

---

## R3. Repository Identity Consistent

Repository name, terminology, goals, and positioning remain consistent with:

* Vision
* Architecture
* Documentation Philosophy

Contradictory messaging should be reported.

---

# Documentation Navigation

This section details the Documentation Navigation.

## R4. Documentation Navigation Complete

The README provides clear navigation to major documentation.

Examples include:

* Vision
* Documentation Philosophy
* Architecture
* Design
* Features
* Engineering
* Prototype
* External Context

Readers should quickly locate detailed documentation.

---

## R5. Repository Structure Explained

Major repository directories are introduced.

Examples include:

* docs/
* src/
* examples/
* tools/

Descriptions should explain purpose rather than implementation.

---

## R6. Ecosystem Relationships Explained

When applicable, the README explains:

* upstream repositories
* downstream repositories
* shared libraries
* ecosystem relationships

Repository relationships should remain understandable.

---

# Documentation Quality

This section details the Documentation Quality.

## R7. No Documentation Duplication

The README summarizes documentation.

It should not reproduce:

* Architecture
* Feature Specifications
* Engineering standards
* Design principles

Detailed documentation belongs in dedicated documents.

---

## R8. Links Accurate

Internal and external links should resolve correctly.

Broken links, stale references, and outdated navigation should be reported.

---

## R9. README Scope Controlled

The README should remain concise.

It should avoid becoming:

* a wiki
* a tutorial
* an architecture document
* an engineering guide

Excessive detail should be reported.

---

## R10. Installation and Quick Start Appropriate

Installation and Quick Start guidance should match the repository purpose.

Examples include:

* prerequisites
* installation
* first command
* verification
* documentation links

Quick Start should help readers begin, not teach the entire system.

---

# Maintainability

This section details the Maintainability.

## R11. Documentation Synchronization

The README remains synchronized with the documentation.

Repository changes reflected in:

* Vision
* Architecture
* Engineering
* Features

should also be reflected where appropriate in the README.

Stale summaries should be reported.

---

## R12. Future Maintainability

The README should remain maintainable.

Documentation growth should not require proportional README growth.

The README should continue acting as a navigation layer rather than accumulating detailed documentation.

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

Scores evaluate repository onboarding quality rather than documentation completeness.

---

# Category Weights

| Category                 | Weight |
| ------------------------ | -----: |
| Repository Introduction  |    30% |
| Documentation Navigation |    30% |
| Documentation Quality    |    25% |
| Maintainability          |    15% |

Weighted scores produce an overall **README Score (0–100)**.

---

# Success Criteria

The README should:

* immediately communicate repository purpose
* define repository responsibilities
* accurately introduce the repository
* guide readers to detailed documentation
* explain repository structure
* describe ecosystem relationships when applicable
* avoid duplicating documentation
* remain concise and maintainable
* stay synchronized with repository documentation

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. README Scores
5. Validation Scores (R1–R12)
6. Trend Analysis
7. Findings (Critical / Major /Minor / Observations)
8. Prioritized Recommendations
9. Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                          | Status            |
| ----------------------------- | ----------------- |
| Repository Introduction       | PASS / FAIL       |
| Documentation Navigation      | PASS / FAIL       |
| Onboarding Experience         | PASS / FAIL       |
| Repository Discoverability    | READY / NOT READY |
| Documentation Synchronization | PASS / FAIL       |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all repository README files.
3. Verify repository identity and purpose.
4. Execute validation checks R1–R12.
5. Score every validation.
6. Score every README.
7. Calculate weighted category scores.
8. Calculate the overall README Score.
9. Compare against the previous report when available.
10. Identify documentation drift, navigation issues, duplicated content, and onboarding gaps.
11. Assess Repository Onboarding Readiness.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/readme/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
