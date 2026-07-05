# CLI Interface — Feature Technical Design

The CLI Interface provides the command-line entry point for all Saṃgraha operations — compilation, registry management, search, audit, and resolution — through a structured subcommand interface.

## Purpose

This document describes the architectural realization of the CLI Interface feature.

The CLI is the primary human interaction point for Saṃgraha. It exposes compilation, search, audit, and information commands through a consistent, discoverable interface. Every CLI command operates offline by default and requires no AI model.

This document applies the architectural principles defined in Component Model, Runtime Boundary, Communication, and Deployment Architecture.

---

## Feature Specification

- **Feature:** docs/raw/feature/cli-interface.md
- **Architecture:** docs/raw/architecture/component-model.md, docs/raw/architecture/runtime-boundary.md, docs/raw/architecture/communication.md, docs/raw/architecture/deployment.md

---

## Participating Components

The Participating Components section identifies the architectural components involved in implementing this feature, their responsibilities, and how they interact to deliver the specified functionality.

### CLI Adapter

The CLI Adapter is the transport adapter responsible for command-line interaction. It translates terminal input into Knowledge Runtime operations and formats responses for human consumption.

### Knowledge Runtime

The Knowledge Runtime executes all engineering operations. The CLI adapter forwards commands to the runtime and receives structured responses.

### Knowledge Services

Knowledge Services implement the actual engineering operations — compilation, search, audit, and information retrieval. The runtime coordinates service execution.

### Knowledge Registry

The Knowledge Registry provides compiled knowledge for search and info commands. The CLI does not access the registry directly.

### Documentation Standards

Documentation Standards define audit contracts. The CLI audit command applies these standards through the Audit Framework.

### Workspace Management

Workspace Management resolves the active workspace context. The CLI init command creates workspace configuration.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| CLI Adapter | Parse command-line arguments, format output, handle exit codes, provide help text, support tab completion |
| Knowledge Runtime | Route commands to Knowledge Services, enforce repository boundaries, manage execution context |
| Knowledge Services | Execute compilation, search, audit, and information operations |
| Knowledge Registry | Provide compiled knowledge for queries |
| Documentation Standards | Define audit rules referenced by the audit command |
| Workspace Management | Provide workspace configuration and discovery |

---

## Component Interactions

```text
Terminal
    │
    ▼
CLI Adapter
    │
    ├── samgraha compile        →  Knowledge Services → Knowledge Compiler
    ├── samgraha search         →  Knowledge Services → Knowledge Registry
    ├── samgraha audit          →  Knowledge Services → Audit Framework
    ├── samgraha info           →  Knowledge Services → Knowledge Registry
    ├── samgraha init           →  Workspace Management
    └── samgraha registry *     →  Repository Registry (via RegistryClient)
            ├── register        →  RegistryClient::register
            ├── unregister      →  RegistryClient::unregister
            ├── sync            →  RegistryClient::sync
            ├── refresh         →  RegistryClient::sync (re-read manifest.json)
            ├── status          →  RegistryClient::get_metadata + RepositoryStatus::compute
            ├── list            →  RegistryClient::list
            └── resolve runtime →  Metadata Cache inspection
    │
    ▼
Formatted Output
```

### Command Dispatch Flow

1. CLI Adapter receives raw terminal input.
2. Adapter parses arguments, flags, and options.
3. Adapter loads optional configuration from samgraha.toml.
4. Adapter validates command syntax and required flags.
5. Adapter forwards the parsed command to the Knowledge Runtime via the service interface.
6. Knowledge Runtime executes the corresponding Knowledge Service.
7. Service returns structured results (success, data, errors, warnings).
8. CLI Adapter formats results for terminal display.
9. Adapter sets the appropriate exit code.
10. Adapter writes output to stdout/stderr.

---

## Runtime Behavior

This section details the Runtime Behavior.

### Command Lifecycle

```
Parse Arguments
        │
        ▼
Load Configuration (optional)
        │
        ▼
Validate Command
        │
        ▼
Execute via Knowledge Runtime
        │
        ▼
Format Output
        │
        ▼
Set Exit Code
        │
        ▼
Write Output
```

### Stateless Execution

Each CLI invocation is independent. No state persists between commands except through the Knowledge Registry and configuration files.

### Offline Operation

All commands execute without network access. AI providers are never required.

---

## Communication Paths

This section details the Communication Paths.

### Terminal → CLI Adapter

The adapter reads stdin for command-line arguments and flags. It may also read piped input for batch operations.

