# Build Audit Report — {{date}}

**Score:** {{score}}% {{score_bar}}

**Session:** {{session_id}}
**Git Revision:** {{git_revision}}

## Contract

| Field | Value |
|-------|-------|
| Contract Name | {{contract_name}} |
| Declared Produces | {{declared_produces}} |
| Execution Success | {{execution_success}} |

## Artifacts

| Artifact | Status |
|----------|--------|
{{artifact_table}}

{{#execution_output}}
## Execution Output

```
{{execution_output}}
```
{{/execution_output}}

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
