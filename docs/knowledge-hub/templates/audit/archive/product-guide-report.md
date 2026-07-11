# Product Guide Audit Report — {{date}}

**Score:** {{score}}% {{score_bar}}

**Session:** {{session_id}}
**Git Revision:** {{git_revision}}
**Readiness:** {{engineering_readiness}}

## Category Scores

| Category | Score |
|----------|-------|
| Coverage | {{coverage_score}}% |
| Navigation | {{navigation_score}}% |
| Quality | {{quality_score}}% |
| Accuracy | {{accuracy_score}}% |

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
