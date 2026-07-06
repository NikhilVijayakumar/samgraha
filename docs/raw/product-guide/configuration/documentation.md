# [repository.documentation] Configuration

## Purpose

Documentation root, domain declaration, and domain exclusion configuration.

## Content

### `root_dir`

Absolute or env-referenced path to the documentation directory:

```toml
[repository.documentation]
root_dir = "${SAMGRAHA_DOCS_DIR}"
```

Falls back to `<repo>/docs` if the env variable is unset. Note: `samgraha compile` actually takes the directory to compile as its own positional `[path]` argument (default: current directory) — `root_dir` is a declared schema field, not currently read to auto-locate the docs directory for `compile` itself.

### `domain`

List of domains this repository declares:

```toml
domain = ["readme", "vision", "feature", "architecture", "engineering"]
```

An empty list means "all builtin standards" (backward-compatible default).

### `domain_exclusion`

Domains to ignore even if listed in `domain`:

```toml
domain_exclusion = ["help", "standards"]
```

Effective domains (`domain` minus `domain_exclusion`) are what `samgraha info` displays in its Standards list. **`domain_exclusion` does not prevent `compile` from picking up files** — it's a display/validation filter only. The actual compile-time safeguard against unwanted content is `[repository.ignore].patterns` (see [repository](repository.md)): if you don't want a domain's files compiled at all, ignore their paths, don't rely on `domain_exclusion`.

`help` and `standards` are reserved domain names (the built-in knowledge stores shipped next to the binary). Declaring `domain = ["help"]` without also listing `help` in `domain_exclusion` fails compile validation — the reserved name would otherwise collide with the built-in store. This is why the example below excludes `help`/`standards` even though they're declared.

### Example

```toml
[repository.documentation]
root_dir = "${SAMGRAHA_DOCS_DIR}"
domain = ["readme", "vision", "architecture", "feature", "feature-design", "feature-technical", "design", "engineering", "external-context", "prototype", "help", "standards"]
domain_exclusion = ["design", "feature-design", "external-context", "prototype", "help", "standards"]
```

## Related

- [samgraha.toml Overview](samgraha-toml.md)
- [Concepts: Domains](../concepts/domains.md)
- [Command: info](../commands/info.md)
