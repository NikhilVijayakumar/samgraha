# Audit Reports

## Purpose

How audit reports are generated, their format, and how to interpret findings.

## Content

### Generating a Report

```bash
samgraha audit --report
```

This produces a Markdown report — there is no HTML or JSON report format.

### Report Contents

The Markdown report includes a header (date, domain, provider, overall score, readiness), a summary table (documents checked/passed, error/warning/suggestion counts), a per-standard category score table, and the full findings list, e.g.:

```
# Saṃgraha Audit Report

**Date:** 2026-07-05 14:30:22
**Domain:** all
**Provider:** deterministic
**Score:** 85.0%
**Readiness:** engineering

## Summary

| Metric | Value |
|---|---|
| Documents Checked | 20 |
| Documents Passed | 18 |
| Errors | 2 |
| Warnings | 5 |
| Suggestions | 5 |

## Category Scores

| Standard | Score |
|---|---|
| feature | 92.0 |
| architecture | 78.0 |
```

### Report Location

`samgraha audit --report` writes to `[report].dir/audit/latest/report.md` and archives a timestamped copy under `[report].dir/audit/archive/`. `[report].dir` defaults to `${SAMGRAHA_REPORT_DIR}`, which resolves to `<repo>/docs/raw/reports` if that environment variable isn't set.

## Related

- [Audit Overview](overview.md)
- [Gates](gates.md)
- [Configuration: Report](../configuration/report.md)
