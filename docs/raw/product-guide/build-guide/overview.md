# Build Guide Overview

## Purpose

How the Samgraha build process works and how to build from source.

## Content

### Build Process

```
Source code (Rust)  →  cargo build --release --bin mcp --bin cli  →  cli.exe + mcp.exe
                                          │
Raw markdown docs   →  cli.exe compile    │
(help)                                  │
        │                                  │
        └─────────────┬────────────────────┘
                      │
             Release package (samgraha/)
             ├── bin/
             │   ├── cli.exe
             │   └── mcp.exe
             ├── docs/raw/          (standards, audit, audit-standards)
             ├── .samgraha/
             ├── samgraha.toml
             ├── run-mcp.cmd / run-mcp.sh
             ├── knowledge.db       (empty schema — register your Knowledge System)
             ├── help.db            (compiled product help)
             └── SHA256SUMS
```

### Building from Source

```bash
git clone https://github.com/your-org/samgraha.git
cd samgraha
cargo build --release --bin mcp --bin cli
```

### Release Build

```bash
./scripts/build-release.ps1
```

This script:
1. Builds `cli.exe` and `mcp.exe` in release mode.
2. Packages them into `<OUTPUT_DIR>/samgraha/` along with config, universal docs (`standards`, `audit`, `audit-standards`), and an empty `.samgraha/`.
3. Compiles `docs/raw/product-guide` into `help.db` using the just-packaged `cli.exe`.
4. Creates an empty `knowledge.db` (schema only) and copies the system scripts for Knowledge System registration.
5. Writes `run-mcp.cmd`/`run-mcp.sh` launchers and a `SHA256SUMS` file.

`.env` at the repo root (`SAMGRAHA_EXPIRY_DAYS`, `SAMGRAHA_EXPIRY_HOURS`, `OUTPUT_DIR`) is the single source of truth for these settings — there are no CLI override flags. The script stops on any build/compile failure; a missing `docs/raw/product-guide` source directory just skips that step with a warning.

## Related

- [Release Checklist](release.md)
- [Packaging](packaging.md)
- [Distribution](distribution.md)
