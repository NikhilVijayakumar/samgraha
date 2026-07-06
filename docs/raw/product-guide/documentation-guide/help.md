# Help Domain Guide

## Purpose

How to write a Help topic — end-user product documentation shipped in `help.db`.

## Content

### Purpose of a Help Topic

Help topics answer "how do I use this?" and "what does this mean?" for a human or an AI agent looking up Samgraha's own behavior. One file per concept, command, or guide — never a catch-all page.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| Title | Yes | Topic title |
| Content | Yes | Main body |
| Purpose | No | What this topic covers |
| Product Context | No | How this capability fits the product |
| Public Contract | No | Public-facing behavior (flags, methods, config keys) |
| Related | No | Cross-references to other topics |

### Writing Tips

- Document public behavior only — implementation details belong in Architecture/Engineering, not here.
- Every CLI command, MCP method, and config section should have a corresponding topic (the Product Guide Audit Pipeline's Coverage checks verify this against the actual code surface).
- Use the `full` packaging profile for topics that need Product Context/Public Contract; `quickref` (title + body) is enough for a short FAQ-style entry.

## Related

- [Standards Reference: Help](../../standards/help.md)
- [Product Guide Audit Pipeline](../concepts/help-audit.md)
- [index.md](../index.md)
