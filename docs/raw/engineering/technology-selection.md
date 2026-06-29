# Technology Selection

## Purpose

This document explains the rationale behind technology choices for the Saṃgraha platform.

Technology selection is driven by engineering principles — determinism, offline-first, minimal dependencies, and repository isolation. Every choice is justified by clear engineering value.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Programming Language: Rust

### Rationale

Rust was selected as the implementation language for the following reasons:

**Memory safety without garbage collection.** The platform handles large documentation repositories and knowledge registries. Predictable memory behavior and absence of garbage collection pauses are essential for deterministic compilation and runtime performance.

**Zero-cost abstractions.** The platform compiles into a single self-contained executable. Rust's zero-cost abstractions enable high-level component organization without runtime overhead or binary bloat.

**Strong type system.** Domain modeling of documentation standards, audit rules, knowledge artifacts, and runtime policies benefits from expressive type systems. Rust's enum and trait system maps naturally to the architectural component model.

**Ecosystem maturity.** The Rust ecosystem provides high-quality crates for CLI development (clap), serialization (serde), content hashing, filesystem operations, and concurrent processing.

**Cross-platform compilation.** The platform targets Windows, macOS, and Linux. Rust's cross-compilation support and consistent standard library behavior across platforms minimize platform-specific code.

**Offline-first alignment.** Rust's compilation model produces statically linked binaries with no runtime dependencies. This aligns with the offline-first principle — the platform runs without interpreters, virtual machines, or package managers.

### Alternatives Considered

- **Go**: Simple concurrency model and fast compilation. Rejected because garbage collection introduces unpredictable pause behavior for large knowledge registry operations, and the type system is less expressive for domain modeling.
- **Python**: Rich ecosystem and rapid development. Rejected because runtime dependency management conflicts with offline-first and single-executable requirements.
- **C++**: Maximum performance. Rejected because memory safety guarantees require external tooling and the build ecosystem is less approachable for contributors.

---

## CLI Framework: clap

### Rationale

clap is the standard CLI argument parsing library for Rust. It provides:

- Derive-based command definition
- Automatic help text generation
- Subcommand support matching the command hierarchy
- Shell completion generation
- Active maintenance and wide adoption

No alternative was seriously considered — clap is the de facto standard for Rust CLI applications.

---

## Serialization: serde + serde_json + toml

### Rationale

- **serde** is the standard Rust serialization framework. It provides derive-based serialization for all data types with zero-cost abstraction.
- **serde_json** is used for structured output (CLI --json flag, runtime responses, audit reports). JSON is the interoperable standard for programmatic consumption.
- **toml** is used for configuration files (samgraha.toml). TOML is human-readable, supports comments, and is standard in the Rust ecosystem.

### Alternatives Considered

- **YAML**: Rejected because of specification complexity, security concerns with untrusted input, and ambiguous parsing rules.
- **RON**: Rejected because it is Rust-specific and not widely understood by tooling outside the Rust ecosystem.
- **MessagePack / BSON**: Rejected because binary formats reduce debuggability and are unnecessary for local-first operation.

---

## Content Hashing: sha2

### Rationale

SHA-256 provides deterministic content hashing for change detection and integrity verification. It is:

- Well-established with no known practical collision attacks relevant to documentation content
- Implemented in pure Rust (sha2 crate) with no external dependencies
- Fast enough for hashing large documentation collections
- Standard — can be verified by external tooling if needed

---

## MCP Protocol: rust-mcp-sdk

### Rationale

The Model Context Protocol (MCP) is the primary runtime interface for AI engineering assistant integration. rust-mcp-sdk provides:

- A Rust-native MCP server implementation
- Protocol adherence to the MCP specification
- Active maintenance aligned with the MCP standard evolution

MCP was selected as the primary AI integration protocol because it is an open standard designed specifically for AI coding tool integration. The SDK abstraction allows future protocol adapters without changing the Knowledge Runtime architecture.

