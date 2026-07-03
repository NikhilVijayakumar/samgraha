# Release

This section details the Release.

## Purpose

A Release is a standalone, portable distribution of Samgraha that runs on any Windows or Linux machine without a Rust toolchain or source checkout. It bundles the compiled MCP server, CLI, documentation corpus, and pre-compiled knowledge base into a single directory.

Releases are time-locked: the MCP binary embeds an expiry date at build time and refuses to serve requests past that date. This ensures consumers rebuild periodically and do not operate on stale knowledge bases.

## Build

`scripts\build-release.ps1` produces the release directory:

```
.\scripts\build-release.ps1 [-ExpiryDays <int>] [-OutputDir <path>]
```

| Parameter | Default | Description |
|-----------|---------|-------------|
| ExpiryDays | 30 | Days from build date until binary expires |
| OutputDir | .\release | Output path (absolute or relative to project root) |

Examples:

```
scripts/build-release.ps1                                          # 30-day expiry, .\release\
scripts/build-release.ps1 -ExpiryDays 90                           # 90-day expiry, .\release\
scripts/build-release.ps1 -ExpiryDays 60 -OutputDir D:\releases    # Custom output dir
```

## Time Lock

The expiry is baked into the binary at compile time via the `SAMGRAHA_EXPIRY` environment variable.

**`crates/mcp/src/main.rs` (check_expiry):**

The binary accepts two formats:

| Format | Example | Precision |
|--------|---------|-----------|
| RFC 3339 | 2026-10-01T23:59:59Z | Second precision, UTC |
| Date only | 2026-10-01 | End of day (23:59:59 UTC) |

At startup, `check_expiry()` compares current UTC time against the baked-in expiry. If past expiry, it prints an error to stderr and exits with code 1:

```
ERROR: This binary expired at 2026-10-01T23:59:59Z UTC. Build a new one.
```

To extend, rebuild with `build-release.ps1` and redistribute the new directory.

## Output Structure

```
release/
  samgraha/
    bin/
      mcp.exe          # MCP JSON-RPC 2.0 stdio server
      cli.exe          # CLI tool (compile, search, audit, etc.)
    docs/raw/           # Documentation corpus + 91 audit knowledge files
    .samgraha/
      knowledge.db     # Pre-compiled knowledge base (SQLite)
    samgraha.toml       # Project configuration
    run-mcp.cmd         # Windows launcher
    run-mcp.sh          # Linux launcher
```

## Requirements

- Runtime: None. The binaries are static PE/ELF with no .NET, JVM, or DLL dependencies.
- Disk: ~5-10 MB for binaries + ~20-30 MB for docs and knowledge base.
- OS: Windows 10+ or Linux (x86-64).

## Usage

Pipe JSON-RPC 2.0 requests into the launcher:

```powershell
# Windows
Get-Content request.json | .\run-mcp.cmd
# or
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | .\run-mcp.cmd
```

```sh
# Linux
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./run-mcp.sh
```

The launcher forwards stdin to `bin/mcp.exe` and prints the JSON response line to stdout.
