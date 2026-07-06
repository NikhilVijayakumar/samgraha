# Workspace Setup

## Purpose

How to set up and configure a multi-repo workspace.

## Content

### Workspace File

`samgraha-workspace.toml` (no leading dot) — either hand-write it or generate it with `samgraha workspace init`:

```bash
samgraha workspace init my-workspace ../core-lib ../api-service ../web-app
```

This writes:

```toml
name = "my-workspace"
repositories = [
    "../core-lib",
    "../api-service",
    "../web-app",
]
```

Paths can be relative (to the workspace file's directory) or absolute. `samgraha compile --workspace` (or running `compile` in a directory containing `samgraha-workspace.toml`) auto-detects it and compiles every member.

### Workspace Commands

There are only three workspace subcommands — there is no `workspace list`/`status`/`info`:

```bash
# Initialize a workspace config
samgraha workspace init <name> <repo-path>...

# Compile every member repository
samgraha workspace compile [--force]

# Search across all member repositories
samgraha workspace search "<query>" [--max N]
```

### Auto-Discovery

Workspace commands walk up from the current directory looking for `samgraha-workspace.toml`. If none is found, `workspace compile`/`workspace search` fail with an error rather than falling back to single-repo behavior.

### Search in Workspace

`samgraha workspace search` spans all workspace members (plain `samgraha search` does not). Results from every member are merged and re-sorted by score, but are not tagged with the source repository name — there is no per-result repo label in the output.

## Related

- [Concepts: Workspace](../concepts/workspace.md)
- [Command: workspace](../commands/workspace.md)
- [Multi-Repo Overview](overview.md)
