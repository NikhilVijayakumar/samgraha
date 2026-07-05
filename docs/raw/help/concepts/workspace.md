# Workspace

## Purpose

The concept of a workspace for managing multiple Samgraha repositories together.

## Content

A **workspace** groups multiple Samgraha repositories so they share a knowledge context. This allows cross-repository search and dependency resolution.

Workspaces are useful when:

- Your product spans multiple Git repositories.
- You need to search documentation across repos.
- One repo depends on another's documentation (e.g., a library's feature specs).

### Workspace File

A workspace is defined by a `samgraha-workspace.toml` file (no leading dot) at the workspace root, listing member repositories:

```toml
name = "my-workspace"
repositories = [
    "../core-lib",
    "../api-service",
    "../web-app",
]
```

### Commands

Workspace operations are their own subcommand group — there is no `list`/`status`; the real actions are `init`, `compile`, and `search`:

```bash
samgraha workspace init <name> <repositories...> [path]   # Create samgraha-workspace.toml
samgraha workspace compile [--force]                      # Compile every member repo
samgraha workspace search <query> [--max <n>]              # Search across all member repos
```

`samgraha compile` also auto-detects a workspace (treats `--workspace` as implied) when it finds a `samgraha-workspace.toml` in the target path — but plain `samgraha search` only searches the current repository plus its own configured dependencies/interests, not the whole workspace; use `samgraha workspace search` for that.

## Related

- [Repository](repository.md)
- [Multi-Repo Guide](../multi-repo-guide/overview.md)
- [Command: workspace](../commands/workspace.md)
