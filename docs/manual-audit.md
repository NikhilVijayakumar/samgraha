# Saṃgraha — Manual Audit Guide

> Run from project root: `E:\Python\samgraha` (Windows) or `~/PycharmProjects/samgraha` (Ubuntu)
>
> All CLI commands: `cargo run --bin cli -- <subcommand>` (virtual workspace — must specify binary)
> Short alias: `cargo run -p cli -- <subcommand>`

---

## Automated Test Runner

Scripts are provided for both platforms:

### Windows (PowerShell)

Script: `scripts/run-tests.ps1`

```powershell
# Quick run (Phase 1a + 1b)
.\scripts\run-tests.ps1

# Full platform + multi-repo
.\scripts\run-tests.ps1 -Full

# Full platform + MCP (requires Node.js)
.\scripts\run-tests.ps1 -WithMCP

# Everything
.\scripts\run-tests.ps1 -Full -WithMCP

# Skip build step (already built)
.\scripts\run-tests.ps1 -SkipBuild
```

### Ubuntu (Bash)

Script: `scripts/run-tests.sh`

```bash
# Quick run (Phase 1a + 1b)
./scripts/run-tests.sh

# Full platform + multi-repo
./scripts/run-tests.sh --full

# Full platform + MCP (requires Node.js)
./scripts/run-tests.sh --with-mcp

# Everything
./scripts/run-tests.sh --full --with-mcp

# Skip build step (already built)
./scripts/run-tests.sh --skip-build
```

Each test prints `OK` or `XX`. Exit code = number of failures.

A markdown report is saved automatically after every run:

```
docs/report/manual-audit/YYYYMMDD-HHmmss-{mode}.md
```

The report contains a failure summary table and captured output (stdout + stderr) for each failing test. Feed the report directly to Claude Code or OpenCode to debug failures.

**Windows (PowerShell):**
```powershell
# Example: hand latest report to Claude Code
claude "here are the failing tests: $(Get-Content (Get-ChildItem docs\report\ | Sort-Object LastWriteTime -Descending | Select-Object -First 1))"
```

**Ubuntu (Bash):**
```bash
# Example: hand latest report to Claude Code
claude "here are the failing tests: $(ls -t docs/report/manual-audit/*.md | head -1 | xargs cat)"
```

---

## Prerequisites

### MCP Server Binary

MCP is a library crate with a binary entry point at `crates/mcp/src/main.rs`.

Build:

```bash
cargo build --bin mcp
```

The binary speaks **MCP JSON-RPC 2.0 over stdio**:
- Reads JSON-RPC 2.0 requests from stdin
- Writes JSON-RPC 2.0 responses to stdout
- Logs/tracing go to stderr

It wraps `McpAdapter` and maps MCP standard methods (`initialize`, `tools/list`, `tools/call`) to internal method dispatch.

### MCP Inspector

Official visual testing tool: https://github.com/modelcontextprotocol/inspector

```bash
npx @modelcontextprotocol/inspector cargo run --bin mcp
```

Starts proxy server on `localhost:6277` and opens web UI at `http://localhost:6274`.

The server must be run from a directory with compiled knowledge (`.samgraha/knowledge.db` + registered repo), otherwise it exits immediately with "Failed to open knowledge registry".

Docs: https://modelcontextprotocol.io/docs/tools/inspector

---

## Phase 1 — Platform Audit (CLI Only)

Goal: validate compiler, registry, resolver, search, audit. No MCP, no AI.

Back up config before Phase 1, restore after:

**Windows (PowerShell):**
```powershell
.\scripts\audit-phase1.ps1          # backup
# ... run Phase 1 commands ...
.\scripts\audit-phase1.ps1 -Restore # restore
```

**Ubuntu (Bash):**
```bash
./scripts/audit-phase1.sh           # backup
# ... run Phase 1 commands ...
./scripts/audit-phase1.sh --restore # restore
```

Sections 1.4 and 1.5 use `scripts/demo-dependency.ps1` / `scripts/demo-dependency.sh` which handle their own backup.

Section 1.7 uses a temp config copy (`--config`) so original TTL is preserved.

---

### 1.1 — Single Repository (Saṃgraha only)

```bash
cargo run --bin cli -- compile
```

Verify:

- `.samgraha/knowledge.db` generated.
- `.samgraha/manifest.json` generated.
- Manifest revision starts at `1`.
- Audit status recorded.
- UUID stable across recompiles.

