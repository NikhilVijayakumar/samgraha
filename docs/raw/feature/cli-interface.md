# CLI Interface

This section details the CLI Interface.

## Purpose

Saṃgraha provides a command-line interface for all platform operations.

The CLI is the primary interaction point for human engineers. It exposes compilation, search, audit, and information commands through a consistent, discoverable interface. Every CLI command operates offline by default and requires no AI model.

The CLI is a single, self-contained executable with no external runtime dependencies.

---

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Compile Command

`samgraha compile [path]` shall:

- discover and compile documentation in the specified repository path
- respect `samgraha.toml` configuration if present
- produce `knowledge.db` at the configured output path
- report compilation statistics (documents processed, chunks generated, errors, warnings)
- exit with non-zero code on compilation failure

If no path is specified, the current directory shall be used.

## FR2. Search Command

`samgraha search <query> [options]` shall:

- search the compiled knowledge index at the specified or discovered path
- support flags for domain filtering (`--domain`), status filtering (`--status`)
- support progressive retrieval level (`--level metadata|summary|section|full`)
- display results in a formatted table (title, domain, score, snippet)
- exit with non-zero code if no index exists

## FR3. Audit Command

`samgraha audit [domain] [options]` shall:

- run deterministic audit checks against documentation standards
- support single-domain audit (`samgraha audit vision`)
- support full-suite audit (`samgraha audit --all`)
- produce structured audit output (pass/fail per rule, summary)
- exit with non-zero code on any audit failure

## FR4. Info Command

`samgraha info [path]` shall:

- display repository configuration
- list discovered documentation domains
- show compilation status and last audit date
- display dependency declarations

## FR5. Init Command

`samgraha init [path]` shall:

- create a `samgraha.toml` with default configuration, if none exists yet
- declare every builtin documentation standard by default, so a new repo's toml shows the full catalog it can opt out of rather than an empty/arbitrary subset
- create the expected documentation directory structure
- generate (or update) `.env.example` alongside the config, documenting every env key samgraha recognizes
- provide a starting template for new repositories
- be safe to re-run on an already-initialized repository: if `samgraha.toml` exists, it shall not be recreated or overwritten — instead, every section/key present in the current schema but missing from the existing file shall be added, and every key already present shall be left exactly as it is (including its formatting and comments). `--force` bypasses this and fully overwrites, for the rare case an actual reset is wanted.

## FR5a. Repository Guard

Every command except `init` and `version` shall refuse to run outside a samgraha repository, the same way `git` refuses to run outside a git repository:

- before executing, walk up from the current directory looking for `.samgraha/` or `samgraha.toml`
- if neither is found in the current directory or any parent, fail with an error naming the problem and pointing to `samgraha init`
- a directory containing only a `.git/` (no samgraha markers) is not sufficient — it must be explicitly initialized with `samgraha init`
- the MCP server applies the identical guard on startup, since it resolves the repository root the same way the CLI does

## FR5b. Domain Exclusions

A repository does not necessarily use every builtin documentation standard. `samgraha.toml`'s `[repository.documentation]` shall support:

- `domain`: the declared/known domains for this repo (init populates this with the full builtin catalog)
- `domain_exclusion`: domains from that catalog this repo deliberately does not use (e.g. no `prototype` docs)

Effective, active domains = `domain` minus `domain_exclusion`. Commands that list domains (e.g. `samgraha info`) shall reflect the effective set, not the raw catalog. An empty `domain` list means "all builtin standards" (back-compat default).

## FR5c. Environment-Resolved Paths

Machine-specific absolute paths — documentation root, report output directory, implementation/source directory, and (optionally) external scripts/test directories — shall be configurable in `samgraha.toml` as `${VAR_NAME}` placeholders, resolved from the process environment at load time:

- if the env var is set, its value is used (as an absolute path, or joined to the repo root if relative)
- if unset, a sensible repo-relative default is used, so the tool works with zero env configuration on a single-machine setup
- a literal path (no `${...}`) is also accepted as-is, bypassing env resolution entirely

`samgraha env [path]` shall generate/update `.env.example` with every recognized key, blank/commented, so a repo can be cloned onto a new machine and have its per-machine values regenerated and filled in. This is additive — it must not remove or overwrite keys the repo already declared for other purposes.

