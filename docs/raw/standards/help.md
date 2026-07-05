# Help Standard

This section details the Help Standard.

## Purpose

This document defines the standard for Help documentation — the product docs shipped in `help.db` next to the binary, written for end users and for an LLM looking up how to use Samgraha or how to interpret an error.

Help topics explain **how to use the product**. They do not define product vision, architecture, or engineering rationale — those belong to their own standards (see [index.md](index.md)).

## Sections

Every Help topic must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Title | `title` | ✓ | Title |
| Content | `body` | ✓ | Body, Details |
| Purpose | `purpose` | | Overview |
| Related | `related` | | See Also, References |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

## Audit Rules

| ID | Check | Severity |
|----|-------|----------|
| `help-001` | Has title | error |
| `help-002` | Has purpose | suggestion |
| `help-003` | Has content | error |

## Usage

Written by whoever ships a feature that needs end-user-facing explanation — a help topic is small and single-purpose, one file per concept/command/guide, never a catch-all. Use `samgraha compile --domain help` (done automatically by the release build for the shipped `docs/raw/help/` tree) and `samgraha audit --domain help` to confirm every topic has a title and body before it ships. Two packaging profiles exist: `quickref` (title + body only, for compact contexts) and `full` (all four sections).

## Related

- [index.md](index.md) — table of contents for the standards collection
- [Standards Reference Standard](standards.md) — the meta-standard this document is itself written against
- `docs/raw/help/index.md` — the actual Help content tree's table of contents (not this file — this one documents the *standard*, that one is the *content*)
