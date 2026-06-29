# Testing Standards

## Purpose

This document defines the testing standards for the Saṃgraha platform.

Testing verifies that implementation matches documented behavior. Tests are organized in layers corresponding to the architectural hierarchy. Every test has a clear scope and verifiable assertions.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Testing Layers

### Unit Tests

Location: `src/*_test.rs` or inline `#[cfg(test)] mod tests { }`

Scope: Single function, module, or type. No external dependencies. No filesystem access. No network access.

Characteristics:
- Test pure functions with deterministic inputs
- Test error cases and edge cases alongside happy paths
- Test serialization round-trips (serde)
- Test validation rules
- Use proptest for property-based testing where applicable

Coverage target: 80%+ of non-trivial functions.

### Integration Tests (Crate-level)

Location: `tests/` directory within each crate, and workspace `tests/` for cross-crate tests.

Scope: A single crate's public API with its dependencies available. Filesystem access permitted in temporary directories. Network access not permitted.

Characteristics:
- Test public API behavior end-to-end within the crate
- Test configuration parsing and validation
- Test compilation pipeline stages
- Test registry operations (create, read, update, delete)
- Test audit execution flows
- Use rstest for fixture-based parameterized tests

### End-to-End Tests

Location: `tests/` in workspace root.

Scope: Multiple crates working together through CLI commands or MCP protocol messages.

Characteristics:
- Test complete workflows
- Test CLI command execution with various arguments
- Test MCP protocol message sequences
- Test error propagation across component boundaries
- Test configuration-driven behavior

---

## Test Patterns

### Deterministic Tests

All tests must be deterministic. Randomness is seeded with a fixed seed:

```rust
// Property-based tests use a fixed seed for reproducibility
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    fn test_document_parsing_roundtrip(doc in document_strategy()) {
        let parsed = parse_document(&doc).unwrap();
        let serialized = serialize_document(&parsed);
        assert_eq!(doc, serialized);
    }
}
```

### Test Naming

- Test functions: `fn test_<module>_<behavior>()`
- Test modules: `mod tests { }`
- Integration test files: `*_test.rs` or `*_spec.rs`

### Assertions

- Use `assert_eq!`, `assert_ne!`, `assert!` for basic assertions.
- Use `Result<()>` return type for tests that can fail: `fn test_foo() -> Result<()> { ... Ok(()) }`
- Use `#[should_panic(expected = "...")]` only when testing panic behavior — prefer Result-based error testing.
- Use `anyhow::Result` for integration tests where error context matters.

### Fixtures

Test fixtures are stored in:
- Within crate: `tests/fixtures/`
- Workspace level: `tests/fixtures/`
- Feature-specific: `tests/fixtures/feature/<name>/`
- Standard-specific: `tests/fixtures/standards/<name>/`

Fixtures must be small. Large fixtures are generated programmatically.

### Golden Files

Golden file tests verify output against expected content:

1. Run the operation
2. Compare output with `tests/fixtures/golden/<test_name>.golden`
3. Update golden files when behavior changes deliberately

Golden files are committed to version control. CI verifies golden file matches.

---

## Test Configuration

Tests use test-specific configuration:

```rust
fn test_config() -> Config {
    Config {
        repository: tempdir().path().join("samgraha.toml"),
        registry: tempdir().path().join("knowledge.db"),
        features: Features::default(),
        providers: Providers::default(),
    }
}
```

Temporary directories are cleaned up automatically when the test completes. Test configuration must never read from the user's actual configuration directory.

---

## Benchmarking

Benchmarks use criterion:

Location: `benches/` directory within each crate.

Characteristics:
- Benchmark critical compilation paths
- Benchmark registry queries
- Benchmark serialization of large documents
- Report throughput and latency distributions

Benchmarks are not run in CI by default. They are run on demand and before performance-sensitive changes.

---

## Test Doubles

- **Mocks**: Used for AI provider tests. Mock providers return predefined responses.
- **Stubs**: Used for provider abstraction tests when the test only needs to verify the abstraction layer.
- **Fakes**: Used for registry tests — an in-memory registry implementation for fast unit tests.

Mock objects are defined in test modules, not in production code. Mocking libraries are not used — manual mock implementations are preferred for clarity and control.

---

## Test Coverage Measurement

- Used as a signal, not a gate
- Coverage reports generated with `cargo-llvm-cov`
- Reports stored in `docs/raw/reports/test-coverage/latest/`
- Coverage trends tracked over time
- Focus on meaningful coverage of documented behavior

---

## Traceability

This document derives from:

- Engineering Principles (deterministic by default)
- Repository Structure
- Build Standards

Testing Standards provide the framework for:

- CI/CD Pipeline
- Quality Assurance

## Build Standards

Tests are compiled and executed as part of the standard build pipeline. Build profiles support test-specific configurations. See [Build Standards](build-standards.md) for test runner integration with the build system.

Traceability:

```
Engineering Principles → Testing Standards → Implementation
```
