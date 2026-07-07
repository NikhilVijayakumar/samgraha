# MCP Tools Reference

## Purpose

Complete reference for all MCP tools provided by the Samgraha MCP server.

## Content

There is no `search_documents`/`search_sections`/`search_features`/`get_summary` — `search` is the one general-purpose search tool (filter by `domain` instead of calling a per-domain tool). The full method list, from `crates/mcp/src/adapter.rs`:

ping, capabilities, init, compile, search, get_sections, audit, info, get_document, get_document_section, list_domains, list_repositories, register_repository, unregister_repository, synchronize_repository, resolve_dependencies, repository_status, workspace_status, get_documents_by_domain, get_section, get_audit_knowledge, get_audit_report, get_section_changed, check_gate, store_section_report, store_document_report, store_cross_domain_report, update_finding_status, sync, get_plan, switch_context, list_contexts.

Tools that return lists (`search`, `get_sections`, `list_repositories`, `repository_status`, `workspace_status`, `get_documents_by_domain`, `get_document_section`) accept `limit`/`offset` and return `{ total, offset, limit, has_more, <key>: [...] }`. `max` is accepted as a backward-compatible alias for `limit`.

### `search`

Search compiled knowledge (repo + built-in, plus dependencies/interests when a multi-repo session is active).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| query | string | yes | Search query |
| domain | string | no | Filter by domain |
| level | string | no | `metadata` \| `summary` \| `section` \| `full` (default: metadata) |
| limit | integer | no | Max results (default: 20) |
| offset | integer | no | Result offset (default: 0) |

### `get_sections`

Get document sections by semantic type (same as CLI `sections`).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| semantic_type | string | yes | Semantic type filter |
| domain | string | no | Filter by domain |
| limit / offset | integer | no | Pagination (default limit: 50) |

### `get_document`

Get document metadata and its section table of contents.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | yes | Document ID |

### `get_document_section`

Get the (paginated) content of a specific section.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | yes | Document ID |
| section | integer or string | yes | Section index or heading text |
| limit / offset | integer | no | Content-line pagination |

### `list_domains`

List domains that have compiled documents. No parameters.

### `init`

Initialize `samgraha.toml` and `.samgraha/` for this repo, or backfill any keys missing from an existing `samgraha.toml` (never overwrites a key already there). Also (re)generates `.env.example` with every env key samgraha reads, additive only — existing keys/content are left untouched. Mirrors the CLI `init` command; call this first in a repo with no `samgraha.toml` before `compile`/`register_repository`.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| force | boolean | no | Overwrite existing `samgraha.toml` with a fresh template instead of backfilling missing keys |

### `compile`

Compile documentation into a knowledge database. Omit `path` to compile the current repo; pass `path` to compile an external repo into its own `.samgraha/knowledge.db`.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| force | boolean | no | Force full recompile |
| domains | array\<string\> | no | Domains to compile |
| path | string | no | Absolute path to an external repository |

### `sync`

Read an external repo's `.samgraha/manifest.json`, register it, and write its `.meta` cache file so the Planner can resolve it offline.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| path | string | yes | Absolute path to the repo root (must contain `.samgraha/manifest.json`) |

### `get_plan`

Return the current Knowledge Plan — which repos are loaded, their priority (primary/dependency/interest), status (`loaded`/`stale`/`outdated`/`missing`/`unresolved`/`required_missing`), and revision. No parameters.

### Multi-repo registry tools

`list_repositories`, `register_repository` (takes `manifest`, a `RepositoryManifest` JSON string), `unregister_repository` (takes `uuid`), `synchronize_repository`, `resolve_dependencies`, `repository_status`, `workspace_status` — mirror the CLI `samgraha registry`/`workspace` subcommands. See [Multi-Repo Guide](../multi-repo-guide/registry.md).

### Audit workflow tools

`audit` — supports `pipeline` (string, default `"doc"`), `inspect_artifact` (boolean, Build Audit only), and `runtime` (boolean, Security Audit only) parameters in addition to existing domain/provider/all/gate/report parameters. For `build`/`test`/`package`/`deploy` pipeline contracts (see [Pipeline Contracts](../concepts/pipeline-contracts.md)), also supports `execute` (boolean, default `false` — runs the declared command instead of verify-only) and `dry_run` (boolean — prints the command without running it). No separate `audit_build`/`audit_security`/etc. tools — one `audit` tool, param-driven, same as every other pipeline type.

`get_documents_by_domain`, `get_section`, `get_audit_knowledge`, `get_audit_report`, `get_section_changed`, `check_gate`, `store_section_report`, `store_document_report`, `store_cross_domain_report`, `update_finding_status` — support an AI agent driving a semantic audit pass and recording findings.

### Session tools

`switch_context`, `list_contexts` — manage which repo's `KnowledgeContext` an MCP session is bound to when working across multiple repos in one session.

## Related

- [MCP Overview](overview.md)
- [Examples](examples.md)
- [Search Guide](../search-guide/overview.md)
