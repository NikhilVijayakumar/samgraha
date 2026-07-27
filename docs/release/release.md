# Release

This section details the Release.

## Purpose

A Release is a standalone, portable distribution of Samgraha that runs on any Windows or Linux machine without a Rust toolchain or source checkout. It bundles the compiled MCP server, CLI, documentation corpus, and pre-compiled knowledge base into a single directory.

Releases are time-locked: the MCP binary embeds an expiry date at build time and refuses to serve requests past that date. This ensures consumers rebuild periodically and do not operate on stale knowledge bases.

## Configuration

All build settings live in `.env` at project root. The build scripts accept no CLI arguments — `.env` is the single source of truth.

| Key | Default | Description |
|-----|---------|-------------|
| `SAMGRAHA_EXPIRY_DAYS` | `30` | Days from build time until binary expires. `-1` = no expiry. |
| `SAMGRAHA_EXPIRY_HOURS` | `0` | Additional hours on top of `EXPIRY_DAYS`. `-1` = treat as 0. |
| `OUTPUT_DIR` | *(required)* | Absolute path for the release package. Use absolute — `.env` is machine-specific. |

Example `.env`:

```env
# 30 days and 12 hours from build time
SAMGRAHA_EXPIRY_DAYS=30
SAMGRAHA_EXPIRY_HOURS=12
OUTPUT_DIR=C:\releases\samgraha
```

```env
# Build that never expires
SAMGRAHA_EXPIRY_DAYS=-1
OUTPUT_DIR=C:\releases\samgraha
```

See `.env.example` for a template.

### Expiry arithmetic

Expiry = build timestamp + (`EXPIRY_DAYS` × 24h) + max(`EXPIRY_HOURS`, 0) × 1h

| `EXPIRY_DAYS` | `EXPIRY_HOURS` | Result |
|---------------|----------------|--------|
| `30` | `0` | 30 days from build |
| `30` | `12` | 30 days 12 hours from build |
| `30` | `-1` | 30 days from build (hours ignored) |
| `-1` | *(any)* | No expiry — binary never expires |

## Build

Two build scripts, same logic, no arguments:

| Platform | Script |
|----------|--------|
| Windows  | `scripts\build-release.ps1` |
| Linux    | `scripts/build-release.sh` |

```powershell
# Windows — edit .env first, then run:
.\scripts\build-release.ps1
```

```sh
# Linux — edit .env first, then run:
./scripts/build-release.sh
```

## Time Lock

`crates/mcp/build.rs` reads `SAMGRAHA_EXPIRY_DAYS` and `SAMGRAHA_EXPIRY_HOURS` from `.env` at compile time, computes the RFC 3339 expiry timestamp, and bakes it into the binary via `cargo:rustc-env=SAMGRAHA_EXPIRY=<value>`. It also emits `cargo:rerun-if-changed` for `.env`, so cargo re-runs the build script whenever `.env` is modified — without this, a changed expiry would be silently ignored.

`cargo build --release` outside the scripts works the same way — `build.rs` handles it entirely.

If `SAMGRAHA_EXPIRY_DAYS=-1`, `build.rs` emits nothing and `option_env!("SAMGRAHA_EXPIRY")` returns `None`, making `check_expiry()` a no-op.

At startup, `check_expiry()` (`crates/mcp/src/main.rs`) compares current UTC time against the baked-in expiry. If past expiry, it prints an error to stderr and exits with code 1:

```
ERROR: This binary expired at 2026-10-01T23:59:59Z UTC. Build a new one.
```

To extend, update `.env` and rebuild.

## Output Structure

```
<OUTPUT_DIR>/
  samgraha/
    bin/
      mcp.exe          # MCP JSON-RPC 2.0 stdio server (mcp on Linux)
      cli.exe          # CLI tool — compile, search, audit, etc. (cli on Linux)
    docs/raw/           # Documentation corpus + audit knowledge files
    .samgraha/
      knowledge.db     # Pre-compiled knowledge base (SQLite)
    samgraha.toml       # Project configuration
    run-mcp.cmd         # Windows launcher
    run-mcp.sh          # Linux launcher
    SHA256SUMS          # SHA-256 hashes of bin/mcp and bin/cli
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

## Verifying Checksums

```sh
# Linux
sha256sum -c SHA256SUMS

# Windows (PowerShell)
Get-Content SHA256SUMS | ForEach-Object {
    $hash, $file = $_ -split '\s+', 2
    $actual = (Get-FileHash $file -Algorithm SHA256).Hash.ToLower()
    if ($actual -eq $hash) { "OK: $file" } else { "FAIL: $file" }
}
```
