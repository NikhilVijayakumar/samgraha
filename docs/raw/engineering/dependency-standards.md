# Dependency Standards

This section details the Dependency Standards.

## Purpose

This document defines the dependency management standards for the Saṃgraha platform.

Dependencies are carefully managed to align with the minimal dependencies principle. Every dependency carries maintenance cost, security risk, and compatibility burden. Selection prioritizes well-maintained, widely-used crates with compatible licenses.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Dependency Philosophy

- Dependencies must be justified by clear engineering value.
- Implement rather than import when the implementation is straightforward and well-understood.
- Prefer standard library and ecosystem-standard crates over niche alternatives.
- Every dependency is audited for maintenance status, safety guarantees, and license compatibility.
- Dependency versions are pinned in Cargo.lock for deterministic builds.
- Transitive dependencies are reviewed — a dependency's dependencies are also dependencies.

---

## Allowed Dependency Categories

This section details the Allowed Dependency Categories.

### Core Infrastructure

Crates that provide essential platform infrastructure with no reasonable alternative:

- serde, serde_json, toml — serialization
- clap — CLI argument parsing
- anyhow, thiserror — error handling
- tracing — logging and diagnostics
- sha2 — content hashing

### Storage

- rusqlite — SQLite bindings for Knowledge Registry persistence

### Concurrency

- rayon — data-parallel processing for compilation

### Testing

- proptest — property-based testing
- rstest — fixture-based testing
- criterion — benchmarking

### Protocol

- rust-mcp-sdk — MCP protocol implementation for AI tool integration

---

## Prohibited Dependency Patterns

This section details the Prohibited Dependency Patterns.

### Async Runtimes

tokio, async-std, smol, and other async runtimes are not used. The platform is CPU-bound rather than I/O-bound. Async adds complexity, binary size, and cognitive overhead without benefit for the primary workload. CLI commands execute synchronously. MCP and future REST adapters use simple threading where needed.

Rationale re-evaluation: If benchmarks demonstrate that async I/O provides measurable benefit for a specific use case (e.g., concurrent AI provider requests during enrichment), async may be introduced as an isolated concern rather than a platform-wide pattern.

### HTTP Clients

Direct HTTP dependencies are isolated within the providers crate. The providers crate abstracts HTTP communication behind the provider trait. Other crates must not depend on HTTP client libraries.

### File Watching (Production)

File system watching (notify crate) is limited to the incremental build watch mode. The watcher is a compile-time optional feature — the core compilation pipeline does not depend on file system notification.

---

## Dependency Management Process

This section details the Dependency Management Process.

### Adding a Dependency

1. Verify the dependency is necessary — can the functionality be implemented with reasonable effort using existing dependencies?
2. Verify the dependency is well-maintained — check GitHub stars, last commit date, open issues, release frequency.
3. Verify license compatibility — must be MIT, Apache 2.0, or dual-licensed. GPL and AGPL are not permitted.
4. Verify no known security advisories — check rustsec.org.
5. Add the dependency and run `cargo audit` to verify no new advisories.
6. Document the rationale in the commit message.

### Updating a Dependency

1. Review the changelog for breaking changes.
2. Update the version in Cargo.toml.
3. Run the full test suite.
4. Run `cargo audit`.
5. Commit with the dependency update scope in the message.

### Auditing

- `cargo audit` runs in CI on every commit.
- `cargo deny` verifies license compatibility.
- Dependencies are reviewed quarterly for outdated or unmaintained crates.
- Minimum Supported Rust Version (MSRV) is tracked and updated deliberately.

---

## Workspace Dependency Management

Shared dependencies are declared at the workspace level in the root Cargo.toml:

```toml
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
clap = { version = "4", features = ["derive"] }
```

Individual crates reference workspace dependencies without specifying versions:

```toml
[dependencies]
serde.workspace = true
anyhow.workspace = true
```

This ensures all crates use consistent dependency versions.

---

## Traceability

This document derives from:

- Engineering Principles (minimal dependencies)
- Technology Selection
- Repository Structure

Dependency Standards provide the framework for:

- Build Standards
- Security Standards

## Build Standards

Dependency declarations are validated during build. Unused or duplicate dependencies are rejected by the build pipeline. See [Build Standards](build-standards.md) for dependency audit configuration.

Traceability:

```
Engineering Principles → Dependency Standards → Implementation
```
