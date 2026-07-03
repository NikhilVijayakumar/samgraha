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
