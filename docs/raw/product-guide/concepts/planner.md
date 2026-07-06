# Planner

## Purpose

How dependency planning works — resolving which repositories provide which knowledge.

## Content

The **Planner** (`crates/services/src/planner.rs`, `Planner::plan()`) builds a `KnowledgePlan` — a deterministic, database-free list of candidate repositories for an MCP session or CLI run. Same `samgraha.toml` + same manifest/`.meta` files always produces the same plan. For each name listed in `[knowledge].dependencies` / `[knowledge].interests`, it:

1. Looks for an explicit path in `[repository].dependencies` (config is authoritative if present).
2. Falls back to the cached `.samgraha/dependencies/<name>.meta` file (written by `samgraha registry sync`) for the path and revision/exports.
3. Checks whether `.samgraha/knowledge.db` exists at the resolved path.
4. Computes a status for the entry (see below) — it never errors out; unresolved/missing entries just stay in the plan with the appropriate status.

The primary (local) repository is always the first entry, with status `Primary`.

### Entry Status (`DepStatus`)

| Status | Meaning |
|--------|---------|
| `Primary` | The local repository — always loaded. |
| `Loaded` | Path resolved, `.meta` fresh, cached revision matches actual. |
| `Stale` | Available, but `.meta` TTL (`resolver.metadata_ttl`, default 24h) has expired — re-sync recommended. |
| `Outdated` | `.meta` fresh, but its cached revision differs from the dependency's actual manifest revision. |
| `Missing` | Path resolved but `.samgraha/knowledge.db` doesn't exist yet (needs `compile`). |
| `Unresolved` | No path in config or `.meta` — never synced. |
| `RequiredMissing` | A required dependency that is missing or unresolved — error state. |

### Priority vs. Dependency Types

Each entry also has a `Priority` (`Primary`, `Dependency`, or `Interest`) that determines *load timing*, not search ranking (see [Knowledge Context](knowledge-context.md)):

- **Dependencies** (`[knowledge].dependencies`) — always loaded eagerly; if `required` and missing, this is an error state (`RequiredMissing`).
- **Interests** (`[knowledge].interests`) — opened lazily, only on first query; missing is non-fatal.

The full dependency *graph* walk (transitive deps, cycle detection, registry-cache-first with manifest fallback) is a separate component, `KnowledgeResolver` — see [Resolver](resolver.md).

## Related

- [Registry](registry.md)
- [Resolver](resolver.md)
- [Multi-Repo Dependencies](../multi-repo-guide/dependencies.md)
- [Multi-Repo Interests](../multi-repo-guide/interests.md)
