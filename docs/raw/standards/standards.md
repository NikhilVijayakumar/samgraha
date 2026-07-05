# Standards Reference Standard

This section details the Standards Reference Standard.

## Purpose

This document defines the standard that governs every document in this `docs/raw/standards/` collection — the reference material that describes each of Samgraha's other documentation standards (README, Vision, Feature, Architecture, and so on), plus itself.

A Standards Reference document explains what a standard requires, why, and how to use it. It does not contain the engineering content that standard governs — that lives in the repository being documented.

## Sections

Every Standards Reference document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Title | `title` | ✓ | Title |
| Purpose | `purpose` | ✓ | Purpose |
| Sections | `sections` | ✓ | Required Sections, Section Schema |
| Audit Rules | `audit_rules` | | Rules, Checks |
| Usage | `usage` | | Best Practices |
| Related | `related` | | See Also |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

## Audit Rules

| ID | Check | Severity |
|----|-------|----------|
| `std-001` | Has title | error |
| `std-002` | Has sections | error |
| `std-003` | Has purpose | warning |

## Usage

Written once per standard, updated when the corresponding `StandardDefinition` in `crates/standards/src/builtin.rs` changes (new required section, new audit rule, new profile) — the two must stay in sync, since this reference is documentation *about* code, not independently authoritative. Use `samgraha compile --domain standards` to validate the collection, and `samgraha search --domain standards` (or the MCP `search`/`get_document` tools) so an LLM can look up a standard's actual required sections before generating or auditing a document under it.

## Related

- [index.md](index.md) — table of contents for this collection
- [overview.md](overview.md) — how the standards system works generally
- [Help Standard](help.md) — the other built-in knowledge domain, product docs rather than standards reference
- [domain-relationships.md](domain-relationships.md) — how all 13 domains relate to each other
