# Coverage Audit Report — {{date}}

**Score:** {{score}}% {{score_bar}}

**Session:** {{session_id}}
**Git Revision:** {{git_revision}}

## Feature Coverage

| Metric | Value |
|--------|-------|
| Features Declared | {{features_count}} |
| Source Files Found | {{src_files_count}} |
| Feature Coverage | {{coverage_bar}} |

## Doc Types Covered

| Type | Status |
|------|--------|
{{doc_types_table}}

{{#uncovered_features_list}}
## Uncovered Features

| Feature |
|---------|
{{uncovered_features_list}}
{{/uncovered_features_list}}

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
