# samgraha workspace

## Purpose

Manage multi-repository workspaces — initializing, compiling, and searching across a group of repos in one operation.

## Content

### Synopsis

```bash
samgraha workspace <init <name> <repositories...> [path]|compile [--force]|search <query> [--max <n>]>
```

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `init <name> <repositories...> [path]` | Create a `samgraha-workspace.toml` at `path` (default: current directory) naming the workspace and listing member repository paths |
| `compile [--force]` | Compile all repositories in the discovered workspace |
| `search <query> [--max <n>]` | Search across all workspace repositories (max default: 20) |

There are no `list`, `status`, or `info` subcommands. This is distinct from `[knowledge].dependencies`/`interests` in `samgraha.toml`, which apply to a single repository's own knowledge context rather than a shared multi-repo workspace.

### Description

A workspace groups multiple Samgraha repositories so `compile`/`search` can operate on all of them together. Workspace discovery walks up from the current directory looking for `samgraha-workspace.toml` — the same file `samgraha compile --workspace` auto-detects.

### Workspace File

Workspace membership is defined in a `samgraha-workspace.toml` file (no leading dot):

```toml
name = "my-workspace"
repositories = [
    "../core-lib",
    "../api-service",
]
```

### Examples

```bash
# Create a workspace
samgraha workspace init my-workspace ../core-lib ../api-service

# Compile all workspace repositories
samgraha workspace compile

# Search across all workspace repositories
samgraha workspace search "authentication" --max 10
```

## Related

- [Concepts: Workspace](../concepts/workspace.md)
- [Multi-Repo Guide: Workspace](../multi-repo-guide/workspace.md)
- [Registry](registry.md)
