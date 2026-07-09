# Audit Plan for Samgraha

## Overview

Samgraha's audit system verifies documentation quality against standard-defined rules and produces actionable findings across multiple pipelines. Two models exist: **Documentation Audit** (15 domains against standards) and **Custom Pipelines** (build, security, consistency, coverage, dependency).

---

## 1. Audit Pipelines — What and Why

### 1.1 Documentation Audit (default, `--pipeline doc`)

Checks compiled documentation against standard-defined rules using configurable providers.

| Domain | Why needed |
|--------|-----------|
| `architecture` | Ensure component model, communication, data flow, security, constraints are documented |
| `vision` | Verify problem statement, solution, success criteria, target audience are clear |
| `philosophy` | Validate guiding principles, values, tradeoffs are articulated |
| `readme` | Ensure README exists, has title, getting-started, documentation links |
| `feature` | Check FRs have purpose, acceptance criteria, inputs/outputs, constraints |
| `feature-design` | Validate UX principles, workflows, states, constraints |
| `feature-technical` | Verify component responsibilities, interactions, data governance, security |
| `design` | Check design principles, accessibility, UX principles, constraints |
| `engineering` | Validate code/build/test/security standards, guiding principles |
| `external-context` | Check integration contracts, dependencies, constraints |
| `prototype` | Verify scope, data model, mock APIs, constraints |
| `help` | Ensure help documentation quality |
| `build-guide` | Check build process is documented |
| `configuration` | Verify configuration docs exist |
| `tutorials` | Check tutorial coverage |

**Providers:**
- `deterministic` (default) — rule-based: section presence (`has_section`), title checks (`has_title`), corpus existence (`corpus_exists`), implementation leakage detection (`no_implementation`), compile diagnostic promotion (missing sections, prohibited content)
- `semantic` — heuristic: short docs (<50 words), placeholder text (TBD/TODO), technology references in vision/feature docs, missing rationale

### 1.2 Architecture Audit (`--pipeline architecture`)

Deep structural analysis of `docs/raw/architecture/` markdown files. 13 checks across 4 weighted categories (25/30/30/15):

| Check | Category | Weight | What it verifies |
|-------|----------|--------|-----------------|
| A1 | Collection Integrity | 6.25% | Modular architecture (≥2 files, or single doc with ≥3 H2 sections) |
| A2 | Collection Integrity | 6.25% | Completeness: system overview, component model, communication, data flow, security |
| A3 | Collection Integrity | 6.25% | Responsibility separation (each doc has clear H1 title) |
| A4 | Collection Integrity | 6.25% | No duplicate H1 titles across files |
| A5 | Structural Integrity | 7.5% | Ownership explicit (owns/responsible/boundary keywords) |
| A6 | Structural Integrity | 7.5% | Boundaries explicit (boundary/interface keywords) |
| A7 | Structural Integrity | 7.5% | Relationships documented (dependencies/interactions) |
| A8 | Structural Integrity | 7.5% | Communication & knowledge flow documented |
| A9 | Consistency | 7.5% | Shared terminology across documents (≥3 common terms) |
| A10 | Consistency | 7.5% | Traceability chain (cross-references, derived-from) |
| A11 | Consistency | 7.5% | Technology independence (no implementation keywords) |
| A12 | Consistency | 7.5% | Feature independence (no feature-specific language) |
| A13 | Cross-Repository | 15% | External architecture references documented |

### 1.3 Build Audit (`--pipeline build`)

Verifies build configuration matches documentation. Three levels:
- **Config-level**: checks `[pipelines.build]` contract exists, command is valid, working directory exists
- **Artifact inspection** (`--inspect-artifact`): verifies binary exists at declared path, has expected metadata
- **Execution** (`--execute`): runs the declared build command, verifies output

### 1.4 Security Audit (`--pipeline security`)

Checks security documentation against config and code. Two modes:
- **Static** (`--security`): verifies security docs exist, threat model present, security configuration matches
- **Runtime** (`--runtime`): connects to running app, verifies auth, TLS, rate limiting