---

## Concurrency: std::thread + rayon

### Rationale

- **std::thread** for coarse-grained parallelism across independent compilation units and concurrent runtime request handling
- **rayon** for data-parallel operations within compilation (document processing, metadata extraction, index building)

rayon provides work-stealing thread pools that adapt to available hardware without over-subscription. It is the standard choice for CPU-bound parallel processing in Rust.

Asynchronous runtimes (tokio, async-std) are intentionally avoided. The platform is CPU-bound rather than I/O-bound for compilation and runtime operations. Async adds complexity without benefit for the primary workload.

---

## Error Handling: anyhow + thiserror

### Rationale

- **anyhow** for application-level error handling in CLI and service orchestration code — context-rich error reporting with backtrace support
- **thiserror** for library-level error types in component implementations — typed errors with From implementations

This is the standard Rust error handling pattern. anyhow provides ergonomic error propagation with context. thiserror provides type-safe error definitions at component boundaries.

---

## Logging: tracing

### Rationale

tracing provides structured, async-aware diagnostic logging with:

- Hierarchical spans for operation tracing (compilation of a document, execution of an audit check)
- Multiple output formats (human-readable, JSON)
- Configurable verbosity at module level
- Integration with standard Rust logging ecosystem

---

## Testing: built-in + proptest + rstest

### Rationale

- Rust built-in test framework for unit tests and integration tests
- **proptest** for property-based testing of document parsing, metadata extraction, and serialization round-trips
- **rstest** for fixture-based integration tests with parameterized test cases

---

## Storage: SQLite via rusqlite

### Rationale

SQLite is the persistence engine for the Knowledge Registry. Rationale:

- Embedded — no separate database process, aligning with offline-first and single-executable requirements
- Reliable — battle-tested, ACID-compliant, deterministic behavior
- Concurrent — WAL mode supports concurrent reads during writes
- Portable — database file is a single file, trivially movable across environments
- Zero configuration — no server setup, no connection management
- Adequate performance — documentation volumes (10K–100K documents) are well within SQLite's range

rusqlite provides idiomatic Rust bindings with compile-time SQL validation via the rustqlite crate.

### Alternatives Considered

- **sled**: Embedded database written in Rust. Rejected because it is less battle-tested than SQLite and the API is less standard.
- **redb**: Embedded Rust database. Rejected for similar reasons — SQLite's ecosystem and reliability are unmatched for this use case.
- **File-based storage (JSON, custom format)**: Rejected because query performance degrades with document count and ACID guarantees require custom implementation.

---

## AI Provider Integration: Custom Provider Abstraction

### Rationale

Rather than depending on a specific AI SDK, the platform defines a minimal provider abstraction:

```rust
trait EnrichmentProvider {
    fn summarize(&self, document: &str) -> Result<Summary>;
    fn keywords(&self, document: &str) -> Result<Vec<Keyword>>;
    fn embed(&self, document: &str) -> Result<Embedding>;
    fn glossary(&self, documents: &[&str]) -> Result<Glossary>;
}
```

This abstraction supports multiple backends (LM Studio, Ollama, llama.cpp, OpenAI-compatible) through uniform configuration. Providers are interchangeable at runtime through configuration changes.

---

## Traceability

This document derives from:

- Architecture: System Overview
- Architecture: Component Model
- Engineering Principles

Technology Selection provides context for:

- Repository Structure
- Build Standards
- Dependency Standards
- Persistence Standards

## Build Standards

Technology selection constrains build toolchain requirements. Rust edition 2021 with specific feature flags drives build configuration. See [Build Standards](build-standards.md) for toolchain version requirements and build profile definitions.

## Testing Standards

Technology choices are validated through cross-platform test suites ensuring compatibility across supported toolchains. See [Testing Standards](testing-standards.md) for platform coverage expectations.

Traceability:

```
Architecture → Engineering Principles → Technology Selection → Implementation
```
