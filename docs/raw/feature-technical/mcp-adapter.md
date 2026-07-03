# MCP Adapter — Feature Technical Design

This section details the MCP Adapter — Feature Technical Design.

## Purpose

This document describes the architectural realization of the MCP (Model Context Protocol) transport adapter for Saṃgraha.

The MCP Adapter exposes Knowledge Runtime capabilities to AI language models over the Model Context Protocol. It enables LLMs to query compiled knowledge, navigate document structure, and retrieve section content progressively — managing context window usage through built-in pagination.

This document applies the architectural principles defined in Component Model, Runtime Boundary, Communication Architecture, and Security Architecture.

---

## Feature Specification

- **Runtime:** docs/raw/feature-technical/knowledge-runtime.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/runtime-boundary.md, docs/raw/architecture/communication.md, docs/raw/architecture/security-architecture.md

---

## Participating Components

This section details the Participating Components.

### MCP Adapter

The MCP Adapter is the transport adapter responsible for MCP communication. It translates JSON-RPC requests from AI consumers into Knowledge Runtime operations and formats responses as MCP-compliant JSON.

### Knowledge Runtime

The Knowledge Runtime executes all engineering operations. The MCP adapter forwards method calls to the runtime and receives structured responses.

### Knowledge Registry

The Knowledge Registry provides compiled knowledge. The MCP adapter never accesses the registry directly — all access routes through the Knowledge Runtime.

### Repository Registry

The Repository Registry provides repository catalog data for workspace-level operations. The adapter queries it through the runtime.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| MCP Adapter | Parse MCP requests, enforce pagination, format paginated responses, dispatch to runtime |
| Knowledge Runtime | Execute operations, enforce repository boundaries, provide knowledge and registry access |
| Knowledge Registry | Provide compiled documents and section indexes |
| Repository Registry | Provide repository catalog for workspace and status queries |

---

## Component Interactions

```text
AI Consumer (LLM)
    │
    ▼
MCP Adapter (stdin/stdout JSON-RPC)
    │
    ├── compile              →  Knowledge Runtime → Compilation Service
    ├── search               →  Knowledge Runtime → Search Service
    ├── get_sections         →  Knowledge Runtime → Registry (section index)
    ├── audit                →  Knowledge Runtime → Audit Framework
    ├── info                 →  Knowledge Runtime → RuntimeInfo
    ├── get_document         →  Knowledge Runtime → Registry (TOC only)
    ├── get_document_section →  Knowledge Runtime → Registry (section body)
    ├── list_domains         →  Knowledge Runtime → StandardRegistry
    ├── list_repositories    →  Repository Registry → RegistryDb
    ├── repository_status    →  Repository Registry → RegistryDb
    └── workspace_status     →  Repository Registry + Metadata Cache
    │
    ▼
Paginated JSON Response
```

### Request Dispatch Flow

1. AI consumer sends an MCP JSON-RPC message over stdin.
2. MCP Adapter deserializes the message.
3. Adapter checks the `method` field and dispatches to the appropriate handler.
4. Handler extracts `limit`, `offset` (or `max` as alias for `limit`) from request params.
5. Handler invokes the Knowledge Runtime.
6. Runtime executes the operation against compiled knowledge or the registry.
7. Handler applies pagination to the result set.
8. Adapter serializes the paginated response as MCP JSON and writes to stdout.

---

## Pagination Contract

All retrieval methods follow a uniform pagination contract. AI consumers that fit data in one pass use the defaults; those managing context window budget override the parameters.

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `limit` | integer | Maximum items (or lines) to return. Alias: `max`. |
| `offset` | integer | Zero-based index of the first item to return. Default: `0`. |

The `max` alias is accepted as a backward-compatible synonym for `limit`. When both are provided, `limit` takes precedence.

### Response Envelope

Every paginated response includes the following fields alongside the result data:

| Field | Type | Description |
|---|---|---|
| `total` | integer | Total items available before pagination. |
| `offset` | integer | Offset applied in this response. |
| `limit` | integer | Limit applied in this response. |
| `has_more` | boolean | `true` if more items exist beyond this page. |

### Default Limits

| Method | Default `limit` | Result unit |
|---|---|---|
| `search` | 20 | documents |
| `get_sections` | 50 | semantic sections |
| `get_document_section` | 100 | lines |
| `list_repositories` | 50 | registry entries |
| `repository_status` | 50 | registry entries |
| `workspace_status` | 50 | registered repositories |

### LLM Override Behavior

Defaults are chosen to fit within a typical LLM context window for each data type. An LLM may pass any `limit` and `offset` values to match its current context budget. There is no server-enforced maximum — the LLM is responsible for requesting an amount it can process.

Example: An LLM with remaining context for 10 results passes `"limit": 10, "offset": 0` on the first call, then `"limit": 10, "offset": 10` for the next page, continuing until `has_more` is `false`.

---

## Method Reference

This section details the Method Reference.

### `compile`

Compiles source documents into the Knowledge Registry.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `path` | string | no | Repository root path. Defaults to current directory. |
| `scope` | string | no | `"repository"` (default) or `"workspace"`. |

