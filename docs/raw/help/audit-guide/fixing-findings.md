# Fixing Audit Findings

## Purpose

How to respond to each type of audit finding with practical fixes.

## Content

### Documentation Audit Findings

Deterministic findings always read as `{rule description}: '{document path}'` — they name the document, not a line or heading.

#### Missing Section (error, `has_section`)

**Finding**: `feat-001: Feature must document its purpose: 'docs/raw/feature/authentication.md'`

**Fix**: Add the missing section with the canonical heading:

```markdown
## Purpose
Handle user authentication for the application.
```

#### Empty Title (error, `has_title`)

**Finding**: `readme-002: README must have a top-level title: 'README.md'`

**Fix**: Add a non-empty `# Title` as the document's first H1.

#### No Documents At All (error, `corpus_exists`)

**Finding**: `readme-001: Repository must have a README.md`

**Fix**: Add at least one document of that standard.

#### Implementation Leakage (warning, `no_implementation`)

**Finding**: `feat-004: Feature must not specify technology: 'docs/raw/feature/authentication.md'`

**Fix**: Move implementation details to the Feature Technical document.

### Semantic Heuristic Findings

The semantic provider (`--provider semantic`) adds its own findings:

- **sem-001** (warning) — document body under 50 words. Fix: add more context.
- **sem-002** (suggestion) — placeholder text (`TBD`/`TODO`/`FIXME`). Fix: replace with real content.
- **sem-003** (suggestion) — vision/feature/design doc names specific technology. Fix: describe capability instead.
- **sem-004** (suggestion) — vision doc uses requirement language. Fix: move to Feature doc.
- **sem-005** (suggestion) — engineering/architecture doc missing rationale. Fix: add explanation.

### Coverage Audit Findings

Forward coverage (doc→code) errors: documented capability not implemented. Fix: implement the capability.

Orphan findings (code→doc) are always **Warning**, never Error. Three valid resolutions:

| Resolution | When to use |
|---|---|
| **Document** | The orphan is intentional — document it in the appropriate domain |
| **Remove** | The orphan is dead code or experimental — delete it |
| **Suppress** | The orphan is acceptable as-is — suppress via `[audit.suppress]` config |

### Build & Security Audit Findings

Config-level findings indicate a mismatch between documentation and configuration. Fix: update either the doc or the config to match.

Artifact-level findings require a built binary. Fix: rebuild with correct configuration.

### Fix Workflow

1. Run `samgraha audit --report` (or `samgraha audit --pipeline <name> --report`) to see all findings.
2. Fix findings starting with errors (highest severity).
3. Rerun audit to verify fixes.
4. Repeat until the gate passes.

## Related

- [Audit Overview](overview.md)
- [Deterministic Audit](deterministic.md)
- [Semantic Audit](semantic.md)
- [Coverage Audit](../concepts/coverage.md)
