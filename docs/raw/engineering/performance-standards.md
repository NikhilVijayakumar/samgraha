# Performance Standards

## Purpose

This document defines the performance expectations and measurement standards for the Saṃgraha platform.

Performance is defined in terms of user-visible behavior — time to complete operations, not synthetic benchmarks. The platform prioritizes predictable performance over raw throughput.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Performance Targets

### Compilation

| Document Count | Target Time | Degradation Threshold |
|---|---|---|
| 100 documents | < 1 second | 3 seconds |
| 1,000 documents | < 5 seconds | 15 seconds |
| 10,000 documents | < 30 seconds | 60 seconds |
| 100,000 documents | < 5 minutes | 10 minutes |

Targets apply to the full compilation pipeline: discovery, parsing, metadata extraction, relationship resolution, and registry update.

### Search

| Registry Size | Target Time | Degradation Threshold |
|---|---|---|
| 1,000 documents | < 100ms | 500ms |
| 10,000 documents | < 200ms | 1 second |
| 100,000 documents | < 500ms | 2 seconds |

### Audit

| Document Count | Target Time | Degradation Threshold |
|---|---|---|
| 100 documents | < 2 seconds | 5 seconds |
| 1,000 documents | < 10 seconds | 30 seconds |
| 10,000 documents | < 1 minute | 3 minutes |

### Registry Operations

| Operation | Target Time | Degradation Threshold |
|---|---|---|
| Load (10K docs) | < 500ms | 2 seconds |
| Single document insert | < 10ms | 50ms |
| Batch insert (100 docs) | < 100ms | 500ms |

---

## Measurement Standards

### Methodology

- Measurements are taken on reference hardware:
  - CPU: Modern x86-64 (AMD Ryzen 5 / Intel i5 equivalent or better)
  - RAM: 16GB+
  - Storage: SSD (NVMe or SATA)
  - OS: Windows 11, Ubuntu 24.04, macOS 14+
- Each measurement is a median of 10 runs
- Cold cache: first run after system boot
- Warm cache: subsequent runs with filesystem cache populated
- Results are reported as: `median (p95) [cold | warm]`

### Tooling

- Criterion benchmarks for micro-benchmarks
- Custom timing harness for compilation pipeline benchmarks
- `--timings` flag for ad-hoc performance measurement
- tracing spans for detailed operation timing

### Reporting

Performance regressions are tracked in:

```
docs/raw/reports/performance/latest/
├── compilation.md           # Compilation benchmarks
├── search.md               # Query benchmarks
├── audit.md                # Audit benchmarks
└── summary.md              # Summary and regression detection
```

---

## Design for Performance

### Batch Processing

- Compilation processes documents in parallel batches
- Batch size is configurable (default: 100 documents)
- Progress is reported per-batch
- Errors in one batch do not stop processing

### Incremental Processing

- Documents with unchanged hashes are skipped
- Only changed documents are re-processed
- Relationship resolution re-processes only affected relationships
- Registry update is incremental for changed documents

### Lazy Loading

- Documents are loaded on demand
- Registry metadata is cached
- Full document bodies are loaded only when needed
- Search results load metadata first, bodies on selection

### Memory Management

- Compilation streams documents rather than loading all into memory
- Large document bodies are memory-mapped
- Unused data is dropped promptly
- Memory usage is bounded by configuration

---

## Acceptable Slow Paths

Some operations are expected to be slow by nature:

- AI enrichment (provider latency, network-bound)
- Full registry integrity check (reads every document)
- Knowledge Package export (disk I/O-bound)

These operations are:
1. Explicitly marked as slow in CLI help text and documentation
2. Provide progress indication
3. Are interruptible (SIGINT / Ctrl+C)

---

## Traceability

This document derives from:

- Engineering Principles (deterministic by default)
- Technology Selection
- Persistence Standards

Performance Standards provide the framework for:

- Compiler Implementation
- Registry Implementation
- CI/CD Pipeline

## Build Standards

Performance benchmarks are integrated into the build pipeline. Release builds enable optimizations relevant to performance measurement. See [Build Standards](build-standards.md) for build profile configuration.

## Testing Standards

Performance is verified through benchmark tests measuring compilation speed, query latency, and memory usage against defined thresholds. See [Testing Standards](testing-standards.md) for benchmarking methodology.

Traceability:

```
Engineering Principles → Performance Standards → Implementation
```
