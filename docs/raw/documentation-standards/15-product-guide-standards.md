# Product Guide Standard

## Purpose

This document defines the standard for Product Guide documentation — the product docs shipped in `help.db` next to the binary, written for end users and for an LLM looking up how to use Samgraha or how to interpret an error.

Product Guide topics explain **how to use the finished product**. They do not define product vision, architecture, engineering rationale, or any other domain's reasoning — those belong to their own standards, and Product Guide is written last, after everything else exists, because it can't be accurate until the product it describes is finished.

---

## Required Sections

Every Product Guide topic must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases |
|---------|--------------|----------|---------|
| Title | `title` | ✓ | Title |
| Content | `body` | ✓ | Body, Details |
| Purpose | `purpose` | | Overview |
| Product Context | `product-context` | | Context, Background |
| Public Contract | `public-contract` | | Interface, API |
| Related | `related` | | See Also, References, Cross-References |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Product Guide aims to:

* Give end users and LLMs a single authoritative place to learn product usage.
* Stay accurate to the shipped product by being written last, not speculatively.
* Keep each topic small enough to be useful as a standalone lookup.

---

## Non-Goals

Product Guide does not define:

* Product Vision or Philosophy
* Architecture or Engineering rationale
* Security threat models
* Feature specifications or technical design
* Source code

These responsibilities belong to other documentation standards.

---

## Success Criteria

Product Guide is successful when:

* A user or LLM can find how to do something without reading source code.
* Every CLI/MCP/config surface has corresponding coverage.
* Topics stay current as the product changes (Accuracy check catches drift).
* A reader lands on the right topic within one or two navigation steps.

---

## Responsibilities

Product Guide is responsible for:

* Explaining how to use a specific command, concept, or workflow
* Documenting the public contract of a CLI/MCP surface (inputs, outputs, error conditions)
* Routing a reader from a symptom (an error, a question) to the right topic
* Staying scoped to one concept per topic — small, single-purpose files

Product Guide is the only domain written after the product is finished, not before.

---

## Scope

Product Guide may describe:

* How to install, configure, and run the product
* What a specific command, flag, or config option does
* Common workflows end to end
* How to interpret a specific error message
* Troubleshooting steps for a specific symptom

Every topic should stay single-purpose — one command, one concept, one guide.

---

## Out of Scope

Product Guide must not describe:

* Product Vision or strategy
* Philosophy or guiding principles
* Architecture or system design
* Engineering rationale or technology choices
* Security threat models
* Feature specifications or technical design
* Source code

These belong to their own documentation standards. Product Guide only explains usage of what already exists.

---

## Inputs

Product Guide derives from the finished product itself — every other domain, once built:

* Vision — for framing what the product is, at a level an end user needs
* Feature / Feature Design — for what a workflow is supposed to do
* Build — for install/run instructions that match what's actually shippable
* README — for the high-level entry point Product Guide expands on

Product Guide should not derive from plans or drafts — only from what's actually shipped.

---

## Outputs

Product Guide provides:

* End-user-facing explanations of how to use the product
* LLM-consumable reference for tool-use and error interpretation (`help.db`)
* The corpus validated by the Product Guide Audit Pipeline (Coverage, Navigation, Quality, Accuracy)

---

## Traceability

```text
Everything else (Vision … Build, README)
              │
              ↓
        Product Guide
```

Product Guide has no formal `relationship(...)` entries in `crates/standards/src/builtin.rs` — it's flat, built-in content, the same as `standards` itself. It doesn't participate in the tiered derivation graph the other 14 domains form; it references all of it informally, through prose and the freeform `Related` section on each topic, rather than through a machine-readable edge. See `00-domain-relationships.md`.

---

## Relationships

Product Guide declares no `relationship(...)` entries — by design, matching how `help` is documented in `00-domain-relationships.md`. Individual topics link elsewhere through their own `Related` section instead of a domain-level graph edge.

---

## Required Characteristics

A Product Guide topic should be:

* Single-purpose — one command, concept, or guide per topic
* Accurate to the shipped product, not aspirational
* Scannable — a reader should find the answer without reading the whole topic
* LLM-consumable — structured enough for `help.db` lookups, not just prose

---

## Generation Rules

When generating Product Guide topics:

* Write for the finished product, not for planned features.
* One topic per command, concept, or workflow.
* Lead with what the user wants to do, not with what the tool is.
* Document the public contract: inputs, outputs, error conditions.
* Use concrete examples over abstract descriptions.
* Keep topics short enough to scan in under a minute.

---

## Enhancement Rules

When enhancing Product Guide topics:

* Verify accuracy against the current shipped product.
* Remove references to features that no longer exist.
* Add topics for new CLI/MCP surfaces not yet covered.
* Improve scannability — headers, short paragraphs, concrete examples.
* Preserve the single-purpose scope of each topic.
* Update cross-references when related topics change.

---

## Audit Rules

| ID | Check | Severity |
|----|-------|----------|
| `help-001` | Has title | error |
| `help-002` | Has purpose | suggestion |
| `help-003` | Has content | error |
| `help-004` | Has product context | suggestion |
| `help-005` | Has public contract | warning |

The corpus is also validated by the **Product Guide Audit Pipeline** (`samgraha audit --pipeline help`) — a separate, deeper check spanning Coverage, Navigation, Quality, and Accuracy against the actual CLI/MCP/config surface, not just per-topic section presence. See [Product Guide Audit Pipeline](../product-guide/concepts/help-audit.md).

---

## Summary

Product Guide is the end-user- and LLM-facing usage documentation for the finished product — commands, workflows, error interpretation, and troubleshooting — written last, after every other domain has settled, because it can only be accurate once the product it describes actually exists. It declares no formal relationships; it references everything informally instead.

---

## Documentation Folder

Product Guide documents live under:

```text
docs/raw/product-guide/
```

---

## Usage

Written by whoever ships a feature that needs end-user-facing explanation — a help topic is small and single-purpose, one file per concept/command/guide, never a catch-all. Use `samgraha compile --domain help` (done automatically by the release build for the shipped `docs/raw/product-guide/` tree) and `samgraha audit --domain help` to confirm every topic has a title and body before it ships. Two packaging profiles exist: `quickref` (title + body only, for compact contexts) and `full` (all six sections).

## Related

- [Readme Standard](14-readme-standards.md) — the entry point Product Guide expands on
- [Build Standard](13-build-standards.md) — install/run instructions Product Guide must stay accurate to
- [Standards Reference Standard](standards.md) — how this standard itself is documented
- `docs/raw/product-guide/index.md` — the actual Product Guide content tree's table of contents (not this file — this one documents the *standard*, that one is the *content*)
