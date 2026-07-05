# [report] Configuration

## Purpose

Report output directory configuration for audit reports.

## Content

### Settings

```toml
[report]
dir = "${SAMGRAHA_REPORT_DIR}"
```

Falls back to `<repo>/docs/raw/reports` if the env variable is unset.

### Report Output

When `samgraha audit --report` is used, a **markdown** report is generated (not HTML or JSON) under:

```
<dir>/audit/latest/report.md      # overwritten each run
<dir>/audit/archive/<timestamp>.md  # kept, one per run — e.g. 20260705-143022.md
```

The report includes the score, provider, domain, and every finding grouped by severity (error/warning/suggestion). There is no `audit-history.db` or other trend-analysis database — history is just the accumulated files under `archive/`.

## Related

- [Command: audit](../commands/audit.md)
- [Audit Guide: Reports](../audit-guide/reports.md)
- [samgraha.toml Overview](samgraha-toml.md)
