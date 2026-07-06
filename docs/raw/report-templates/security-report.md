# Security Audit Report — {{date}}

**Score:** {{score}}% {{score_bar}}

**Session:** {{session_id}}
**Git Revision:** {{git_revision}}

## Threat Summary

{{threat_summary}}

## Scans

| Check | Count |
|-------|-------|
| Secrets Scanned | {{secrets_scanned}} |
| Secrets Found | {{secrets_found}} |
| Runtime Checks | {{runtime_checks}} |
| Runtime Issues | {{runtime_issues}} |
| High Risk Findings | {{high_risk_count}} |

## Findings

{{#errors}}
### Errors ({{errors_count}})
| Check | Location | Message |
|-------|----------|---------|
{{errors_table}}
{{/errors}}

{{#warnings}}
### Warnings ({{warnings_count}})
| Check | Location | Message |
|-------|----------|---------|
{{warnings_table}}
{{/warnings}}

{{#suggestions}}
### Suggestions ({{suggestions_count}})
| Check | Location | Message |
|-------|----------|---------|
{{suggestions_table}}
{{/suggestions}}

## Improvements

| Category | Suggestion | Priority |
|----------|------------|----------|
{{improvements}}
