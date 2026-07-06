# [ai] Configuration

## Purpose

AI integration configuration — settings for semantic audit and AI-assisted features.

## Content

### Settings

```toml
[ai]
provider = "lms"

[ai.lms]
endpoint = "http://localhost:1234"
model = "..."
api_key = "..."

# [ai.ollama] and [ai.openai] take the same endpoint/model/api_key shape
```

There is no top-level `api_endpoint`/`model` field — per-provider settings live in the matching `[ai.lms]`, `[ai.ollama]`, or `[ai.openai]` sub-table, each with `endpoint: Option<String>`, `model: Option<String>`, `api_key: Option<String>`.

### Current Status

As of this writing, `provider` is a schema field that isn't yet wired to a real model call: any value other than `"rule-based"` (or leaving `provider` unset) currently falls back to a no-op rule-based provider, with a warning logged. Setting `provider` to a non-empty value does turn on the `enrichment` and (if `[audit].providers` includes `semantic`) `semantic-audit` capabilities reported in the repository manifest, but the semantic audit provider itself is presently a heuristic/rule-based implementation (checks for short documents, placeholder text like "TODO"/"TBD", scope violations) — it does not call out to `lms`/`ollama`/`openai` yet.

### Purpose (intended)

The `[ai]` section is intended to configure optional AI integration for:

- **Semantic audit** — AI-powered content quality analysis beyond deterministic rules
- **Enrichment** — AI-generated summaries and keywords during compilation

AI is never required at runtime — compile, search, and audit all work with `[ai]` left unset.

## Related

- [Audit Guide: Semantic](../audit-guide/semantic.md)
- [samgraha.toml Overview](samgraha-toml.md)
- [Environment Setup](../getting-started/environment.md)