```bash
cargo run --bin cli -- registry register
```

Verify:

- Registry contains exactly one repository.
- Metadata matches manifest.
- No documents stored in registry.

Test both modes:

- **Manual**: `compile` → manifest generated, registry empty → then `registry register`
- **Automatic** (future): `compile` → auto-sync to registry

---

### 1.2 — Registry Commands

```bash
cargo run --bin cli -- registry list
```

Expected (text mode):

```
Registered repositories (1)
------------------------------------------------------------------------
  samgraha (abc123..) -- rev 1 | audit: PASS
```

```bash
cargo run --bin cli -- registry status
```

Shows all repos with computed status (Registered / Stale / Expired / Missing).

---

### 1.3 — Resolver (single repo)

```bash
cargo run --bin cli -- registry resolve runtime
```

Expected: no dependency resolution, no registry lookup, just local `knowledge.db`.

Verifies: Resolver, Locator, Knowledge Package.

**Invariant**: Resolver never inspects Markdown. Only compiled artifacts (`knowledge.db` + metadata cache).

---

### 1.4 — Simulate Dependency

**Windows (PowerShell):**
```powershell
.\scripts\demo-dependency.ps1
```

**Ubuntu (Bash):**
```bash
./scripts/demo-dependency.sh
```

Creates temp fixture `id = "astra"`, compiles, registers, lists registry, cleans up. Use `-Keep` (PowerShell) / `--keep` (Bash) to preserve the fixture dir.

See `scripts/demo-dependency.ps1` / `scripts/demo-dependency.sh` for implementation — mirrors `New-TestFixture` pattern from the test runner scripts.

Registry now has `samgraha` (project root) + `astra`.

---

### 1.5 — Dependency Resolution

Use the demo script with the resolve flag:

**Windows (PowerShell):**
```powershell
.\scripts\demo-dependency.ps1 -Resolve
```

**Ubuntu (Bash):**
```bash
./scripts/demo-dependency.sh --resolve
```

This will:
1. Create temp fixture `id = "astra"`, compile, register
2. Add dependency to `samgraha.toml` (script uses `path` to point at compiled fixture):

```toml
[[repository.dependencies]]
name = "astra"
path = "C:/Users/.../Temp/samgraha-demo-XXXXX"
required = true
```

3. Run `cargo run --bin cli -- registry resolve runtime`

Expected: `Current → Astra → Knowledge Package`. No DB copy.

Verify generated Knowledge Package:

- Included repositories list
- Repository order (current first, then dependencies)
- No duplicated knowledge

**Cycle detection test** — simulate a dependency cycle:

```
A → B → C → A
```

Resolver must report:

```
Dependency cycle detected: A → B → C → A
```

**Missing dependency test** — add `astra` to dependencies, delete `samgraha-copy/`. Resolver must report `Repository not found` (not panic).

---

### 1.6 — Metadata Cache

```bash
# First resolve creates metadata cache
cargo run --bin cli -- registry resolve runtime
```

First run: `Registry → Metadata Cache → Resolver`

```bash
# Second resolve (immediate) hits cache
cargo run --bin cli -- registry resolve runtime
```

Expected:

```
Metadata Cache → Resolver
```

Cache hit — Registry not queried.

```bash
# Delete cache, resolve again
rm -rf .samgraha/dependencies/
cargo run --bin cli -- registry resolve runtime
```

*(Windows: `Remove-Item -Recurse -Force .samgraha/dependencies/`)*

Cache miss — Registry queried again.

---

### 1.7 — TTL Expiration

Use a temp config copy to avoid changing the real `samgraha.toml`:

**Windows (PowerShell):**
```powershell
$TtlCfg = "$env:TEMP\samgraha-ttl-test.toml"
Copy-Item samgraha.toml $TtlCfg
Add-Content $TtlCfg "`n[resolver]`nmetadata_ttl = `"5s`""
```

**Ubuntu (Bash):**
```bash
TtlCfg=/tmp/samgraha-ttl-test.toml
cp samgraha.toml "$TtlCfg"
cat >> "$TtlCfg" << EOF