**Response:** Compilation result with success status, document counts, errors, and duration.

No pagination — this is a write operation, not a retrieval.

---

### `search`

Searches compiled knowledge by query string with optional filters.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `query` | string | yes | Full-text search query. |
| `domain` | string | no | Filter by domain. |
| `status` | string | no | Filter by document status. |
| `repository` | string | no | Filter by repository name. |
| `level` | string | no | Retrieval level: `metadata`, `summary`, `section`, `full`. Default: `metadata`. |
| `limit` / `max` | integer | no | Max results to return. Default: `20`. |
| `offset` | integer | no | Zero-based start index. Default: `0`. |

**Response:**
```json
{
  "results": [ { "document_id": 1, "path": "...", "title": "...", "domain": "...", "score": 0.9, "snippet": "..." } ],
  "total": 42,
  "offset": 0,
  "limit": 20,
  "has_more": true
}
```

The service fetches all matching documents and paginates the result set in memory. This ensures consistent `total` counts regardless of page position.

---

### `get_sections`

Retrieves semantic sections of a given type across the repository or workspace.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `semantic_type` | string | yes | Section type (e.g., `"functional_requirements"`, `"business_rules"`). |
| `domain` | string | no | Filter by domain. |
| `limit` / `max` | integer | no | Max sections to return. Default: `50`. |
| `offset` | integer | no | Zero-based start index. Default: `0`. |

**Response:**
```json
{
  "sections": [ { "id": 1, "document_id": 2, "document_title": "...", "semantic_type": "...", "content": "..." } ],
  "total": 120,
  "offset": 0,
  "limit": 50,
  "has_more": true
}
```

---

### `audit`

Runs audit checks against compiled documents.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `domain` | string | no | Limit audit to a specific domain. |
| `providers` | array | no | List of audit provider names to run. |

**Response:** Audit report with findings, severities, and pass/fail status. No pagination — audit results are always returned in full.

---

### `info`

Returns runtime information about the active repository.

**Request params:** None.

**Response:** Repository name, registry path, document count, registered standards, and active services.

---

### `get_document`

Returns document metadata and table of contents (section list) without body content. The LLM uses the TOC to identify which section to request via `get_document_section`.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `id` | integer | yes | Document ID as returned by `search` or `get_sections`. |

**Response:**
```json
{
  "id": 1,
  "title": "...",
  "standard": "feature",
  "path": "docs/raw/feature/example.md",
  "hash": "abc123",
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-02T00:00:00Z",
  "total_lines": 312,
  "section_count": 8,
  "sections": [
    {
      "index": 0,
      "heading": "Purpose",
      "semantic_type": "purpose",
      "level": 2,
      "required": true,
      "line_start": 5,
      "line_end": 12,
      "subsection_count": 0
    }
  ]
}
```

`get_document` never returns body content. Its purpose is orientation — the LLM inspects the TOC and requests only the sections it needs.

---

### `get_document_section`

Returns paginated line content for a specific section within a document.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `id` | integer | yes | Document ID. |
| `section` | integer or string | yes | Section index (from `get_document` TOC) or section heading (case-insensitive substring match). |
| `limit` / `max` | integer | no | Max lines to return. Default: `100`. |
| `offset` | integer | no | Zero-based line start index within the section. Default: `0`. |

**Section lookup:**
- If `section` is an integer: treated as a zero-based index into the document's section list.
- If `section` is a string: matched against section headings using case-insensitive substring match. First match wins.

**Response:**
```json
{
  "heading": "Functional Requirements",
  "semantic_type": "functional_requirements",
  "required": true,
  "content": "line 1\nline 2\n...",
  "total_lines": 45,
  "offset": 0,
  "limit": 100,
  "has_more": false,
  "subsections": ["Acceptance Criteria", "Constraints"]
}
```

An LLM reading a 312-line document with 8 sections retrieves each section individually, fitting each within its context window, rather than receiving the entire document body in one response.

---

### `list_domains`

Returns all registered documentation standards (domains) in the active repository.

**Request params:** None.

**Response:** Array of domain names. No pagination — domain lists are small.

---

### `list_repositories`

Returns all repositories registered in the Repository Registry.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `limit` / `max` | integer | no | Max entries. Default: `50`. |
| `offset` | integer | no | Zero-based start index. Default: `0`. |

**Response:**
```json
{
  "repositories": [ { "id": "...", "uuid": "...", "path": "...", "revision": "..." } ],
  "total": 12,
  "offset": 0,
  "limit": 50,
  "has_more": false
}
```

---

### `repository_status`

Returns status of one or more repositories from the Registry.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `id` | string | no | Repository ID to filter. If absent, returns all. |
| `limit` / `max` | integer | no | Max entries. Default: `50`. |
| `offset` | integer | no | Zero-based start index. Default: `0`. |

**Response:**
```json
{
  "repositories": [ { "id": "...", "status": "Current", "expires": "...", "audit": "PASS" } ],
  "total": 12,
  "offset": 0,
  "limit": 50,
  "has_more": false
}
```

