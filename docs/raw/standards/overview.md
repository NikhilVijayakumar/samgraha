# Standards System Overview

## Purpose

Explains how the standards system works in general — what a `StandardDefinition` is, how the compiler assigns a document to one, how sections are matched, and what versioning and lifecycle exist today. Read this before [index.md](index.md)'s individual standard pages if you haven't touched Samgraha's compiler internals before.

## Section Schema

Every standard (defined in `crates/standards/src/builtin.rs`, registered via `StandardRegistry::with_builtins()`) declares:

- **`id` / `domain`** — the standard's identifier and the `standard` value stamped onto every `Document` compiled under it. `domain` is what you pass to `--domain` on `compile`/`search`/`audit`.
- **`required_sections`** — a list of `SectionDefinition`s, each with a `canonical_name` (the heading text as it should appear), a `semantic_type` (a stable snake_case key used for storage/querying — e.g. `purpose`, `functional_requirements`), a list of case-insensitive `aliases` (alternate heading spellings that count as a match), and a `required` flag (documentation-only today — see Versioning & Lifecycle).
- **`audit_rules`** — `AuditRuleDef`s with a `check_type` (`corpus_exists`, `has_title`, `has_section`, or `no_implementation`) and a `severity` (`error`, `warning`, `suggestion`). These are what `samgraha audit --domain <domain>` actually runs.
- **`profiles`** — named subsets of sections (`ProfileDef`) used by the packaging/resolution services to include only what a given consumer needs.
- **`relationships`** — declared links to other domains (see [domain-relationships.md](domain-relationships.md)).
- **`prohibited_content`** — free-text list of things that standard shouldn't contain, checked by the `no_implementation`-style audit rules where applicable.

A heading in a compiled document matches a section when its text equals a `canonical_name` or one of its `aliases`, case-insensitively. Headings that don't match anything in a standard's `required_sections` are stored with `semantic_type: "generic"` — preserved and searchable, just not queryable by type.

## Versioning & Lifecycle

Every built-in standard currently declares `version: "1.0.0"` as a plain string — there is no migration mechanism between standard versions today. `required_sections[].required` is populated for documentation purposes (and used by `StandardDefinition::missing_required()`/`section_is_required()` if a caller wants it) but is not enforced as a hard compile failure; the only enforcement mechanism today is the explicit `audit_rules` list. If a document is missing a section marked `required: true` but there's no matching `has_section` audit rule for it, nothing will flag it. When authoring a new standard, add an audit rule for anything that actually needs to be enforced — don't rely on the `required` flag alone.

`StandardRegistry` is append-only and keyed by `"{id}@{version}"` (`crates/standards/src/registry.rs`) — registering a new standard, or a new version of an existing one, can't break existing lookups by earlier versions.

## Usage

Read this once when you're adding a new built-in standard or debugging why a `has_section` audit rule isn't firing the way you expected. Use `samgraha info` to see every registered standard's domain, and `samgraha audit --domain <domain>` to see which audit rules actually run against a given document set.

## Related

- [index.md](index.md) — table of contents for the standards collection
- [domain-relationships.md](domain-relationships.md) — how the 13 domains relate to each other
- [best-practices.md](best-practices.md) — cross-standard usage patterns and anti-patterns
- [Standards Reference Standard](standards.md) — the schema this document collection itself follows
