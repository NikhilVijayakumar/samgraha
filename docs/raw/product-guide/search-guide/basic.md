# Basic Search

## Purpose

How to perform basic search queries and understand results.

## Content

### Simple Query

```bash
samgraha search "authentication"
```

Returns sections matching "authentication" across all domains.

### Multiple Words

```bash
samgraha search "user authentication flow"
```

The query is split into individual words; there is no phrase matching. Each word is scored independently (title match scores higher than a body match), so a document matching all three words ranks higher than one matching only one — but a single-word match is still returned.

### Multiple Terms

```bash
samgraha search "authentication registration"
```

Matches documents containing either or both terms (OR behavior) — any document with at least one matching word is returned.

### Result Format

Text output is a table:

```
Title                Domain    Score   Snippet
──────────────────── ───────── ─────── ────────────────────────
Authentication        feature   10.00   Handle user authentication...
Login Flow             feature-design 5.00   1. User enters credentials
──────────────────── ───────── ─────── ────────────────────────
2 result(s) in 3ms
```

Each result shows:
- Document title and domain
- Relevance score
- A one-line matched snippet (not the whole matched section)

Results do not carry a "source repository" tag — that applies only within a multi-repo MCP session, and even there results are not labeled per-repo (see [Cross-Repository Search](cross-repo.md)).

## Related

- [Search Overview](overview.md)
- [Filtering](filtering.md)
- [Pagination](pagination.md)
