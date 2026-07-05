# Frequently Asked Questions

## Purpose

Common questions about Samgraha and their answers.

## Content

### General

**Q: What is Samgraha?**

A: A knowledge compilation and audit tool for engineering documentation. It compiles structured markdown into a searchable SQLite database, validates it against defined standards, verifies artifacts against contracts, and serves knowledge via CLI and MCP for AI-assisted development.

**Q: What is an audit?**

A: The process of verifying artifacts against declared contracts. Samgraha supports multiple audit types: Documentation Audit (docs against standards), Implementation Audit (docs vs code), Build Audit (build docs vs config vs artifacts), Security Audit (security docs vs config vs code vs runtime), Consistency Audit (layer alignment), Coverage Audit (bidirectional doc↔code coverage + orphan detection), and Dependency Governance (dependency justification and health).

**Q: What is an orphan finding?**

A: An orphan is code that exists without corresponding documentation. Coverage Audit detects all orphans. Orphans are always Warning severity, never Error. Resolution: document the orphan, remove the orphan, or suppress the finding.

**Q: How is Build Audit different from Documentation Audit?**

A: Documentation Audit checks that build documentation is well-formed and complete (B1-B12). Build Audit additionally verifies that the documented build strategy is faithfully realized in configuration files and produced artifacts (BC1-BC10), optionally inspecting the binary via `--inspect-artifact`.

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
