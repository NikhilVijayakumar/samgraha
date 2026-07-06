# Documentation Standards

## Purpose

What documentation standards are and how they define the structure and quality of documentation.

## Content

A **documentation standard** is a named specification that defines:

- **Required sections** — Which markdown headings a document must have.
- **Prohibited content** — Content that must not appear in the document.
- **Audit rules** — Automated checks that verify compliance.
- **Profiles** — Named subsets of sections for packaging.
- **Relationships** — How this standard relates to other standards.

### Built-in Standards

Samgraha ships with standards for these domains:

| Domain | Purpose |
|--------|---------|
| readme | Repository entry point |
| vision | Product vision and direction |
| philosophy | Design principles and values |
| architecture | System structural organization |
| feature | Atomic functional capabilities |
| feature-design | User-centered feature design |
| feature-technical | Architectural feature realization |
| design | Design language and UX standards |
| engineering | Engineering decisions and standards |
| external-context | External system dependencies |
| prototype | Executable simulations |
| help | Product documentation |
| standards | Standards reference docs |

### Custom Standards

The schemas for custom/extended standards exist in code (`StandardDeclaration`, `StandardLoader::load_from_declarations`/`discover_from_path` in `crates/standards`), but there is currently no `samgraha.toml` field or CLI flag that wires them in — `KnowledgeRuntime` and `WorkspaceService` always build the standard registry from `StandardRegistry::with_builtins()` only. In practice, today's 13 built-in standards are fixed; a repo can only select a subset of them via `[repository.documentation].domain` / `domain_exclusion`, not add new ones.

## Related

- [Domains](domains.md)
- [Profiles](profiles.md)
- [Documentation Guide](../documentation-guide/overview.md)
- [Standards Reference](../../standards/overview.md)
