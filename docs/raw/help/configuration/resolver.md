# [resolver] Configuration

## Purpose

Resolver behavior — how the Knowledge Resolver caches and refreshes dependency metadata via the local Repository Registry.

## Content

### Settings

```toml
[resolver]
metadata_cache = true
metadata_ttl = "24h"
knowledge_ttl = "720h"
auto_refresh = true
registry_type = "file"
# registry_url = "https://example.com/registry"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `metadata_cache` | bool | `true` | Whether to cache dependency metadata (`.meta` files) rather than re-resolving every time |
| `metadata_ttl` | String | `"24h"` | How long cached dependency metadata is valid before `sync`/`refresh` is needed. Parsed with a duration suffix: `s`, `m`, `h`, `d` |
| `knowledge_ttl` | String | `"720h"` | How long an assembled Knowledge Package/session is valid |
| `auto_refresh` | bool | `true` | Whether to auto-refresh metadata when it expires |
| `registry_type` | `"file"` \| `"http"` | `"file"` | Registry storage backend. `"file"` uses the local `.samgraha/registry.db`. `"http"` is reserved for future remote registry support — not implemented yet |
| `registry_url` | `Option<String>` | unset | Remote registry URL, only meaningful once `registry_type = "http"` is implemented |

### Behavior

`registry sync`/`registry refresh` use `metadata_ttl` to decide whether cached dependency metadata (`.meta` files, per registered repository) needs refreshing. There is no `max_depth` or `timeout` setting in this section.

## Related

- [Concepts: Resolver](../concepts/resolver.md)
- [Documentation Section](documentation.md)
- [Multi-Repo Guide](../multi-repo-guide/overview.md)
