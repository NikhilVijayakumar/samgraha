# Audit System

This section details the Saṃgraha Audit System.

## Getting Started

This directory contains audit specifications for the Saṃgraha knowledge platform. To understand the audit system, start with the [Artifact Contract](#artifact-contract-philosophy) philosophy, then read the [Taxonomy](#taxonomy) to understand which audit applies to your need, then the [Pipeline Model](#pipeline-model) for execution.

---

## Artifact Contract Philosophy

Every engineering artifact publishes a contract declaring what it **Provides** and what it **Consumes**:

| Artifact | Provides | Consumes |
|---|---|---|
| Vision | Goals, constraints, success criteria | — |
| Architecture | Responsibilities, components, boundaries | Vision |
| Feature | Capabilities, business rules, inputs, outputs | Architecture |
| Feature Technical | Realization decisions, API surfaces, data flow | Feature, Architecture |
| Engineering | Runtime contracts, communication, dependencies | Feature Technical |
| Implementation | Modules, types, functions, configuration | Engineering, Feature Technical |
| Build | Artifact spec, runtime spec, targets, outputs | Engineering, Architecture |
| Security | Security properties, trust boundaries, access model | Architecture, Engineering |
| Dependency | Ownership, version policy, supply-chain policy | Engineering, External Context |

**Audit verifies: Consumer implements Producer Contract.**

An audit collects evidence that the consumer artifact satisfies the contract declared by the producer artifact. Findings reference the specific Producer → Consumer → Contract relationship.

---

## Taxonomy

| Audit | Artifacts Compared | Direction | File |
|---|---|---|---|
| Documentation Audit (15 specs) | docs ↔ standards | doc→standard | `vision-audit.md` through `readme-audit.md` |
| Implementation Audit | docs ↔ source code | doc→code | `implementation-audit.md` |
| Build Audit | build docs ↔ config ↔ artifact | doc→config→artifact | `build-audit.md` |
| Security Audit | security docs ↔ config ↔ code ↔ runtime | doc→config→code→runtime | `security-audit.md` |
| Consistency Audit | adjacent layers + cross-doc terminology | layer→layer | `consistency-audit.md` |
| Coverage Audit | docs ↔ implementation (bidirectional) | doc↔code | `coverage-audit.md` |
| Dependency Governance | docs ↔ dependency manifest | doc→manifest←code | `dependency-audit.md` |

Note: `build-audit.md` and `security-audit.md` each carry checks for two audit types — their existing B1-B12 / SEC1-SEC12 checks feed Documentation Audit, while their new BC / SC checks feed Build / Security Audit. Same file, two audit types consuming it.

---

## Authority Chain

Every audit validates against specific checks defined in its own spec file.

| Audit | Checks | Source |
|---|---|---|
| Vision Audit | V1–V12 | `vision-audit.md` |
| Architecture Audit | A1–A13 | `architecture-audit.md` |
| Design Audit | D1–D12 | `design-audit.md` |
| Feature Audit | F1–F14 | `feature-audit.md` |
| Feature Design Validation | FD1–FD15 | `feature-design-validation.md` |
| Feature Technical Audit | FT1–FT15 | `feature-technical-audit.md` |
| Prototype Audit | P1–P15 | `prototype-audit.md` |
| External Context Audit | EC1–EC12 | `external-context-audit.md` |
| External Context Ownership Audit | EC1–EC7 | `external-context-ownership-audit.md` |
| Engineering Audit | E1–E12 | `engineering-audit.md` |
| Build Audit (doc) | B1–B12 | `build-audit.md` |
| Build Audit (conformance) | BC1–BC10 | `build-audit.md` |
| Security Audit (doc) | SEC1–SEC12 | `security-audit.md` |
| Security Audit (conformance) | SC1–SC11 | `security-audit.md` |
| Deterministic Runtime Audit | S1–S12 | `deterministic-runtime-audit.md` |
| Implementation Audit | I1–I15 | `implementation-audit.md` |
| Readme Audit | R1–R12 | `readme-audit.md` |
| Consistency Audit | C1–C12 | `consistency-audit.md` |
| Coverage Audit | CV1–CV15 | `coverage-audit.md` |
| Dependency Governance | D1–D8 | `dependency-audit.md` |

---

## Pipeline Independence

Each pipeline runs standalone and is invoked on-demand via `--pipeline <name>`.

- There is no fixed global execution order.
- Consistency Audit's C6 (Build→Implementation Alignment) and C7 (Security→Implementation Alignment) are more accurate if Build and Security audits have run recently, but this is advisory, not enforced.
- Pipelines do not share state or depend on each other's outputs.

---

## Pipeline Model

Every audit follows the same conceptual flow:

```
Pipeline
  ↓
Evidence Collection
  ↓
Verification (against contracts)
  ↓
Findings (each references Producer → Consumer → Contract)
  ↓
Report
```

**Documentation Audit** uses the existing 4-stage pipeline (Deterministic → Section → Document → CrossDomain), implemented by `AuditFramework`.

**All other audits** (Build, Security, Consistency, Coverage, Dependency) use custom pipelines defined as standalone structs implementing the `Pipeline` trait:

| Pipeline | Evidence Collection | Verification |
|---|---|---|
| Build | Cargo.toml, CI YAML, build.rs, Dockerfile, binary artifact | Artifact Spec, Runtime Spec |
| Security | Security docs, config files, source code patterns, runtime behavior | Security Properties |
| Consistency | All documentation layers, build config, security config, impl structure | Pairwise alignment, terminology, contradiction |
| Coverage | Compiled knowledge base (docs), source code (parser), manifest | Forward match, reverse match, orphan detection |
| Dependency | Engineering docs, Cargo.toml, lockfile, External Context | Justification, policy, health, scope |

---

## Evidence Collection

Every pipeline collects evidence before verification:

```
Evidence Collection
├── Parse docs (extract contracts)
├── Scan config (Cargo.toml, CI YAML, build.rs, Dockerfile)
├── Analyze code (static analysis, pattern matching)
├── Inspect artifact (binary, embedded files) [opt-in]
└── Verify runtime (syscall inspection, behavior) [opt-in]
```

Documentation Audit evidence: documents under `docs/raw/` + standards under `docs/raw/standards/`.
Build Audit evidence: build docs + Cargo.toml + CI YAML + (opt-in) binary.
Security Audit evidence: security docs + config + source code + (opt-in) runtime.
Consistency Audit evidence: all documentation layers.
Coverage Audit evidence: compiled docs + source code + manifest.
Dependency Governance evidence: engineering docs + Cargo.toml + lockfile.

---

## Finding Format

Every finding references:

```
Producer:   <source artifact + path>
Consumer:   <target artifact + path>
Contract:   <audit check ID + description>
Evidence:   <specific evidence collected>
Severity:   error | warning | suggestion
Status:     open | fixed | accepted | ignored | false_positive
```

**Severity rules:**
- Orphan findings (code without documentation) are always **Warning**, never Error.
- Missing implementations (documented features not implemented) are **Error**.
- Grep-based parser findings are **Suggestion** (promote to Warning when tree-sitter parser ships).

**Status lifecycle:** open → fixed / accepted / ignored / false_positive.

---

## Report Format

Every audit report MUST contain the following sections:

### 1. Executive Summary
Summarizes corpus health, resolved findings from prior audit, new findings, and overall trajectory.

```
- **Overall Assessment:** [Poor / Fair / Good / Excellent]
- **Audit Score:** [X.X/10]
- **Critical Findings (P0):** [N]
- **Major Findings (P1):** [N]
- **Minor Findings (P2):** [N]
- **Informational (P3):** [N]
- **Documents Audited:** [N]
```

### 2. Score Details
Breakdown of how each dimension contributed to the score. Each audit defines its own dimensions based on its validation checklist items.

### 3. Findings by Severity
- **P0** — Critical: blocks correctness or safety. Must fix before next cycle.
- **P1** — Major: violates a mandatory Audit Rule. Must fix within 1 cycle.
- **P2** — Minor: violates a non-mandatory rule or quality concern. Should fix.
- **P3** — Informational: observations, suggestions, no rule violation.

### 4. Findings Detail
Each finding includes: ID, severity, file path + line, violated check, description.

### 5. Remediation Tracking
Findings from prior report listed with status: Resolved / Unresolved / New.

---

## Scoring Models

Each audit defines its own scoring model and category weights:

| Audit | Scoring |
|---|---|
| Documentation Audit (each domain) | Domain-specific category weights, overall 0–100 |
| Implementation Audit | Architectural Conformance (30%) + Feature Conformance (25%) + Engineering Conformance (20%) + Documentation Integrity (15%) + Implementation Quality (10%) |
| Build Audit | Engineering Strategy (25%) + Documentation Quality (20%) + Engineering Readiness (25%) + Build Conformance (30%) |
| Security Audit | Security Strategy (25%) + Documentation Quality (20%) + Security Readiness (25%) + Security Conformance (30%) |
| Consistency Audit | Layer Alignment (50%) + Cross-Layer Integrity (50%) |
| Coverage Audit | Bidirectional: `(forward_score + reverse_score) / 2` |
| Dependency Governance | Justification (40%) + Version Policy (25%) + Health (25%) + Cross-References (10%) |

---

## Report Lifecycle

Reports go in `docs/raw/reports/<domain>/latest/`. Previous reports rotate to `archive/`.

### Report Rotation

Before writing a new report, rotate the previous report:

```powershell
$domain = "<domain>"
$reportDir = "docs/raw/reports/$domain"
if (Test-Path "$reportDir/latest") {
    Move-Item -Path "$reportDir/latest/*" -Destination "$reportDir/archive/" -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Path "$reportDir/latest" -Force | Out-Null
```

This ensures every audit cycle has a before/after comparison to measure improvement.

---

## Exit Criteria

Each audit produces a report. All checks must pass before the corresponding artifact is accepted. An audit fails if any mandatory check is not satisfied or if the target artifact references a non-existent source.

Coverage Audit: forward coverage misses produce Error findings; reverse coverage misses (orphans) produce Warning findings. Forward coverage is expected to pass; reverse coverage (orphans) may be accepted or suppressed by the user.
