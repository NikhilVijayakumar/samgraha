# Deterministic Audit

## Purpose

Rule-based audit checks that run locally with no external dependencies.

## Content

### Overview

The deterministic audit checks documents against rules defined in each standard. In the new architecture, these checks are provided by system `validate` scripts via capability dispatch — the script performs all checks for domains it owns. There is no built-in fallback: a domain with no working `validate` script fails clearly instead of running a built-in deterministic engine.

Checks are fast and reliable once a system provides a script — but availability depends on that script existing, not on a permanent built-in engine.

### Check Types

| Check Type | What It Checks | Example Rule |
|------------|---------------|--------------|
| `corpus_exists` | Whether any documents exist in the domain | readme-001: README exists |
| `has_title` | Whether the document's parsed title (its first H1 heading text) is non-empty | readme-002: Has title |
| `has_section` | Whether a specific required section exists | feat-001: Has Purpose |
| `no_implementation` | Whether the document contains implementation details | feat-004: Technology independent |

### How Rules Are Defined

Two mechanisms coexist, not a primary path plus a fallback:

- **The `Doc`-kind path** (`audit()`, not capability dispatch) — rules are
  declarative `AuditRuleDef` rows loaded from the registered standard's
  own data, interpreted generically by `DeterministicAuditProvider`
  (no domain knowledge in samgraha's source, just row interpretation):
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
- **Named-domain capability dispatch** (`validate` scripts) — rules live
  entirely inside the system's own script and are never visible to
  samgraha's source code. A named domain (e.g. `architecture`,
  `vision`, ...) with no `validate` script has no way to run this kind of
  check at all — the `Doc`-kind path is not a fallback for it.

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