### CLI Adapter → Knowledge Runtime

The adapter invokes the runtime through the service interface. The runtime is embedded in the same process for CLI deployments.

### CLI Adapter → Workspace Management

The init command interacts directly with Workspace Management to create initial repository configuration.

### Repository Guard

Before dispatching to any command handler except `init` and `version`, `Cli::execute` calls `ensure_samgraha_repo()` (`crates/cli/src/commands.rs`). It walks up from the current directory checking for `.samgraha/` or `samgraha.toml`; if neither exists anywhere up to the filesystem root, it returns an error ("fatal: not a samgraha repository ... Run `samgraha init` first") instead of running the command. This mirrors `git`'s behavior of refusing to operate outside a git repository — a plain directory, or one that only has `.git/`, is not a valid samgraha repository until `samgraha init` has run.

`discover_repository_root()` (`crates/cli/src/config.rs`) performs the same walk-up-and-check to resolve the actual root path once the guard has passed; it is not a second, looser check — same two markers, same failure.

The MCP server (`crates/mcp/src/main.rs::discover_root`) enforces the identical guard at process startup, so an MCP client cannot point the server at an uninitialized directory either. See docs/raw/feature-technical/mcp-adapter.md § Repository Guard.

### Init Is Additive, Not All-or-Nothing

`execute_init` (`crates/cli/src/commands.rs`) no longer bails when `samgraha.toml` already exists. Instead, without `--force`:

