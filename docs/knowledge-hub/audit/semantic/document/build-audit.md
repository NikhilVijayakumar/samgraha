# Build Audit

This section details the Build Audit.

## Purpose

Verifies that the Build Engineering documentation defines a complete, consistent, maintainable, and implementation-ready build strategy for the repository, **and** that Build Configuration and produced artifacts conform to that strategy.

This audit evaluates the **build documentation as an integrated engineering collection**, ensuring that build decisions, technology choices, packaging, automation, reproducibility, and deployment preparation are fully documented and aligned with the Architecture.

Build documentation describes **engineering strategy**. The audit then verifies that strategy is faithfully realized in configuration and artifacts.

---

# Authority

Audit rules are defined by the validation checks in this document (B1–B12) for documentation quality, and (BC1–BC10) for build conformance.

---

# Scope

Applies to build-related engineering documentation under:

```text
docs/raw/engineering/
```

Examples include:

* build-strategy.md
* packaging.md
* release-strategy.md
* ci-cd.md
* toolchain.md

Configuration files may be referenced for verification only.

Examples include:

* Cargo.toml
* package.json
* pyproject.toml
* tsconfig.json
* GitHub Actions
* GitLab CI
* Azure Pipelines

Configuration files are **implementation artifacts**, not engineering documentation.

---

# Validation Checklist

---

# Engineering Strategy

This section details the Engineering Strategy.

## B1. Build Principles Defined

Repository-wide build principles are documented.

Examples include:

* deterministic builds
* reproducible builds
* incremental builds
* offline builds
* dependency isolation
* packaging philosophy

Engineering principles should guide every build decision.

---

## B2. Technology Selection Includes Rationale

Every major build technology includes documented rationale.

Examples:

* build system
* compiler
* package manager
* testing framework
* release tooling
* CI platform

Documentation explains:

* why it was selected
* architectural alignment
* major alternatives considered
* trade-offs

---

## B3. Architecture Alignment

Build engineering supports the documented Architecture.

The build strategy should align with:

* deployment architecture
* runtime architecture
* workspace architecture
* component model

Engineering decisions must not contradict architectural principles.

---

## B4. External Context Applied

Relevant external dependencies are documented through External Context.

Examples include:

* Cargo
* Rust toolchain
* GitHub Actions
* Azure Pipelines
* Docker
* Package registries

External documentation should be referenced rather than duplicated.

---

# Documentation Quality

This section details the Documentation Quality.

## B5. Modular Documentation

Build Engineering is organized into focused documents.

Examples:

* build strategy
* packaging
* release
* CI/CD
* toolchain

Monolithic documentation should be decomposed.

---

## B6. Responsibility Separation

Responsibilities remain clearly separated.

Examples:

* Build
* Packaging
* Release
* Deployment
* Dependencies

Documentation should reference related concerns rather than duplicate them.

---

## B7. Repository-wide Scope

Build Engineering defines repository-wide engineering standards.

Feature-specific build procedures should not appear unless the repository architecture explicitly requires them.

---

## B8. Implementation Independence

Engineering documentation explains engineering decisions rather than implementation.

Engineering documentation should not contain:

* build scripts
* CI YAML
* Makefiles
* shell scripts
* Cargo commands
* source code

Implementation belongs in repository configuration.

---

## B9. Explicit Engineering Rationale

Every significant engineering decision explains:

* what
* why
* alternatives considered
* trade-offs
* architectural impact

Engineering intent should never rely on implied knowledge.

---

# Engineering Readiness

This section details the Engineering Readiness.

## B10. Build Process Completeness

An engineer unfamiliar with the repository should understand:

* how the project is built
* packaging strategy
* release strategy
* artifact generation
* dependency management
* build lifecycle

without reading implementation files.

---

## B11. Cross-Document Consistency

Build Engineering remains consistent with:

* Architecture
* Engineering
* External Context
* Dependency documentation
* Deployment strategy

Contradictory engineering guidance is not permitted.

---

## B12. Future Maintainability

Documentation should remain maintainable.

Technology changes should require updates only within the appropriate engineering documents.

Engineering responsibilities should remain modular.

---

# Build Conformance

This section defines checks that verify Build Configuration and produced artifacts conform to Build Documentation.

Evidence for these checks comes from the repository's own declared Pipeline
Contract (`samgraha.toml [pipelines.build]`) — command, working directory,
and declared artifacts — not from any assumption about which build system
the repository uses. A repository built with Cargo, npm, a Makefile, or a
custom script all satisfy these checks the same way: by declaring their own
`[pipelines.build]` contract and keeping it truthful. See
`help/concepts/pipeline-contracts.md`.

