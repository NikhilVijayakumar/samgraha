# Initialization

## Purpose

How to initialize a Samgraha repository and understand the generated `.samgraha/` directory structure.

## Content

### `samgraha init`

Run this command in the root of any repository you want to manage with Samgraha (`samgraha init [path] [--force]`):

```bash
cd my-project
samgraha init
```

This writes `samgraha.toml` at the repository root (pre-filled with every built-in standard domain and a generated repository UUID), creates the `.samgraha/` directory, and generates `.env.example`. Re-running `init` on an existing `samgraha.toml` is safe — it backfills any missing keys/sections without touching values you've already set or overwriting your comments; pass `--force` to regenerate `samgraha.toml` from scratch instead.

Note the `.samgraha/` directory itself is empty right after `init` — its contents are populated by later commands:

- `knowledge.db` — Written by `samgraha compile`. The compiled output: documents, sections, enrichment. This is what CLI/MCP queries read.
- `manifest.json` — Written by `samgraha compile`. Declares the repository's identity, revision, and exported domains for multi-repo resolution.
- `dependencies/<name>.meta` — Written by `samgraha registry sync`. Cached metadata (path, revision, exports) for a declared dependency/interest, used by the Planner when no explicit path is configured.
- `registry.db` — Written by `samgraha registry` commands (register/sync/etc). The local repository registry backing store.

Every command except `init` and `version` requires being run inside a recognized samgraha repository (it walks up from the current directory looking for `.samgraha/` or `samgraha.toml`, the same way git looks for `.git/`) — otherwise it errors with "not a samgraha repository".

### Environment File

Samgraha loads configuration from a `.env` file in the repo root (optional):

```
SAMGRAHA_DOCS_DIR=E:\Python\samgraha\docs
SAMGRAHA_REPORT_DIR=E:\Python\samgraha\docs\reports
SAMGRAHA_SCRIPTS_DIR=E:\Python\samgraha\scripts
SAMGRAHA_TESTS_DIR=E:\Python\samgraha\tests
SAMGRAHA_IMPLEMENTATION_DIR=E:\Python\samgraha\src
```

## Related

- [Installation](installation.md)
- [Environment Setup](environment.md)
- [samgraha.toml Configuration](../configuration/samgraha-toml.md)
