# Installation

## Purpose

This topic covers system requirements and installation methods for Samgraha.

## Content

### System Requirements

- **OS**: Windows 10+, macOS 12+, Linux (x86-64)
- **Storage**: ~50 MB for binary + built-in knowledge databases
- **Runtime**: None required (statically linked binary)

### Download Prebuilt Binary

1. Go to the [Releases](https://github.com/your-org/samgraha/releases) page.
2. Download the archive for your platform.
3. Extract the archive to a directory on your PATH.
4. Verify installation:

```bash
samgraha --version
```

### Build from Source

Requires Rust 1.85+ (see `rust-version` in the workspace `Cargo.toml`):

```bash
git clone https://github.com/your-org/samgraha.git
cd samgraha
cargo build --release
./target/release/cli --version
```

### Directory Layout

The workspace produces two binaries — `cli` (the `samgraha` CLI) and `mcp` (the MCP server) — named `cli.exe`/`mcp.exe` on Windows. `standards.db` and `help.db` (the built-in standards reference and product help, compiled separately) must sit next to whichever binary you run; they're loaded automatically from the binary's own directory at startup, and silently skipped if missing:

```
<install-dir>/
├── cli(.exe)             # CLI binary
├── mcp(.exe)             # MCP server binary
├── standards.db          # Built-in standards reference
└── help.db               # Built-in help documentation
```

## Related

- [Environment Setup](environment.md)
- [Initialization](initialization.md)
- [First Project](first-project.md)
