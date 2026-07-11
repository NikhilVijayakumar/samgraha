# Build Plan

## Table of Contents
- [Purpose](#purpose)
- [Documentation Quality](#documentation-quality)
- [Security Checks](#security-checks)
- [Size Checks](#size-checks)
- [ML Artifact Management](#ml-artifact-management)
- [CI/CD Validation](#cicd-validation)
- [Obfuscation & Optimization](#obfuscation--optimization)
- [Versioning & Naming](#versioning--naming)
- [Required Sections](#required-sections)
- [Goals](#goals)
- [Non-Goals](#non-goals)
- [Success Criteria](#success-criteria)
- [Responsibilities](#responsibilities)
- [Scope](#scope)
- [Out of Scope](#out-of-scope)
- [Inputs](#inputs)
- [Outputs](#outputs)
- [Traceability](#traceability)
- [Relationships](#relationships)
- [Required Characteristics](#required-characteristics)
- [Generation Rules](#generation-rules)
- [Enhancement Rules](#enhancement-rules)
- [Audit Rules](#audit-rules)
- [Validation Rules](#validation-rules)
- [Summary](#summary)
- [Common Mistakes](#common-mistakes)
- [Documentation Folder](#documentation-folder)
- [Usage](#usage)
- [Related](#related)
- [Revision History](#revision-history)

---

## Purpose

> **semantic_type:** `purpose`
> **scope:** How artifacts are generated, validated, and packaged — the build pipeline contract for the documentation ecosystem
> **out_of_scope:** Implementation details, code structure, QA testing strategy, deployment procedures
> **contributes:** Provides the packaging and validation layer that takes verified Implementation(13) and produces shippable artifacts
> **relationships:** Consumes Implementation(13) verified code; validates against QA(12) test results; produces artifacts for deployment
> **responsibilities:** Define artifact generation, quality checks, security validation, and packaging standards
> **generation_rules:** Start from the project profile (web app, CLI, ML pipeline, library); identify applicable build stages; define quality gates
> **enhancement_rules:** Add build stages when new artifact types emerge; improve quality checks as tooling matures; keep stages aligned with project needs
> **validation_rules:** Build stages cover all applicable artifact types; quality gates are defined; not all stages are mandatory
> **audit_rules:** Must exist; must define applicable build stages; must have quality gates; must not define code implementation

This document defines the standard for Build Plans — artifact generation and validation pipelines that take verified code and produce shippable packages.

Unlike other standards that define what to build or how to verify it, Build defines how to package, validate, and deliver what was built. Not every build stage applies to every project — the plan defines applicability conditions.

---

## Documentation Quality

> **semantic_type:** `documentation_quality`
> **scope:** Validating that documentation meets its own standards — structure, completeness, consistency, and accuracy
> **out_of_scope:** Documentation content decisions, writing style, translation
> **contributes:** Ensures the documentation ecosystem is internally consistent and valid before packaging
> **relationships:** Validates all documentation standards (01-16); references samgraha audit pipeline
> **responsibilities:** Define documentation quality checks, completeness verification, and consistency validation
> **generation_rules:** Run samgraha compile and audit against all domains; verify structure completeness; check cross-reference validity
> **enhancement_rules:** Add quality checks as new standards are added; improve validation accuracy as tooling matures
> **validation_rules:** All documentation compiles without errors; audit passes for all applicable domains; cross-references are valid
> **audit_rules:** Must exist; must validate all applicable documentation domains; must verify cross-reference integrity

Documentation quality is the first build stage. If the documentation is invalid, nothing downstream should proceed.

---

## Security Checks

> **semantic_type:** `security_checks`
> **scope:** Validating that artifacts are free from known vulnerabilities — dependency scanning, SAST, secrets detection
> **out_of_scope:** Security architecture, threat modeling, security policy definition
> **contributes:** Ensures the built artifact does not introduce known security risks
> **relationships:** Validates against Security(03) requirements; references Engineering(07) security standards; applies to Implementation(13) artifacts
> **responsibilities:** Define which security checks apply, severity thresholds for pass/fail, and remediation requirements
> **generation_rules:** Run dependency vulnerability scanning; perform SAST on source code; scan for secrets and credentials; set severity thresholds
> **enhancement_rules:** Add new security check types as threats evolve; update severity thresholds as risk posture changes
> **validation_rules:** All checks pass severity thresholds; no critical/high vulnerabilities; secrets scan is clean
> **audit_rules:** Must exist; must reference Security(03); must define severity thresholds; must not allow critical vulnerabilities

Security checks are mandatory for all projects. Map checks to Security(03) threat categories.

---

## Size Checks

> **semantic_type:** `size_checks`
> **scope:** Validating that artifacts meet size constraints — documentation bloat, binary size, dependency footprint
> **out_of_scope:** Performance testing, load testing, runtime behavior
> **contributes:** Ensures artifacts remain within acceptable size boundaries
> **relationships:** References Engineering(07) size constraints; validates Implementation(13) artifact size
> **responsibilities:** Define size limits, measurement methods, and enforcement rules
> **generation_rules:** Measure artifact size against defined limits; check documentation line counts; verify dependency footprint
> **enhancement_rules:** Adjust size limits as project evolves; add size checks for new artifact types
> **validation_rules:** Size limits are defined; measurement methods are explicit; enforcement rules are clear
> **audit_rules:** Must exist for size-sensitive projects; must define measurable limits; must have enforcement rules

Size checks are conditional — required for projects with size constraints (mobile apps, embedded systems, CLI tools).

---

## ML Artifact Management

> **semantic_type:** `ml_artifact_management`
> **scope:** Managing ML-specific artifacts — model versioning, data versioning, experiment tracking, model artifacts
> **out_of_scope:** Model training, model architecture, feature engineering
> **contributes:** Ensures ML artifacts are versioned, reproducible, and traceable
> **relationships:** References DVC or equivalent for data/model versioning; validates against Feature(04) ML requirements
> **responsibilities:** Define model versioning scheme, data versioning approach, experiment tracking requirements
> **generation_rules:** Version models with semantic versioning; track experiments with MLflow or equivalent; use DVC for data versioning
> **enhancement_rules:** Add versioning schemes for new artifact types; improve experiment tracking as tooling matures
> **validation_rules:** Models are versioned; data is versioned; experiments are tracked; artifacts are reproducible
> **audit_rules:** Must exist for ML projects; must define versioning scheme; must be reproducible

ML artifact management is conditional — required for projects with ML models. Use DVC for data versioning, MLflow for experiment tracking.

---

## CI/CD Validation

> **semantic_type:** `cicd_validation`
> **scope:** Validating that the CI/CD pipeline enforces all applicable quality gates — automated checks that run on every commit
> **out_of_scope:** Pipeline implementation details, specific CI/CD tool choices, deployment automation
> **contributes:** Ensures quality gates are enforced automatically, not manually
> **relationships:** References Engineering(07) CI/CD standards; validates against QA(12) test results; enforces Security(03) checks
> **responsibilities:** Define which gates run when, how failures are handled, and what blocks deployment
> **generation_rules:** Define gate sequence (lint → test → security → build); set failure policies; define deployment blockers
> **enhancement_rules:** Add gates as new checks become available; improve failure handling as pipeline matures
> **validation_rules:** All applicable gates are defined; failure policies are explicit; deployment blockers are identified
> **audit_rules:** Must exist for projects with CI/CD; must define gate sequence; must have failure policies

CI/CD validation is conditional — required for projects with automated pipelines. Define gates in the right sequence.

---

## Obfuscation & Optimization

> **semantic_type:** `obfuscation_optimization`
> **scope:** Post-build transformations — code obfuscation, minification, tree-shaking, dead code elimination
> **out_of_scope:** Runtime optimization, algorithm optimization, performance tuning
> **contributes:** Produces smaller, more secure, more efficient final artifacts
> **relationships:** References Engineering(07) optimization standards; applies to Build(14) output artifacts
> **responsibilities:** Define which transformations apply, their configuration, and their trade-offs
> **generation_rules:** Apply obfuscation for security-sensitive artifacts; optimize for size in constrained environments; preserve debug info for development builds
> **enhancement_rules:** Add new transformation types as tooling matures; adjust configuration as requirements change
> **validation_rules:** Transformations do not break functionality; debug info is preserved where needed; size reduction is measurable
> **audit_rules:** Must exist for release builds; must not break functionality; must preserve debug info for dev builds

Obfuscation and optimization are conditional — apply to release builds, not development builds. Document the trade-offs.

---

## Versioning & Naming

> **semantic_type:** `versioning_naming`
> **scope:** How artifacts are versioned and named — version schemes, naming conventions, compatibility rules
> **out_of_scope:** Feature versioning, API versioning, documentation versioning
> **contributes:** Ensures artifacts are identifiable, traceable, and compatible
> **relationships:** References Engineering(07) versioning standards; applies to all Build(14) output artifacts
> **responsibilities:** Define version scheme (semantic, calendar, etc.), naming conventions, and compatibility rules
> **generation_rules:** Use semantic versioning for libraries; use calendar versioning for applications; follow naming conventions consistently
> **enhancement_rules:** Add versioning schemes for new artifact types; improve naming conventions as patterns emerge
> **validation_rules:** Version scheme is defined; naming conventions are explicit; compatibility rules are documented
> **audit_rules:** Must exist; must define version scheme; must have naming conventions; must document compatibility rules

Versioning and naming are mandatory for all projects. Define the scheme once, apply it consistently.

---

## Required Sections

Every Build Plan must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Documentation Quality | `documentation_quality` | ✓ | Doc Quality, Documentation Checks |
| Security Checks | `security_checks` | ✓ | Security Validation, Vulnerability Scanning |
| Size Checks | `size_checks` | | Size Validation, Bloat Checks |
| ML Artifact Management | `ml_artifact_management` | | ML Versioning, Model Management |
| CI/CD Validation | `cicd_validation` | | Pipeline Validation, Gate Enforcement |
| Obfuscation & Optimization | `obfuscation_optimization` | | Optimization, Minification |
| Versioning & Naming | `versioning_naming` | ✓ | Versioning, Naming Conventions |
| Purpose | `purpose` | | Overview, Summary |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

A Build Plan aims to:

* Define which build stages apply to the project
* Specify quality gates that block deployment
* Ensure artifacts are validated before packaging
* Provide versioning and naming consistency
* Enable automated enforcement of quality standards

---

## Non-Goals

Build Plans do not define:

* Product vision or strategy
* Architecture or system design
* Implementation details or code structure
* QA testing strategy
* Deployment automation or release management

These belong to their own documentation standards.

---

## Success Criteria

A Build Plan is successful when:

* All applicable build stages are defined
* Quality gates enforce standards automatically
* Artifacts are validated before deployment
* Versioning and naming are consistent
* Security checks catch vulnerabilities before release

---

## Responsibilities

Build Plans are responsible for:

* Defining artifact generation and validation pipeline
* Enforcing quality gates automatically
* Ensuring artifacts meet security, size, and quality standards
* Providing versioning and naming consistency
* Recording build results for traceability

---

## Scope

Build Plans may describe:

* Documentation quality validation
* Security scanning and vulnerability detection
* Size constraint enforcement
* ML artifact versioning and management
* CI/CD pipeline gate configuration
* Obfuscation and optimization settings
* Versioning schemes and naming conventions

Not all build stages apply to every project. The plan defines applicability conditions.

---

## Out of Scope

Build Plans must not describe:

* Product vision or strategy
* Architecture or system design
* Code implementation details
* QA testing strategy (what to test)
* Deployment procedures or release management

These belong to their own documentation standards.

---

## Inputs

Build Plans derive from:

* Implementation(13) — what to package
* QA(12) — test results that gate the build
* Engineering(07) — code standards and CI/CD configuration
* Security(03) — security check requirements
* External Context(08) — external build constraints

---

## Outputs

Build Plans provide:

* Validated, packaged artifacts ready for deployment
* Build reports with quality gate results
* Versioned artifacts with naming consistency
* Security scan results for compliance records

---

## Traceability

```text
Implementation(13) ── as-built code
QA(12) ────────────── test results
Engineering(07) ───── code standards
Security(03) ──────── security requirements
External Context(08) ─ external constraints
         ↓
    Build(14) ──────── artifact generation + validation
         ↓
    Deployment ─────── validated artifact
```

---

## Relationships

| Document | Relationship |
|---|---|
| Implementation(13) | Provides code to package |
| QA(12) | Provides test results that gate the build |
| Engineering(07) | Provides code standards and CI/CD config |
| Security(03) | Provides security check requirements |
| External Context(08) | Provides external build constraints |

---

## Required Characteristics

Build Plans should be:

* Applicable — not every stage applies to every project
* Automated — quality gates run without manual intervention
* Measurable — pass/fail criteria are quantifiable
* Traceable — build results link back to source code and tests
* Secure — security checks are mandatory, not optional

---

## Audit Rules

| ID | Check | Severity |
|----|-------|----------|
| `build-001` | Has documentation quality checks | error |
| `build-002` | Has security checks | error |
| `build-003` | Has versioning and naming scheme | error |
| `build-004` | Conditional stages have applicability conditions | warning |
| `build-005` | Quality gates define pass/fail criteria | error |
| `build-006` | Security checks reference Security(03) | error |

---

## Validation Rules

A Build Plan is considered valid when:

* All mandatory stages exist (documentation quality, security, versioning)
* Conditional stages have clear applicability conditions
* Quality gates define measurable pass/fail criteria
* Security checks reference Security(03) requirements
* Build results are recorded for traceability

---

## Generation Rules

When generating a Build Plan:

* Start from the project profile (web app, CLI, ML pipeline, library)
* Identify which build stages apply
* Define quality gates with measurable pass/fail criteria
* Map security checks to Security(03) requirements
* Define versioning scheme before the first release
* Ensure CI/CD pipeline enforces all applicable gates

---

## Enhancement Rules

When enhancing a Build Plan:

* Add build stages when new artifact types emerge
* Improve quality gates as tooling matures
* Update security checks as threats evolve
* Refine versioning conventions as patterns solidify
* Keep applicability conditions current with project needs

---

## Summary

Build Plans are the artifact generation and validation layer of the documentation ecosystem. They take verified Implementation(13) code and produce shippable packages through a series of quality gates. Not every build stage applies to every project — the plan defines applicability conditions based on project type. Mandatory stages (documentation quality, security, versioning) apply everywhere; conditional stages (size checks, ML artifact management, obfuscation) apply only when relevant. Every build stage has measurable pass/fail criteria enforced automatically through CI/CD.

---

## Common Mistakes

Examples of incorrect Build content include:

* Making all stages mandatory when some are conditional
* No pass/fail criteria — gates exist but don't block anything
* Security checks as optional instead of mandatory
* No versioning scheme defined before the first release
* Manual quality gates instead of automated enforcement

---

## Documentation Folder

Build Plans live under:

```text
docs/raw/build/
```

---

## Usage

Written when the project's build pipeline is defined or changes; read by anyone configuring CI/CD, setting up quality gates, or packaging artifacts. Use `samgraha compile --domain build` to validate structure, and `samgraha audit --domain build` to verify the build plan covers all applicable stages.

## Related

- [Implementation Plan](13-implementation-standards.md) — provides code to package
- [QA Standard](12-qa-standards.md) — provides test results that gate the build
- [Engineering Standard](07-engineering-standards.md) — provides code standards and CI/CD config
- [Security Standard](03-security-standards.md) — provides security check requirements
- [README Standard](15-readme-standards.md) — references build artifacts
- [Standards Reference Standard](standards.md) — how this standard itself is documented

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| Draft | — | — | Initial proposal. Artifact generation plan replacing generic standard. |
