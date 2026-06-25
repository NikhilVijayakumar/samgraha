# Saṃgraha

> **Compile engineering documentation into AI-ready knowledge packages for coding agents.**

---

## Overview

Saṃgraha is a **Knowledge Engineering Platform** designed for AI-assisted software engineering.

Traditional documentation is written for humans.

Saṃgraha transforms engineering documentation into structured, searchable, implementation-ready knowledge that can be consumed efficiently by AI coding agents.

Instead of treating Markdown as static documentation, Saṃgraha treats it as **source code for knowledge**.

Documentation remains the single source of truth.

Everything else is generated.

---

# Philosophy

Documentation should drive implementation.

Source documentation should be:

* Human readable
* Version controlled
* Technology independent
* Architecture focused

Generated artifacts should provide:

* Fast retrieval
* AI-friendly structure
* Deterministic compilation
* Offline operation

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

# Core Concepts

## Documentation

Documentation is the authoritative source of knowledge.

Examples include:

* Vision
* Features
* Architecture
* Engineering Decisions
* External Context
* Ownership
* Protocols
* Invariants
* Security
* Runtime
* APIs

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
Markdown Documentation
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
          │
          ▼
     MCP Runtime
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
Parser
    │
    ▼
Metadata
    │
    ▼
Chunking
    │
    ▼
Knowledge Enrichment (Optional)
    │
    ▼
Knowledge Registry
    │
    ▼
Knowledge Package
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
Prana

depends on

- Astra
- React
- Electron
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

Saṃgraha integrates with MCP-compatible coding agents.

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

Example repository:

```
Prana/

    samgraha.toml

    docs/

    architecture/

    features/

    engineering/

    external-context/

    .samgraha/

        knowledge.db
```

---

# Design Principles

* Documentation is the source of truth.
* Documentation remains technology independent.
* Engineering decisions explain implementation choices.
* Knowledge is compiled, never handwritten.
* Generated artifacts are disposable.
* AI enhances compilation, not execution.
* Metadata before vectors.
* Progressive retrieval before full document loading.
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

