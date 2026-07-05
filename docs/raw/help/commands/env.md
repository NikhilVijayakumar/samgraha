# samgraha env

## Purpose

Generate a `.env.example` file listing every environment variable key samgraha reads.

## Content

### Synopsis

```bash
samgraha env [path]
```

`path` is a positional argument, optional (default: current repository root).

### Description

`env` writes (or updates) `.env.example` at the repository root, listing every env var key samgraha reads, each with a comment explaining what it's for and its fallback default. It does not print resolved values or their sources — it only generates the template file. Existing keys already present in `.env.example` are left untouched; only missing keys are appended. (`samgraha init` calls this same generator automatically.)

The keys written are:

| Variable | Purpose | Fallback if unset |
|----------|---------|--------------------|
| `SAMGRAHA_DOCS_DIR` | Documentation root for this repository | `<repo>/docs` |
| `SAMGRAHA_REPORT_DIR` | Where generated reports (e.g. `samgraha audit --report`) are written | `<repo>/docs/raw/reports` |
| `SAMGRAHA_IMPLEMENTATION_DIR` | Implementation/source directory (reserved for future traceability checks) | `<repo>/src` |
| `SAMGRAHA_SCRIPTS_DIR` | External scripts directory (only relevant if `[repository.scripts]` is set) | — |
| `SAMGRAHA_TESTS_DIR` | Test directory, if kept outside the implementation dir (only relevant if `[repository.tests]` is set) | — |

### Example Output

```
Generated E:\Python\samgraha\.env.example
```

This is useful to bootstrap or update the `.env.example` template committed alongside `samgraha.toml`.

## Related

- [Command: info](info.md)
- [Getting Started: Environment](../getting-started/environment.md)
- [Configuration: samgraha.toml](../configuration/samgraha-toml.md)
