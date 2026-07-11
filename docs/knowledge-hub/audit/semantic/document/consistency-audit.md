# Consistency Audit

This section details the Consistency Audit.

## Purpose

Verifies that adjacent layers in the documentation and implementation chain maintain a consistent, traceable, and contradiction-free relationship.

Consistency Audit covers the full layer chain — Vision → Architecture → Feature → Feature Technical → Engineering → Implementation — and extends to **Build→Implementation** and **Security→Implementation** alignment. It also owns cross-document terminology consistency.

This audit does **not** detect orphan items (undocumented code or unimplemented features). Orphan detection belongs exclusively to Coverage Audit.

---

# Authority

Audit rules are defined by the validation checks in this document (C1–C12).

---

# Scope

All documentation layers, build configuration, security configuration, and implementation source code.

| Layer | Artifact Location |
|---|---|
| Vision | `docs/raw/vision/` |
| Architecture | `docs/raw/architecture/` |
| Feature | `docs/raw/feature/` |
| Feature Technical | `docs/raw/feature-technical/` |
| Engineering | `docs/raw/engineering/` |
| Implementation | Declared implementation folder |
| Build | `docs/raw/engineering/` + `Cargo.toml` + CI config |
| Security | `docs/raw/engineering/` + config files |

---

# Validation Checklist

---

# Layer Alignment

This section details the Layer Alignment.

## C1. Vision→Architecture Alignment

Architecture fulfills promises made in the Vision document.

Every capability, constraint, and success criterion declared in Vision should have a corresponding architectural component or principle.

---

## C2. Architecture→Feature Alignment

Features are scoped within Architecture.

Every Feature should trace to a documented architectural component or boundary. Features must not introduce architectural concepts absent from Architecture documentation.

---

## C3. Feature→Feature Technical Alignment

Technical designs faithfully implement their parent Feature.

Feature Technical documents should realize the capabilities, business rules, inputs, and outputs declared by the Feature. Technical designs must not add undocumented capabilities.

---

## C4. Feature Technical→Engineering Alignment

Engineering documentation supports the technical design.

Engineering decisions (build, dependencies, runtime, deployment) should accommodate the needs of Feature Technical designs without contradicting them.

---

## C5. Engineering→Implementation Alignment

Implementation follows Engineering documentation.

The declared build strategy, dependency policy, runtime contracts, and repository organization should be reflected in source code and configuration.

---

## C6. Build→Implementation Alignment

Build Spec (targets, features, outputs) matches the actual implementation and its configuration.

The declared build targets, feature flags, binary outputs, and embedded files in Build documentation must be realizable from the implementation. Implementation must not depend on build configuration that contradicts the Build Spec.

---

## C7. Security→Implementation Alignment

Security Properties documented in Security documentation are reflected in implementation.

Authentication, authorization, secrets management, data protection, and trust boundaries declared in Security documentation must be present in source code and configuration. Implementation must not bypass documented security mechanisms.

---

# Cross-Layer Integrity

This section details the Cross-Layer Integrity.

## C8. No Layer Skip

Every document references its parent layer.

Each artifact should cite the upstream document it implements or extends. A Feature document that references a Vision goal without mentioning Architecture is skip-risking — the Architecture may not agree.

---

## C9. Cross-Document Terminology Consistency

The same term carries the same meaning across all layers.

Every named concept (component, capability, boundary, role) used in two or more documents must agree on definition, scope, and semantics. Contradictory definitions should be resolved.

---

## C10. Constraint Propagation

Downstream layers respect constraints established upstream.

A capability constraint in Vision (e.g., "must work offline") must appear in Architecture, Feature, Engineering, Build, and Implementation as a binding constraint, not a suggestion.

---

## C11. No Contradiction

No document in the chain contradicts another.

Contradiction resolution:
- Contradictions against **Vision or Architecture** (upstream intent) are always resolved by fixing the code or downstream documents — those docs do not yield.
- Contradictions found only between **Feature-Technical-and-below** docs and code may be resolved by updating either side, same as orphans (document / remove / suppress).

---

## C12. Traceability Complete

Every artifact in the chain traces its parent.

Starting from any artifact, a reader should be able to follow the chain upward to Vision and downward to Implementation without gaps.

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

Scores evaluate consistency quality rather than pass/fail compliance.

---

# Category Weights

| Category             | Weight |
| -------------------- | -----: |
| Layer Alignment      |    50% |
| Cross-Layer Integrity |    50% |

Weighted scores produce an overall **Consistency Score (0–100)**.

---

# Success Criteria

The documentation and implementation chain should:

* maintain alignment between every adjacent layer
* avoid layer skips in traceability
* use consistent terminology across all documents
* propagate constraints faithfully downstream
* contain no contradictory statements
* trace bidirectionally from any artifact

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Validation Scores (C1–C12)
5. Trend Analysis
6. Findings (Critical / Major / Minor / Observations)
7. Prioritized Recommendations
8. Consistency Risk Assessment
9. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                     | Status              |
| ------------------------ | ------------------- |
| Layer Alignment          | PASS / FAIL         |
| Terminology Consistency  | PASS / FAIL         |
| Constraint Propagation   | PASS / FAIL         |
| Consistency Risk         | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all documentation layers and their relationships.
3. Load Vision, Architecture, Feature, Feature Technical, Engineering, Build, Security, and Implementation artifacts.
4. Execute pairwise alignment checks C1–C7.
5. Execute cross-layer integrity checks C8–C12.
6. Score every validation.
7. Calculate weighted category scores.
8. Calculate the overall Consistency Score.
9. Compare against the previous report when available.
10. Identify contradictions, terminology drift, and traceability gaps.
11. Assess Consistency Risk and terminology alignment.
12. Generate the audit report using the Standard Audit Report format.
13. Write the report to:

```text
docs/raw/reports/consistency/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
