# Section Queries

## Purpose

How to query compiled sections by document, domain, and semantic type.

## Content

### By Semantic Type

`semantic_type` is a required positional argument — there is no way to list "all sections" without naming a type:

```bash
samgraha sections purpose
```

Shows all sections with semantic type `purpose` across all documents.

### Filtered by Domain

```bash
samgraha sections functional_requirements --domain feature
```

Shows all `functional_requirements` sections within the `feature` domain. There is no `--document` flag to filter by a single document's title in the CLI.

### Section Types

Each standard defines semantic section types:

| Domain | Types |
|--------|-------|
| feature | purpose, functional_requirements, acceptance_criteria, constraints, ... |
| architecture | purpose, system_overview, component_model, communication_paths, ... |
| help | title, purpose, content, related |

### Limiting Results

```bash
samgraha sections purpose --max 10
```

`--max` caps the number of results (default: 50). There is no `--offset`/paging flag on the CLI.

## Related

- [Search Overview](overview.md)
- [Filtering](filtering.md)
- [Command: sections](../commands/sections.md)
