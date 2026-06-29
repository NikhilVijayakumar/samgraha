# Repository Structure

## Purpose

This document defines the repository organization for the Saṃgraha platform.

Repository structure follows the architectural component model. Each architectural component maps to a Cargo crate with a single responsibility. The workspace organization enables independent compilation, testing, and versioning of each component.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Workspace Organization

The platform is organized as a Cargo workspace with the following crates:

```
samgraha/
│
├── Cargo.toml                  # Workspace root
├── samgraha.toml               # Platform configuration (dogfooding)
│
├── crates/
│   ├── common/                 # Shared configuration and utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── schemas/                # Shared domain types
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── standards/              # Documentation Standards
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── services/               # Knowledge Services + Knowledge Runtime
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── runtime/        # Knowledge Runtime (inside services)
│   │
│   ├── compiler/               # Knowledge Compiler
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── registry/               # Knowledge Registry + Repository Registry
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── audit/                  # Audit Framework
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── providers/              # Provider Integrations
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── cli/                    # CLI Adapter
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   └── mcp/                    # MCP Adapter
│       ├── Cargo.toml
│       └── src/
│
├── docs/                       # Documentation
│   └── raw/
│       ├── standards/
│       ├── architecture/
│       ├── feature/
│       ├── feature-technical/
│       └── engineering/
│
└── tests/                      # Integration tests
    ├── Cargo.toml
    └── src/
```

---

## Crate Responsibilities

| Crate | Architectural Component | Responsibility |
|---|---|---|
| common | Shared Utilities | Configuration types, shared utilities, path validation |
| schemas | Shared Schemas | Domain types, serialization, validation shared across all crates |
| standards | Documentation Standards | Standard definitions, contract types, audit rule types |
| services | Knowledge Services + Knowledge Runtime | Service orchestration, workflow execution, KnowledgeRuntime (at `src/runtime/`) |
| compiler | Knowledge Compiler | Document processing, metadata extraction, relationship resolution |
| registry | Knowledge Registry + Repository Registry | Compiled knowledge storage + repository metadata catalog, RegistryClient trait |
| audit | Audit Framework | Audit execution, scoring, reporting, provider interface |
| providers | Provider Integrations | AI provider abstraction, HTTP clients, response parsing |
| cli | CLI Adapter | Argument parsing, output formatting, terminal interaction |
| mcp | MCP Adapter | MCP protocol implementation, transport management |
| tests | — | Integration and end-to-end tests across crate boundaries |

---

## Dependency Graph

Crate dependencies follow architectural layering:

```
common (shared config/utilities — minimal deps)
    ↑
schemas (depends on common)
    ↑
standards (depends on schemas)
    ↑
services (depends on standards, schemas, common)
    ↑
compiler ──────┐
audit ─────────┤
providers ─────┤
               ↓
registry (depends on compiler, audit, schemas)
    ↑
services/runtime (KnowledgeRuntime, inside services, depends on registry)
    ↑
cli ───────────┤
mcp ───────────┤
               ↓
tests (depends on all crates)
```

Note: `KnowledgeRuntime` is implemented at `crates/services/src/runtime/`. There is no standalone `runtime` crate.

Circular dependencies are not permitted. Every dependency must be justified and explicit.

---

## Crate Naming Conventions

- Crate names match directory names
- Workspace members use `samgraha-` prefix in published form (e.g., `samgraha-compiler`)
- Internal dependencies reference crates by path: `compiler = { path = "../compiler" }`

---

## Module Organization (Within Crates)

Each crate follows a consistent internal structure:

```
compiler/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API, re-exports
│   ├── discovery/          # Documentation discovery
│   ├── processing/         # Document processing pipeline
│   ├── extraction/         # Metadata and knowledge extraction
│   ├── resolution/         # Relationship resolution
│   ├── generation/         # Registry artifact generation
│   └── config/             # Compilation configuration
│
├── tests/                  # Integration tests (crate-level)
│
└── benches/                # Criterion benchmarks
```

Public API modules expose the component interface. Internal modules are private to the crate. Cross-crate communication occurs only through public API types.

---

## File Naming Conventions

- Source files: `snake_case.rs`
- Test files: `*_test.rs` or `*_spec.rs`
- Benchmark files: `*_bench.rs`
- Configuration: `samgraha.toml`
- Documentation: `kebab-case.md`

---

## Generated Artifacts

Generated artifacts are excluded from version control:

| Pattern | Contents |
|---|---|---|
| `/target/` | Build artifacts |
| `knowledge.db` | Knowledge Registry |
| `manifest.json` | Repository Manifest (compiler output, consumed by Repository Registry) |
| `*.knowledge‑package` | Knowledge Packages |
| `docs/raw/reports/*/archive/` | Rotated audit reports |

Generated artifacts are disposable. Documentation is the authoritative source.

---

## Traceability

This document derives from:

- Architecture: Component Model
- Engineering Principles
- Technology Selection

Repository Structure provides the foundation for:

- Build Standards
- Dependency Standards
- CI/CD Pipeline

## Build Standards

Repository structure defines the compilation boundaries. Each crate is compiled independently with defined dependencies. See [Build Standards](build-standards.md) for build system integration with workspace structure.

## Testing Standards

Repository structure is validated through workspace-level tests ensuring correct dependency resolution and feature flag isolation. See [Testing Standards](testing-standards.md) for testing expectations per crate.

Traceability:

```
Architecture → Engineering Principles → Repository Structure → Implementation
```
