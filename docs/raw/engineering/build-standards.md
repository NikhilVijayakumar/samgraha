# Build Standards

This section details the Build Standards.

## Purpose

This document defines the build system standards for the Saṃgraha platform.

Build standards ensure deterministic, reproducible compilation across environments. The build process is explicit and configurable. Every build profile is documented and justified.

---

## Engineering Principles

The project follows a set of core engineering principles including Documentation First, Architecture First, Deterministic by Default, Offline First, Local First, Minimal Dependencies, Explicit Configuration, Fail Fast, Secure by Default, Observable Systems, Progressive Enhancement, and Repository Isolation. See [Engineering Principles](engineering-principles.md) for the full description of each principle and the decision framework.

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Build Profiles

This section details the Build Profiles.

### Development (`cargo build` / `cargo run`)

- Debug profile
- Fast compilation, unoptimized
- Full debug information
- Used for day-to-day development
- All default features enabled

### Release (`cargo build --release`)

- Release profile
- Optimized for binary size (opt-level = "z")
- LTO enabled (thin)
- Codegen-units = 1 for maximum optimization
- Debug symbols stripped
- Single, self-contained executable for distribution

Rationale for size optimization: Saṃgraha is a developer tool distributed as a single binary. Small binary size reduces download time, installation friction, and CI pipeline overhead. Performance is not throughput-critical — the platform processes documentation collections on the order of seconds, not sub-millisecond.

### CI (`cargo build --profile ci`)

Custom profile defined in Cargo.toml:

```toml
[profile.ci]
inherits = "release"
debug = 0
incremental = false
strip = "symbols"
```

Removes debuginfo and disables incremental compilation for fastest CI build times.

---

## Feature Flags

Features enable optional capabilities without changing the core platform:

| Feature | Default | Description |
|---|---|---|
| `mcp` | yes | MCP protocol adapter |
| `ai` | no | AI provider integrations |
| `watch` | no | File system watching for incremental compilation |
| `static-sqlite` | yes | Statically link SQLite |

Features are additive — enabling a feature must never disable or alter core behavior. Core functionality is always available.

---

## Build Configuration (Cargo.toml)

Root workspace Cargo.toml defines:

- Workspace members
- Shared dependency versions
- Profile configurations
- Metadata for packaging

Individual crate Cargo.toml defines:

- Crate dependencies (referencing workspace where possible)
- Crate features
- Crate metadata (description, authors, license)

---

## Pre-commit Checks

Before committing, run:

```
cargo check      # Type check only (fast)
cargo test       # Unit and integration tests
cargo clippy     # Lint checks
cargo fmt        # Formatting (must be applied)
cargo audit      # Security advisories
```

These checks also run in CI. CI must pass before merge.

---

## CI Pipeline

CI runs on every push to all branches:

1. `cargo check` — type check
2. `cargo fmt --check` — formatting verification
3. `cargo clippy` — lint
4. `cargo test` — tests
5. `cargo build --profile ci` — release build
6. `cargo audit` — security audit
7. `cargo deny check licenses` — license verification

CI artifacts:

- Compiled binary for the target platform
- Test results (JUnit format)
- Audit report
- Lint report

Platforms: windows-latest, ubuntu-latest, macos-latest.

---

## Deterministic Builds

- Cargo.lock is committed to version control
- MSRV is documented and verified in CI
- Builds must not depend on network access for compilation (cargo vendor for offline)
- Builds must not embed timestamps or environment-dependent metadata

---

## Workspace Compilation

The workspace uses `--workspace` flag to compile all crates together:

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

Individual crate compilation is also supported for faster iteration:

```
cargo build -p samgraha-compiler
```

---

## Compilation Output Artifacts

Knowledge compilation produces two distinct artifacts.

| Artifact | Location | Consumer |
|---|---|---|
| Compiled knowledge database | `.samgraha/knowledge.db` | Knowledge Registry |
| Repository manifest | `.samgraha/manifest.json` | Repository Registry |

Both artifacts are generated on successful compilation. Failed compilations do not update either artifact.

The manifest contains only repository metadata — identity, revision, capabilities, exports, dependencies. It never contains engineering knowledge.

The manifest is written in JSON format (see Repository Registry Architecture for schema).

---

## Traceability

This document derives from:

- Engineering Principles (deterministic by default, offline first)
- Technology Selection
- Repository Structure
- Repository Registry Architecture

Build Standards provide the framework for:

- CI/CD Pipeline
- Packaging Standards
- Repository Registry Implementation

## Testing Standards

Build correctness is verified through CI pipeline tests covering compilation, artifact generation, and profile-specific behavior. See [Testing Standards](testing-standards.md) for the testing framework.

Traceability:

```
Engineering Principles → Build Standards → Repository Registry → Implementation
```
