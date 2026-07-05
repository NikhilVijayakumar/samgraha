# Quality Gates

## Purpose

How quality gates work and how to use them in CI/CD pipelines.

## Content

### What is a Gate?

A quality gate is a minimum score threshold. If the audit score falls below the threshold, the command exits with a non-zero exit code, which can fail a CI/CD pipeline.

### Using Gates

```bash
# Gate at 80 (fail if overall score < 80)
samgraha audit --gate 80

# Gate per domain (domain is a positional argument)
samgraha audit feature --gate 90

# --gate with no value defaults to 100.0
samgraha audit --gate
```

### CI/CD Integration

Example GitHub Actions workflow:

```yaml
- name: Audit documentation
  run: samgraha audit --report --gate 80
```

### Failure Behavior

When a gate fails:
1. The full report is still generated (if `--report` was passed).
2. Exit code is 2 (`AuditFailure`).
3. All findings are listed so the team knows what to fix.

### Gate Thresholds by Environment

| Environment | Recommended Gate |
|-------------|-----------------|
| Development | 50 (don't block work) |
| Pull Request | 70 (catch major issues) |
| Main branch | 85 (production quality) |
| Release | 90 (ship quality) |

### MCP Gate (Per Stage)

This CLI gate is separate from the MCP `check_gate` tool, which blocks per `AuditStage` (Deterministic/Section/Document/CrossDomain) instead of by score threshold — see [Stages](stages.md).

## Related

- [Scores](scores.md)
- [Reports](reports.md)
- [Stages](stages.md)
- [Configuration: Audit](../configuration/audit.md)
