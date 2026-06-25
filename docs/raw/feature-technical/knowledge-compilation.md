# Knowledge Compilation — Feature Technical Design

## Purpose

This document describes the architectural realization of the Knowledge Compilation feature.

Knowledge Compilation transforms engineering documentation into deterministic, structured knowledge artifacts. It is the foundation of the Saṃgraha platform — every downstream capability depends on compiled knowledge.

This document applies the architectural principles defined in Component Model, Knowledge Flow, Persistence Architecture, and Runtime Boundary. It describes how the compilation pipeline participates in the overall knowledge lifecycle.

---

## Feature Specification

- **Feature:** docs/raw/feature/knowledge-compilation.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/knowledge-flow.md, docs/raw/architecture/persistence.md

---

## Participating Components

### Knowledge Compiler

The Knowledge Compiler is the primary component realizing this feature. It owns the compilation pipeline from documentation discovery through knowledge registry generation.

### Documentation Standards

Documentation Standards define the contracts that the compiler applies during processing. The compiler reads standards to determine document structure, expected sections, and validation rules.

### Knowledge Services

Knowledge Services coordinate compilation execution. They receive compilation requests, validate inputs against Documentation Standards, and invoke the compiler.

### Knowledge Registry

The Knowledge Registry is the persistent output target. The compiler writes compiled knowledge, metadata, indexes, and dependency information into the registry.

### Knowledge Enrichment

Knowledge Enrichment optionally reads compiled knowledge after compilation completes. It is not part of the compilation pipeline but depends on compilation output.

### Incremental Build

Incremental Build coordinates with the compiler to determine which artifacts require regeneration. The compiler provides dependency information; Incremental Build determines rebuild scope.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Knowledge Compiler | Discover documentation, validate sources, extract knowledge, resolve relationships, generate registry artifacts |
| Documentation Standards | Define compilation contracts (structure, validation, metadata rules) |
| Knowledge Services | Receive and validate compilation requests, invoke compiler, report results |
| Knowledge Registry | Accept and store compiled knowledge, maintain integrity metadata |
| Incremental Build | Track artifact dependencies, determine rebuild scope, invalidate stale artifacts |

---

## Component Interactions

```text
Repository Configuration
        │
        ▼
Knowledge Services
        │
        ▼
Knowledge Compiler
        │
        ├── Documentation Standards (read contracts)
        ├── Repository Documentation (read sources)
        │
        ▼
Knowledge Registry
        │
        ▼
Knowledge Services (report completion)
```

### Compilation Request Flow

1. Knowledge Services receive a compilation request from CLI, Incremental Build, or programmatic API.
2. Services validate the request against Workspace and Repository Configuration.
3. Services invoke the Knowledge Compiler with the validated scope.
4. Compiler reads Documentation Standards to determine processing rules.
5. Compiler discovers documentation sources within the configured scope.
6. Compiler processes each source document: validates structure, extracts metadata, extracts knowledge content, resolves relationships.
7. Compiler writes compiled artifacts to the Knowledge Registry.
8. Compiler returns compilation metadata (documents processed, errors, warnings).
9. Knowledge Services report results to the caller.

---

## Runtime Behavior

### Compilation Lifecycle

```
Initialize Compilation
        │
        ▼
Load Configuration
        │
        ▼
Discover Documentation
        │
        ▼
Apply Documentation Standards
        │
        ▼
Process Documents
        │
        ├── Validate Structure
        ├── Extract Metadata
        ├── Extract Knowledge
        └── Resolve Relationships
        │
        ▼
Generate Registry Artifacts
        │
        ▼
Report Completion
```

### Concurrency

Document processing may proceed in parallel for independent documents. Documents with cross-references require coordinated processing. The compiler ensures that relationship resolution occurs after all documents are processed.

### Determinism

Compilation is deterministic. Identical input documentation produces identical output artifacts. No runtime state, network access, or random values influence compilation output.