### 1.5 Consistency Audit (`--pipeline consistency`)

Verifies alignment across adjacent documentation layers:
- Layer inventory → pairwise comparison → traceability check → contradiction scan → terminology check
- Detects: terminology drift, contradictory statements, missing cross-references

### 1.6 Coverage Audit (`--pipeline coverage`)

Bidirectional traceability between documentation and source code:
- **Forward** (doc→code): documented capabilities must have corresponding implementation
- **Reverse** (code→doc): code without documentation → orphan (warning, not error)
- Orphan resolution: Document / Remove / Suppress via `[audit.suppress]`

### 1.7 Dependency Governance (`--pipeline dependency`)

Spec-only (enabled = false by default). Verifies dependency justification, policy compliance, and health.

### 1.8 Semantic Audit Stages (MCP-only)

Four-stage pipeline for AI coding agents via MCP, gated by previous stage:

| Stage | How populated | Gate condition |
|-------|--------------|----------------|
| Deterministic | `samgraha audit` (CLI) | Always passes (no ERROR findings) |
| Section | `store_section_report` (MCP) | Deterministic gate passes |
| Document | `store_document_report` (MCP) | Section gate passes |
| Cross-Domain | `store_cross_domain_report` (MCP) | Document gate passes |

Each stage stores findings with status (`Open` / `Fixed` / `Accepted` / `Ignored` / `FalsePositive`).

---

## 2. How to Trigger Audits

### CLI

```bash
# Documentation audit (all domains, deterministic provider)
samgraha audit

# Single domain
samgraha audit architecture

# All domains explicitly
samgraha audit --all

# With semantic provider
samgraha audit --provider deterministic --provider semantic

# Quality gate (fail if score < 80)
samgraha audit --gate 80

# Generate markdown report
samgraha audit --report

# Pipeline-specific
samgraha audit --pipeline architecture
samgraha audit --pipeline build --inspect-artifact
samgraha audit --pipeline security --runtime
samgraha audit --pipeline consistency
samgraha audit --pipeline coverage

# Build execution (runs the declared build command)
samgraha audit --pipeline build --execute
```

### MCP Tools (for AI agents)

| Tool | Purpose |
|------|---------|
| `audit` | Run audit programmatically |
| `check_gate` | Verify stage gate is clear |
| `get_audit_knowledge` | Get audit criteria for a section type |
| `get_audit_report` | Read stored report |
| `get_section_changed` | Check if section changed since last audit |
| `store_section_report` | Submit section-level findings |
| `store_document_report` | Submit document-level findings |
| `store_cross_domain_report` | Submit cross-domain findings |
| `update_finding_status` | Mark finding as fixed/accepted/ignored |

### Samgraha Tools (via MCP)

```rust
samgraha_audit(domain: "architecture", providers: ["deterministic"])
samgraha_check_gate(stage: "section", document_id: 1)
samgraha_get_audit_knowledge(domain: "architecture", section_type: "component_model")
```

---

## 3. Reports Generated

### Output Locations

| Audit type | Report path (default) |
|-----------|----------------------|
| Documentation Audit | `docs/raw/reports/audit/latest/report.md` |
| Architecture Pipeline | `docs/raw/reports/architecture/latest/report.md` |
| Build Pipeline | `docs/raw/reports/build/latest/report.md` |
| Security Pipeline | `docs/raw/reports/security/latest/report.md` |
| Consistency Pipeline | `docs/raw/reports/consistency/latest/report.md` |
| Coverage Pipeline | `docs/raw/reports/coverage/latest/report.md` |
| Dependency Pipeline | `docs/raw/reports/dependency/latest/report.md` |

Each has `archive/` subdirectory with timestamped copies.

### Report Structure

```
# Audit Report
- Date, pipeline, provider
- Overall score
- Category scores (per domain or per check group)
- Findings table:
  [SEVERITY] check_id — message (location)
```

