# Saṃgraha

> **Compile engineering documentation into AI-ready knowledge packages for coding agents.**

---

## Overview

Saṃgraha is a **Knowledge Engineering Platform** that transforms engineering
documentation into structured, verified, deterministic knowledge packages for
AI coding agents.

Every engineering repository should become **self-describing** — a repository
that communicates why it exists, what it does, how it's designed, and why
decisions were made without requiring an AI to reverse-engineer source code.

Documentation is governed by **standards** — contracts that define what each
domain must contain. **Audits** verify compliance before knowledge is compiled.
**MCP** serves only audit-passing documentation to AI agents.

Documentation is the authoritative engineering specification. Everything else
is generated.

---

# Philosophy

### Documentation First

Documentation defines the system. Implementation realizes it.

### Standards Before Tools

Standards define quality. Tools enforce quality.

### AI-Readable by Default

Documentation should be understandable by both humans and AI systems. Clarity
benefits both.

### Audit-Gated Knowledge

Only verified documentation enters the knowledge pipeline. Two-tier audit:
deterministic checks (always run, offline) and semantic checks (optional
AI-assisted). Audit failures are engineering quality feedback.

### Deterministic Compilation

The same documentation always produces the same knowledge package. No hidden
state. No manual synchronization.

### Knowledge Before Retrieval

Knowledge should be organized before it is searched. Retrieval quality depends
on knowledge quality.

### Prototype Before Production

Systems should be validated before they are implemented.

### Offline First

Compilation and knowledge delivery should not require cloud services or AI
models. AI is optional enrichment, never a runtime dependency.

### Technology Independent

Documentation methodology remains independent of programming languages,
frameworks, databases, infrastructure, and AI providers.

---

# Goals

* Documentation-first engineering
* Repository-aware knowledge retrieval
* Offline-first operation
* Deterministic compilation
* Local-first execution
* Zero mandatory AI dependencies
* AI-assisted implementation
* Build artifacts instead of persistent state
* Technology agnostic documentation

---

# Documentation Domains

Engineering knowledge is separated into ten explicit documentation domains,
each governed by a Documentation Standard:

Repository-wide knowledge:
- **Vision** — product purpose, direction, and success criteria
- **Design** — system design decisions and rationale
- **Architecture** — system organization and communication paths
- **Engineering** — engineering principles, build system, security, invariants
- **External Context** — external systems, dependencies, and their ownership

Feature-specific knowledge:
- **Feature** — what a feature does and its behavior
- **Feature Design** — how a feature is designed
- **Feature Technical Design** — how a feature is implemented

Validation:
- **Prototype** — validation approach and results

Repository navigation:
- **README** — repository structure and getting started

Every domain has one responsibility. Together they form a complete engineering
specification.

---

# Quality-Gated Knowledge

Saṃgraha's documentation pipeline has three layers that guarantee knowledge
quality before it reaches AI agents:

```text
Standards (Contracts)
    ↓
Audit (Verification)
    ↓
MCP (Delivery)
```

**Standards** define exactly what each documentation domain must contain.
Each standard is an independent contract specifying Purpose, Responsibilities,
Scope, Inputs, Outputs, Relationships, Validation Rules, Audit Rules,
Generation Rules, Enhancement Rules, and Success Criteria. Architecture docs
describe organization, not code. Feature docs describe capabilities, not
implementation.

**Audits** verify compliance against those contracts before documentation
enters the knowledge pipeline. Two tiers:

- **Deterministic checks** — metadata completeness, link validity, file
  existence, one-to-one mapping. Always run, offline, zero AI.
- **Semantic checks** — technology independence, scope appropriateness,
  implementation leakage. Optional AI-assisted enrichment.

Each audit check traces directly to a specific Audit Rule in the corresponding
standard. Every standard has a matching audit; every Audit Rule is covered.

Audit results are deterministic metadata carried into the knowledge package
(audit_pass, failed_rules, last_audit). Compilation respects audit status —
domains that fail audit are excluded from the package.

**MCP** serves only audit-passing documentation to AI agents. Audit status is
exposed in search result metadata so agents understand the verification level
of every retrieved document. The result: LLMs like Claude Code and OpenCode
receive consistent, complete, traceable, verified knowledge — not ad-hoc docs
with unpredictable quality.

Without standards and audits, MCP serves whatever documentation exists,
with whatever quality it happens to have. With them, MCP serves only
contract-compliant knowledge — verified, structured, and reliable.

---

# Core Concepts

## Documentation

Documentation is the authoritative engineering specification.

