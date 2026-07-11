# QA Standard

## Table of Contents
- [Purpose](#purpose)
- [Test Strategy](#test-strategy)
- [Unit Testing](#unit-testing)
- [Integration Testing](#integration-testing)
- [End-to-End Testing](#end-to-end-testing)
- [Smoke Testing](#smoke-testing)
- [Load Testing](#load-testing)
- [Scalability Testing](#scalability-testing)
- [Security Testing](#security-testing)
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
- [Audit Rules](#audit-rules)
- [Validation Rules](#validation-rules)
- [Generation Rules](#generation-rules)
- [Enhancement Rules](#enhancement-rules)
- [Summary](#summary)
- [Common Mistakes](#common-mistakes)
- [Documentation Folder](#documentation-folder)
- [Usage](#usage)
- [Related](#related)

---

## Purpose

> **semantic_type:** `purpose`
> **scope:** How features are verified against their specifications — the testing contract for the documentation ecosystem
> **out_of_scope:** Implementation details, build pipelines, deployment procedures, release management
> **contributes:** Provides the verification layer that confirms Implementation(13) fulfills what Feature(04), Architecture(05), Design(06), Engineering(07), and Security(03) require
> **relationships:** Feature(04) defines what to test; Architecture(05) defines system boundaries for testing; Design(06) defines UX to validate; Engineering(07) defines test infrastructure; Security(03) defines security test requirements; Implementation(13) is what gets tested
> **responsibilities:** Define which test types are needed, their applicability conditions, and how they verify upstream documentation
> **generation_rules:** Start from Feature requirements; identify applicable test types based on project profile; define verification targets against upstream docs
> **enhancement_rules:** Add test types when new risk areas emerge; remove test types that no longer apply; keep test strategy aligned with project evolution
> **validation_rules:** Test types are clearly defined; applicability conditions are explicit; verification chain to upstream docs is complete
> **audit_rules:** Must exist; must define test types with applicability conditions; must reference upstream standards; must not define implementation details

This document defines the standard for Quality Assurance documentation within the engineering documentation ecosystem.

QA Documentation defines how features are verified against their specifications — the testing strategy, test types, and verification chain from feature requirements to implementation.

Unlike other standards that define what to build, QA defines how to verify that what was built matches what was specified.

---

## Test Strategy

> **semantic_type:** `test_strategy`
> **scope:** The overall testing approach — which test types apply, in what order, and with what priority
> **out_of_scope:** Specific test implementations, framework choices, CI/CD configuration
> **contributes:** Provides the testing roadmap that guides all test-related decisions
> **relationships:** Derived from Feature(04) requirements and Architecture(05) system boundaries; referenced by Implementation(13) for verification
> **responsibilities:** Define the testing pyramid appropriate for the project; specify which test types are mandatory vs. conditional
> **generation_rules:** Start from project profile (web app, CLI, ML pipeline, library); map test types to risk areas; prioritize by impact
> **enhancement_rules:** Adjust test mix as project evolves; add test types for new risk areas; remove tests for deprecated features
> **validation_rules:** Test strategy covers all applicable risk areas; priorities are justified; no gaps in verification chain
> **audit_rules:** Must exist; must define test type applicability; must reference project profile; must be justified by risk analysis

Every QA document must define the overall test strategy before individual test types. The strategy maps test types to project needs.

| Test Type | Applicability | Priority |
|-----------|--------------|----------|
| Unit Testing | All projects | Mandatory |
| Integration Testing | Projects with multiple components | Mandatory |
| End-to-End Testing | Applications with user-facing interfaces | Conditional |
| Smoke Testing | Deployed applications | Conditional |
| Load Testing | Applications expecting concurrent users | Conditional |
| Scalability Testing | Applications expecting growth | Conditional |
| Security Testing | All projects | Mandatory |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Unit Testing

> **semantic_type:** `unit_testing`
> **scope:** Testing individual components or functions in isolation — the foundation of the testing pyramid
> **out_of_scope:** Integration between components, user-facing behavior, performance characteristics
> **contributes:** Verifies that individual building blocks work correctly before they are composed
> **relationships:** Validates Feature(04) functional requirements at the component level; references Engineering(07) code standards for test quality
> **responsibilities:** Define unit test coverage targets, naming conventions, and assertion standards
> **generation_rules:** Derive from Feature(04) functional requirements; test one behavior per test; follow Arrange-Act-Assert pattern
> **enhancement_rules:** Add tests for new behaviors; update tests when behavior changes; remove tests for deleted features
> **validation_rules:** Coverage targets are defined; naming conventions are explicit; assertion standards are clear
> **audit_rules:** Must exist; must define coverage targets; must not test implementation details; must test behavior, not structure

Unit testing is applicable to all projects. Define coverage targets appropriate to the project's risk profile.

---

## Integration Testing

> **semantic_type:** `integration_testing`
> **scope:** Testing how components interact — verifying that composed units work together correctly
> **out_of_scope:** Individual component behavior, user-facing workflows, performance under load
> **contributes:** Verifies that the component model defined in Architecture(05) actually works when components communicate
> **relationships:** Validates Architecture(05) component interactions; references Feature(04) cross-component requirements
> **responsibilities:** Define which integration boundaries to test, contract verification approach, and data flow validation
> **generation_rules:** Derive from Architecture(05) component model; test each communication path; verify data contracts between components
> **enhancement_rules:** Add integration tests for new component connections; update when interfaces change; remove tests for deprecated paths
> **validation_rules:** Integration boundaries are defined; contract verification is explicit; data flow validation is complete
> **audit_rules:** Must exist for multi-component projects; must reference Architecture(05) component model; must test actual communication paths

Integration testing is mandatory for projects with multiple components. Map test coverage to Architecture(05) component boundaries.

---

## End-to-End Testing

> **semantic_type:** `e2e_testing`
> **scope:** Testing complete user workflows from start to finish — verifying the full stack works as the user experiences it
> **out_of_scope:** Individual component behavior, internal API contracts, implementation details
> **contributes:** Verifies that the user experience defined in Design(06) actually works in the running application
> **relationships:** Validates Design(06) user workflows; references Feature(04) user-facing requirements; tests against Implementation(13) as-built
> **responsibilities:** Define critical user journeys to test, expected outcomes, and acceptance criteria
> **generation_rules:** Derive from Design(06) user workflows and Feature(04) acceptance criteria; test the happy path first, then edge cases
> **enhancement_rules:** Add tests for new user journeys; update when workflows change; remove tests for deprecated features
> **validation_rules:** Critical user journeys are covered; expected outcomes are explicit; acceptance criteria are testable
> **audit_rules:** Must exist for user-facing applications; must reference Design(06) workflows; must have clear pass/fail criteria

End-to-end testing is conditional — required for applications with user-facing interfaces. Map test coverage to Design(06) user workflows.

---

## Smoke Testing

> **semantic_type:** `smoke_testing`
> **scope:** Quick sanity checks after deployment — verifying the application starts and core functions work
> **out_of_scope:** Deep functional testing, performance testing, edge case validation
> **contributes:** Provides the first line of defense after deployment; catches critical failures before users do
> **relationships:** References Implementation(13) deployed artifacts; validates Build(14) deployment output
> **responsibilities:** Define smoke test scope (what "core functions" means), pass/fail criteria, and execution timing
> **generation_rules:** Start from the most critical user journey; test that the application starts; verify core data flows
> **enhancement_rules:** Add smoke tests for new critical paths; update when core functions change; keep the suite fast
> **validation_rules:** Smoke tests are fast (< 5 minutes); cover critical paths; have clear pass/fail criteria
> **audit_rules:** Must exist for deployed applications; must be fast enough for post-deployment execution; must cover critical paths

Smoke testing is conditional — required for deployed applications. Must be fast enough to run after every deployment.

---

## Load Testing

> **semantic_type:** `load_testing`
> **scope:** Testing application behavior under expected and peak load — verifying performance meets requirements
> **out_of_scope:** Functional correctness, security vulnerabilities, deployment procedures
> **contributes:** Verifies that the application meets the performance requirements implied by Feature(04) and Architecture(05)
> **relationships:** References Architecture(05) scalability constraints; validates Engineering(07) performance standards
> **responsibilities:** Define load profiles (expected, peak, stress), performance targets, and acceptable degradation
> **generation_rules:** Derive from expected user load; define baseline, target, and stress profiles; set measurable performance targets
> **enhancement_rules:** Update load profiles as user base grows; adjust targets as requirements evolve; add tests for new critical paths
> **validation_rules:** Load profiles are realistic; performance targets are measurable; degradation behavior is defined
> **audit_rules:** Must exist for applications expecting concurrent users; must define realistic load profiles; must have measurable targets

Load testing is conditional — required for applications expecting concurrent users. Define realistic load profiles based on expected usage.

---

## Scalability Testing

> **semantic_type:** `scalability_testing`
> **scope:** Testing how the application behaves as load increases beyond normal — verifying it can grow gracefully
> **out_of_scope:** Current performance validation, functional testing, deployment procedures
> **contributes:** Verifies that Architecture(05) scalability decisions actually work under growth pressure
> **relationships:** References Architecture(05) scalability constraints; validates system design handles growth
> **responsibilities:** Define growth scenarios, breaking points, and scaling behavior expectations
> **generation_rules:** Start from Architecture(05) scalability model; test scaling behavior at 2x, 5x, 10x expected load
> **enhancement_rules:** Update growth scenarios as architecture evolves; adjust breaking point expectations; add tests for new scaling dimensions
> **validation_rules:** Growth scenarios are defined; breaking points are documented; scaling behavior is characterized
> **audit_rules:** Must exist for applications expecting growth; must reference Architecture(05) scalability model; must document breaking points

Scalability testing is conditional — required for applications expecting significant growth. Document where the system breaks and how it scales.

---

## Security Testing

> **semantic_type:** `security_testing`
> **scope:** Verifying that Security(03) requirements are enforced in the implementation — testing for vulnerabilities, compliance, and data protection
> **out_of_scope:** Security architecture design, threat modeling, security policy definition
> **contributes:** Verifies that Security(03) requirements and Engineering(07) security standards are actually implemented correctly
> **relationships:** Validates Security(03) threat model and data classification; references Engineering(07) security standards; tests Implementation(13) for vulnerabilities
> **responsibilities:** Define security test types (SAST, DAST, dependency scanning, secrets detection), coverage targets, and severity thresholds
> **generation_rules:** Derive from Security(03) threat model; map test types to threat categories; set severity thresholds for pass/fail
> **enhancement_rules:** Add tests for new threat categories; update thresholds as risk posture changes; remove tests for mitigated threats
> **validation_rules:** All Security(03) threat categories have corresponding tests; severity thresholds are defined; coverage targets are measurable
> **audit_rules:** Must exist; must reference Security(03) threat model; must cover all mandatory threat categories; must have measurable severity thresholds

Security testing is mandatory for all projects. Map test coverage to Security(03) threat categories. Severity thresholds determine pass/fail.

---

## Required Sections

Every QA document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Test Strategy | `test_strategy` | ✓ | Testing Strategy, QA Strategy |
| Unit Testing | `unit_testing` | ✓ | Unit Tests, Component Tests |
| Integration Testing | `integration_testing` | ✓ | Integration Tests, Contract Tests |
| End-to-End Testing | `e2e_testing` | | E2E Tests, UI Tests, Acceptance Tests |
| Smoke Testing | `smoke_testing` | | Sanity Tests, Post-Deploy Checks |
| Load Testing | `load_testing` | | Performance Tests, Stress Tests |
| Scalability Testing | `scalability_testing` | | Scale Tests, Growth Tests |
| Security Testing | `security_testing` | ✓ | Security Tests, Vulnerability Tests |
| Purpose | `purpose` | | Overview, Summary |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

A QA document aims to:

* Define which test types are needed for the project
* Specify applicability conditions so teams know what applies
* Create a verification chain from Feature(04) requirements to Implementation(13)
* Define measurable pass/fail criteria for each test type
* Guide test infrastructure investment based on risk

---

## Non-Goals

QA Documentation does not define:

* Test implementation details or framework choices
* CI/CD pipeline configuration
* Deployment procedures
* Release management processes
* Bug tracking or issue management

These responsibilities belong to Engineering(07) and Build(14).

---

## Success Criteria

QA Documentation is successful when:

* Every applicable test type has clear coverage targets
* The verification chain from requirements to implementation is complete
* Pass/fail criteria are measurable and automated where possible
* Test strategy evolves with the project without becoming outdated
* Teams know exactly which tests to write and why

---

## Responsibilities

QA Documentation is responsible for defining:

* Which test types apply to the project
* What each test type verifies against upstream documentation
* Coverage targets and pass/fail criteria
* The verification chain from requirements to implementation
* How test results map to documentation compliance

QA defines the verification contract. Implementation fulfills it. Build validates it.

---

## Scope

QA Documentation may describe:

* Test strategy and test type applicability
* Unit test coverage targets and conventions
* Integration test boundaries and contract verification
* End-to-end test scenarios and acceptance criteria
* Smoke test scope and execution timing
* Load test profiles and performance targets
* Scalability test growth scenarios and breaking points
* Security test coverage and severity thresholds

---

## Out of Scope

QA Documentation must not describe:

* Product vision or strategy
* Architecture or system design
* Implementation details or code structure
* Build pipeline configuration
* Deployment procedures
* Release management

These belong to their own documentation standards.

---

## Inputs

QA Documentation derives from:

* Feature(04) — what to verify
* Architecture(05) — system boundaries and component model
* Design(06) — user experience to validate
* Engineering(07) — test infrastructure and code standards
* External Context(08) — external testing constraints
* Security(03) — security test requirements
* Implementation(13) — what was actually built (as-built verification)

---

## Outputs

QA Documentation provides direction for:

* Test implementation and execution
* Build(14) test pipeline configuration
* Implementation(13) verification records
* Security(03) compliance verification

---

## Traceability

```text
Feature(04) ──── defines what to test
Architecture(05) ── defines system boundaries for testing
Design(06) ──────── defines UX to validate
Engineering(07) ─── defines test infrastructure
External Context(08) ─ defines external test constraints
Security(03) ────── defines security test requirements
       ↓
  QA(12) ──────── defines testing strategy and verification chain
       ↓
  Implementation(13) ── gets tested against QA(12) contract
       ↓
  Build(14) ────── validates test results in pipeline
```

---

## Relationships

| Document | Relationship |
|---|---|
| Feature(04) | Defines what QA verifies |
| Architecture(05) | Defines system boundaries for test scope |
| Design(06) | Defines UX workflows for E2E testing |
| Engineering(07) | Provides test infrastructure standards |
| External Context(08) | Provides external testing constraints |
| Security(03) | Defines security test requirements |
| Implementation(13) | Is what QA tests against |
| Build(14) | Validates QA results in pipeline |

---

## Required Characteristics

QA Documentation should be:

* Risk-driven — test types prioritized by impact
* Measurable — coverage targets and pass/fail criteria are quantifiable
* Applicable — test types have clear applicability conditions
* Traceable — every test type maps to upstream documentation
* Evolving — test strategy grows with the project

---

## Audit Rules

| ID | Check | Severity |
|----|-------|----------|
| `qa-001` | Has test strategy | error |
| `qa-002` | Test strategy defines applicability conditions | error |
| `qa-003` | Unit testing section exists | error |
| `qa-004` | Security testing section exists | error |
| `qa-005` | Integration testing section exists (multi-component projects) | warning |
| `qa-006` | Each test type references upstream documentation | error |
| `qa-007` | Coverage targets are measurable | error |
| `qa-008` | Pass/fail criteria are defined | error |

---

## Validation Rules

QA Documentation is considered valid when:

* Test strategy defines which test types apply
* Each applicable test type has measurable coverage targets
* Every test type references the upstream documentation it verifies
* Pass/fail criteria are explicit and automatable
* The verification chain is complete from requirements to implementation
* Security testing covers all Security(03) threat categories

---

## Generation Rules

When generating QA Documentation:

* Start from Feature(04) requirements — what needs to be verified
* Map test types to project profile (web app, CLI, ML pipeline, library)
* Define applicability conditions for each test type
* Create verification chain from test types to upstream documentation
* Set measurable coverage targets based on risk profile
* Define pass/fail criteria that can be automated

---

## Enhancement Rules

When enhancing QA Documentation:

* Add test types when new risk areas emerge
* Update coverage targets as project maturity increases
* Remove tests for deprecated features
* Improve test strategy alignment with current project profile
* Preserve the verification chain when upstream documentation changes

---

## Summary

QA Documentation is the verification layer of the documentation ecosystem. It defines which tests to write, what they verify, and how to know when the implementation is correct. Every test type maps to upstream documentation — Feature requirements, Architecture boundaries, Design workflows, Engineering standards, and Security requirements. The test strategy is risk-driven and project-specific: not every test type applies to every project, but every project needs a clear verification contract.

---

## Common Mistakes

Examples of incorrect QA content include:

* Defining test implementations instead of test contracts
* Skipping applicability conditions — assuming all test types apply everywhere
* No verification chain — tests exist but don't trace to requirements
* Coverage targets without measurement method
* Security testing as an afterthought instead of mandatory
* Defining CI/CD pipeline details instead of test strategy

---

## Documentation Folder

QA documents live under:

```text
docs/raw/qa/
```

---

## Usage

Written when the project's testing strategy is defined or changes; read by anyone writing tests or configuring test pipelines. Use `samgraha compile --domain qa` to validate structure, and `samgraha audit --domain qa` to verify the test strategy covers all applicable risk areas.

## Related

- [Feature Standard](04-feature-standards.md) — defines what QA verifies
- [Architecture Standard](05-architecture-standards.md) — defines system boundaries for test scope
- [Security Standard](03-security-standards.md) — defines security test requirements
- [Implementation Plan](13-implementation-standards.md) — what QA tests against
- [Build Plan](14-build-standards.md) — validates QA results in pipeline
- [Standards Reference Standard](standards.md) — how this standard itself is documented
