# Distribution

## Purpose

How Samgraha binaries and built-in knowledge are distributed.

## Content

### Distribution Package

`scripts/build-release.ps1` produces a `samgraha/` package directory containing:

```
samgraha/
├── bin/
│   ├── cli.exe                # CLI binary
│   └── mcp.exe                # MCP server binary
├── docs/raw/                  # standards, audit, audit-standards (universal docs)
├── .samgraha/                 # empty, for the package's own use
├── samgraha.toml
├── run-mcp.cmd / run-mcp.sh   # launcher scripts — the recommended way to start mcp.exe
├── knowledge.db                # empty schema — users register their Knowledge System
├── help.db                     # compiled built-in product help
├── knowledge-system-scripts/    # system scripts for Knowledge System registration
└── SHA256SUMS                  # checksums for cli.exe and mcp.exe
```

`knowledge.db` and `help.db` sit at the package root, next to `bin/` — not inside `bin/` and not inside `.samgraha/`.

### Platform Support

The build script as it stands targets Windows (produces `.exe` binaries and `.cmd`/`.sh` launchers side by side). Building on macOS/Linux via `cargo build --release --bin mcp --bin cli` produces unsuffixed `cli`/`mcp` binaries; there is no separate cross-platform packaging script yet.

### Installation

Users extract/copy the `samgraha/` package directory and run `bin/cli.exe` / `bin/mcp.exe` directly, or use the `run-mcp.cmd`/`run-mcp.sh` launcher for the MCP server.

### Built-in Knowledge Distribution

`help.db` is compiled during the release build (from `docs/raw/product-guide`) and placed in the package root. `knowledge.db` ships with an empty schema — no system rows, no domains, no rules. Users register their Knowledge System with `standards register`, which creates local data and syncs to the global store.

At runtime, `cli.exe` and `mcp.exe` look for these files next to whichever binary is running (via `std::env::current_exe()?.parent()`). If either is missing, that's a non-fatal degraded state — `samgraha info` reports built-in store status explicitly.

## Related

- [Build Overview](overview.md)
- [Release Checklist](release.md)
- [Getting Started: Installation](../getting-started/installation.md)
