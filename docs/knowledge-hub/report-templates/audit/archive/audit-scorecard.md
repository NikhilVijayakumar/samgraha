# {{pipeline|title}} Audit Scorecard — {{date}}

**Score:** {{score}}% {{score_bar}}

## Category Scores

| Category | Score |
|---|---|
{{categories}}

## Deterministic Findings

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

## Semantic Review

{{#semantic_done}}
### Judged ({{semantic_done_count}})
| Type | Location | Result |
|---|---|---|
{{semantic_done_table}}
{{/semantic_done}}

{{#semantic_pending}}
### Pending ({{semantic_pending_count}})
| Type | Document | Title |
|---|---|---|
{{semantic_pending_table}}
{{/semantic_pending}}
