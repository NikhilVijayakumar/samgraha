# CLI Interface

## Purpose

Saṃgraha provides a command-line interface for all platform operations.

The CLI is the primary interaction point for human engineers. It exposes compilation, search, audit, and information commands through a consistent, discoverable interface. Every CLI command operates offline by default and requires no AI model.

The CLI is a single, self-contained executable with no external runtime dependencies.

---

## Functional Requirements

## FR1: Compile Command

`samgraha compile [path]` shall:

- discover and compile documentation in the specified repository path
- respect `samgraha.toml` configuration if present
- produce `knowledge.db` at the configured output path
- report compilation statistics (documents processed, chunks generated, errors, warnings)
- exit with non-zero code on compilation failure

If no path is specified, the current directory shall be used.

## FR2: Search Command

`samgraha search <query> [options]` shall:

- search the compiled knowledge index at the specified or discovered path
- support flags for domain filtering (`--domain`), status filtering (`--status`)
- support progressive retrieval level (`--level metadata|summary|section|full`)
- display results in a formatted table (title, domain, score, snippet)
- exit with non-zero code if no index exists

## FR3: Audit Command

`samgraha audit [domain] [options]` shall:

- run deterministic audit checks against documentation standards
- support single-domain audit (`samgraha audit vision`)
- support full-suite audit (`samgraha audit --all`)
- produce structured audit output (pass/fail per rule, summary)
- exit with non-zero code on any audit failure

## FR4: Info Command

`samgraha info [path]` shall:

- display repository configuration
- list discovered documentation domains
- show compilation status and last audit date
- display dependency declarations

## FR5: Init Command

`samgraha init [path]` shall:

- create a `samgraha.toml` with default configuration
- create the expected documentation directory structure
- provide a starting template for new repositories

## FR6: Progress Reporting

All commands shall report progress information:

- compilation progress (documents processed / total)
- audit progress (checks completed / total)
- error locations with file path and line number
- warnings without interrupting execution

## FR7: Exit Codes

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

## Success Criteria

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

## Traceability

This feature derives from the following Vision commitments:

- "Knowledge Services execute engineering workflows using Documentation Standards."
- "Offline First — Compilation and Knowledge Services should operate without requiring cloud infrastructure."
- "Deterministic Engineering — The same documentation should always produce the same engineering knowledge."

Traceability: Vision → Feature: CLI Interface
