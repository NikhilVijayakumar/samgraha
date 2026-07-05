# Deterministic Audit Provider

## Purpose

Rule-based audit checks that run locally with no external dependencies.

## Content

### Overview

The deterministic provider checks documents against hardcoded rules defined in each standard. These checks are fast, reliable, and always available.

### Check Types

| Check Type | What It Checks | Example Rule |
|------------|---------------|--------------|
| `corpus_exists` | Whether any documents exist in the domain | readme-001: README exists |
| `has_title` | Whether the document's parsed title (its first H1 heading text) is non-empty | readme-002: Has title |
| `has_section` | Whether a specific required section exists | feat-001: Has Purpose |
| `no_implementation` | Whether the document contains implementation details | feat-004: Technology independent |

### How Rules Are Defined

Rules are defined in the standard definition (Rust code for built-in standards):

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

### Running Deterministic Audit

```bash
# Default provider is deterministic
samgraha audit

# Explicitly specify deterministic
samgraha audit --provider deterministic
```

## Related

- [Semantic Audit](semantic.md)
- [Stages](stages.md)
- [Fixing Findings](fixing-findings.md)
