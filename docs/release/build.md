# Build

How to produce a standalone, distributable Samgraha release — the
compiled `mcp`/`cli` binaries plus what they need at the destination
machine. Not about `samgraha.toml`'s runtime configuration (see
`knowledge-standard.md` and `repository-registration.md` for that) —
this is specifically the release-packaging pipeline.

## Purpose

A release is a portable distribution that runs on any Windows or Linux
machine without a Rust toolchain or source checkout: the compiled MCP
server, the CLI, `samgraha.toml`, and a reference copy of the SQL schema.

Releases are time-locked: the MCP binary embeds an expiry date at build
time and refuses to serve requests past it. This forces periodic
rebuilds rather than an installation drifting indefinitely from the
source it was built from.

**Not bundled**: a pre-compiled knowledge base. `knowledge.db` is
created fresh, on demand, by whichever repo the binary runs against —
`register_standard`/`activate_standard` create and migrate it the first
time a standard is registered there (`registry::core_schema::ensure_current_schema`,
called from `register_standard.rs`/`step_execution.rs` on every open). A
release ships an empty `.samgraha/` directory, not a `.db` file.

## Two separate env-reading mechanisms — don't conflate them

1. **`.env` at the repo root** — read only by `crates/mcp/build.rs`, only
   at compile time, for exactly the two keys below plus `OUTPUT_DIR`
   (read by the build *scripts*, not `build.rs`). Has nothing to do with
   `samgraha.toml`.
2. **Process environment variables** consulted at runtime by
   `samgraha.toml`'s own `${VAR}`-pattern fields (`samgraha_dir`,
   `report.dir`, etc.) — see
   `docs/proposal/samgraha-toml-configuration-contract-proposal.md` for
   the full rule. Unrelated to `.env`; nothing here reads that file.

This section is about mechanism 1 only.

## Configuration (`.env`)

The build scripts accept no CLI arguments — `.env` is the single source
of truth for a build.

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

See `.env.example` for a template — it also documents the *runtime*
`samgraha.toml` env vars (`SAMGRAHA_DIR`, `SAMGRAHA_DOCS_DIR`, etc.) in
the same file for convenience, even though those are mechanism 2, not
read by anything in this build pipeline.

### Expiry arithmetic

Expiry = build timestamp + (`EXPIRY_DAYS` × 24h) + max(`EXPIRY_HOURS`, 0) × 1h

| `EXPIRY_DAYS` | `EXPIRY_HOURS` | Result |
|---------------|----------------|--------|
| `30` | `0` | 30 days from build |
| `30` | `12` | 30 days 12 hours from build |
| `30` | `-1` | 30 days from build (hours ignored) |
| `-1` | *(any)* | No expiry — binary never expires |

## Running a build

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

Both scripts: read `.env` for `OUTPUT_DIR`/expiry (falling back to
`.\release` with a warning if `OUTPUT_DIR` is unset), run
`cargo build --release --bin mcp --bin cli`, then assemble the package
directory described below.

## Time Lock

`crates/mcp/build.rs` reads `SAMGRAHA_EXPIRY_DAYS` and
`SAMGRAHA_EXPIRY_HOURS` from `.env` at compile time (a hand-rolled
`key=value` line scan, not a full `.env` parser — `build.rs:13-24`),
computes the RFC 3339 expiry timestamp, and bakes it into the binary via
`cargo:rustc-env=SAMGRAHA_EXPIRY=<value>`. It also emits
`cargo:rerun-if-changed` for `.env`, so cargo re-runs the build script
whenever `.env` changes — without this, a changed expiry would be
silently ignored on an incremental build.

`cargo build --release` outside the scripts works the same way —
`build.rs` handles the expiry lock unconditionally; the scripts add
packaging on top.

If `SAMGRAHA_EXPIRY_DAYS=-1`, `build.rs` emits nothing and
`option_env!("SAMGRAHA_EXPIRY")` returns `None`, making `check_expiry()`
a no-op.

At startup, `check_expiry()` (`crates/mcp/src/main.rs`) compares current
UTC time against the baked-in expiry. If past expiry, it prints an error
to stderr and exits with code 1:

```
ERROR: This binary expired at 2026-10-01T23:59:59Z UTC. Build a new one.
```

To extend, update `.env` and rebuild.

## Output Structure

Verified directly against `scripts/build-release.ps1`/`.sh` (both
scripts produce the identical layout):

```
<OUTPUT_DIR>/
  samgraha/
    bin/
      mcp.exe          # MCP JSON-RPC 2.0 stdio server (mcp on Linux)
      cli.exe          # CLI tool (cli on Linux)
    .samgraha/          # empty — knowledge.db/registry.db created on first use, not shipped
    samgraha.toml       # copied verbatim from repo root at build time
    schema/
      registration/*.sql  # registry.db reference schema (registry::migration::REGISTRY_MIGRATIONS)
      knowledge/*.sql      # knowledge.db reference schema (registry::core_schema::CORE_MIGRATIONS)
      standards/*.sql      # standards.db reference schema (registry::standards_db::STANDARDS_MIGRATIONS)
    run-mcp.cmd         # Windows launcher
    run-mcp.sh          # Linux launcher
    SHA256SUMS          # SHA-256 hashes of bin/mcp.exe and bin/cli.exe
```

The `schema/*.sql` files are **not read by anything at runtime** —
`register_standard`/`run_script_step`/`activate_standard` create and
migrate every database via the inline Rust migration constants
(`CORE_MIGRATIONS`, `REGISTRY_MIGRATIONS`, `STANDARDS_MIGRATIONS`), never
by executing a `.sql` file from disk. They're shipped purely as a
human-readable reference for anyone integrating with a raw
`knowledge.db`/`registry.db`/`standards.db` file directly, without going
through the MCP server.

`metadata/*.schema.json` (the standard-metadata and proposal-envelope
JSON Schemas) are **compiled into the binary** via `include_str!`
(`crates/services/src/metadata_validate.rs`) — not shipped as separate
files, and don't need to be; there is no separate copy to keep in sync
with what the binary actually validates against.

## Requirements

- Runtime: None. The binaries are static PE/ELF with no .NET, JVM, or
  DLL dependencies. (A standard's own scripts may need an interpreter —
  Python, PowerShell, etc. — but that's the standard's requirement, not
  the release binary's.)
- Disk: ~5-10 MB for binaries + a few hundred KB for the reference SQL
  schema.
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
```

```powershell
# Windows (PowerShell)
Get-Content SHA256SUMS | ForEach-Object {
    $hash, $file = $_ -split '\s+', 2
    $actual = (Get-FileHash $file -Algorithm SHA256).Hash.ToLower()
    if ($actual -eq $hash) { "OK: $file" } else { "FAIL: $file" }
}
```
