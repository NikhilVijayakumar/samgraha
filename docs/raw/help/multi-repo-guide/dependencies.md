# Declaring Dependencies

## Purpose

How to declare a dependency on another repository's documentation.

## Content

### samgraha.toml Declaration

Dependencies are declared in the consumer repo's own `samgraha.toml`, not in `.samgraha/manifest.json` (that file is generated output from `compile`, not something you hand-edit for this). Name the dependency under `[knowledge]`, then optionally give it an explicit local path under `[[repository.dependencies]]`:

```toml
[knowledge]
dependencies = ["core-lib"]

[[repository.dependencies]]
name = "core-lib"
path = "../core-lib"
required = true
```

### Resolution

The Planner (`crates/services/src/planner.rs`) resolves each name in `[knowledge].dependencies` to a root path, in this order:

1. An explicit `[[repository.dependencies]]` entry's `path` (config wins if present).
2. A cached `.samgraha/dependencies/<name>.meta` file (JSON, written by `samgraha registry sync`, TTL from `resolver.metadata_ttl` — default 24h).

Once resolved, the dependency's `.samgraha/knowledge.db` is opened and its documents are merged into search/get_sections results for that session — but only within an MCP session (`KnowledgeContext`), not the plain CLI `samgraha search`. See [Search Guide: Cross-Repository Search](../search-guide/cross-repo.md).

### Hard vs Soft

There's no per-dependency `required: true/false` toggle — hardness is determined by which list a name is in:

- **`[knowledge].dependencies`** — always required/hard. An unresolved or missing entry here is a `RequiredMissing` status (error state).
- **`[knowledge].interests`** — always optional/soft (see [Interests](interests.md)).

### Circular Dependencies

Samgraha detects dependency cycles via DFS during dependency-graph resolution (`samgraha registry resolve runtime` / `KnowledgeResolver`) and aborts with the full cycle path.

## Related

- [Multi-Repo Overview](overview.md)
- [Interests](interests.md)
- [Concepts: Planner](../concepts/planner.md)