## Artifact Spec Declaration

Build Documentation should embed an Artifact Spec and Runtime Spec that declare the contract downstream artifacts must satisfy:

```markdown
## Artifact Spec

- name: samgraha
- targets: [x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu, x86_64-apple-darwin]
- outputs: [bin/samgraha.exe (windows), bin/samgraha (unix)]
- embedded_files: [help.db, standards.db]
- features: [sqlite, mcp-server]

## Runtime Spec

- requires: [sqlite, stdio]
- embedded_db: [help.db, standards.db]
- offline: true
- network: none
```

---

## BC1. Build Principles Realized

Doc-declared build principles are visible in Build Configuration.

**Level:** Config (always runs)

---

## BC2. Target Platforms Conform

Doc-declared target platforms are built by CI and produce binaries.

**Level:** Artifact* (opt-in — requires `--inspect-artifact`)

---

## BC3. Feature Completeness

Every doc-declared feature exists in the repository's declared Pipeline
Contract (`samgraha.toml [pipelines.build]`), not any single build system's
own feature-flag syntax.

**Level:** Config (always runs)

---

## BC4. Dependency Rationale

Every dependency declared in config has rationale in build documentation.

**Level:** Config (always runs)

**Note:** Undocumented dependency detection is owned by Coverage Audit (CV12).

---

## BC5. CI Platform Alignment

Doc-declared CI platform matches the actual CI configuration.

**Level:** Config (always runs)

---

## BC6. Output Completeness

Doc-declared binary outputs exist as declared `artifacts` in the Pipeline
Contract, and the produced artifact matches.

**Level:** Artifact* (opt-in)

---

## BC7. Build Config Self-Consistency

Build configuration contains no contradicting settings.

**Level:** Config (always runs)

---

## BC8. External Context Applied

External dependencies referenced in build configuration are documented through External Context.

**Level:** Config (always runs)

---

## BC9. Artifact Contents Match Spec

Declared embedded files are present in the produced binary.

**Level:** Artifact* (opt-in)

---

## BC10. Future Maintainability

Build configuration changes require updates only within the appropriate engineering documents.

**Level:** Config (always runs)

---

`*` = Artifact-level checks require a built binary and are opt-in via `--inspect-artifact` flag or `[audit.pipelines.build] artifact_inspection = "always"` config.

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

Scores measure engineering quality rather than pass/fail compliance.

---

# Category Weights

| Category              | Weight |
| --------------------- | -----: |
| Engineering Strategy  |    25% |
| Documentation Quality |    20% |
| Engineering Readiness |    25% |
| Build Conformance     |    30% |

Weighted scores produce an overall **Build Engineering Score (0–100)**.

---

# Success Criteria

The Build Engineering documentation and conformance should:

* define repository-wide build principles
* justify engineering decisions
* align with Architecture
* remain implementation independent
* maintain modular responsibilities
* reference External Context appropriately
* prepare engineers for implementation
* remain maintainable over time
* realize documented principles in Build Configuration
* produce artifacts matching the Artifact Spec

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Overall Score
3. Category Scores
4. Document Scores
5. Validation Scores (B1–B12, BC1–BC10)
6. Trend Analysis
7. Findings (Critical / Major / Minor / Observations)
8. Prioritized Recommendations
9. Engineering Readiness Assessment
10. Audit Metadata

---

# Readiness Assessment

Assess:

| Area                      | Status            |
| ------------------------- | ----------------- |
| Engineering Documentation | PASS / FAIL       |
| Build Strategy            | READY / NOT READY |
| CI/CD Readiness           | READY / NOT READY |
| Release Readiness         | READY / NOT READY |
| Implementation Readiness  | READY / NOT READY |
| Build Conformance         | PASS / FAIL       |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all build-related engineering documents.
3. Verify required engineering concerns are present.
4. Execute documentation validation checks B1–B12.
5. Execute conformance checks BC1–BC10 (config-level always; artifact-level opt-in).
6. Score every validation.
7. Score every engineering document.
8. Calculate weighted category scores.
9. Calculate the overall Build Engineering Score.
10. Compare against the previous report when available.
11. Identify findings and prioritized improvements.
12. Assess Engineering Readiness and Build Conformance.
13. Generate the audit report using the Standard Audit Report format.
14. Write the report to:

```text
docs/raw/reports/build/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