---

### `workspace_status`

Returns the workspace overview: registered repositories, their sync status, and aggregate audit state.

**Request params:**
| Param | Type | Required | Description |
|---|---|---|---|
| `limit` / `max` | integer | no | Max repositories. Default: `50`. |
| `offset` | integer | no | Zero-based start index. Default: `0`. |

**Response:**
```json
{
  "workspace": "...",
  "repositories": [ { "id": "...", "status": "Current", "registered": true } ],
  "total": 5,
  "offset": 0,
  "limit": 50,
  "has_more": false
}
```

---

## Progressive Document Retrieval

The recommended pattern for an LLM to retrieve document content:

```
1. search(query: "...")                  → get document_id values
2. get_document(id: N)                   → get TOC, identify sections of interest
3. get_document_section(id: N,
       section: "Purpose")               → get first section content
4. get_document_section(id: N,
       section: "Functional Requirements",
       limit: 50, offset: 0)            → get first 50 lines
5. get_document_section(id: N,
       section: "Functional Requirements",
       limit: 50, offset: 50)           → get next 50 lines (if has_more)
```

This pattern lets the LLM:
- Pay one call to orient itself (TOC only)
- Skip sections irrelevant to its task
- Read large sections in context-sized chunks

---

## Protocol Details

This section details the Protocol Details.

### Transport

The MCP Adapter reads from stdin and writes to stdout. Each message is a newline-delimited JSON object following the Model Context Protocol specification (version `2025-03-26`).

### Message Types

| Type | Direction | Purpose |
|---|---|---|
| `request` | Consumer → Adapter | Method invocation |
| `response` | Adapter → Consumer | Method result |
| `notification` | Adapter → Consumer | Async event (initialization) |
| `error` | Adapter → Consumer | Method failure |

### Capabilities Declaration

On initialization, the adapter publishes its capabilities:

```json
{
  "version": "0.1.0",
  "protocol_version": "2025-03-26",
  "methods": [
    "compile", "search", "get_sections", "audit", "info",
    "get_document", "get_document_section", "list_domains",
    "list_repositories", "repository_status", "workspace_status"
  ]
}
```

---

## Data Ownership

| Data | Owner | Adapter Access |
|---|---|---|
| Compiled Knowledge | Knowledge Registry | Read via Runtime |
| Repository Catalog | Repository Registry | Read via Runtime |
| Request Parameters | MCP Adapter | Transient |
| Pagination State | MCP Adapter | Transient (per-request) |
| Consumer Session | Consumer | None (stateless) |

---

## Runtime Constraints

- Adapter must never store state between requests.
- Adapter must accept `limit` and `offset` on every retrieval method.
- Adapter must include `total`, `offset`, `limit`, `has_more` in every paginated response.
- Adapter must accept `max` as a synonym for `limit`.
- Default limits must be honored when neither `limit` nor `max` is provided.
- `get_document` must never return body content — TOC only.
- `get_document_section` must support both integer index and string heading lookup.

---

## Architectural Constraints

- Adapter must carry no engineering logic — all computation belongs to the Knowledge Runtime.
- Adapter must not cache results between requests.
- Adapter must remain stateless with respect to consumer sessions.
- Adapter must not access the Knowledge Registry or Repository Registry directly.

---

## Security Considerations

- All repository boundary enforcement occurs in the Knowledge Runtime, not the adapter.
- The adapter applies no authentication — access control is enforced at the process level.
- Request parameters are passed through to the runtime without shell interpolation.
- No credential material passes through the MCP adapter.

---

## Performance Considerations

- Pagination is applied in memory after the registry query; the registry is queried once per request regardless of page size.
- `get_document_section` splits body content by newline at request time — no pre-built line index.
- Default limits are sized to fit within a typical LLM context window while minimizing round trips.
- The adapter adds no latency beyond JSON serialization and parameter extraction.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Unknown method | Return MCP error with code `-32601` (Method Not Found) |
| Missing required param | Return MCP error with code `-32602` (Invalid Params) |
| Document not found | Return `null` result with `not_found: true` |
| Section not found | Return MCP error with code `-32602` and descriptive message |
| Registry unavailable | Return MCP error with code `-32603` (Internal Error) |
| Invalid `offset` or `limit` | Clamp to valid range (offset to 0, limit to default) |

---

## Extension Points

This section details the Extension Points.

### New Methods

Additional runtime capabilities are exposed by adding a handler to the method dispatch and registering the method name in the capabilities declaration.

### Alternative Transports

The adapter is wired for stdin/stdout. Alternative transports (Unix socket, named pipe) may be added by wrapping the same handler dispatch without changing method behavior.

---

## Traceability

This document derives from:

- Feature: Knowledge Runtime
- Architecture: Component Model
- Architecture: Runtime Boundary
- Architecture: Communication Architecture
- Architecture: Security Architecture

This document provides technical context for:

- Engineering MCP Strategy
- MCP Pagination Implementation (`crates/mcp/src/adapter.rs`)
