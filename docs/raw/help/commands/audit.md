# samgraha audit

## Purpose

Audit documentation against standard-defined rules and produce quality reports.

## Content

### Synopsis

```bash
samgraha audit [domain] [--pipeline <name>] [--provider <name>]... [--all] [--gate [<score>]] [--report] [--inspect-artifact] [--runtime]
```

### Description

`audit` runs configured audit checks. Default mode checks documentation against standards (Documentation Audit). Use `--pipeline` to select a different audit type.

### Options

| Flag | Description |
|------|-------------|
| `domain` | Positional, optional. Audit only documents in the given domain (Documentation Audit only) |
| `--pipeline <name>` | Audit type: `doc` (default), `build`, `security`, `consistency`, `coverage`, `dependency`. When not `doc`, `domain` and `--provider` are rejected |
| `--provider <name>` | Audit provider: `deterministic` (default) or `semantic`. Documentation Audit only |
| `--all` | Audit all domains explicitly (Documentation Audit only) |
| `--gate [<score>]` | Fail if quality score is below `score`. Defaults to 100.0 |
| `--report` | Save a markdown report under `[report].dir/{pipeline}/{latest,archive}/` |
| `--inspect-artifact` | Enable artifact-level checks (Build Audit only). Requires a built binary |
| `--runtime` | Enable runtime-level checks (Security Audit only). Requires running app |

### Severity Levels

| Severity | Meaning | Effect on score |
|----------|---------|-------------|
| `error` | Must fix | Counts against score |
| `warning` | Should fix | Reported, does not affect score |
| `suggestion` | Consider fixing | Reported, does not affect score |

Orphan findings (Coverage Audit reverse checks) are always **warning**, never error.

### Examples

```bash
# Full documentation audit
samgraha audit

# Audit with report and quality gate
samgraha audit --report --gate 80

# Build Audit: config-level checks only
samgraha audit --pipeline build

# Build Audit: with artifact inspection
samgraha audit --pipeline build --inspect-artifact

# Security Audit: static + config checks
samgraha audit --pipeline security

# Security Audit: with runtime verification
samgraha audit --pipeline security --runtime

# Consistency Audit
samgraha audit --pipeline consistency

# Coverage Audit
samgraha audit --pipeline coverage
```

## Related

- [Compile](compile.md)
- [Audit Guide: Overview](../audit-guide/overview.md)
- [Configuration: audit](../configuration/audit.md)
