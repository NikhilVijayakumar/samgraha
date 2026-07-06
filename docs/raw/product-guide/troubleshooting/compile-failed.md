# Compilation Failed

## Purpose

Common compilation failures and their solutions.

## Content

### Problem: "No documents found in domain"

**Cause**: No markdown files exist in the configured docs directory for the specified domain.

**Solution**:
```bash
# Check docs directory exists
ls docs/raw/feature/

# Verify configuration
samgraha env
```

### Problem: Recompile doesn't pick up a changed file

**Cause**: Incremental compile compares each document's stored content hash (kept in `.samgraha/knowledge.db`, not a separate meta file) against the file on disk, and only reprocesses what changed. If a file was touched without its content actually changing (e.g. some git operations only update mtime), it can look unchanged.

**Solution**: Force a full recompile:
```bash
samgraha compile --force
```

### Problem: "Domain not in standard registry" / unrecognized domain

**Cause**: Samgraha's standards are a fixed built-in catalog (`feature`, `architecture`, `vision`, `design`, `engineering`, `help`, `standards`, etc.) — there is currently no way to register an arbitrary custom domain at compile time (the `StandardLoader` scaffolding for runtime-loaded standard definitions exists in the `standards` crate but isn't wired into the compile pipeline).

**Solution**: Check the domain name for typos against the built-in catalog. If the domain is a built-in one your repo excluded, remove it from `domain_exclusion` in `samgraha.toml`.

### Problem: Compile seems to use default config, ignoring `samgraha.toml`

**Cause**: `load_config()` only checks `samgraha.toml` in the **current working directory** — it does not search upward through parent directories (unlike `samgraha init`/repo-guard detection, which does search upward). Running a command from a subdirectory without `--config` silently falls back to an all-defaults config instead of erroring.

**Solution**: Run commands from the repository root, or pass `--config <path-to-samgraha.toml>` explicitly.

### Problem: Build script compile step fails

**Cause**: The `build-release.ps1` compile step encountered an error.

**Solution**: Check that all markdown files follow the standard template. Run compile manually (`path` is positional, not a `--path` flag):
```bash
samgraha compile docs/raw/help --domain help --force
```

## Related

- [Command: compile](../commands/compile.md)
- [Command: env](../commands/env.md)
- [Troubleshooting Index](mcp-wont-start.md)
