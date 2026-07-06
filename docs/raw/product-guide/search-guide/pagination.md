# Search Pagination

## Purpose

How result counts are capped in the CLI, and how true limit/offset pagination works over MCP.

## Content

### CLI: `--max` Only, No Offset

The CLI does not support offset-based paging. `--max` just caps how many results come back:

```bash
samgraha search "authentication" --max 5     # search: default 20
samgraha sections purpose --max 5            # sections: default 50
```

There is no `--offset`/`--limit` flag on `samgraha search` or `samgraha sections`. To see more results, raise `--max` (or narrow the query/`--domain`) and re-run — there's no way to fetch "the next page" from the CLI.

### Total Count

Both commands report how many results were found, but not as a page range:

```
5 result(s) in 3ms
```

### MCP: Real Limit/Offset Pagination

MCP tools (`search`, `get_sections`, `get_documents_by_domain`, `list_repositories`, `repository_status`, `workspace_status`, `get_document_section`) accept `limit` and `offset` and return a pagination envelope:

```json
{ "total": 47, "offset": 5, "limit": 5, "has_more": true, "results": [ ... ] }
```

`max` is accepted as a backward-compatible alias for `limit`. See [MCP Tools Reference](../mcp-guide/tools.md).

### Performance

Matching is done in memory over documents loaded from SQLite (no FTS5 index) — the limit/offset slicing happens after all matches are scored and sorted, not at the SQLite query level.

## Related

- [Basic Search](basic.md)
- [Filtering](filtering.md)
- [Sections](sections.md)