[resolver]
metadata_ttl = "5s"
EOF
```

Wait 6s.

```bash
cargo run --bin cli -- --config "$TtlCfg" registry resolve runtime
```

Expected: metadata expired → registry refresh → resolver.

Cleanup:

```bash
rm "$TtlCfg"
```

*(Windows: `Remove-Item $TtlCfg -Force`)*

---

### 1.8 — Offline Mode

Break registry path. Run resolve:

- Before TTL: uses cache.
- After TTL: warning printed, stale cache used (not an error).

---

### 1.9 — Manifest Integrity

After compile, verify `.samgraha/manifest.json`:

| Field | JSON path |
|---|---|
| Repository ID | `repository.id` |
| UUID | `repository.uuid` |
| Revision | `revision` |
| Exports | `exports` |
| Capabilities | `capabilities` |
| Dependencies | `dependencies` |
| Audit Status | `audit.status` |

Compile twice without changes → revision unchanged.

Change one doc, compile → revision++.

---

### 1.10 — Knowledge DB Recovery

```bash
rm -f .samgraha/knowledge.db
cargo run --bin cli -- compile
```

*(Windows: `Remove-Item .samgraha/knowledge.db`)*

Expected: DB regenerated, manifest updated, registry synced.

---

### 1.11 — Registry Sync

Edit `.samgraha/manifest.json`. Run:

```bash
cargo run --bin cli -- registry sync
```

Expected: revision, audit, capabilities, exports update. Nothing else.

**Registry invariant**: Registry reads/writes manifest ONLY. Never touches `knowledge.db`.

---

### 1.12 — Search

Replace `<term>` with an actual query (e.g., `"repository"`, `"registry"`, `"compiler"`):

```bash
cargo run --bin cli -- search "repository"
```

Exit code `4` (InputError) means zero results — try a different term.

Architectural invariant: search reads `knowledge.db` only. Never touches registry.

---

### 1.13 — Audit

#### 1.13.1 — Basic Audit

```bash
cargo run --bin cli -- audit
```

Verify: audit runs against all domains, PASS/FAIL per category, overall readiness score printed.

#### 1.13.2 — Full Audit (all checks)

```bash
cargo run --bin cli -- audit --all
```

Verify: uses all available providers, comprehensive check output, no skipped categories.

#### 1.13.3 — Audit Gate

```bash
cargo run --bin cli -- audit --gate
```

Verify: exits with code `2` (AuditFailure) if any check fails; MCP returns no document for failed audits.

---

### 1.14 — Incremental Compile

Change one file:

```bash
cargo run --bin cli -- compile
```

Expected: only the changed file rebuilt (not full recompile).

---

### 1.15 — Dependency Removal (Cleanup)

After dependency testing, restore `samgraha.toml` to its original state (no dependencies).

If you ran `.\scripts\demo-dependency.ps1 -Resolve` / `./scripts/demo-dependency.sh --resolve`, this happens automatically — the script backs up `samgraha.toml`, patches in `[[repository.dependencies]]`, runs resolve, then restores.

Manual cleanup:

```bash
# If samgraha.toml.bak exists from a previous run:
mv samgraha.toml.bak samgraha.toml

# Otherwise, remove the [[repository.dependencies]] section:
```

*(Windows: `Move-Item samgraha.toml.bak samgraha.toml -Force`)*

Remove the added section from `samgraha.toml`:

```toml
# Delete these lines if present:
# [[repository.dependencies]]
# name = "astra"
# required = true
```

Verify no dependencies remain:

```bash
cargo run --bin cli -- registry resolve runtime
```

Should resolve with only the current repo.

---

## Phase 2 — MCP Binary + Inspector (Functional Audit)

Goal: validate MCP server works. No AI.

---

### 2.1 — Build MCP binary

```bash
cargo build --bin mcp
```

Verify binary runs and accepts JSON-RPC on stdin:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run --bin mcp
```

Expected: JSON-RPC response with `serverInfo`.

---

### 2.2 — Start MCP Inspector (web UI)

```bash
npx @modelcontextprotocol/inspector cargo run --bin mcp
```

Starts proxy server on `localhost:6277` and opens web UI at `http://localhost:6274`.

The `cargo run --bin mcp` command is the MCP server the inspector spawns. The server must be run from a directory with compiled knowledge (`.samgraha/knowledge.db` + registered repo), otherwise it exits immediately with "Failed to open knowledge registry".

In the Inspector UI:

1. Configure command: `cargo run --bin mcp`
2. Connect
3. Test tool calls

---

### 2.3 — Test scenarios in Inspector UI

