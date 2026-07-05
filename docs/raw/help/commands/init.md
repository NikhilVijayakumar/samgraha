# samgraha init

## Purpose

Initialize a new `samgraha.toml` configuration for a repository (or merge missing keys into an existing one).

## Content

### Synopsis

```bash
samgraha init [path] [--force]
```

`path` is a positional argument, optional (default: current directory). `init` is one of only two commands (with `version`) that don't require the repo guard — it works on a directory that isn't a samgraha repo yet.

### Description

`init` creates a `.samgraha/` directory (empty at this point — `knowledge.db`, `manifest.json`, etc. are created later by `compile`/`registry`) and writes a full-schema default `samgraha.toml` if one doesn't already exist. The generated config declares every built-in standard in `[repository.documentation].domain` and assigns a repository `id`, `name` (from the directory name), and a new random `uuid`. It also generates a `.env.example` file listing every env var key samgraha reads.

If `samgraha.toml` already exists and `--force` is not passed, `init` is merge-ready: it adds any keys missing from the existing file (comparing against the full default schema) without overwriting or deleting anything already present, then reports how many keys were added.

### Options

| Flag | Description |
|------|-------------|
| `path` | Positional, optional. Directory to initialize (default: current directory) |
| `--force` | Overwrite the existing `samgraha.toml` with a fresh default instead of merging missing keys into it |

### Examples

```bash
# Initialize current directory
samgraha init

# Initialize a specific path
samgraha init ../other-repo

# Overwrite existing config with fresh defaults
samgraha init --force
```

## Related

- [Getting Started: Initialization](../getting-started/initialization.md)
- [Getting Started: First Project](../getting-started/first-project.md)
- [Configuration: samgraha.toml](../configuration/samgraha-toml.md)
