# External Context Audit

## Purpose

Verifies that the External Context Documentation completely, consistently, and accurately defines every external dependency used by the repository.

This audit evaluates the **External Context documentation as an integrated dependency knowledge base**, ensuring that every external dependency is documented once, referenced consistently, justified architecturally, and correctly applied throughout the documentation ecosystem.

External Context defines **external engineering knowledge**.

It does not document repository architecture, implementation, or feature behavior.

---

# Authority

`docs/raw/standards/external-context.md` — **Audit Rules** section.

---

# Scope

Applies to:

```text
docs/raw/external-context/
```

and validates references originating from:

```text
docs/raw/vision/
docs/raw/architecture/
docs/raw/design/
docs/raw/feature/
docs/raw/feature-design/
docs/raw/feature-technical/
docs/raw/engineering/
docs/raw/prototype/
```

The audit evaluates the entire documentation ecosystem.

---

# Validation Checklist

---

# Dependency Coverage

## EC1. External Dependencies Complete

Every external dependency referenced anywhere within the repository has a corresponding External Context document.

Missing dependencies are reported.

Unused External Context documents are also reported.

---

## EC2. One Dependency Per Document

Each External Context document describes exactly one dependency.

Multiple unrelated dependencies should never share one document.

Related technologies belonging to one engineering product may be documented together only when they represent one logical dependency.

---

## EC3. Dependency Purpose Explicit

Every dependency clearly explains:

* what it is
* why it exists
* why the repository depends on it
* architectural role
* engineering role

Purpose should never be implied.

---

## EC4. Constraints Documented

Every dependency documents relevant constraints.

Examples include:

* platform limitations
* performance constraints
* compatibility requirements
* licensing
* operational restrictions
* API limitations

Constraints should be actionable by downstream documentation.

---

# Documentation Integration

## EC5. Downstream Application

Documentation that references an external dependency correctly applies the documented constraints.

Examples include:

* Architecture
* Engineering
* Feature Technical Design
* Prototype

Referenced constraints should influence downstream decisions.

---

## EC6. Referenced Rather Than Duplicated

Documentation should reference External Context rather than duplicate external knowledge.

Duplicated documentation should be reported.

---

## EC7. Repository Relevance

Every External Context document clearly explains its relevance to the repository.

Generic technology descriptions are not sufficient.

Documentation should explain why the dependency matters to this repository.

---

## EC8. Repository Isolation

External Context documents describe only external systems.

Repository architecture, implementation details, feature behavior, or engineering decisions should not appear.

Internal knowledge belongs to repository documentation.

---

# Consistency

## EC9. Terminology Consistency

Dependency names, terminology, purposes, and constraints remain consistent across every documentation domain.

Contradictory descriptions are not permitted.

---

## EC10. Architecture Alignment

External Context supports the documented Architecture.

Dependencies should reinforce architectural decisions rather than contradict them.

Architectural responsibilities remain within Architecture Documentation.

---

## EC11. Engineering Alignment

Engineering Documentation correctly reflects the technologies and constraints described within External Context.

Engineering should not introduce undocumented external dependencies.

---

## EC12. Future Maintainability

External Context remains modular and maintainable.

Technology upgrades should require updates only to the relevant External Context document.

Downstream documentation should remain stable through references.

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

Scores evaluate documentation quality rather than pass/fail compliance.

---

# Category Weights

| Category                  | Weight |
| ------------------------- | -----: |
| Dependency Coverage       |    35% |
| Documentation Integration |    35% |
| Consistency               |    30% |

Weighted scores produce an overall **External Context Score (0–100)**.

---

# Success Criteria

The External Context Documentation should:

* document every required external dependency
* contain one dependency per document
* justify every dependency
* document relevant constraints
* remain repository-focused
* avoid duplicated external knowledge
* remain consistent throughout the documentation ecosystem
* support both Architecture and Engineering

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores
5. Validation Scores (EC1–EC12)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Engineering Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                             | Status            |
| -------------------------------- | ----------------- |
| Documentation Quality            | PASS / FAIL       |
| Dependency Documentation         | PASS / FAIL       |
| Architecture Support             | READY / NOT READY |
| Engineering Support              | READY / NOT READY |
| Feature Technical Design Support | READY / NOT READY |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all External Context documents.
3. Discover every external dependency referenced throughout the documentation repository.
4. Match every dependency to its corresponding External Context document.
5. Execute validation checks EC1–EC12.
6. Score every validation.
7. Score every External Context document.
8. Calculate weighted category scores.
9. Calculate the overall External Context Score.
10. Compare against the previous report when available.
11. Identify findings and prioritized recommendations.
12. Assess Engineering Readiness.
13. Generate the audit report using the Standard Audit Report format.
14. Write the report to:

```text
docs/raw/reports/external-context-ownership/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
