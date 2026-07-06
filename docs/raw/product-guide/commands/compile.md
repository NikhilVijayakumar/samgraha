# samgraha compile

## Purpose

Compile markdown documentation into a structured SQLite knowledge database.

## Content

### Synopsis

```bash
samgraha compile [path] [--domain <domain>]... [--force] [--watch] [--workspace]
```

### Description

`compile` discovers markdown files in the documentation directory, parses them according to their domain standard, extracts sections, and writes them to the repository's `knowledge.db`. `path` is a positional argument (default: current directory) — there is no `--path` flag.

### Options

| Flag | Description |
|------|-------------|
| `path` | Positional. Directory to compile instead of the configured docs root (default: current directory) |
| `--domain <domain>` | Compile only documents in the given domain. Repeatable — pass multiple times to select several domains |
| `--force` | Recompile all documents, ignoring content hashes |
| `--watch` | Watch for file changes and recompile automatically |
| `--workspace` | Compile all repositories in a `samgraha-workspace.toml` workspace. Auto-enabled if `samgraha-workspace.toml` is found in `path` even without this flag |

### Behavior

- Recursively walks `**/*.md` files.
- Skips files listed in `[repository.ignore].patterns` (glob patterns; this is the actual compile-time filter — `domain_exclusion` only affects what `samgraha info` displays, it does not stop compile from picking up files).
- Uses content hashing for incremental compilation (only changed files are reprocessed).
- Generates `.samgraha/manifest.json` on successful completion.
- On success, automatically syncs the repository registry (best-effort — failures don't fail the compile).
- Requires being run inside a recognized samgraha repo (see the repo guard note under [init](init.md)).

### Examples

```bash
# Full compile
samgraha compile

# Compile only feature domain
samgraha compile --domain feature

# Compile multiple domains
samgraha compile --domain feature --domain architecture

# Compile an external directory
samgraha compile /path/to/docs --domain help --force
```

## Related

- [Audit](audit.md)
- [Search](search.md)
- [Configuration: compilation](../configuration/compilation.md)
- [Concepts: Knowledge Database](../concepts/knowledge-db.md)
