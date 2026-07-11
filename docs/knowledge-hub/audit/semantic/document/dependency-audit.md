# Dependency Governance Audit

This section details the Dependency Governance Audit.

## Purpose

Verifies every dependency is justified, documented, allowed, correctly owned, respects version policy, and meets health and licensing standards.

This audit is framed as **governance** — ensuring the project's dependency posture is intentional, documented, and maintainable — rather than mere validation.

**Status: Specification only. Automated checks are not yet implemented.** This document defines the governance framework and future automated check surface. Manual review is possible today; automated execution is deferred.

---

# Authority

Audit rules are defined by the validation checks in this document (D1–D8).

---

# Scope

All dependency declarations across the repository:

* `Cargo.toml` (or equivalent package manifest)
* Lock file (e.g., `Cargo.lock`)
* Dependency documentation under `docs/raw/engineering/`
* External Context documentation referenced from dependency decisions

Dependencies include:
* Runtime dependencies (crates, packages, modules loaded at runtime)
* Development dependencies (test frameworks, build tools)
* Build dependencies (code generators, compilers, CI tooling)

---

# Validation Checklist

---

# Dependency Justification

This section details the Dependency Justification.

## D1. Every Dependency Justified

Every dependency declared in the manifest has a documented rationale in Engineering or External Context documentation.

Rationale should explain:
* what the dependency provides
* why it was chosen over alternatives
* what architectural or functional need it satisfies

---

## D2. Every Dependency Documented

Every dependency has an owner, purpose, and version policy documented in Engineering docs.

Documentation should include:
* owner (team or individual responsible)
* purpose (specific capability the dependency provides)
* version policy (semver range, pinning rules, update cadence)
* upstream source (crate registry, git, fork)

---

## D3. Dependency Ownership Explicit

Every dependency has an explicitly documented owner responsible for:
* reviewing updates and security advisories
* evaluating replacement or removal
* maintaining the dependency's documentation

---

# Version Policy

This section details the Version Policy.

## D4. Version Policy Respected

Every dependency's actual version constraint in the manifest respects the declared version policy.

Examples:
* `"^1.0"` is compatible with a "semver-compatible updates only" policy
* `"=0.5.3"` is compatible with a "pinned exact version" policy
* `"*"` is compatible with no policy (flagged as risk)

---

## D5. Supply-Chain Policy Applied

Dependencies sourced outside the primary registry (git, path, fork) must be declared in Engineering docs with:
* reason for alternative source
* trust assessment of the source
* synchronization strategy (if a fork)

---

# Dependency Health

This section details the Dependency Health.

## D6. Dependency Health Check

Every dependency is evaluated for:
* **Deprecated:** upstream marks it deprecated
* **Unmaintained:** no commits or releases in >2 years (or project-declared EOL)
* **Yanked:** removed from registry

Findings from health checks should be actionable: replace, fork, or accept with documented rationale.

---

## D7. Dependency Scope Correct

Every dependency is classified and declared as the correct scope:
* **Runtime:** linked into the production binary or loaded at runtime
* **Dev:** test frameworks, build tools, documentation generators
* **Build:** code generators, compile-time dependencies, CI tooling

Misclassified dependencies (e.g., a dev-only crate in `[dependencies]`) should be reported.

---

# Cross-References

This section details the Cross-References.

## D8. Dependency Governance Cross-Reference

Orphan dependency detection (dependencies in the manifest that lack documentation) is owned by **Coverage Audit** (CV12).

Dependency Governance focuses on justification, ownership, version policy, supply-chain policy, health, and scope — not orphan detection.

For dependencies that lack documentation, see CV12.

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

---

# Category Weights

| Category                | Weight |
| ----------------------- | -----: |
| Dependency Justification |    40% |
| Version Policy          |    25% |
| Dependency Health       |    25% |
| Cross-References        |    10% |

Weighted scores produce an overall **Dependency Governance Score (0–100)**.

---

# Success Criteria

The dependency posture should:

* every dependency be justified in Engineering docs
* every dependency have an owner and version policy
* version constraints match declared policy
* supply-chain sourcing be declared and assessed
* deprecated, unmaintained, and yanked dependencies be managed
* dependency scope (runtime/dev/build) be correctly declared
* orphan detection be deferred to Coverage Audit (CV12)

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Validation Scores (D1–D8)
5. Dependency Inventory with Status
6. Findings (Critical / Major / Minor / Observations)
7. Prioritized Recommendations
8. Governance Risk Assessment
9. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                     | Status              |
| ------------------------ | ------------------- |
| Dependency Justification | PASS / FAIL         |
| Version Policy Adherence | PASS / FAIL         |
| Dependency Health        | PASS / FAIL         |
| Governance Risk          | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all dependencies from the manifest and lock file.
3. Load Engineering dependency documentation and External Context.
4. Execute justification checks D1–D3.
5. Execute version policy checks D4–D5.
6. Execute dependency health checks D6–D7.
7. Execute cross-reference check D8 (deferral to Coverage Audit).
8. Score every validation.
9. Calculate weighted category scores.
10. Calculate the overall Dependency Governance Score.
11. Compare against the previous report when available.
12. Identify dependencies without justification, policy violations, and health risks.
13. Assess Governance Risk.
14. Generate the audit report using the Standard Audit Report format.
15. Write the report to:

```text
docs/raw/reports/dependency/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