### Finding Format

```
Producer:   <source artifact + path>
Consumer:   <target artifact + path>
Contract:   <check ID + description>
Evidence:   <specific evidence>
Severity:   error | warning | suggestion
Status:     open | fixed | accepted | ignored | false_positive
```

### Severity Impact

| Severity | Effect on score |
|----------|----------------|
| `error` | Counts against score |
| `warning` | Reported, does not affect score |
| `suggestion` | Reported, does not affect score |

### Readiness Levels

| Score Range | Readiness |
|------------|-----------|
| ≥90%, no errors | Production |
| ≥80% | Implementation |
| ≥70% | Engineering |
| ≥60% | Design |
| ≥50% | Architecture |
| <50% | Product |

---

## 4. How to Fix Issues in Reports

### Workflow

1. **Run** `samgraha audit --report` to generate full findings
2. **Prioritize**: fix errors first (highest severity), then warnings, then suggestions
3. **Fix** the underlying document or code issue
4. **Recompile**: `samgraha compile`
5. **Re-audit**: `samgraha audit` to verify fixes
6. **Repeat** until gate passes (`--gate 80`)

### Finding Type → Fix

| Finding pattern | check_id prefix | Fix |
|----------------|----------------|-----|
| Missing required section | `compile-missing-section` | Add the missing `## Heading` to the document |
| Prohibited content in section | `compile-prohibited-content` | Remove implementation details from the section |
| Must document purpose | `*-001` (has_section) | Add `## Purpose` section |
| Must have a top-level title | `*-002` (has_title) | Add `# Title` as first H1 |
| Must not specify technology | `*-004` (no_implementation) | Move code/technology details to feature-technical doc |
| Document body under 50 words | `sem-001` | Add more context/content |
| Placeholder text found | `sem-002` | Replace TBD/TODO/FIXME with real content |
| Names specific technology | `sem-003` | Describe capability instead of technology |
| Architecture not modular | `A1` | Split single doc into multiple focused docs |
| Missing architectural concern | `A2` | Add missing sections: System Overview, Component Model, etc. |
| Technology independence | `A11` | Remove implementation keywords, describe responsibilities |
| Architecture lacks traceability | `A10` | Add cross-references to higher/lower-level docs |
| Coverage orphan | Coverage | Document the code, remove dead code, or suppress via config |
| Config mismatch | Build/Security | Update doc or config to match |

### MCP Finding Status Updates (for AI agents)

After fixing a documented finding, update its status:

```rust
samgraha_update_finding_status(
    report_id: <id>,
    criterion_id: "<check_id>",
    status: "fixed"           // or "accepted", "ignored", "false_positive"
)
```

### Suppressing False Positives

For findings that are intentional (e.g., a documented tech choice that looks like implementation leakage), use `[audit.suppress]` in `samgraha.toml`:

```toml
[audit.suppress]
patterns = ["feat-004:docs/raw/feature/specific-feature.md"]
```

---

## 5. Configuration

```toml
[audit]
default_severity = "suggestion"
providers = ["deterministic"]

[audit.gates.feature]
enabled = true
min_score = 80.0
min_readiness = "implementation"

[audit.pipelines.build]
enabled = true
artifact_inspection = "optional"

[audit.pipelines.security]
enabled = true
runtime_verification = "optional"

[audit.pipelines.consistency]
enabled = true

[audit.pipelines.coverage]
enabled = true
scanner = "simple"
```

---

## 6. Quick Reference

```bash
# Discover + compile docs (prerequisite)
samgraha compile

# Full doc audit with report and quality gate
samgraha audit --all --report --gate 80

# Architecture deep-dive
samgraha audit --pipeline architecture --report

# Build verification
samgraha audit --pipeline build --inspect-artifact

# Coverage check (bidirectional doc↔code)
samgraha audit --pipeline coverage --report

# List available pipeline reports
samgraha report --list-sessions
samgraha report --list-templates
```
