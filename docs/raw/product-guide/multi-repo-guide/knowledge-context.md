# Multi-Repo Knowledge Context

## Purpose

How the runtime knowledge context handles multiple repositories for unified search.

## Content

### Store Aggregation

At runtime, all resolved knowledge stores are aggregated into a single search context:

```
Search Context
├── Primary repo (current repository)
├── Dependency repos (hard + soft dependencies)
├── Interest repos (declared interests)
├── Standards store (built-in)
└── Help store (built-in)
```

### Query Routing

This aggregation only happens for MCP requests (`search`/`get_sections`), inside a `KnowledgeContext` session — not for the plain CLI `samgraha search`. When a query arrives:

1. Every loaded store's documents/sections are concatenated into one list — primary, dependencies, interests (opened lazily on first use), then built-in help/standards.
2. The combined list is scored and sorted by relevance (or merged as-is for `get_sections`), then domain-filtered if `domain` was passed.
3. There is no deduplication and no priority/ranking tier between stores — a document from an "interest" repo can outrank one from the primary repo if it scores higher. The load order above only affects *when* a store is opened (eager vs. lazy), not how results are ranked.

### Load Order (not a ranking)

| Order | Store | Loaded |
|-------|-------|--------|
| 1 | Primary repo | Eagerly, at session creation |
| 2 | Dependencies | Eagerly, at session creation |
| 3 | Interests | Lazily, on first query needing them |
| 4 | Built-in (help + standards) | Eagerly, at session creation |

## Related

- [Multi-Repo Overview](overview.md)
- [Concepts: Knowledge Context](../concepts/knowledge-context.md)
- [Concepts: Resolver](../concepts/resolver.md)
