# Search Filtering

## Purpose

How to filter search results by domain, retrieval level, and other criteria.

## Content

### Domain Filtering

Restrict search to a specific domain:

```bash
# Search only feature docs
samgraha search "authentication" --domain feature

# Search only help docs
samgraha search "compile" --domain help
```

### Retrieval Level

Control how much content is returned:

```bash
# Just titles and domains
samgraha search "authentication" --level metadata

# Titles + purpose sections
samgraha search "authentication" --level summary

# Full matched sections (default)
samgraha search "authentication" --level section

# Complete document bodies
samgraha search "authentication" --level full
```

### Combined Filters

```bash
samgraha search "authentication" --domain feature --level summary --max 5
```

### Domain Behavior

- `--domain feature` — searches only the feature domain in the repo store
- `--domain help` — searches only the built-in help store
- `--domain standards` — searches only the built-in standards store
- No domain flag — searches all stores

## Related

- [Basic Search](basic.md)
- [Sections](sections.md)
- [Pagination](pagination.md)
