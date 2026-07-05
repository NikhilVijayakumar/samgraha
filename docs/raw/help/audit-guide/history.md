# Audit History

## Purpose

How audit history is tracked and how to use it for quality trend analysis.

## Content

There is no `audit-history.db`, no `--history` flag, and no trend/graph command today. History exists in two much simpler forms:

### Archived Markdown Reports

Every `samgraha audit --report` run writes `[report].dir/audit/latest/report.md` (overwritten each time) and also archives a timestamped copy to `[report].dir/audit/archive/<YYYYMMDD-HHMMSS>.md`. The archive directory is your history — browse or diff the files directly; there's no built-in command to query or summarize them.

### Manifest Audit Status

`.samgraha/manifest.json` tracks only the *most recent* run: `audit.status` (`PASS`/`FAIL`, based on whether any error-severity finding exists) and `audit.last_audit` (timestamp). It is overwritten on every `samgraha audit` run, not a history log.

### Semantic Report Revisions

For the per-section/document/cross-domain semantic reports submitted via MCP (`store_section_report`, `store_document_report`, `store_cross_domain_report` — see [Stages](stages.md)), each report path keeps a `history/` subfolder: when a new report is stored for the same document/section, the previous `latest.json` is rotated into `history/<timestamp>-rev<N>.json` before the new one is written. This is a real revision trail, but it's per-section, not a whole-repository quality trend.

## Related

- [Reports](reports.md)
- [Stages](stages.md)
- [Gates](gates.md)
- [Configuration: Report](../configuration/report.md)
