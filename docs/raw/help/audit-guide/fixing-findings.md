# Fixing Audit Findings

## Purpose

How to respond to each type of audit finding with practical fixes.

## Content

Deterministic findings always read as `{rule description}: '{document path}'` — they name the document, not a line or heading. There is no fuzzy "did you mean" suggestion and no per-section word-count check; keep that in mind when interpreting messages below.

### Missing Section (error, `has_section`)

**Finding**: `feat-001: Feature must document its purpose: 'docs/raw/feature/authentication.md'`

**Fix**: Add the missing section with the canonical heading (or one of its aliases):

```markdown
## Purpose
Handle user authentication for the application.
```

### Empty Title (error, `has_title`)

**Finding**: `readme-002: README must have a top-level title: 'README.md'`

**Fix**: Add a non-empty `# Title` as the document's first H1 — the parser takes its text as the document title.

### No Documents At All (error, `corpus_exists`)

**Finding**: `readme-001: Repository must have a README.md`

**Fix**: This only fires when zero documents exist in the domain — add at least one document of that standard.

### Implementation Leakage (warning, `no_implementation`)

**Finding**: `feat-004: Feature must not specify technology: 'docs/raw/feature/authentication.md'`

This check flags code fences (```` ```rust ````, ```` ```python ````, etc.) and keywords like `fn `, `impl `, `struct `, `pub `, `let `, `cargo install`, `npm install`, `pip install`.

**Fix**: Move implementation details to the Feature Technical document. Keep the Feature doc focused on requirements.

### Semantic Heuristic Findings

The semantic provider (see [Semantic Audit](semantic.md)) adds its own findings when run with `--provider semantic`:

- **sem-001** (warning) — document body under 50 words. Fix: add more context.
- **sem-002** (suggestion) — placeholder text like `TBD`/`TODO`/`FIXME`/`coming soon`. Fix: replace with real content.
- **sem-003** (suggestion) — a `vision`/`feature`/`design` doc names a specific technology (React, PostgreSQL, Rust, ...). Fix: describe the capability instead.
- **sem-004** (suggestion) — a `vision` doc uses requirement-level language (`SHALL`, `MUST`, `FR1`, `API`, `endpoint`). Fix: move that language into a Feature document.
- **sem-005** (suggestion) — an `engineering`/`architecture` doc has no rationale/decision language. Fix: add a sentence explaining why the approach was chosen.

### Fix Workflow

1. Run `samgraha audit --report` to see all findings.
2. Fix findings starting with errors (highest severity).
3. Rerun audit to verify fixes.
4. Repeat until the gate passes.

## Related

- [Audit Overview](overview.md)
- [Deterministic Audit](deterministic.md)
- [Semantic Audit](semantic.md)
