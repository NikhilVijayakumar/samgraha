# [audit] Configuration

## Purpose

Audit defaults — severity thresholds, provider selection, gating behavior, and pipeline configuration.

## Content

### Settings

```toml
[audit]
default_severity = "suggestion"
providers = ["deterministic"]

[audit.gates.feature]
enabled = true
min_score = 80.0
min_readiness = "implementation"

[audit.pipelines.build]
enabled = true
artifact_inspection = "optional"

[audit.pipelines.security]
enabled = true
runtime_verification = "optional"

[audit.pipelines.consistency]
enabled = true

[audit.pipelines.coverage]
enabled = true
scanner = "simple"        # "simple" | "treesitter" (future)

[audit.pipelines.dependency]
enabled = false            # spec only
```

### `default_severity`

Default: `"suggestion"`. The minimum severity level for reporting.

### `providers`

Default: `["deterministic"]`. List of audit providers for Documentation Audit only.

### `gates`

`HashMap<String, {enabled, min_score, min_readiness}>`, keyed by domain name. Applies to Documentation Audit only.

### `pipelines`

Pipeline-specific settings:

| Pipeline | Key | Values |
|---|---|---|
| build | `artifact_inspection` | `"optional"` (default), `"always"`, `"never"` |
| security | `runtime_verification` | `"optional"` (default), `"always"`, `"never"` |
| coverage | `scanner` | `"simple"` (default, grep-based), `"treesitter"` (future) |
| dependency | `enabled` | `false` (default, spec only) |

## Related

- [Command: audit](../commands/audit.md)
- [Audit Guide: Overview](../audit-guide/overview.md)
- [samgraha.toml Overview](samgraha-toml.md)
