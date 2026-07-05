# Audit

## Purpose

What audit is, why it matters, and how it enforces documentation quality.

## Content

**Audit** is the process of verifying artifacts against declared contracts. It produces a quality or conformance report with findings classified by severity.

### Artifact Contract Principle

Every engineering artifact publishes a contract: what it **Provides** and what it **Consumes**. Audit verifies that the consumer implements the producer's contract.

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

### Why Audit Matters

- **Correctness** — Implementation matches documentation
- **Consistency** — All documents follow structural rules; adjacent layers align
- **Completeness** — Required sections present; no undocumented orphans
- **Purity** — Prohibited content flagged
- **Security** — Declared security properties are realized
- **Governance** — Dependencies are justified and healthy

### Audit Types

| Audit | What it checks |
|---|---|
| **Documentation Audit** (default) | Docs against standards (15 domain specs) |
| **Implementation Audit** | Docs vs source code |
| **Build Audit** | Build docs vs config vs artifacts |
| **Security Audit** | Security docs vs config vs code vs runtime |
| **Consistency Audit** | Adjacent layer alignment + terminology |
| **Coverage Audit** | Bidirectional doc↔code + all orphan detection |
| **Dependency Governance** | Dependency justification, policy, health (spec only) |

### Pipeline Model

Each audit type runs as an independent pipeline:

```
Pipeline → Evidence Collection → Verification → Findings → Report
```

**Documentation Audit** uses the existing 4-stage pipeline (Deterministic/Section/Document/CrossDomain). All other audits use custom pipelines with their own evidence collection.

### Documentation Audit Stages

1. **Deterministic** — Section existence, title, prohibited content (instant)
2. **Section quality** — Per-section heuristics
3. **Document quality** — Cross-section consistency
4. **Cross-domain** — Relationship validation between standards

### Orphan Findings

Orphans (code that exists without documentation) are always **Warning**, never Error. Resolution: document / remove / suppress. Coverage Audit owns all orphan detection.

### Scores

Each audit defines its own scoring model. Documentation Audit produces 0-100 per document/per domain. Coverage Audit uses a bidirectional formula.

### Running Audit

`samgraha audit [domain] [--pipeline <name>] [--provider <name>]... [--all] [--gate [<score>]] [--report] [--inspect-artifact] [--runtime]` — `--pipeline` selects audit type (default: `doc`), `--inspect-artifact` (Build Audit), `--runtime` (Security Audit).

## Related

- [Build Audit](build-audit.md)
- [Security Audit](security-audit.md)
- [Consistency Audit](consistency.md)
- [Coverage Audit](coverage.md)
- [Dependency Governance](dependency.md)
- [Audit Guide: Overview](../audit-guide/overview.md)
- [Command: audit](../commands/audit.md)
- [Configuration: audit](../configuration/audit.md)
