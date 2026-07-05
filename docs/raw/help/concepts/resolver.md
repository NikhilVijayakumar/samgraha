# Resolver

## Purpose

How knowledge resolution works — making remote documentation available for search.

## Content

"Resolver" covers two related but distinct things in Samgraha:

- The **`[resolver]` config section** (`ResolverConfig`) — controls how dependency metadata is cached and refreshed: `metadata_cache` (bool, default true), `metadata_ttl` (default "24h"), `knowledge_ttl` (default "720h" — governs `KnowledgeContext` staleness), `auto_refresh` (bool, default true), `registry_type` (`file` or `http`, default `file`), `registry_url`.
- The **runtime resolution path** — `KnowledgeResolver` (`crates/services/src/resolution.rs`) walks the dependency graph (DFS with cycle detection), preferring the SQLite-backed registry cache and falling back to reading a dependency's `manifest.json` directly on a cache miss. This is what backs `samgraha registry resolve runtime` and `samgraha package`. At MCP-session scope, `RuntimePackage`/`KnowledgeContext` do the equivalent job of opening each planned repository's `.samgraha/knowledge.db`.

### Merging, Not Ranking

After the Planner produces a plan and the stores are opened, search/section queries read **all** stores (primary repo + dependencies + interests + built-in) and concatenate the results — there is no priority ranking of one repository's results over another's. See [Knowledge Context: Merging, Not Ranking](knowledge-context.md) for the mechanics.

### Built-in Knowledge

Standards and Help knowledge are automatically available in every session. They are loaded from `standards.db` and `help.db` adjacent to the running binary (`load_builtin_stores()`), appended to the store list alongside the primary/dependency/interest stores with no special priority — just another set of documents in the merged result set, filterable by `--domain help` / `--domain standards`.

## Related

- [Planner](planner.md)
- [Registry](registry.md)
- [Knowledge Context](knowledge-context.md)
- [Configuration: resolver](../configuration/resolver.md)
