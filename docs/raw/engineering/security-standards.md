# Security Standards

This section details the Security Standards.

## Purpose

This document defines the security engineering standards for the Saṃgraha platform.

Security is a platform property, not a feature. Path traversal, repository isolation, and knowledge integrity are enforced at the architectural level. Security-sensitive operations are validated at component boundaries. The platform operates on local files only — remote interaction is optional and isolated.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Threat Model

This section details the Threat Model.

### In Scope

| Threat | Mitigation |
|---|---|
| Path traversal via crafted documentation paths | Path validation at every component boundary |
| Knowledge registry corruption | Content hashing, integrity checks, WAL mode |
| Malicious knowledge package import | Hash verification, format validation |
| Unauthorized access to compiled knowledge | Repository-scoped registry, filesystem permissions |
| AI provider prompt injection | Isolated provider abstraction, input sanitization |
| Supply chain via dependencies | cargo audit, license verification, dependency review |

### Out of Scope

| Threat | Rationale |
|---|---|
| Physical access to the machine | OS-level security, disk encryption |
| Network-level attacks | Platform operates locally by default |
| Memory analysis | Standard OS process isolation |
| Side-channel attacks | Not relevant for a developer documentation tool |

---

## Path Validation

All filesystem operations validate paths against the repository root:

```rust
// Paths must be within the repository root
fn validate_path(path: &Path, root: &Path) -> Result<CanonicalPath> {
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(root) {
        return Err(anyhow!("Path {} is outside repository root {}", 
            canonical.display(), root.display()));
    }
    Ok(CanonicalPath(canonical))
}
```

- Paths are canonicalized before any filesystem operation
- Symlink traversal is permitted only within the repository
- Archive extraction (knowledge packages) validates all paths against the extraction target
- Glob patterns in configuration do not escape the repository root

---

## Repository Isolation

- The Knowledge Registry is scoped to a single repository
- Cross-repository queries require explicit import
- Registry database is stored within the repository
- Knowledge Package import validates repository origin
- Repository identity is derived from the repository root path

---

## Input Validation

This section details the Input Validation.

### Configuration

- Config file is validated against the defined schema
- Unknown fields produce errors (deny_unknown_fields)
- Path values are validated as existing or creatable
- Enum values are validated against allowed variants

### CLI Arguments

- Path arguments are validated and canonicalized
- String arguments are length-limited (reasonable limits per argument)
- Arguments are parsed through clap's type system

### MCP Protocol

- MCP message sizes are limited (configurable, default 1MB)
- MCP message format is validated against the protocol schema
- Unrecognized message types are rejected

---

## Secure Defaults

- AI providers are disabled by default — explicit opt-in required
- File watching is disabled by default
- Network access is never initiated without explicit configuration
- Verbose output is disabled by default
- JSON output does not include environment variables or system information

---

## Secrets Management

- API keys are configured through environment variables, not config files
- Environment variables are never exposed in error messages
- Environment variables are never exposed in JSON output
- Environment variables are never included in Knowledge Packages
- Secrets are not logged

---

## Dependency Security

- `cargo audit` runs in CI on every commit
- `cargo deny` verifies license compatibility
- Dependencies with known advisories are upgraded immediately
- Transitive dependencies are reviewed for supply chain risk
- Vendored dependencies are pinned and verified

---

## Code Security

- No unsafe code without explicit justification and audit
- Unsafe blocks are documented with safety invariants
- Format strings are never constructed from user input
- Command injection is not possible — the platform does not execute shell commands
- Temporary files use secure permissions (0600 for files)
- The `unwrap()` and `expect()` calls are permitted only when failure is impossible (proven by invariants)

---

## Traceability

This document derives from:

- Architecture: Security Architecture
- Engineering Principles (secure by default)
- Dependency Standards

Security Standards provide the framework for:

- Registry Implementation
- Provider Integration
- CLI Design

## Build Standards

Security-sensitive builds apply hardening flags and dependency audit checks. Build pipeline verifies no known-vulnerability dependencies are included. See [Build Standards](build-standards.md) for security build configuration.

## Testing Standards

Security is verified through dependency audits, fuzzing, and penetration testing. Security tests validate authentication, authorization, and data isolation guarantees. See [Testing Standards](testing-standards.md) for security testing methodology.

Traceability:

```
Architecture → Engineering Principles → Security Standards → Implementation
```
