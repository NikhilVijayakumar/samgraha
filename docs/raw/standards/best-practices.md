# Standards Best Practices

## Purpose

Patterns and anti-patterns that repeat across all 13 standards, pulled together in one place instead of copy-pasted into every individual standard document. Read the relevant standard first (see [index.md](index.md)) — this document assumes you already know which one you're writing against.

## Sections

### Cross-Standard Patterns

**Reference, don't duplicate.** External Context, Design, Architecture, and Engineering are all explicitly meant to be referenced from downstream documents (Feature Design, Feature Technical Design) rather than re-explained. If you find yourself copying a paragraph from one standard's document into another, link to it instead.

**One document, one responsibility.** Feature, Feature Design, Feature Technical, External Context, and Prototype are all explicitly atomic — one file per feature/dependency/concern. Architecture, Design, and Engineering are explicitly modular collections for the same reason. A document trying to cover two unrelated things should be split.

**Technology independence, until it's architecturally significant.** Vision, Feature, Feature Design, and Design must stay implementation-independent entirely. Architecture and Feature Technical Design may name a technology only when it's structurally significant (e.g. "Electron Main Process" is fine, "React Hooks" is not) — the dividing line is whether the name describes a boundary/responsibility or an implementation choice.

**Traceability runs one direction.** Everything traces back toward Vision (see [domain-relationships.md](domain-relationships.md)). A downstream document (e.g. Feature Technical Design) may reference an upstream one (Architecture) to justify a decision; an upstream document should never need to reference a downstream one to make sense on its own.

**Say the trade-off, not just the value.** Philosophy in particular exists to name what's given up, not just what's chosen. A "value" with no corresponding cost is usually just marketing language, not a decision-making tool.

### Anti-Patterns Seen Across Multiple Standards

- **Combining multiple features/dependencies in one document** — breaks the atomicity every per-feature standard requires (Feature, Feature Design, Feature Technical, External Context, Prototype).
- **Redefining shared guidance instead of referencing it** — most commonly Feature Design/Feature Technical Design re-explaining Design/Architecture principles instead of linking to them.
- **Implementation leakage** — source code, algorithms, or specific APIs showing up in Vision, Feature, Feature Design, Design, or Architecture, where the standard explicitly prohibits it.
- **Missing the required sections for the standard you're writing against** — check the standard's own `Required Sections`/`Sections` table (linked from [index.md](index.md)) before writing, not after; the audit rules exist precisely to catch this after the fact but it's cheaper to get it right first.

## Usage

Skim this once, then keep the relevant individual standard document open while writing. Run `samgraha audit --domain <domain>` before treating a new document as done — the anti-patterns above are exactly what the deterministic audit rules (`has_section`, `no_implementation`, etc.) are built to catch.

## Related

- [index.md](index.md) — table of contents for the standards collection
- [overview.md](overview.md) — how standards, sections, and audit rules work mechanically
- [domain-relationships.md](domain-relationships.md) — the traceability chain these practices assume
