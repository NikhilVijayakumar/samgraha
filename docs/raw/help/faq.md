# Frequently Asked Questions

## Purpose

Common questions about Samgraha and their answers.

## Content

### General

**Q: What is Samgraha?**

A: A knowledge compilation and audit tool for engineering documentation. It compiles structured markdown into a searchable SQLite database, validates it against defined standards, and serves it via CLI and MCP for AI-assisted development.

**Q: How is Samgraha different from a wiki or Confluence?**

A: Samgraha is designed for AI consumption. Documentation is compiled into structured data (sections with semantic types), not just rendered pages. This enables AI assistants to query specific sections by type, not just full-text search.

### Setup

**Q: Do I need a database server?**

A: No. Samgraha uses SQLite locally. No server to install, configure, or maintain.

**Q: Can I use Samgraha without MCP?**

A: Yes. The CLI (`samgraha search`, `samgraha sections`, `samgraha audit`) works independently. MCP is optional.

### Standards

**Q: Can I define my own standards?**

A: Yes. Custom standards can be defined by extending built-in standards. See the [Custom Standard tutorial](tutorials/custom-standard.md).

**Q: What if my document doesn't match any standard?**

A: It will be compiled with a `generic` document body type. Sections are still stored and searchable.

### Multi-Repo

**Q: Can Samgraha handle microservice architectures?**

A: Yes. Each service is a Samgraha repository. Workspaces and dependency resolution connect them.

**Q: Is the registry distributed?**

A: The registry is local. Remote repository manifests are fetched and cached. There is no global server.

### Performance

**Q: How many documents can Samgraha handle?**

A: Documents are stored in SQLite, but search is currently a Rust-side term-matching scan over the documents loaded from the database — not a SQLite FTS5 index. It scores title/body substring matches per query. This works well for typical project/documentation-sized corpora (tens to low thousands of documents); it isn't built for millions of documents.

**Q: How long does compilation take?**

A: Incremental compilation (after the first full compile) processes only changed files. Typical incremental compiles take under a second.

## Related

- [Getting Started: Installation](getting-started/installation.md)
- [Concepts: Repository](concepts/repository.md)
- [Troubleshooting Index](troubleshooting/compile-failed.md)