| Step | Action | Expected |
|---|---|---|
| Initialize | Auto on connect | Server info + protocol version |
| `tools/list` | Click | 15 tools listed with schemas |
| `search` | Call with `query: "repository"` | Results from knowledge.db |
| `compile` | Call with `force: true` | Compilation result |
| `get_document` | Call with `id: 1` | Document metadata + section TOC |
| `get_document_section` | Call with `id: 1, section: 0` | Paginated section content |
| `info` | Call | Runtime info |
| `list_domains` | Call | Domain list |
| `list_repositories` | Call | All registered repos |
| `repository_status` | Call | Status per repository |
| Invalid tool | Call `nonexistent` | Error response |

---

### 2.4 — Inspector CLI mode (single-call)

```bash
npx @modelcontextprotocol/inspector --cli cargo run --bin mcp --method tools/list

npx @modelcontextprotocol/inspector --cli cargo run --bin mcp --method tools/call -t search -a query=repository
```

Note: MCP server must be run from a directory with compiled knowledge (`.samgraha/knowledge.db` + registered repo), otherwise the server exits immediately with "Failed to open knowledge registry".

### 2.5 — Scriptable CLI testing (stdin JSON-RPC)

For CI/automation, pipe JSON-RPC directly to the MCP binary (no Node.js or inspector needed):

```bash
# tools/list
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | cargo run --bin mcp

# tools/call search
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"query":"repository"}}}' | cargo run --bin mcp

# tools/call get_document
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_document","arguments":{"id":"1"}}}' | cargo run --bin mcp
```

See Phase 2.5 in `scripts/run-tests.ps1` / `scripts/run-tests.sh` for protocol edge-case testing via the same method.

---

## Phase 2.5 — Protocol Compliance (Edge Cases)

Still using the Inspector. Test error handling and robustness:

| Test | Input | Expected |
|---|---|---|
| Malformed JSON | `not json` | Parse error (-32700) |
| Missing method | `{}` | Parse error |
| Unknown method | valid JSON, random method | Method not found (-32601) |
| Missing tool name | `tools/call` without `name` | Invalid params (-32602) |
| Wrong param types | `query: 123` instead of string | Server handles gracefully |
| Oversized query | 10KB query string | No crash |
| Rapid calls | 50 sequential calls | No memory leak, all respond |
| Shutdown | Close Inspector | Process exits cleanly |

---

## Phase 3 — AI Integration

Only when Phases 1, 2, and 2.5 pass.

### 3.1 — Claude Code

Configure MCP server in Claude Code's `mcp.json`:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "cargo",
      "args": ["run", "--bin", "mcp"]
    }
  }
}
```

Test prompts:

- "How does Knowledge Resolution work?"
- "Search for 'repository registry'"
- "What documents are available?"

### 3.2 — OpenCode

Configure MCP server in OpenCode's settings.

Verify knowledge retrieval works.

### 3.3 — Codex CLI / future IDE integrations

Test compatibility.

---

## CLI Reference

| Test | Command |
|---|---|
| Compile | `cargo run --bin cli -- compile` |
| Audit | `cargo run --bin cli -- audit` |
| Register | `cargo run --bin cli -- registry register` |
| List | `cargo run --bin cli -- registry list` |
| Status | `cargo run --bin cli -- registry status` |
| Sync | `cargo run --bin cli -- registry sync` |
| Resolve | `cargo run --bin cli -- registry resolve runtime` |
| Search | `cargo run --bin cli -- search <query>` |
| Build MCP binary | `cargo build --bin mcp` |
| Run MCP binary | `cargo run --bin mcp` |
| MCP Inspector | `npx @modelcontextprotocol/inspector cargo run --bin mcp` |

---

## Architecture Invariants

1. **Search** → `knowledge.db` ONLY. Registry NEVER.
2. **Registry** → `manifest.json` ONLY. `knowledge.db` NEVER.
3. **Resolver** → Never inspects Markdown. Only compiled artifacts.
4. **MCP** → Thin protocol layer over Knowledge Runtime. No business logic.

---

## Notes

- `resolve` is `registry resolve runtime`, not top-level `resolve`.
- `status` shows all repos (no per-name argument).
- List output shows audit PASS/FAIL, not computed status.
- Test reports auto-saved to `docs/report/manual-audit/` (gitignored) after each test runner run.
