# Deterministic Audit

## Purpose

Rule-based audit checks that run locally with no external dependencies.

## Content

### Overview

The deterministic audit checks documents against rules defined in each standard. In the new architecture, these checks are provided by system `validate` scripts via capability dispatch. For domains with a working system script, the script performs all checks. For domains without a system script, the built-in deterministic engine serves as a fallback.

Checks are fast, reliable, and always available.

### Check Types

| Check Type | What It Checks | Example Rule |
|------------|---------------|--------------|
| `corpus_exists` | Whether any documents exist in the domain | readme-001: README exists |
| `has_title` | Whether the document's parsed title (its first H1 heading text) is non-empty | readme-002: Has title |
| `has_section` | Whether a specific required section exists | feat-001: Has Purpose |
| `no_implementation` | Whether the document contains implementation details | feat-004: Technology independent |

### How Rules Are Defined

For built-in fallback, rules are defined in the standard definition:

```rust
AuditRuleDef {
    id: "feat-001",
    name: "Has purpose",
    description: "Feature must document its purpose",
    severity: "error",
    check_type: "has_section",
    scope: "Purpose",
}
```

For system-provided `validate` scripts, rules are defined inside the system's script and are not visible to samgraha's source code.

### Running Deterministic Audit

```bash
# Default — dispatches to system validate scripts where available
samgraha audit

# Explicitly specify a domain
samgraha audit --domain feature
```

## Related

- [Semantic Audit](semantic.md)
- [Stages](stages.md)
- [Fixing Findings](fixing-findings.md)
