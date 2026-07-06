# Audit History

## Purpose

How audit history is tracked and how to use it for quality trend analysis across all audit types.

## Content

There is no dedicated history database or `--history` flag today. History exists in archived markdown reports and revision trails.

### Archived Markdown Reports

Every `samgraha audit --report` run writes `[report].dir/{pipeline}/latest/report.md` (overwritten each time) and archives a timestamped copy to `[report].dir/{pipeline}/archive/<YYYYMMDD-HHMMSS>.md`. The archive directory is your history — browse or diff the files directly.

Each audit pipeline (doc, build, security, consistency, coverage) maintains its own report directory under `docs/raw/reports/`.

### Manifest Audit Status

`.samgraha/manifest.json` tracks the most recent Documentation Audit run: `audit.status` (`PASS`/`FAIL`) and `audit.last_audit` (timestamp). It is overwritten on every run, not a history log.

### Semantic Report Revisions

For per-section/document/cross-domain semantic reports submitted via MCP, each report path keeps a `history/` subfolder. When a new report is stored for the same document/section, the previous `latest.json` is rotated into `history/<timestamp>-rev<N>.json`.

## Related

- [Reports](reports.md)
- [Stages](stages.md)
- [Gates](gates.md)
- [Configuration: Report](../configuration/report.md)
