# Consistency Audit Report — {{date}}

**Score:** {{score}}% {{score_bar}}

**Session:** {{session_id}}
**Git Revision:** {{git_revision}}

## Structure

| Check | Status |
|-------|--------|
| Vision Document Exists | {{vision_exists}} |
| Architecture Document Exists | {{architecture_exists}} |
| Structure Score | {{structure_score}}% |
| Cross-References | {{cross_references}} |

{{#naming_issues_table}}
## Naming Issues

| Issue | Location |
|-------|----------|
{{naming_issues_table}}
{{/naming_issues_table}}

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
