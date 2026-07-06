# MCP Overview

## Purpose

What the Model Context Protocol (MCP) is and how Samgraha uses it.

## Content

### What is MCP?

The Model Context Protocol (MCP) is an open protocol that enables AI assistants (Claude, OpenCode, Codex CLI) to access tools and knowledge from external services. Samgraha implements an MCP server that exposes compiled documentation as searchable knowledge.

### How Samgraha Uses MCP

The `mcp.exe` binary (built from `crates/mcp`, a separate binary from the `samgraha`/`cli.exe` binary — there is no `samgraha mcp` subcommand) implements a JSON-RPC 2.0 server over stdio and exposes one general-purpose `search` tool plus several others, including:

- `search` — Search across all loaded knowledge stores, with an optional domain filter and retrieval level (metadata/summary/section/full) — one tool, not split per-domain (`search_documents`/`search_sections`/etc. do not exist)
- `get_sections` — Query sections by semantic type across the registry
- `get_document` — Retrieve document metadata and its section table of contents by ID
- `get_document_section` — Retrieve a specific section's content
- `list_domains` — List domains with compiled documents

See [Tools Reference](tools.md) for the full list.

### Benefits

- AI assistants get real-time access to your project's documentation.
- Documents are retrieved by relevance, not by file name.
- Built-in standards and help are merged into results automatically (no config, no opt-in) — alongside project docs, with no ranking between them, just concatenation then domain filter.
- Declared multi-repo dependencies and interests are resolved and merged in automatically for MCP sessions (see [Multi-Repo Guide](../multi-repo-guide/knowledge-context.md)) — this does not apply to the plain CLI `search` command.

## Related

- [Installation](installation.md)
- [Tools Reference](tools.md)
- [Claude Integration](claude-code.md)
- [OpenCode Integration](opencode.md)
