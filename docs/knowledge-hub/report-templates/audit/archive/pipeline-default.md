# {{pipeline|title}} Report — {{date}}

**Score:** {{score}}% {{score_bar}}

## Category Scores

| Category | Score |
|---|---|
{{categories}}

## Findings

{{#errors}}
### Errors ({{errors_count}})
| Check | Location | Message |
|---|---|---|
{{errors_table}}
{{/errors}}

{{#warnings}}
### Warnings ({{warnings_count}})
| Check | Location | Message |
|---|---|---|
{{warnings_table}}
{{/warnings}}

{{#suggestions}}
### Suggestions ({{suggestions_count}})
| Check | Location | Message |
|---|---|---|
{{suggestions_table}}
{{/suggestions}}

{{#comments}}
## Comments
{{comments}}
{{/comments}}
