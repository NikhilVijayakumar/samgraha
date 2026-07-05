# [knowledge] Configuration

## Purpose

Declares which other repositories' compiled knowledge this repository loads, by name — not by database path.

## Content

### Settings

```toml
[knowledge]
dependencies = ["core-lib"]
interests = ["api-service"]
```

There is no `extra_stores` field — repositories are referenced by name (matching a registered/workspace repository), not by a direct path to a `.db` file.

| Field | Type | Description |
|-------|------|-------------|
| `dependencies` | `Vec<String>` | Repository names always loaded eagerly. Required — these repos must be resolvable |
| `interests` | `Vec<String>` | Repository names always loaded but lazily-opened on first query. Optional — missing/unresolvable interests don't fail the command |

### Built-in Stores

The `help` and `standards` stores are separate from `[knowledge]` — they're always available and loaded automatically from the binary directory (see [Command: info](../commands/info.md)); they don't need to be, and aren't, listed here.

## Related

- [Concepts: Knowledge Context](../concepts/knowledge-context.md)
- [samgraha.toml Overview](samgraha-toml.md)
- [Resolver Section](resolver.md)
