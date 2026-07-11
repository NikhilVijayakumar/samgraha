# Coverage Audit

This section details the Coverage Audit.

## Purpose

Bidirectional contract verification between documentation and implementation: every documented capability should be implemented, and every implemented capability should be documented.

Coverage Audit **owns all orphan detection**. No other audit spec (Build, Security, Implementation, Dependency) contains orphan checks. Orphans found by Coverage are the single source of truth for undocumented code or unimplemented features.

**Phase 1 scope:** documentation ↔ implementation (manifest + simple pattern parser).
**Future phases** (noted but not implemented): documentation ↔ configuration, documentation ↔ build.

Documentation ↔ tests (CV6) is implemented — see CV6 below for the `[pipelines.test]` contract.

---

# Authority

Audit rules are defined by the validation checks in this document (CV1–CV15).

---

# Scope

Applies to all documentation layers and the implementation source code.

Coverage compares documentation against source code (and vice versa), using:
- Compiled knowledge base (documents)
- Source code (simple pattern matching)
- Manifest files (`Cargo.toml`, configuration files)
- Repository structure

Evidence is collected automatically. The scanner uses a manifest-driven approach with grep-based pattern matching in Phase 1.

---

# Scanner Notes

**Phase 1 parser:** manifest + simple grep-based pattern matching.
- Deliberately imprecise — matches by convention, not by AST.
- Findings produced by the grep-based parser are emitted at **Suggestion** severity.
- Severity promotes to **Warning** once a tree-sitter-based parser replaces grep.
- Tree-sitter integration is tracked as a future enhancement; no timeline is set.
- Suppression: individual findings may be suppressed via `[audit.suppress]` config.

**Orphan findings are always Warning severity** (never Error), regardless of scanner precision. The user decides the resolution: document / remove / suppress.

**Source location:** the scanner reads `repository.implementation.dir` and
`repository.tests.dir` from `samgraha.toml` (with `repository.source_exclude`
applied) to find source — not a hardcoded `src/`. A repository declares where
its own code lives once; the scanner never guesses a convention.

---

# Validation Checklist

---

# Forward Coverage (doc → code)

This section verifies that documented items exist in implementation.

## CV1. Documented Features Implemented

Every Feature document has corresponding code.

Producer: Feature documentation → Consumer: Implementation
Severity: Error (missing implementation)

---

## CV2. Architecture Components Exist

Every Architecture component declared in docs has a corresponding module or structure.

Producer: Architecture documentation → Consumer: Implementation
Severity: Error (missing component)

---

## CV3. Documented APIs Available

Every API documented in Feature Technical or Engineering docs has a corresponding function signature.

Producer: Feature Technical / Engineering documentation → Consumer: Implementation
Severity: Error (missing API)

---

## CV4. Documented CLI Commands Work

Every CLI command declared in documentation exists in the argument parser.

Producer: Engineering / Feature documentation → Consumer: CLI configuration
Severity: Error (missing command)

---

## CV5. Documented Config Keys Accepted

Every configuration key documented in Engineering docs is handled by the config parser.

Producer: Engineering documentation → Consumer: Configuration
Severity: Error (missing config key)

---

## CV6. Documented Capabilities Tested

Every documented capability has test coverage. Backed by a repo-declared `[pipelines.test]` contract (same shape as `[pipelines.build]`): the repository supplies a script (`pwsh`/`sh`, whatever fits its own stack — `cargo test` + `tarpaulin`, `pytest` + `coverage.py`, etc.) that runs its unit and e2e suites and writes structured JSON results to the contract's first declared `artifacts` path:

```jsonc
{
  "unit": { "total": 42, "passed": 40, "failed": 2, "skipped": 0, "failures": [{"name": "test_foo", "message": "assertion failed"}] },
  "e2e":  { "total": 5,  "passed": 5,  "failed": 0, "skipped": 0, "failures": [] },
  "coverage_percent": 78.4
}
```

Run with `samgraha audit --pipeline coverage --execute` to run the script and read results, or without `--execute` to read a previously-produced results file if one exists. No `[pipelines.test]` declared, or no results produced yet, falls back to the advisory "not adopted" message.

Full field-by-field reference and the machine-validatable JSON Schema: [Test Report Schema](test-report-schema.md).

**Advisory** — real results, but still non-blocking: CV6 never contributes to the Forward Coverage pass/fail count (weight 0.0, see Severity Weighting below), only to its own finding severity (Warning when tests are failing, Suggestion otherwise).

Producer: Feature / Engineering documentation → Consumer: Test suite
Severity: Suggestion (passing) / Warning (failures present)

---

## CV7. Documented Build Targets Exist

Every build target declared in Build documentation exists in the build configuration.

Producer: Build documentation → Consumer: Build configuration
Severity: Error (missing target)

---

# Reverse Coverage (code → doc)

This section verifies that implemented items are documented. All findings here are **orphans**.

All orphan findings are **Warning** severity, never Error. Resolution: document / remove / suppress.

---

## CV8. No Orphan Source Components