1. It builds `template: SamgrahaConfig` — the same full-defaults config a fresh `init` would write (all builtin domains declared, fresh `id`/`name`/`uuid` if this were new).
2. It parses the *existing* `samgraha.toml` as a `toml_edit::DocumentMut` (format-preserving — comments, key order, and array formatting all survive) and serializes `template` to a second `toml_edit::DocumentMut`.
3. `merge_missing_keys(existing: &mut toml_edit::Table, defaults: &toml_edit::Table) -> usize` walks the template's table recursively: a key absent from `existing` gets inserted from the template (whole subtree, at whatever depth it's missing); a key already present in `existing` is never touched — if it's a table, the function recurses into it looking for missing sub-keys, but a scalar or array value that's already there is left completely alone, even if it looks stale.
4. Only if the merge actually added something does the file get rewritten. The rewritten config is then parsed back into `SamgrahaConfig` (verifying it's still valid) for the printed summary/output.

`--force` skips all of this and takes the old all-overwrite path. Without it, re-running `init` on an existing repo is safe and idempotent: running it twice in a row adds nothing the second time. This is what lets the schema grow (new sections like `[report]`, `[repository.implementation]`) across samgraha versions without every existing repo's toml needing manual migration — `init` backfills exactly what's missing, whenever it's next run, and never regresses a hand-authored value or comment.

`toml_edit` (already resolved transitively via `toml` 0.8, pinned at the same version so no new fetch) is used specifically because plain `toml::Value` round-tripping would re-serialize the whole file from scratch — losing comments and reformatting arrays/tables even for keys that didn't change. `toml_edit::DocumentMut` preserves the original document's formatting for anything the merge doesn't touch.

### Environment-Resolved Paths

`common::config::resolve_configured_dir(raw: &str, root: &Path, fallback_rel: &str) -> PathBuf` (`crates/common/src/config.rs`) resolves machine-specific path config fields:

- if `raw` is exactly `"${VAR_NAME}"`, read `VAR_NAME` from the process environment; if set, use its value (absolute as-is, relative joined to `root`); if unset, fall back to `root.join(fallback_rel)`
- if `raw` is any other string, it's used directly as a literal path (absolute as-is, relative joined to `root`) — env resolution is bypassed

Absolute paths (via env) are the intended usage for multi-developer repos and MCP: with `n` dependency/interest repos potentially open in one session, a relative path is ambiguous about *which* repo's root it's relative to, while an absolute path resolved per-machine is not. `samgraha.toml`'s own placeholders stay portable/committed; only the `.env` values are machine-specific.

`resolve_configured_dir` reads `std::env::var` directly — it does not parse `.env` itself. `common::env::load_dotenv()` (`crates/common/src/env.rs`) is what makes `.env` values visible to it: called once at the very start of both `cli::main()` and `mcp::main()` (before any config is loaded), it walks up from the current directory for a `.env` file and sets any key not already present in the real process environment (real env vars always win, standard dotenv precedence). This mirrors the manual parser already in `crates/mcp/build.rs` — that one only runs at *compile* time to bake `SAMGRAHA_EXPIRY` into the binary; `load_dotenv()` is the runtime counterpart, needed so editing `.env` actually changes what a `${VAR}` placeholder resolves to on the next run, without exporting anything in the shell.

Config fields using this today:

- `[repository.documentation].root_dir` (`DocumentationConfig`, default `"${SAMGRAHA_DOCS_DIR}"`, fallback `docs`) — the documentation root; replaces the old unused `paths: Vec<String>` field (zero consumers, removed rather than left dead alongside a newly-wired equivalent)
- `[report].dir` (`ReportConfig`, default `"${SAMGRAHA_REPORT_DIR}"`, fallback `docs/raw/reports`) — consumed by `samgraha audit --report` (`crates/cli/src/commands.rs::execute_audit`)
- `[repository.implementation].dir` (`ImplementationConfig`, default `"${SAMGRAHA_IMPLEMENTATION_DIR}"`, fallback `src`) — schema only; reserved for future traceability/audit checks, no consumer yet
- `[repository.scripts].dir` / `[repository.tests].dir` (`ScriptsConfig`/`TestsConfig`, both `Option<..>`, no default) — optional; for repos that keep an external scripts directory or tests outside `implementation.dir` (e.g. this repo's own top-level `scripts/` and `tests/`, separate from `crates/`). Absent unless the repo declares them; `init` does not populate them. Schema only, no consumer yet.

`samgraha init` and `samgraha env [path]` both call `write_env_example()` (`crates/cli/src/commands.rs`), which writes/updates `.env.example` at the repo root with a commented, blank line per recognized key (`SAMGRAHA_REPORT_DIR`, `SAMGRAHA_DOCS_DIR`, `SAMGRAHA_IMPLEMENTATION_DIR`, `SAMGRAHA_SCRIPTS_DIR`, `SAMGRAHA_TESTS_DIR`). It's additive: it reads the existing file (if any) and only appends keys not already present by name, so it never clobbers keys a repo added for unrelated purposes (e.g. this repo's own `SAMGRAHA_EXPIRY_DAYS`/`OUTPUT_DIR` release-build keys, read separately by `crates/mcp/build.rs`). The real `.env` (actual per-machine values) is gitignored and never generated — only the template.

### Domain Exclusions

`DocumentationConfig` (`crates/common/src/config.rs`) has `domain: Vec<String>` (declared/known domains; empty means "all builtin standards") and `domain_exclusion: Vec<String>` (declared domains this repo deliberately doesn't use). `samgraha init` populates `domain` with the full builtin catalog (`standards::all_builtin_standards()`, mapped to `.domain`) so the generated toml shows every standard the repo could adopt; a repo without, say, `prototype` docs adds `"prototype"` to `domain_exclusion` rather than deleting it from `domain`.

The effective/active set (`domain` minus `domain_exclusion`) is computed in `KnowledgeRuntime::info()` (`crates/services/src/runtime/runtime.rs`) and shown by `samgraha info`. This is presentation-only: compilation and audit already only ever act on documents that physically exist under `docs/raw/<domain>/`, so a missing directory for an excluded (or simply unused) domain was never an error — `domain_exclusion` exists to make that intent explicit in the generated config and in `info` output, not to gate compilation/audit behavior.

### Dependency/Interest Auto-Registration

`FileRegistryClient::sync()` (`crates/services/src/registry_client.rs`) walks `[[repository.dependencies]]`, resolves each declared path, reads its manifest, and writes/refreshes its `CachedRepoMetadata` in the local registry — this already covers both `knowledge.dependencies` (required) and `knowledge.interests` (optional) names, since both resolve through the same `repository.dependencies` name→path table.

What was previously missing was an automatic trigger: `sync()` had to be invoked by hand (`samgraha registry sync`). Both compile entry points now call it automatically after a successful compile, gated on `[resolver].auto_refresh` (already existed in `ResolverConfig`, default `true`, previously unread by any code):

- CLI: `execute_compile` (`crates/cli/src/commands.rs`) calls `sync_registry_best_effort(&root, &runtime.context.config)` after `result.success`, which no-ops if `auto_refresh` is `false`
- MCP: `McpAdapter::handle_compile` (`crates/mcp/src/adapter.rs`) checks `result.success && self.runtime.context.config.resolver.auto_refresh` before calling `self.registry.sync(...)`, for the primary-repo compile path only (not `compile_external`, which compiles some *other* repo's own `knowledge.db`)

Setting `auto_refresh = false` disables the automatic trigger without touching manual `registry sync`/`register`.

This is a hybrid model: automatic sync piggybacks on the command a user already runs regularly (compile), while `samgraha registry register`/`sync` remain fully available as an explicit, manual path for any repository at any time. Sync failures are logged (`tracing::warn!`) and do not fail the compile.

### Report Directory

`samgraha audit --report` (`execute_audit` in `crates/cli/src/commands.rs`) resolves `[report].dir` via `resolve_configured_dir` and writes into a per-report-type layout:

```
<report-dir>/audit/
  latest/report.md      — overwritten every run, fixed filename
  archive/<timestamp>.md — one file per run, kept for local history
```

`latest/report.md` uses a fixed filename so it can be checked into version control (`docs/raw/reports/*/latest/report.md` is the one path explicitly un-ignored in `.gitignore`, everything else under `docs/raw/reports/` is ignored) — history then comes from `git log` on that one file rather than a bespoke mechanism. `archive/` is local-only scratch for anyone who wants every run's output on disk.

Note: `crates/services/src/reporting.rs` (`write_report`, per-section JSON snapshots with its own `latest.json`/`history/` rotation) is a separate, pre-existing mechanism for MCP semantic-audit section reports — it has no current callers outside its own tests and is unrelated to this markdown report path.

### CLI Adapter → stdout/stderr

Formatted output is written to stdout (results, data) and stderr (progress, errors, warnings).

---

## Data Ownership

| Data | Owner | CLI Access |
|---|---|---|
| Command Arguments | CLI Adapter | Transient |
| Configuration | Repository | Read |
| Compiled Knowledge | Knowledge Registry | Read via Runtime |
| Audit Results | Audit Framework | Read via Runtime |
| Output | CLI Adapter | Transient |

---

## Integration Points

This section details the Integration Points.

### Knowledge Runtime

All commands route through the Knowledge Runtime. The runtime provides the single execution entry point.

### Configuration Files

The CLI reads samgraha.toml for repository and workspace configuration. Configuration discovery follows workspace hierarchy.

### Filesystem

Compile and init commands read from and write to the filesystem. All filesystem access is validated against repository boundaries.

---

## External Dependency Integration

The CLI has no external dependencies. It is a self-contained executable operating entirely on local resources.

Optional: When configured, semantic audit and knowledge enrichment may use local AI providers. These features are never required for CLI operation.

---

## Runtime Constraints

- CLI must start within 100ms.
- CLI must output first progress indicators within 500ms.
- Help output must be available via --help on every command.
- JSON output must be available via --json flag.
- Output must respect NO_COLOR environment variable.
- CLI must handle paths with spaces, special characters, and Unicode.
- Tab completion must be supported for common shells.

---

## Architectural Constraints

- CLI must remain a single self-contained executable.
- CLI must not require external runtime dependencies.
- CLI must never require AI model availability.
- CLI must operate identically on Windows, macOS, and Linux.
- CLI must not contain engineering logic — all behavior belongs to Knowledge Services.

---

## Security Considerations

- The CLI executes with the privileges of the invoking user.
- Path traversal is prevented by repository boundary validation.
- Configuration is read from trusted repository locations.
- The CLI never reads or transmits credentials.
- All filesystem writes are restricted to configured output directories.

---

## Performance Considerations

- Command parsing must complete in under 10ms.
- Help text generation must not require loading platform components.
- Progress reporting must not significantly impact command execution time.
- JSON output must be serializable without additional processing passes.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Invalid arguments | Print error with usage information, exit code 4 |
| Configuration error | Print error with file path and line, exit code 3 |
| Compilation failure | Print errors with file paths and line numbers, exit code 1 |
| Audit failure | Print failed rules, exit code 2 |
| Internal error | Print error, exit code 5 |
| Missing command | Print available commands via help |

---

## Extension Points

Extension Points identify the interfaces and hooks where the feature can be extended with additional functionality, custom providers, or alternative implementations without modifying core code.

### Custom Commands

The CLI supports registration of additional commands through the Knowledge Service extension mechanism. New commands follow the same dispatch lifecycle.

### Output Formatters

Custom output formatters may be registered for machine-readable formats beyond JSON.

### Shell Completions

Completion scripts may be generated for additional shells without modifying the CLI adapter.

---

## Traceability

This document derives from:

- Feature: CLI Interface
- Architecture: Component Model
- Architecture: Runtime Boundary
- Architecture: Communication Architecture
- Architecture: Deployment Architecture

This document provides technical context for:

- Engineering CLI Strategy
- Engineering Distribution Strategy

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