On startup, before loading any config, the CLI and MCP server shall load a `.env` file (found by walking up from the current directory) into the process environment, without overriding any variable already set for real — so a checked-in `samgraha.toml` full of `${VAR}` placeholders resolves to real per-machine values just from editing `.env`, no shell export required.

## FR5d. Report Directory

`samgraha audit --report` shall write reports under a configurable base directory (`[report].dir`, env-resolvable per FR5c), with each report type in its own subdirectory:

- `<report-dir>/<type>/latest/report.md` — the most recent report (fixed filename, meant to be version-controlled so its history is `git log`, not a separate mechanism)
- `<report-dir>/<type>/archive/<timestamp>.md` — every past run, kept for local history

Default base directory: `docs/raw/reports`.

## FR5f. Optional Scripts/Tests Directories

Some repositories keep external scripts or tests outside their implementation directory (e.g. a top-level `scripts/` or `tests/` alongside `src/`). `samgraha.toml` shall support declaring these as optional, env-resolvable directories (`[repository.scripts].dir`, `[repository.tests].dir`, per FR5c). Neither is required — `init` does not populate them; a repo adds them only when applicable.

## FR5e. Dependency/Interest Auto-Registration

Declared dependencies (`[[repository.dependencies]]`) and interests (`knowledge.interests`) shall be kept registered in the local repository registry without requiring a separate manual step every time:

- a successful `compile` (CLI or MCP) automatically syncs/registers any declared dependency with a resolvable local path and manifest
- `samgraha registry register` / `samgraha registry sync` remain available as an explicit, manual path at any time, for any repository — auto-sync does not replace or restrict them
- sync/registration failures are logged as warnings and do not fail the compile

## FR6. Progress Reporting

All commands shall report progress information:

- compilation progress (documents processed / total)
- audit progress (checks completed / total)
- error locations with file path and line number
- warnings without interrupting execution

## FR7. Exit Codes

| Condition | Exit Code |
|-----------|-----------|
| Success | 0 |
| Compilation error | 1 |
| Audit failure | 2 |
| Configuration error | 3 |
| Input error | 4 |
| Internal error | 5 |

---

## Business Rules

- All commands must operate offline.
- No command shall require an AI model.
- Output must be human-readable by default.
- JSON output must be available via `--json` flag for programmatic consumption.
- Error messages must include actionable information.
- Help output must be available via `--help` on every command.
- Tab completion must be supported for common shells.

---

## Inputs

- Command-line arguments and flags
- `samgraha.toml` configuration file (optional per command)

---

## Outputs

- Formatted terminal output (tables, progress bars, summaries)
- Optional JSON output for programmatic use
- Exit codes for script integration

---

## Acceptance Criteria

- Every platform capability is exposed through the CLI.
- Commands are discoverable through `--help`.
- Common workflows require minimal flags.
- Errors provide clear, actionable messages.
- CLI operates identically on Windows, macOS, and Linux.
- Tab completion is available for common shells.

---

## Constraints

- Single self-contained executable.
- No runtime dependencies beyond the OS.
- Must respect `NO_COLOR` environment variable for non-colorized output.
- Must handle paths with spaces, special characters, and Unicode.
- Help output must not exceed 80 characters per line for readability.

---

## Dependencies

- **Knowledge Compilation** — compile command depends on compiler.
- **Knowledge Search** — search command depends on search capability.
- **Automated Audit** — audit command depends on audit capability.
- **Knowledge Registry** — info command depends on registry queries.

---

## Non-Goals

The CLI does not:

* compile documentation in the background
* monitor files for changes
* provide a graphical interface
* expose every platform capability through every command

Those responsibilities belong to other platform components.

---

## Future Extensions

The CLI may support future capabilities, including:

* watch mode for automatic recompilation
* tab completion installation command
* output format plugins
* shell integration scripts
* interactive configuration wizard

Future CLI capabilities should integrate without changing the command structure.

---

## Traceability

This feature derives from the following Vision commitments:

- "Knowledge Services execute engineering workflows using Documentation Standards."
- "Offline First — Compilation and Knowledge Services should operate without requiring cloud infrastructure."
- "Deterministic Engineering — The same documentation should always produce the same engineering knowledge."

Traceability: Vision → Feature: CLI Interface
