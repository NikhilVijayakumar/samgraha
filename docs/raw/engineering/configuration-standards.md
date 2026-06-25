# Configuration Standards

## Purpose

This document defines the configuration standards for the Saṃgraha platform.

Configuration is explicit, validated, and documented. Every configurable behavior has a documented default. Invalid configuration produces clear, actionable error messages at startup.

---

## Configuration File: samgraha.toml

The platform is configured through a single `samgraha.toml` file. The file is optional — all settings have documented defaults. When present, the file overrides defaults.

### Location Discovery

1. `--config <path>` CLI flag (explicit, highest priority)
2. `$SAMGRAHA_CONFIG` environment variable
3. Current working directory: `./samgraha.toml`
4. Repository root: `<repo-root>/samgraha.toml`
5. User config directory: `$XDG_CONFIG_HOME/samgraha/samgraha.toml` or platform equivalent

The first location found is used. No merging occurs across locations.

### Schema

```toml
[repository]
# Path to the repository root
# Default: current working directory
# root = "/path/to/repo"

[repository.ignore]
# Glob patterns for files to exclude from documentation discovery
# Default: ["**/node_modules/**", "**/target/**", "**/.git/**"]
patterns = ["**/node_modules/**", "**/target/**"]

[registry]
# Path to the Knowledge Registry database
# Default: <repository-root>/knowledge.db
# path = "/path/to/knowledge.db"

[compilation]
# Whether to enable automatic compilation on source changes
# Default: false
# watch = true

[compilation.documentation]
# Documentation standards to apply during compilation
# Default: determined by discovered standards docs
# standards = ["adr", "prd", "spec"]

[ai]
# AI provider configuration (optional)
# provider = "lms"  # "lms" | "ollama" | "openai"

[ai.lms]
# endpoint = "http://localhost:1234"
# model = "llama-3.2-3b-q4"

[ai.ollama]
# endpoint = "http://localhost:11434"
# model = "llama3.2"

[ai.openai]
# endpoint = "https://api.openai.com/v1"
# model = "gpt-4o"

[audit]
# Default severity for audit issues
# Default: "suggestion"
# default-severity = "warning"

[output]
# Default output format
# Default: "text"
# format = "json"
```

---

## Environment Variables

Environment variables provide an alternative configuration mechanism for CI and containerized environments:

| Variable | Corresponding Config | Purpose |
|---|---|---|
| `SAMGRAHA_CONFIG` | — | Path to config file |
| `SAMGRAHA_REPOSITORY_ROOT` | `repository.root` | Repository root path |
| `SAMGRAHA_REGISTRY_PATH` | `registry.path` | Registry database path |
| `SAMGRAHA_AI_PROVIDER` | `ai.provider` | AI provider selection |
| `SAMGRAHA_AI_ENDPOINT` | `ai.*.endpoint` | AI provider endpoint |
| `SAMGRAHA_AI_MODEL` | `ai.*.model` | AI model selection |
| `SAMGRAHA_OUTPUT_FORMAT` | `output.format` | Output format |
| `SAMGRAHA_LOG` | — | Log level (tracing directives) |

Environment variables take precedence over config file values. CLI flags take precedence over environment variables.

---

## Configuration Validation

Configuration is validated at startup:

1. Parse the config file (if present)
2. Apply environment variable overrides
3. Apply CLI flag overrides
4. Validate all values
5. Report errors with clear messages:

```
Error: Invalid configuration
  → samgraha.toml:12:5 — unknown field "reposotory", did you mean "repository"?
  → samgraha.toml:15:10 — invalid value "frob" for "compilation.documentation.standards[2]", expected one of: "adr", "prd", "spec"
```

Validation errors are fatal. The platform does not start with invalid configuration.

---

## Configuration Schema Documentation

The full configuration schema is documented in the user-facing documentation. Each field includes:

- Type
- Default value
- Description
- Example

The schema is maintained synchronously with the configuration parsing code. Schema generation from code is preferred over hand-maintained schema documentation.

---

## Dogfooding

The platform's own configuration uses the same samgraha.toml it manages. The repository root `samgraha.toml` documents the platform's own documentation standards.

This ensures the platform eats its own dogfood — configuration documentation and validation are exercised by the platform's own development workflow.

---

## Traceability

This document derives from:

- Engineering Principles (explicit configuration)
- Architecture: System Overview
- Technology Selection

Configuration Standards provide the framework for:

- Repository Structure
- CLI Design

Traceability:

```
Engineering Principles → Configuration Standards → Implementation
```
