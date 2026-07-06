# samgraha search

## Purpose

Search compiled knowledge databases for relevant document sections.

## Content

### Synopsis

```bash
samgraha search <query> [--domain <domain>] [--level metadata|summary|section|full] [--max <n>]
```

### Description

`search` runs full-text search across compiled knowledge databases (primary repo + dependencies + built-in stores). `query` is a required positional argument. There is no pagination/offset flag.

### Options

| Flag | Description |
|------|-------------|
| `<query>` | Search query string (required, positional) |
| `--domain <domain>` | Restrict search to a specific domain |
| `--level <level>` | Retrieval level: `metadata`, `summary`, `section`, `full` (default: `metadata`) |
| `--max <n>` | Maximum results (default: 20) |

### Retrieval Levels

| Level | Returns |
|-------|---------|
| `metadata` | Document title and domain only (default) |
| `summary` | Title + purpose section |
| `section` | Full matched sections |
| `full` | Complete document body |

### Examples

```bash
# Basic search
samgraha search "authentication"

# Domain-scoped search
samgraha search "compile" --domain help

# Limit results
samgraha search "feature" --max 5

# Summary level
samgraha search "architecture" --level summary
```

## Related

- [Sections](sections.md)
- [Search Guide: Overview](../search-guide/overview.md)
- [MCP Tools](../mcp-guide/tools.md)
