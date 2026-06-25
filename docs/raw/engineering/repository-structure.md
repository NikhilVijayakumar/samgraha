# Repository Structure

## Purpose

This document defines the repository organization for the Saṃgraha platform.

Repository structure follows the architectural component model. Each architectural component maps to a Cargo crate with a single responsibility. The workspace organization enables independent compilation, testing, and versioning of each component.

---

## Workspace Organization

The platform is organized as a Cargo workspace with the following crates:

```
samgraha/
│
├── Cargo.toml                  # Workspace root
├── samgraha.toml               # Platform configuration (dogfooding)
│
├── schemas/                    # Shared domain types
│   ├── Cargo.toml
│   └── src/
│
├── standards/                  # Documentation Standards
│   ├── Cargo.toml
│   └── src/
│
├── services/                   # Knowledge Services
│   ├── Cargo.toml
│   └── src/
│
├── compiler/                   # Knowledge Compiler
│   ├── Cargo.toml
│   └── src/
│
├── registry/                   # Knowledge Registry
│   ├── Cargo.toml
│   └── src/
│
├── runtime/                    # Knowledge Runtime
│   ├── Cargo.toml
│   └── src/
│
├── audit/                      # Audit Framework
│   ├── Cargo.toml
│   └── src/
│
├── providers/                  # Provider Integrations
│   ├── Cargo.toml
│   └── src/
│
├── cli/                        # CLI Adapter
│   ├── Cargo.toml
│   └── src/
│
├── mcp/                        # MCP Adapter
│   ├── Cargo.toml
│   └── src/
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
| schemas | Shared Schemas | Domain types, serialization, validation shared across all crates |
| standards | Documentation Standards | Standard definitions, contract types, audit rule types |
| services | Knowledge Services | Service orchestration, workflow execution, service registry |
| compiler | Knowledge Compiler | Document processing, metadata extraction, relationship resolution |
| registry | Knowledge Registry | Storage, retrieval, indexing, integrity management |
| runtime | Knowledge Runtime | Request routing, service coordination, policy enforcement |
| audit | Audit Framework | Audit execution, scoring, reporting, provider interface |
| providers | Provider Integrations | AI provider abstraction, HTTP clients, response parsing |
| cli | CLI Adapter | Argument parsing, output formatting, terminal interaction |
| mcp | MCP Adapter | MCP protocol implementation, transport management |
| tests | — | Integration and end-to-end tests across crate boundaries |

---

## Dependency Graph

Crate dependencies follow architectural layering:

```
schemas (foundation — zero dependencies within workspace)
    ↑
standards (depends on schemas)
    ↑
services (depends on standards, schemas)
    ↑
compiler ──────┐
audit ─────────┤
providers ─────┤
               ↓
registry (depends on compiler, audit, schemas)
    ↑
runtime (depends on registry, services, schemas)
    ↑
cli ───────────┤
mcp ───────────┤
               ↓
tests (depends on all crates)
```

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
|---|---|
| `/target/` | Build artifacts |
| `knowledge.db` | Knowledge Registry |
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

Traceability:

```
Architecture → Engineering Principles → Repository Structure → Implementation
```