Public source modules and types that are not referenced in any documentation.

Direction: code → doc. Every public component should appear in Architecture or Feature Technical docs.

Severity: Warning

---

## CV9. No Orphan APIs

Function signatures, endpoints, or handlers exported from the implementation that lack documentation.

Severity: Warning

---

## CV10. No Orphan CLI Commands

Subcommands in the CLI parser that are not documented in Engineering or Feature docs.

Severity: Warning

---

## CV11. No Orphan Config Options

Config keys accepted by the parser that are not documented in Engineering docs.

Note: This check replaces Build Conformance's BC10 (No Orphan Config) and BC4 (No Undocumented Features) — those checks were deleted from the Build spec; orphan config detection lives here.

Severity: Warning

---

## CV12. No Orphan Dependencies

Crates or packages declared in `Cargo.toml` (or equivalent) that have no rationale or owner documented anywhere.

Note: This check replaces Dependency Governance's D3 — that check was deleted from the Dependency spec; orphan dependency detection lives here.

Severity: Warning

---

## CV13. No Orphan Features

Cargo features (or equivalent build features) that are not documented in Build or Engineering docs.

Note: This check replaces Build Conformance's BC4 (No Undocumented Features) — that check was deleted from the Build spec; orphan feature detection lives here.

Severity: Warning

---

## CV14. No Orphan Modules

Source modules that exist in the file tree but have no architectural home in documentation.

Severity: Warning

---

## CV15. No Orphan Security Mechanisms

Security-relevant code (authentication, authorization, encryption, secret handling, access control) that is not documented in Security docs.

Note: This check replaces Security Conformance's SC9 (No Undocumented Security Mechanisms) — that check was deleted from the Security spec; orphan security detection lives here.

Severity: Warning

---

# Coverage Scoring

## Bidirectional Score

```
coverage_score = (forward_score + reverse_score) / 2
```

| Component | Formula |
|---|---|
| forward_score | `implemented_doc_items / total_doc_items` |
| reverse_score | `documented_code_items / total_code_items` |

**Zero denominator:** if `total_doc_items == 0` or `total_code_items == 0`, that sub-score is 100% (nothing to fail). The average is computed normally.

## Severity Weighting

| Check Type | Severity | Weight |
|---|---|---|
| Forward coverage miss | Error | 1.0 |
| Reverse coverage miss (orphan) | Warning | 0.5 |
| Advisory (CV6) | Suggestion | 0.0 (informational only) |

---

# Success Criteria

The documentation ↔ implementation contract should:

* every documented feature have corresponding code
* every architecture component exist as a module or structure
* every documented API have a function signature
* every CLI command be implemented
* every config key be accepted by the parser
* every build target exist in configuration
* every public source component be documented (or explicitly suppressed)
* every API be documented (or explicitly suppressed)
* every CLI command be documented (or explicitly suppressed)
* every config option be documented (or explicitly suppressed)
* every dependency be justified in documentation (or explicitly suppressed)
* every Cargo feature be documented (or explicitly suppressed)
* every module have an architectural home (or explicitly suppressed)
* every security mechanism be documented (or explicitly suppressed)

---

# Audit Report Requirements

The report must follow the Standard Audit Report format and include:

1. Executive Summary
2. Coverage Score
3. Forward Coverage Scores (CV1–CV7)
4. Reverse Coverage Scores (CV8–CV15)
5. Orphan Inventory
6. Gap Analysis
7. Findings (Error / Warning / Suggestion)
8. Prioritized Recommendations
9. Coverage Risk Assessment
10. Audit Metadata

Every finding references:

```
Producer:   <source artifact + path>
Consumer:   <target artifact + path>
Contract:   <CV check ID + description>
Evidence:   <specific evidence collected>
Severity:   error | warning | suggestion
Status:     open | fixed | accepted | ignored | false_positive
```

---

# Readiness Assessment

Assess:

| Area                     | Status              |
| ------------------------ | ------------------- |
| Forward Coverage         | PASS / FAIL         |
| Reverse Coverage         | PASS / FAIL         |
| Orphan Resolution        | ON TRACK / AT RISK  |
| Coverage Risk            | LOW / MEDIUM / HIGH |

Provide justification for every assessment.

---

# Procedure

1. Rotate the previous report according to `docs/raw/audit/README.md#report-rotation`.
2. Inventory all documentation items (features, components, APIs, CLI, config keys, build targets).
3. Inventory all code items (modules, functions, CLI subcommands, config keys, dependencies, features, security mechanisms).
4. Execute forward coverage checks CV1–CV7 (doc → code match).
5. Execute reverse coverage checks CV8–CV15 (code → doc match, orphans).
6. Calculate bidirectional coverage score.
7. Score every validation.
8. Identify findings — orphans at Warning, missing implementations at Error.
9. Assess Coverage Risk.
10. Generate the audit report using the Standard Audit Report format.
11. Write the report to:

```text
docs/raw/reports/coverage/latest/
```

following:

```text
docs/raw/audit/README.md#standard-report-format
```
