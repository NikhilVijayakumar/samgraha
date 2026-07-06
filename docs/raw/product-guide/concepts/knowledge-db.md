# Knowledge Database

## Purpose

What the `.samgraha/knowledge.db` database contains and how it works.

## Content

`knowledge.db` is a SQLite database produced by `samgraha compile`. It stores all compiled documentation in a structured, queryable format.

### Schema Overview

```
documents         → Document metadata (id, path, hash, standard/domain, title, body)
document_sections → Extracted sections (semantic_type, canonical_name, content, order, hash)
search_index       → Term → document frequency index used by search
enrichment         → Per-document generated artifacts (summary, keywords, etc.), tagged by provider
glossary           → Extracted glossary terms
audit_results      → Stored audit findings per document
```

### How It Is Populated

1. `samgraha compile` discovers `**/*.md` files in `docs/raw/`.
2. Each file is parsed by the standard matching its domain.
3. Sections are extracted and stored in `document_sections`.
4. Content hashes enable incremental recompilation.
5. Enrichment (summary, keywords, glossary) is auto-generated.

### How It Is Queried

- **CLI**: `samgraha search`, `samgraha sections`
- **MCP**: `search`, `get_sections`, `get_document`, `get_document_section` methods
- **Direct**: SQLite queries (for advanced users)

### Built-in Knowledge

Samgraha ships with `standards.db` and `help.db` — compiled knowledge databases for the built-in standards reference and product help, shipped next to the `cli`/`mcp` binary. They're loaded automatically at startup (both CLI and MCP) by `load_builtin_stores()`; a missing file is skipped silently, never fatal.

## Related

- [Compilation](../commands/compile.md)
- [Search](../commands/search.md)
- [Sections](../commands/sections.md)
- [Registry](registry.md)
