# Audit Reports

## Purpose

How audit reports are generated, their format, and how to interpret findings.

## Content

### Generating a Report

```bash
samgraha audit --report                           # Documentation Audit
samgraha audit --pipeline build --report          # Build Audit
samgraha audit --pipeline coverage --report       # Coverage Audit
```

This produces a Markdown report.

### Report Contents

The Markdown report includes a header (date, pipeline, provider, overall score), a summary table, category scores, and the full findings list.

### Finding Format

Every finding references:

```
Producer:   <source artifact + path>
Consumer:   <target artifact + path>
Contract:   <audit check ID + description>
Evidence:   <specific evidence collected>
Severity:   error | warning | suggestion
Status:     open | fixed | accepted | ignored | false_positive
```

Example:

```
Producer:   feature/offline-mode.md:42  (Feature Documentation)
Consumer:   src/offline.rs:15            (Implementation)
Contract:   CV1 — Documented Features Implemented
Evidence:   Feature declares offline mode capability. No implementation found.
Severity:   error
```

### Report Location

`samgraha audit --report` writes to `[report].dir/{pipeline}/latest/report.md` and archives a timestamped copy under `[report].dir/{pipeline}/archive/`. `[report].dir` defaults to `${SAMGRAHA_REPORT_DIR}`, which resolves to `<repo>/docs/raw/reports` if unset.

## Related

- [Audit Overview](overview.md)
- [Gates](gates.md)
- [Configuration: Report](../configuration/report.md)
