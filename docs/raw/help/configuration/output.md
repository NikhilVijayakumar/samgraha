# [output] Configuration

## Purpose

Output formatting configuration for CLI commands.

## Content

### Settings

```toml
[output]
format = "text"
color = true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `format` | String | `"text"` | Schema field for the default output format |
| `color` | bool | `true` | Schema field for whether to colorize output |

### Actual Output Format Control

In the current CLI, output format is controlled entirely by the **global `--json` flag** (`samgraha <command> --json`), not by this config section or a per-command `--format` flag — there is no `--format` flag on any command, and no `jsonl` format exists. `[output].format`/`color` are schema fields not yet read by any command; use `--json`/`--no-color` instead:

```bash
samgraha search "authentication" --json
samgraha audit --no-color
```

## Related

- [samgraha.toml Overview](samgraha-toml.md)
- [Command: search](../commands/search.md)
- [Command: audit](../commands/audit.md)
