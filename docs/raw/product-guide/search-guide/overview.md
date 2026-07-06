# Search Overview

## Purpose

How search works in Samgraha — the concepts behind full-text knowledge retrieval.

## Content

### How Search Works

1. You provide a query string (`samgraha search "authentication"`).
2. Samgraha scores documents by keyword match (title and body, case-insensitive term matching) — there is no FTS5 or SQL full-text index involved.
3. Results are sorted by relevance score, high to low.
4. The CLI merges the repository's own documents with the built-in help/standards stores. Dependency and interest repos are only merged in when the search runs inside an MCP session (see [Cross-Repository Search](cross-repo.md)) — plain `samgraha search` does not pull in declared dependencies.

### Search Stores

| Store | Content | When it's included |
|-------|---------|---------------------|
| Repository | User's own documentation | Always |
| Standards | Built-in standards reference | Always (unless excluded via `domain_exclusion`) |
| Help | Built-in product help | Always (unless excluded via `domain_exclusion`) |
| Dependencies / Interests | Other repos declared in `[knowledge]` | Only via an MCP session, not the plain CLI `search` command |

There is no priority/ranking between stores — documents from every included store are concatenated into one list, then sorted by relevance score.

### Retrieval Levels

| Level | Content Returned |
|-------|-----------------|
| metadata | Document title and domain (default) |
| summary | Title + first non-empty line of body |
| section | A relevant matched line/snippet |
| full | Same as section (snippet is not the full body) |

### Output Formats

```bash
samgraha search "query"          # Human-readable table (default)
samgraha --json search "query"   # Machine-readable JSON (global --json flag)
```

## Related

- [Basic Search](basic.md)
- [Filtering](filtering.md)
- [Sections](sections.md)
- [Command: search](../commands/search.md)