Documentation is organized into ten explicit domains — see
[Documentation Domains](#documentation-domains) above.

Documentation is never generated.

Documentation is written by engineers.

---

## Knowledge Compiler

The compiler converts documentation into structured knowledge.

Responsibilities include:

* Markdown parsing
* AST processing
* Metadata extraction
* Chunk generation
* Cross-reference resolution
* Dependency resolution
* SQLite index generation
* Knowledge package generation

---

## Knowledge Registry

The Knowledge Registry is the compiled representation of every registered repository.

It contains:

* Repository metadata
* Document metadata
* Chunk indexes
* Cross references
* Search indexes
* Optional enrichments

The registry is generated.

It is never edited manually.

---

## Knowledge Resolver

The resolver determines which knowledge is required for a repository or implementation task.

Rather than loading every document, it assembles only the relevant knowledge.

Examples:

* Current repository
* Internal dependencies
* External context
* Referenced architecture
* Related features

---

## Knowledge Package

A Knowledge Package is the runtime artifact consumed by AI coding agents.

A package may contain:

* Repository documentation
* Internal dependencies
* External context
* Metadata
* Summaries
* Search indexes
* Original Markdown

Knowledge Packages are generated artifacts.

They are never manually edited.

---

# Architecture

```text
Documentation Standards
          │
          ▼
 Markdown Documentation
          │
          ▼
   Automated Audit (quality gate)
          │
          ▼
   Saṃgraha Compiler
          │
          ▼
   Knowledge Registry
          │
          ▼
  Knowledge Resolver
          │
          ▼
  Knowledge Package
  (audit metadata embedded)
          │
          ▼
    MCP Runtime (quality gate)
          │
          ▼
    AI Coding Agents
```

---

# Progressive Knowledge Retrieval

Saṃgraha minimizes AI context by progressively increasing the amount of retrieved knowledge.

```
Metadata
    │
    ▼
Summary
    │
    ▼
Relevant Section
    │
    ▼
Original Markdown
```

Most implementation tasks never require the entire document.

---

# Knowledge Enrichment

Knowledge enrichment is optional.

It enhances generated artifacts without changing the original documentation.

Possible enrichments include:

* Document summaries
* Architecture summaries
* Dependency summaries
* Keywords
* Embeddings
* Glossaries
* Future enrichments

Enrichment occurs during compilation.

Never during retrieval.

---

# AI Is Optional

Saṃgraha never requires an AI model.

The compiler works without any LLM.

```
Markdown
    │
    ▼
Compiler
    │
    ▼
Knowledge Package
```

If an AI provider is available, additional enrichments are generated.

```
Markdown
    │
    ▼
Compiler
    │
    ▼
AI Enrichment (Optional)
    │
    ▼
Knowledge Package
```

If AI is unavailable, compilation still succeeds.

---

# Build Pipeline

```
Markdown
    │
    ▼
Automated Audit (quality gate, deterministic checks)
    │
    ▼
Parser
    │
    ▼
Metadata
    │
    ▼
Chunking
    │
    ▼
Semantic Audit (optional AI-assisted checks)
    │
    ▼
Knowledge Enrichment (Optional)
    │
    ▼
Knowledge Registry
    │
    ▼
Knowledge Package (includes audit metadata)
```

---

# AI Providers

Saṃgraha is provider-agnostic.

Any OpenAI-compatible endpoint may be used.

Examples include:

* LM Studio
* Ollama
* llama.cpp
* vLLM
* OpenAI-compatible local servers

AI providers are used only during compilation.

They are never required during runtime.

---

# Runtime

The runtime requires only the generated Knowledge Package.

No AI model is required.

```
Coding Agent
      │
      ▼
MCP Runtime
      │
      ▼
SQLite
      │
      ▼
Knowledge Package
```

---

# Search Strategy

Knowledge retrieval follows a deterministic pipeline.

1. Metadata
2. Full Text Search (SQLite FTS5)
3. Optional Vector Search
4. Ranking
5. Progressive Retrieval

Vectors are optional.

Metadata remains the primary retrieval mechanism.

---

# Multi-Repository Support

Repositories explicitly declare knowledge dependencies.

Example:

```
my-project

depends on

- saṃgraha (compiler)
- provider-api (AI integration)
- vector-db (embeddings storage)
```

The resolver automatically assembles the required Knowledge Package.

Unrelated repositories are excluded.

---

# Multi-Machine

Generated artifacts are never committed.

Every machine generates its own registry and packages from the same documentation.

```
Git Clone

↓

Documentation

↓

Compiler

↓

Knowledge Package
```

Deterministic builds eliminate synchronization issues.

---

# MCP Integration

Saṃgraha integrates with MCP-compatible coding agents via deterministic
knowledge packages. All documentation served through MCP has passed the
audit pipeline — it is contract-compliant, traceable to Vision, and free
of implementation leakage.

Examples include:

* Claude Code
* OpenCode
* Codex CLI
* Antigravity
* Future MCP clients

The runtime exposes tools such as:

* search_documents
* search_features
* search_architecture
* search_dependencies
* search_external_context
* get_summary
* get_section
* get_document

Each tool returns only audit-passing documentation, ensuring AI agents
receive reliable, structured knowledge without noise.

---

# Repository Structure

```
samgraha/

    compiler/

    resolver/

    registry/

    enrichment/

    runtime/

    mcp/

    cli/

    schemas/

    providers/
```

# Documentation Structure

Documentation lives in `docs/raw/` with three sub-systems:

```
docs/raw/

    philosophy/             # Core documentation philosophy and lifecycle
        documentation-philosophy.md

    vision/                 # Product purpose and direction
        vision.md

    standards/              # Independent domain contracts (10 docs)
        vision.md
        design.md
        architecture.md
        feature.md
        feature-design.md
        feature-technical.md
        prototype.md
        engineering.md
        external-context.md
        readme.md

    audit/                  # Compliance verification against standards
        README.md
        vision-audit.md
        architecture-audit.md
        design-audit.md
        feature-audit.md
        feature-design-validation.md
        feature-technical-audit.md
        prototype-audit.md
        readme-audit.md
        implementation-audit.md
        statelessness-audit.md
        build-audit.md
        security-audit.md
        ownership-audit.md
        external-context-ownership-audit.md
```

Philosophy defines the lifecycle. Standards define the contracts. Audits verify
compliance against Audit Rules defined in each standard. Every Audit Rule in
every standard is covered by at least one check. Engineering standard rules
are distributed across build-audit, security-audit, statelessness-audit, and
implementation-audit due to their distinct domains.

Example repository:

```
repo/

    samgraha.toml

    features/

    docs/

        architecture/

        engineering/

        external-context/

    .samgraha/

        knowledge.db
```

---

# Design Principles

* Documentation is the authoritative engineering specification.
* Standards define contracts. Audits verify compliance.
* Documentation remains technology independent.
* Audit failures are engineering quality feedback.
* Knowledge is compiled, never handwritten.
* Generated artifacts are disposable.
* AI enhances compilation, not execution.
* Metadata before vectors.
* Progressive retrieval before full document loading.
* Knowledge before retrieval.
* Prototype before production.
* Local-first.
* Offline-first.
* Deterministic builds.

---

# Why Rust?

Rust provides:

* Native performance
* Single executable distribution
* Predictable memory usage
* Excellent filesystem performance
* Parallel processing
* Cross-platform binaries
* Zero runtime installation

Saṃgraha behaves like a compiler rather than an application server.

---

# Why TOML?

TOML is designed for configuration.

Compared to YAML it offers:

* Explicit typing
* Better readability
* Comment support
* Familiar compiler tooling conventions
* Reduced ambiguity

Configuration should be explicit and deterministic.

---

# Why SQLite?

SQLite provides:

* Embedded storage
* Zero configuration
* Offline operation
* Single-file database
* FTS5 search
* Optional vector storage
* Excellent local performance

No external database server is required.

---

# Roadmap

## Version 0.1

* Markdown Compiler
* Metadata Extraction
* SQLite FTS5
* CLI Search

## Version 0.2

* Knowledge Registry
* Knowledge Resolver
* Repository Packages

## Version 0.3

* MCP Runtime
* Workspace Support
* Incremental Compilation

## Version 0.4

* AI Enrichment
* Summaries
* Keywords
* Embeddings

## Version 0.5

* Automated Audit (deterministic checks)
* Audit metadata in knowledge packages
* MCP quality gate (serve only audit-passing docs)
* Audit CLI command (single domain and full suite)

## Version 0.6

* Semantic Audit (optional AI-assisted checks)
* Audit report generation
* CI audit integration
* Audit configuration (samgraha.toml)

## Version 1.0

* Stable Package Format
* Incremental Knowledge Compilation
* Cross Repository Resolution
* Multi-Agent Support
* CI Integration

---

# Non-Goals

Saṃgraha is **not**:

* A documentation editor
* A note-taking application
* A wiki
* A documentation generator
* A vector database
* A RAG framework
* A replacement for engineering documentation

Saṃgraha is a **Knowledge Engineering Platform** that compiles engineering documentation into deterministic, implementation-ready knowledge packages for AI coding agents.

