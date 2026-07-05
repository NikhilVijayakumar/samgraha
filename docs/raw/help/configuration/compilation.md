# [compilation] Configuration

## Purpose

Compilation behavior settings — watching for file changes, debounce, batching, and per-standard overrides.

## Content

### Settings

```toml
[compilation]
watch = false
debounce_ms = 100
batch_size = 100

[compilation.documentation]
standards = []
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `watch` | bool | `false` | Also settable via the `--watch` CLI flag; the config value is the default when the flag is omitted |
| `debounce_ms` | u64 | `100` | Debounce window in milliseconds used by `samgraha compile --watch`: after the first file-change event, it drains further events for this long before recompiling once |
| `batch_size` | usize | `100` | Batch size used internally during compilation |
| `[compilation.documentation].standards` | `Vec<String>` | `[]` (empty) | Reserved for a future per-standard compilation override list |

### `watch`

When `true` (or `--watch` is passed), `samgraha compile` stays running and watches the documentation directory for file changes. Changed `.md`/`.toml` files are recompiled incrementally after the `debounce_ms` window. This is useful during active documentation editing.

### Incremental Compilation

Regardless of `watch`, compilation is always incremental. Content hashes are stored in the knowledge database itself (not a separate file) and only changed files are reprocessed on the next `compile`.

### Full Recompilation

To force a full recompile (ignore content hashes):

```bash
samgraha compile --force
```

## Related

- [Command: compile](../commands/compile.md)
- [Concepts: Knowledge Database](../concepts/knowledge-db.md)
- [samgraha.toml Overview](samgraha-toml.md)
