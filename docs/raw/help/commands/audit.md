# samgraha audit

## Purpose

Audit documentation against standard-defined rules and produce quality reports.

## Content

### Synopsis

```bash
samgraha audit [domain] [--provider <name>]... [--all] [--gate [<score>]] [--report]
```

### Description

`audit` runs configured audit checks against compiled documentation. It produces findings classified by severity and calculates quality scores. `domain` is a positional argument (optional; default is all domains).

### Options

| Flag | Description |
|------|-------------|
| `domain` | Positional, optional. Audit only documents in the given domain |
| `--provider <name>` | Audit provider to run: `deterministic` (default) or `semantic`. Repeatable — pass multiple times to run several providers |
| `--all` | Audit all domains explicitly |
| `--gate [<score>]` | Fail the command if the quality score is below `score`. If passed with no value, defaults to a minimum of 100.0 |
| `--report` | Save a markdown report under `[report].dir/audit/{latest,archive}/` |

### Severity Levels

| Severity | Meaning | Effect on score |
|----------|---------|-------------|
| `error` | Must fix | Counts against the quality score (score = passed/total findings %) |
| `warning` | Should fix | Reported, does not affect the score |
| `suggestion` | Consider fixing (default `default_severity`) | Reported, does not affect the score |

Because the default gate threshold (`--gate` with no value) is 100.0, a single error finding is enough to fail the gate.

### Examples

```bash
# Full audit
samgraha audit

# Audit with report and quality gate
samgraha audit --report --gate 80

# Audit only feature domain with semantic provider
samgraha audit feature --provider semantic

# Audit everything with the default gate (100.0)
samgraha audit --all --gate
```

## Related

- [Compile](compile.md)
- [Audit Guide: Overview](../audit-guide/overview.md)
- [Configuration: audit](../configuration/audit.md)
