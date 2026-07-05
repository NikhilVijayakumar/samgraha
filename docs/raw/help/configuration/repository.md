# [repository] Configuration

## Purpose

The `[repository]` section — repository-level configuration and ignore patterns.

## Content

### `[repository]` top-level fields

| Field | Type | Description |
|-------|------|--------------|
| `root` | `Option<PathBuf>` | Overrides where the repository's `knowledge.db` lives, if different from the config file's own directory |
| `id` | `Option<String>` | Repository identity name, used in manifests and the registry (`samgraha init` sets this to the directory name) |
| `name` | `Option<String>` | Display name |
| `uuid` | `Option<Uuid>` | Repository UUID, generated once by `samgraha init` |
| `dependencies` | `Vec<{name, path, required}>` | Other repositories this one declares a dependency on (name, optional local path, required flag) |

```toml
[repository]
id = "samgraha"
name = "samgraha"
uuid = "..."

[[repository.dependencies]]
name = "core-lib"
path = "../core-lib"
required = true

[repository.workspace]
workspace_id = "my-workspace"
```

### `[repository.ignore]`

Defines glob patterns for files to exclude from compilation. **This is the actual compile-time filter** — it's not the same as `domain_exclusion` (see [documentation](documentation.md)), which only affects what `samgraha info` displays and does not stop compile from picking up files.

```toml
[repository.ignore]
patterns = [
    "**/node_modules/**",
    "**/target/**",
    "**/.git/**",
    "**/audit-standards/**",
]
```

These four are the built-in defaults, used when `patterns` is not set at all.

### `[repository.documentation]`

Documentation root and domain configuration — see the [documentation section](documentation.md) for details.

### `[repository.implementation]`

Reserved for future traceability checks:

```toml
[repository.implementation]
dir = "${SAMGRAHA_IMPLEMENTATION_DIR}"
```

### `[repository.scripts]` and `[repository.tests]`

Directory paths for scripts and tests — primarily informational:

```toml
[repository.scripts]
dir = "${SAMGRAHA_SCRIPTS_DIR}"

[repository.tests]
dir = "${SAMGRAHA_TESTS_DIR}"
```

## Related

- [samgraha.toml Overview](samgraha-toml.md)
- [Environment Variables](../getting-started/environment.md)
- [Concepts: Repository](../concepts/repository.md)
