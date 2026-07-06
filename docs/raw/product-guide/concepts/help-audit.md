# Product Guide Audit

## Purpose

Verifies the Help/Product Guide corpus is complete, navigable, well-written, and an accurate representation of the product — not just that each topic has a title and body.

## Content

**Product Guide Audit** (`samgraha audit --pipeline help`) is a pipeline distinct from the [Help Standard](../../standards/help.md)'s per-topic section checks. The standard asks "does this one topic have a title and body?"; this pipeline asks "does the whole corpus reflect reality?" across four categories:

| Category | Weight | Asks |
|----------|--------|------|
| Coverage (PC1-PC7) | 30% | Is every CLI command, MCP method, config field, domain standard, feature, and architecture decision documented? |
| Navigation (PN1-PN4) | 20% | Can users find what they need — TOC, links, heading structure? |
| Quality (PQ1-PQ5) | 25% | Placeholder text, empty sections, duplicated content, short pages, stale version references? |
| Accuracy (PA1-PA7) | 25% | Does the guide match the actual code surface, Vision's stated goals, and Engineering's public-facing decisions? |

### Coverage checks derive from actual code

PC1/PC2/PC3 (and their Accuracy counterparts PA1-PA3) don't hand-maintain a list of commands/methods/config fields to check against — they read `schemas::code_inventory`, a build-time macro that parses `crates/cli/src/commands.rs`'s `Commands` enum, `crates/mcp/src/adapter.rs`'s dispatch match, and `crates/common/src/config.rs`'s `SamgrahaConfig` struct directly. A new CLI command with no doc in `commands/` is a real, deterministic finding, not a guess.

### Repository Metadata (PA7)

PA7 compares the compiled `repository_metadata` table (written during `compile`) against the live `samgraha.toml` — catching a config edit that hasn't been recompiled yet. Before the first `compile`, this table is empty and PA7 reports that state as an informational note rather than a failure.

## Related

- [Standards Reference: Help](../../standards/help.md)
- [Help Domain Guide](../documentation-guide/help.md)
- [Audit Concept](audit.md)
- [Build Audit](build-audit.md)
