# Environment Configuration

## Purpose

Reference for environment variables used by Samgraha to resolve paths for documentation, reports, scripts, and more.

## Content

### Overview

Samgraha uses environment variables to resolve filesystem paths. This avoids hardcoded paths in `samgraha.toml` that break across machines. Variables are loaded from a `.env` file (optional) or set in the shell environment.

Note: `.env` discovery walks upward from the current directory (like `.git` discovery) until it finds a file — it isn't limited to exactly the repository root. This is different from `samgraha.toml` itself, which is only ever read from the current working directory (see [samgraha.toml Configuration](../concepts/repository.md) for that gotcha).

### Variable Reference

| Variable | Purpose | Default Fallback |
|----------|---------|-----------------|
| `SAMGRAHA_DOCS_DIR` | Root directory for raw documentation files | `<repo>/docs` |
| `SAMGRAHA_REPORT_DIR` | Output directory for audit reports | `<repo>/docs/raw/reports` |
| `SAMGRAHA_SCRIPTS_DIR` | Repository scripts directory | `<repo>/scripts` |
| `SAMGRAHA_TESTS_DIR` | Repository tests directory | `<repo>/tests` |
| `SAMGRAHA_IMPLEMENTATION_DIR` | Source code root for traceability | `<repo>/src` |

### Resolution Order

1. Shell environment variable (highest priority)
2. `.env` file in repo root
3. Default fallback path

### `.env` File Example

```env
SAMGRAHA_DOCS_DIR=E:\Python\samgraha\docs
SAMGRAHA_REPORT_DIR=E:\Python\samgraha\docs\reports
SAMGRAHA_SCRIPTS_DIR=E:\Python\samgraha\scripts
SAMGRAHA_TESTS_DIR=E:\Python\samgraha\tests
SAMGRAHA_IMPLEMENTATION_DIR=E:\Python\samgraha\src
```

### In samgraha.toml

Config values reference env variables with `${VAR_NAME}` syntax:

```toml
[repository.documentation]
root_dir = "${SAMGRAHA_DOCS_DIR}"
```

### Generating a Template

```bash
samgraha env [path]
```

This does not print currently-resolved values — it (re)writes `.env.example` at the repository root with every env key Samgraha reads, each commented out with its fallback default explained. It's additive: keys you already have in `.env.example` are left untouched, only missing ones are appended. `samgraha init` calls this automatically, so `env` is mainly for regenerating the template after an update adds a new key.

## Related

- [Initialization](initialization.md)
- [Configuration: samgraha.toml](../configuration/samgraha-toml.md)
- [Command: env](../commands/env.md)
