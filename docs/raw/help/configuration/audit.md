# [audit] Configuration

## Purpose

Audit defaults — severity thresholds, provider selection, and gating behavior.

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
```

Note the key is `default_severity` (underscore, not `default-severity`), and `providers` is plural — a list, not a single `provider` string. There is no scalar `gate` field; quality gates are a `gates` table keyed by domain name.

### `default_severity`

Default: `"suggestion"`. Documented as the minimum severity level; the CLI's own `--gate` behavior is driven purely by the numeric score (see below), not by this field directly.

### `providers`

Default: `["deterministic"]`. List of audit providers to run when `samgraha audit` is invoked without `--provider`. `--provider` on the CLI can be repeated to override this per-invocation. The two providers currently registered are `deterministic` and `semantic` (see [Configuration: ai](ai.md) for `semantic`'s actual current implementation).

### `gates`

`HashMap<String, {enabled, min_score, min_readiness}>`, keyed by domain name. This configures per-domain quality gates (distinct from the CLI's own `--gate [<score>]` flag, which applies a single overall-score threshold for the invocation, defaulting to 100.0 if passed with no value). Fields:

- `enabled: bool`
- `min_score: Option<f64>`
- `min_readiness: Option<String>`

## Related

- [Command: audit](../commands/audit.md)
- [Audit Guide: Overview](../audit-guide/overview.md)
- [samgraha.toml Overview](samgraha-toml.md)
