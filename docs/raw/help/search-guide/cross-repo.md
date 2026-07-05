# Cross-Repository Search

## Purpose

How to search across multiple repositories using workspaces and dependency resolution.

## Content

### Workspace Search

Plain `samgraha search` never spans multiple repositories — it only searches the current repo plus the built-in help/standards stores. To search across a workspace's member repos, use the workspace subcommand explicitly:

```bash
samgraha workspace search "authentication"
```

This merges results from every repository listed in `samgraha-workspace.toml`, re-sorted by score. Results are not tagged with which repository they came from — `SearchResult` carries no source-repository field, so there is no `[core-lib]`-style label in the output.

### Dependency/Interest Search (MCP only)

Declaring `[knowledge].dependencies`/`interests` in `samgraha.toml` does **not** make plain `samgraha search` include those repos. That merge only happens inside an MCP session: the MCP `search`/`get_sections` tools resolve a Knowledge Plan (primary + dependencies + interests + built-in) via the Planner and merge every loaded store's documents before scoring — see [Multi-Repo Guide: Knowledge Context](../multi-repo-guide/knowledge-context.md). There is no CLI equivalent that resolves `[knowledge].dependencies` into a live search.

### Excluding Built-in Stores

To exclude built-in help and standards from results:

```bash
samgraha search "compile" --domain feature
```

Domain filtering only matches the named domain, so built-in stores (which use "help" and "standards" domains) are excluded.

## Related

- [Search Overview](overview.md)
- [Multi-Repo Guide](../multi-repo-guide/overview.md)
- [Workspace](../commands/workspace.md)
