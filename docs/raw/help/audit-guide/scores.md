# Audit Scores and Readiness Levels

## Purpose

How audit scores are calculated and what readiness levels mean.

## Content

### Score Calculation

There is no per-document percentage. A document counts as "passed" only if it has zero error-severity findings; scores are the passed-document ratio:

```
Domain Score  = (documents in domain − error-severity findings in domain) / documents in domain × 100
Overall Score = (total documents − total error-severity findings) / total documents × 100
```

Overall score is computed independently across all documents — it is not an average of the per-domain scores. Warning and suggestion findings never affect the score, only errors do.

### Readiness Levels

Readiness is derived from the overall score (and whether any errors exist):

| Condition | Readiness |
|-----------|-----------|
| score ≥ 90 and zero errors | Production |
| score ≥ 80 | Implementation |
| score ≥ 70 | Engineering |
| score ≥ 60 | Design |
| score ≥ 50 | Architecture |
| below 50 | Product |

(A `None` readiness value also exists in the schema but is not produced by the current scoring logic.)

### Quality Gates

Gates enforce a minimum overall score via the CLI flag — there is currently no `samgraha.toml` config key for it (an `[audit.gates.<name>]` section with `min_score`/`min_readiness` exists in the config schema, but it isn't wired into `samgraha audit` yet):

```bash
samgraha audit --gate 80
```

Fails with exit code 2 (`AuditFailure`) if the overall score is below 80. This is designed for CI/CD pipelines.

## Related

- [Audit Overview](overview.md)
- [Gates](gates.md)
- [Configuration: Audit](../configuration/audit.md)
