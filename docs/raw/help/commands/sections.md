# samgraha sections

## Purpose

Query compiled document sections by semantic type across the whole registry (not full-text search).

## Content

### Synopsis

```bash
samgraha sections <semantic_type> [--domain <domain>] [--max <n>]
```

### Description

`sections` returns every compiled section matching a semantic type — e.g. `functional_requirements`, `business_rules` — across all documents in the registry. `semantic_type` is a required positional argument. There is no `--document` or `--type` flag and no pagination/offset; use `--domain` to narrow scope and `--max` to cap results.

### Options

| Flag | Description |
|------|-------------|
| `<semantic_type>` | Semantic section type to query (required, positional), e.g. `functional_requirements`, `business_rules` |
| `--domain <domain>` | Filter by domain (standard) |
| `--max <n>` | Maximum results (default: 50) |

### Examples

```bash
# All functional requirements sections across the registry
samgraha sections functional_requirements

# Business rules within a specific domain
samgraha sections business_rules --domain feature

# Cap results
samgraha sections functional_requirements --max 10
```

## Related

- [Search](search.md)
- [Compile](compile.md)
- [Search Guide: Sections](../search-guide/sections.md)
