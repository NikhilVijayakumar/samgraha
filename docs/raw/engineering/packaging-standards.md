# Packaging Standards

## Purpose

This document defines the packaging and distribution standards for the Saṃgraha platform.

The platform is distributed as a single self-contained executable. No runtime dependencies, interpreters, or package managers are required. The executable is the primary distribution artifact.

---

## Distribution Artifacts

### Primary: Single Executable

```
samgraha.exe    (Windows, x86-64)
samgraha        (Linux, x86-64)
samgraha        (macOS, x86-64)
samgraha        (macOS, ARM64)
```

The executable is statically linked and self-contained. It includes:
- All platform code
- All Rust standard library code
- Static SQLite library
- C runtime (statically linked on Linux, bundled on macOS and Windows)

### Secondary: Archive

```
samgraha-x86_64-pc-windows-msvc.zip
samgraha-x86_64-unknown-linux-gnu.tar.gz
samgraha-x86_64-apple-darwin.tar.gz
samgraha-aarch64-apple-darwin.tar.gz
```

Archive contents:

```
samgraha/
├── samgraha{,.exe}
├── LICENSE
├── README.md
└── completions/
    ├── samgraha.bash
    ├── samgraha.zsh
    └── samgraha.powershell
```

### Package Manager Distribution (Future)

- `cargo install samgraha` — crates.io
- `winget install samgraha` — Windows
- `brew install samgraha` — macOS
- Nix package — Linux (NixOS)

---

## Build Process

### Release Build

```
cargo build --release
```

Build configuration (Cargo.toml):

```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = "thin"          # Link-time optimization
codegen-units = 1     # Maximum optimization opportunity
strip = true          # Strip debug symbols
panic = "abort"       # Abort on panic (smaller binary)
```

Rationale for size optimization: Developer tools are installed frequently — in CI pipelines, on new machines, in container images. Small binary reduces friction. Performance is not throughput-critical.

### Cross-Compilation

Cross-compilation for target platforms uses:

```
cargo build --release --target <target-triple>
```

Cross-compilation toolchains are documented in CONTRIBUTING.md. CI uses native builders where possible.

### CI Artifacts

CI produces:
1. Compiled binary for each target platform
2. Archive with completions and license
3. SHA-256 checksum file
4. SBOM (Software Bill of Materials)

---

## Versioning

Versioning follows Semantic Versioning 2.0.0:

- **MAJOR**: Breaking changes to CLI interface or MCP protocol
- **MINOR**: New features, backward-compatible
- **PATCH**: Bug fixes, performance improvements

Pre-release versions use: `-alpha.N`, `-beta.N`, `-rc.N`

Version is defined in a single source:

```
cli/Cargo.toml  →  version = "0.1.0"
```

All crates share the same version. Individual crate versioning is not used.

---

## Release Process

1. Version bump in cli/Cargo.toml
2. Changelog update
3. Tag: `v<version>` (e.g., `v0.1.0`)
4. CI build and test across all targets
5. CI produces release artifacts
6. GitHub Release with artifacts and changelog
7. Published to crates.io (if crate-based distribution)
8. Published to package managers (if applicable)

Release checklist:

- [ ] All tests pass on all platforms
- [ ] cargo audit passes
- [ ] Documentation is up to date
- [ ] Changelog is current
- [ ] Version is bumped
- [ ] Tag is created
- [ ] Release artifacts are verified (checksums match)

---

## Minimum Supported Rust Version (MSRV)

MSRV is documented in Cargo.toml:

```toml
[package]
rust-version = "1.85"
```

MSRV is verified in CI. MSRV bumps are MINOR version changes and are documented in changelog.

---

## Traceability

This document derives from:

- Engineering Principles (offline first, deterministic by default)
- Build Standards
- Technology Selection

Packaging Standards provide the framework for:

- CI/CD Pipeline
- Release Process

Traceability:

```
Engineering Principles → Build Standards → Packaging Standards → Distribution
```
