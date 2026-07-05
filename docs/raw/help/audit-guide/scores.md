# Audit Scores and Readiness Levels

## Purpose

How audit scores are calculated across different audit types.

## Content

### Documentation Audit

Score is the passed-document ratio:

```
Domain Score  = (documents in domain − error-severity findings) / documents in domain × 100
Overall Score = (total documents − total error-severity findings) / total documents × 100
```

Warning and suggestion findings never affect the score, only errors do.

### Other Audit Types

Each audit defines its own scoring:

| Audit | Scoring Model |
|---|---|
| Build Audit | Engineering Strategy (25%) + Doc Quality (20%) + Engineering Readiness (25%) + Conformance (30%) |
| Security Audit | Security Strategy (25%) + Doc Quality (20%) + Security Readiness (25%) + Conformance (30%) |
| Consistency Audit | Layer Alignment (50%) + Cross-Layer Integrity (50%) |
| Coverage Audit | `(forward_score + reverse_score) / 2`. Zero denominator → 100%. |
| Dependency Governance | Justification (40%) + Version Policy (25%) + Health (25%) + Cross-References (10%) |

### Readiness Levels (Documentation Audit)

Readiness is derived from the overall score:

| Condition | Readiness |
|-----------|-----------|
| score ≥ 90 and zero errors | Production |
| score ≥ 80 | Implementation |
| score ≥ 70 | Engineering |
| score ≥ 60 | Design |
| score ≥ 50 | Architecture |
| below 50 | Product |

### Quality Gates

Gates enforce a minimum overall score:

```bash
samgraha audit --gate 80           # Documentation Audit
samgraha audit --pipeline build --gate 80  # Build Audit
```

Fails with exit code 2 (`AuditFailure`) if the overall score is below the threshold.

## Related

- [Audit Overview](overview.md)
- [Gates](gates.md)
- [Configuration: Audit](../configuration/audit.md)