---

## Communication Paths

### Compiler → Documentation Standards

The compiler reads standards to obtain document structure rules, metadata schemas, and validation criteria. Standards are read-only during compilation.

### Compiler → Repository Documentation

The compiler reads source documentation from the repository filesystem. Documentation is never modified.

### Compiler → Knowledge Registry

The compiler writes compiled knowledge artifacts to the registry. The registry is the sole persistent output destination.

### Compiler → Knowledge Services

The compiler returns compilation metadata including processed document count, errors, warnings, and artifact identifiers.

---

## Data Ownership

| Data | Owner | Compiler Access |
|---|---|---|
| Repository Documentation | Repository | Read |
| Documentation Standards | Standards | Read |
| Compiled Knowledge | Knowledge Registry | Write |
| Compilation Metadata | Knowledge Services | Write |
| Build Configuration | Repository | Read |

---

## Integration Points

### CLI Interface

The CLI exposes compilation through `samgraha compile [path]`. This maps to Knowledge Services invoke compilation.

### Incremental Build

Incremental Build invokes the compiler with a scope limited to invalidated artifacts. The compiler processes only the specified documents and writes only affected registry entries.

### Knowledge Registry

The registry provides the persistence boundary for all compilation output. The compiler writes through a defined registry interface.

### Documentation Standards

Standards are loaded at compilation start. The compiler caches standard definitions for the duration of compilation.

---

## External Dependency Integration

The compiler operates entirely offline. No external services, AI providers, or network resources participate in compilation.

Optional integration points include:
- Repository configuration (samgraha.toml) — defines compilation scope and behavior
- Filesystem — source documentation and configuration storage

---

## Runtime Constraints

- Compilation must complete without network access.
- Compilation must not modify source documentation.
- Compilation must not depend on previous compilation state.
- Compilation must produce identical output from identical input.
- Compilation must report all errors without aborting on the first failure.
- Compilation must handle large repositories (10,000+ documents) without exhaustion.

---

## Architectural Constraints

- The compiler must not bypass Documentation Standards.
- The compiler must not write outside the Knowledge Registry boundary.
- The compiler must not assume a specific documentation format.
- The compiler must preserve repository isolation.
- The compiler must produce artifacts that are independently verifiable.

---

## Security Considerations

- Source documentation is read-only; the compiler never modifies source files.
- Path traversal attacks are prevented through input validation.
- Output is restricted to the configured Knowledge Registry location.
- Configuration is loaded from trusted repository locations.
- The compiler does not execute embedded content in documentation.

---

## Performance Considerations

- Document processing should scale linearly with document count.
- Independent documents may be processed in parallel.
- Metadata extraction should require a single document pass.
- Relationship resolution requires a second pass after all documents are processed.
- Large document hierarchies should not cause disproportionate resource consumption.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Invalid documentation structure | Report error, skip document, continue compilation |
| Missing Documentation Standards | Report error, abort compilation |
| Registry write failure | Report error, abort compilation, preserve prior registry |
| Configuration error | Report error, abort compilation |
| Resource exhaustion | Report error, abort compilation |
| Unsupported document type | Report warning, skip document, continue |

The compiler guarantees that a failed compilation never corrupts an existing valid registry.

---

## Extension Points

### Document Processors

The compiler supports additional documentation formats through processor extensions. New processors integrate without modifying the compilation pipeline.

### Metadata Extractors

Custom metadata extraction logic may be registered for specific documentation domains.

### Relationship Resolvers

Custom relationship resolution strategies may extend the default reference resolution.

---

## Traceability

This document derives from:

- Feature: Knowledge Compilation
- Architecture: Component Model
- Architecture: Knowledge Flow
- Architecture: Persistence Architecture
- Architecture: Runtime Boundary
- Architecture: Communication Architecture

This document provides technical context for:

- Engineering Compilation Strategy
- Incremental Build Technical Design
- Knowledge Registry Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
