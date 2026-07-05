# Knowledge Context

## Purpose

Runtime knowledge context at the MCP server level — lifecycle and query routing.

## Content

The **Knowledge Context** (`KnowledgeContext` in `crates/services/src/context.rs`) is the runtime object that aggregates all available knowledge stores for a single MCP session's search queries. It wraps a `RuntimePackage` plus a TTL, and survives MCP disconnects/reconnects — it's only rebuilt when its TTL expires (default from `resolver.knowledge_ttl`, 720h) or when an already-loaded repository's on-disk revision has changed since assembly. (The CLI has no equivalent long-lived context — each CLI invocation builds a fresh `KnowledgeRuntime` for the one repo it's running against.)

### Lifecycle

1. **Session creation** — The Planner produces a `KnowledgePlan` from `samgraha.toml` + manifest/`.meta` files; a `RuntimePackage` opens stores from it: primary + dependency stores eagerly, interest stores lazily (opened on first query).
2. **Built-in stores** — `standards.db` and `help.db` are loaded from the binary's own directory and appended to every session automatically, no config needed.
3. **Query serving** — A query reads all documents from every open store (primary, deps, interests, built-in) and concatenates them; there is no ranking between sources.
4. **Staleness / disposal** — On next use, if the TTL has expired or a loaded repo's revision changed on disk, the context is considered invalid and gets rebuilt.

### Merging, Not Ranking

There is no store-priority or relevance ranking between sources. `RuntimePackage::all_documents()` simply extends one document list across every store in load order (primary, dependencies, interests, then built-in), and the search/section query runs over that combined set. A `--domain` filter narrows the *combined* result set to matching documents — it does not route to a single "owning" store; built-in `help`/`standards` domains and a repo's own domains are filtered exactly the same way.

## Related

- [Resolver](resolver.md)
- [MCP Guide: Overview](../mcp-guide/overview.md)
- [Search Guide: Filtering](../search-guide/filtering.md)
