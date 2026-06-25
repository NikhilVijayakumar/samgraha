# Knowledge Search — Feature Technical Design

## Purpose

This document describes the architectural realization of the Knowledge Search feature.

Knowledge Search enables deterministic discovery and progressive retrieval of compiled engineering knowledge. It operates on the Knowledge Registry rather than raw documentation, providing both human engineers and automated consumers with relevant, ranked results.

This document applies the architectural principles defined in Component Model, Runtime Boundary, and Persistence Architecture.

---

## Feature Specification

- **Feature:** docs/raw/feature/knowledge-search.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/runtime-boundary.md, docs/raw/architecture/persistence.md

---

## Participating Components

### Knowledge Runtime

The Knowledge Runtime owns search execution. Search is a Knowledge Service invoked through the runtime. The runtime resolves the active repository and delegates to the search service.

### Knowledge Registry

The Knowledge Registry provides the data source for search. It stores compiled documents, metadata, indexes, and relationships that the search system queries.

### Knowledge Enrichment

Knowledge Enrichment optionally produces search metadata including summaries, keywords, and embeddings. When present, enrichment artifacts improve search relevance.

### Transport Adapters

Transport adapters (CLI, MCP) expose search to external consumers. Adapters translate search requests into runtime operations and return formatted results.

### Incremental Build

Incremental Build ensures the Knowledge Registry is up to date. Search operates only on compiled knowledge; incremental compilation guarantees search reflects the latest documentation state.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Knowledge Runtime | Resolve search context, invoke search service, enforce repository boundaries |
| Knowledge Registry | Store compiled documents, indexes, metadata, relationships |
| Knowledge Search Service | Execute queries, rank results, support progressive retrieval |
| Knowledge Enrichment | Generate optional search metadata (summaries, keywords, embeddings) |
| Transport Adapters | Translate search protocols, format results for consumers |

---

## Component Interactions

```text
Consumer
    │
    ▼
Transport Adapter
    │
    ▼
Knowledge Runtime
    │
    ▼
Knowledge Registry (query indexes, retrieve documents)
    │
    ▼
Knowledge Enrichment (optional — summaries, keywords)
    │
    ▼
Knowledge Runtime (compose response)
    │
    ▼
Transport Adapter (format response)
    │
    ▼
Consumer
```

### Search Request Flow

1. Consumer submits a search request through a transport adapter (CLI, MCP).
2. Adapter translates the request into a runtime operation.
3. Knowledge Runtime resolves the active repository or workspace context.
4. Runtime invokes the Knowledge Search service with the query and context.
5. Search service queries the Knowledge Registry indexes.
6. Search service applies structured filters (domain, status, repository, audit status).
7. Search service ranks results by relevance.
8. Search service retrieves progressive content (metadata, summary, sections, full document) based on requested level.
9. Runtime composes the response with results and navigation metadata.
10. Transport adapter formats and returns the response.

---

## Runtime Behavior

### Search Lifecycle

```
Receive Search Request
        │
        ▼
Resolve Repository Context
        │
        ▼
Query Indexes
        │
        ▼
Apply Filters
        │
        ▼
Rank Results
        │
        ▼
Retrieve Progressive Content
        │
        ▼
Return Results
```

### Stateless Execution

Each search request executes independently. The search service maintains no session state between requests. All state is contained in the Knowledge Registry.

### Determinism

Identical queries against identical registry content produce identical results. Search does not depend on external services, random values, or runtime state.

---

## Communication Paths

### Consumer → Transport Adapter

Consumers submit search queries through the adapter interface. Queries include search terms, filters, retrieval level, and repository scope.

### Transport Adapter → Knowledge Runtime

The adapter forwards validated requests to the runtime. The adapter adds protocol-specific metadata (session, authentication when applicable).

### Knowledge Runtime → Knowledge Registry

The runtime queries registry indexes and retrieves compiled documents. The registry is read-only during search.

### Knowledge Runtime → Knowledge Enrichment

The runtime optionally queries enrichment metadata to augment search results with summaries, keywords, or semantic relationships.

---

## Data Ownership

| Data | Owner | Search Access |
|---|---|---|
| Compiled Documents | Knowledge Registry | Read |
| Search Indexes | Knowledge Registry | Read |
| Metadata | Knowledge Registry | Read |
| Enrichment Artifacts | Knowledge Registry | Read |
| Search Session | Knowledge Runtime | Transient |
| Query Results | Knowledge Runtime | Transient |

---

## Integration Points

### CLI Interface

`samgraha search <query> [options]` — the CLI adapter translates command-line arguments into a search request.

### MCP Interface

AI assistants search through the MCP transport. The MCP adapter handles progressive content delivery.

### Knowledge Registry

Search reads registry indexes and compiled documents. Indexes are generated during compilation and updated during incremental builds.

---

## External Dependency Integration

Core search operates entirely offline with no external dependencies.

Optional: When Knowledge Enrichment is configured with AI providers, semantic search features may use generated embeddings. Search relevance improves but core functionality does not depend on enrichment.

---

## Runtime Constraints

- Search must execute without network access.
- Search must complete within predictable time bounds.
- Search must support large Knowledge Registries (100,000+ documents).
- Search must respect repository isolation.
- Search must not modify registry content.
- Search must support concurrent consumers.

---

## Architectural Constraints

- Search operates only on compiled knowledge.
- Search must not access source documentation.
- Search must not depend on AI providers.
- Search must preserve repository boundaries.
- Search results must include repository and ownership metadata.

---

## Security Considerations

- Search never exposes source documentation path or content outside the compiled registry.
- Repository isolation prevents cross-repository knowledge leakage.
- Consumer context is validated before results are returned.
- Enrichment metadata is clearly distinguished from authoritative compiled knowledge.

---

## Performance Considerations

- Index lookups must complete in sub-millisecond time.
- Full-text search across large registries must return within 500ms.
- Progressive retrieval minimizes data transfer — metadata first, full content on demand.
- Index updates during incremental builds must not block concurrent readers.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Registry unavailable | Return error, indicate search unavailable |
| Index corruption | Return error, suggest recompilation |
| Invalid query | Return error with actionable message |
| Enrichment unavailable | Return deterministic results without enrichment augmentation |
| Resource exhaustion | Return partial results with continuation support |

---

## Extension Points

### Search Providers

Alternative search implementations may be registered through the search provider interface. Default implementation uses deterministic indexing.

### Ranking Strategies

Custom ranking algorithms may be registered without modifying the search pipeline.

### Filter Providers

Repository-specific or organization-specific filter implementations extend the default filter set.

---

## Traceability

This document derives from:

- Feature: Knowledge Search
- Architecture: Component Model
- Architecture: Runtime Boundary
- Architecture: Persistence Architecture
- Architecture: Communication Architecture

This document provides technical context for:

- Engineering Search Strategy
- MCP Runtime Technical Design
- CLI Interface Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
