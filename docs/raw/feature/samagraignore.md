# SamagraIgnore

This section details SamagraIgnore.

## Purpose

SamagraIgnore provides configurable file and directory exclusion for the documentation compilation pipeline.

Users specify exclusion patterns in `.samagraignore` at repository root, in `samgraha.toml [repository.ignore]`, or both. Patterns use `.gitignore`-style glob syntax. The DiscoveryEngine applies the merged pattern set during markdown file collection.

This feature replaces hardcoded exclusion lists in the compiler with a fully configurable, user-controlled mechanism.

---

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Config-Based Ignore Patterns

`samgraha.toml` shall accept a `[repository.ignore]` section with a `patterns` field.

```toml
[repository.ignore]
patterns = [
  "**/node_modules/**",
  "**/target/**",
  "docs/draft/**",
]
```

Patterns use `.gitignore`-style glob syntax.

Default patterns always apply: `**/node_modules/**`, `**/target/**`, `**/.git/**`, `**/audit-standards/**`.

---

## FR2. File-Based Ignore

A `.samagraignore` file at the repository root shall be recognized as an additional exclusion source.

Each non-comment, non-empty line is treated as a glob pattern.

Comments begin with `#`. Blank lines are ignored.

Example:

```
# Exclude draft documentation
docs/draft/**

# Exclude generated output
**/generated/**
```

An absent or empty `.samagraignore` is valid and equivalent to no file-based patterns.

---

## FR3. Pattern Merging

Patterns from `samgraha.toml [repository.ignore]` and `.samagraignore` are merged into a single `IgnoreConfig`.

Both sets are applied during discovery. Exclusion is OR logic: a file is excluded if it matches any pattern from either source.

There is no precedence conflict between sources. A pattern in one source does not override or suppress a pattern in another.

---

## FR4. Remove Hardcoded Exclusions

`discovery.rs` shall contain no hardcoded exclusion lists.

All exclusions shall flow through `IgnoreConfig`, which is populated from config-based patterns, file-based patterns, and built-in defaults.

The built-in defaults (node_modules, target, .git, audit-standards) are seeded into `IgnoreConfig` at construction and are not present as literal strings in pipeline or discovery logic.

---

## FR5. Glob Syntax

Patterns shall use glob-style matching with the following supported forms:

* `**/node_modules/**` — exclude any `node_modules` directory at any depth
* `docs/draft/**` — exclude a specific subtree relative to repository root
* `*.tmp` — exclude files by extension at any depth when prefixed with `**/`
* `build/` — exclude a named directory at repository root

Patterns apply to paths relative to the repository root.

---

## FR6. Discovery Integration

The DiscoveryEngine shall receive the merged `IgnoreConfig` and apply it during markdown file collection.

A file whose relative path matches any pattern in `IgnoreConfig` shall be excluded from the compilation input set.

Discovery shall correctly handle glob patterns at any path depth.

---

## Business Rules

* An absent `.samagraignore` is valid. Its absence is equivalent to no file-based patterns.
* Comment lines begin with `#` and are not treated as patterns.
* Patterns apply to relative paths from the repository root.
* Default patterns always apply. They cannot be removed by omitting them from config.
* `audit-standards` is always excluded via the built-in default pattern `**/audit-standards/**`.
* Negation patterns (`!include`) are not supported in the initial implementation.

---

## Ignore Resolution Lifecycle

```text
samgraha.toml                  .samagraignore
[repository.ignore]            (optional file)
        │                            │
        ▼                            ▼
  Config Patterns            File Patterns
        │                            │
        └──────────┬─────────────────┘
                   │
                   ▼
            IgnoreConfig
        (defaults + merged)
                   │
                   ▼
          DiscoveryEngine
                   │
                   ▼
     Filtered Markdown File Set
                   │
                   ▼
       Compilation Pipeline
```

---

## Inputs

SamagraIgnore consumes:

* `samgraha.toml [repository.ignore].patterns` — config-sourced glob patterns
* `.samagraignore` at repository root — file-sourced glob patterns (optional)
* built-in default patterns seeded at `IgnoreConfig` construction

---

## Outputs

SamagraIgnore produces:

* a merged `IgnoreConfig` passed to the DiscoveryEngine
* a filtered set of markdown files as input to the compilation pipeline

---

## Constraints

SamagraIgnore shall:

* apply patterns only during discovery; registry queries, search, and audit are unaffected
* support glob patterns at any path depth
* not support negation patterns (`!`) in the initial implementation
* treat the built-in defaults as non-removable
* not require a `.samagraignore` file to be present

---

## Dependencies

SamagraIgnore depends upon:

* `config.rs` — `IgnoreConfig` struct and deserialization from `samgraha.toml`
* `discovery.rs` — DiscoveryEngine that applies `IgnoreConfig` during file collection
* `compilation.rs` — CompilationService that receives the filtered file set from discovery

---

## Non-Goals

SamagraIgnore does not:

* apply exclusion patterns to registry queries, search results, or audit output
* support negation patterns (`!include`) in the initial implementation
* provide a global ignore file shared across all repositories
* affect which repositories are registered or resolved

Those capabilities are deferred or belong to other platform components.

---

## Future Extensions

* Negation pattern support (`!include`) for opt-in within an excluded subtree.
* Global ignore file at the workspace level, applied across all registered repositories.
* `cli check-ignore <path>` command to diagnose why a file is or is not excluded.
* Validation on load: warn on patterns that match no files.

---

## Acceptance Criteria

The feature is successful when:

* files matching patterns in `samgraha.toml [repository.ignore]` are excluded from compilation
* files matching patterns in `.samagraignore` are excluded from compilation
* removing hardcoded exclusions does not break `audit-standards` exclusion, which moves to `IgnoreConfig` defaults
* discovery correctly handles glob patterns matching at any path depth
* an absent `.samagraignore` produces no error and no change in behavior
* `discovery.rs` contains no hardcoded exclusion strings

---

## Traceability

This feature derives from the following Vision commitments:

* **Configuration over convention. Users control what the platform sees.**
* **The compilation pipeline is deterministic and auditable.**
* **Hardcoded behavior is a maintenance liability. Configurable behavior is a platform feature.**

**Traceability**

Vision → Feature: SamagraIgnore
