# Build Guide Overview

## Purpose

How the Samgraha build process works and how to build from source.

## Content

### Build Process

```
Source code (Rust)  →  cargo build --release --bin mcp --bin cli  →  cli.exe + mcp.exe
                                          │
Raw markdown docs   →  cli.exe compile    │
(help + standards)                        │
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
             ├── standards.db
             ├── help.db
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
3. Compiles `docs/raw/standards` and `docs/raw/help` using the just-packaged `cli.exe`, and copies the resulting knowledge databases out as `standards.db`/`help.db` in the package root.
4. Writes `run-mcp.cmd`/`run-mcp.sh` launchers and a `SHA256SUMS` file.

`.env` at the repo root (`SAMGRAHA_EXPIRY_DAYS`, `SAMGRAHA_EXPIRY_HOURS`, `OUTPUT_DIR`) is the single source of truth for these settings — there are no CLI override flags. The script stops on any build/compile failure; a missing `docs/raw/standards` or `docs/raw/help` source directory just skips that step with a warning.

## Related

- [Release Checklist](release.md)
- [Packaging](packaging.md)
- [Distribution](distribution.md)
