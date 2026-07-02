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

Expected JSON output:
```json
{
  "action": "resolve",
  "mode": "runtime",
  "output": ".samgraha/resolved",
  "repositories": 2,
  "success": true
}
```

No DB copy — registry reads manifest only.

Verify generated Knowledge Package at `.samgraha/resolved/`:

- Included repositories list
- Repository order (current first, then dependencies)
- No duplicated knowledge

**Cycle detection test** — simulate a dependency cycle `A → B → C → A`:

Resolver must exit with error:

```
Error: Dependency cycle detected: A → B → C → A
```

**Missing dependency test** — add `astra` to dependencies, delete `samgraha-copy/`. Resolver must exit with error (not panic):

```
Error: Required dependency 'astra' is not available at "..."
```

---

### 1.6 — Metadata Cache

```bash
# First resolve: creates metadata cache (reads manifest from disk)
cargo run --bin cli -- registry resolve runtime
```

Expected: JSON resolve output (no prior cache → reads manifest from disk, caches it).

```bash
# Second resolve (immediate): hits cache, no disk I/O
cargo run --bin cli -- registry resolve runtime
```

Expected: same JSON output (same `success: true`). Cache hit — manifest not re-read.

**Windows (PowerShell):**
```powershell
# Delete metadata cache (SQLite registry.db), resolve again — cache miss
Remove-Item -Force .samgraha/registry.db
cargo run --bin cli -- registry resolve runtime
```

**Ubuntu (Bash):**
```bash
# Delete metadata cache (SQLite registry.db), resolve again — cache miss
rm -f .samgraha/registry.db
cargo run --bin cli -- registry resolve runtime
```

Cache miss — manifest read from disk again, re-cached.

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

Goal: validate MCP server works. No AI. Server now exposes **25 tools** (15 core + 10 semantic audit).

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
| `tools/list` | Click | 25 tools listed with schemas |
| `search` | Call with `query: "repository"` | Results from knowledge.db |
| `compile` | Call with `force: true` | Compilation result |
| `get_document` | Call with `id: 1` | Document metadata + section TOC |
| `get_document_section` | Call with `id: 1, section: 0` | Paginated section content |
| `info` | Call | Runtime info |
| `list_domains` | Call | Domain list |
| `list_repositories` | Call | All registered repos |
| `repository_status` | Call | Status per repository |
| `get_documents_by_domain` | Call with `domain: "feature"` | Documents in that domain |
| `get_section` | Call with `section_id: 1` | Section by PK |
| `get_audit_knowledge` | Call with `domain: "feature", section_type: "functional_requirements"` | Knowledge file content |
| `get_audit_report` | Call with `domain: "feature", stage: "section"` | Latest report |
| `get_section_changed` | Call with `section_id: 1` | Changed status + previous report ID |
| `check_gate` | Call with `stage: "section", document_id: 1` | Gate PASS/BLOCK |
| `store_section_report` | Call with full report JSON | Report stored |
| `store_document_report` | Call with full report JSON | Report stored |
| `update_finding_status` | Call with `report_id: 1, criterion_id: "C1", status: "fixed"` | Status updated |
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

## Phase 3 — Semantic Audit Pipeline (Core Logic)

Goal: validate the 4-stage pipeline (Deterministic → Section → Document → Cross-Domain) and knowledge file serving. No AI required for most steps — section change detection, gates, and report persistence are all deterministic.

---

### 3.1 — Verify Audit Knowledge Files Exist

Knowledge files live under `docs/raw/audit-standards/`. The P0 files are:

| File | Purpose |
|---|---|
| `orchestration.md` | Pipeline order, gate conditions, incremental skip logic |
| `_prompt-template.md` | Shared prompt shell for LLM audit agents |
| `registry.md` | Domain→section_type→strategy mapping |
| `feature/functional-requirements.md` | Audit contract for FR completeness |
| `feature/constraints.md` | Audit contract for constraint specificity |
| `feature/success-criteria.md` | Audit contract for verifiability |
| `architecture/component-model.md` | Audit contract for component clarity |
| `architecture/constraints.md` | Audit contract for architectural constraint quality |

Verify the files exist and are parseable:

```bash
ls docs/raw/audit-standards/feature/
ls docs/raw/audit-standards/architecture/
```

---

### 3.2 — Verify Knowledge File Serving via CLI

The MCP tool `get_audit_knowledge` reads these files and returns their content.

Test via stdin JSON-RPC:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_audit_knowledge","arguments":{"domain":"feature","section_type":"functional_requirements"}}}' | cargo run --bin mcp
```

Expected: JSON response with file content (score criteria, red flags, evidence schema).

---

### 3.3 — Verify Section Change Detection

This is deterministic — it compares hashes. After a compile, each section has a SHA-256 hash stored in `section_audit_hashes`.

Test via MCP:

```bash
# First compile to populate hashes
cargo run --bin cli -- compile

# Check a section
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_section_changed","arguments":{"section_id":1}}}' | cargo run --bin mcp
```

Expected: `{changed: false}` on first call (no previous hash to compare against meaning we can't know, or if a section_id exists in the compiled output, it may show changed).

Modify a source file and recompile:

```bash
# Touch a doc to change it
echo "" >> docs/raw/some-doc.md
cargo run --bin cli -- compile

# Check same section — should now show changed
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_section_changed","arguments":{"section_id":1}}}' | cargo run --bin mcp
```

Expected: `{changed: true}` (hash mismatch).

---

### 3.4 — Verify Gate Logic

Gates block pipeline progression when the previous stage has unresolved ERROR findings.

```bash
# Check gate for deterministic stage (always passes — no deterministic audit configured)
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"check_gate","arguments":{"stage":"deterministic","document_id":1}}}' | cargo run --bin mcp
```

Expected: gate passes if no ERROR-severity findings exist for the deterministic stage on that document.

---

### 3.5 — Verify Report Store Round-Trip

Full cycle: write a section report → read it back → update a finding status.

```bash
# Write a report
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"store_section_report","arguments":{"report_json":{"domain":"feature","stage":"section","document_id":1,"section_id":1,"strategy":"completeness","score":85,"findings":[{"criterion_id":"C1","passed":true,"severity":"error","confidence":0.95,"evidence":{"section_id":1,"paragraph_index":0,"excerpt":"FR1 exists"},"message":"All requirements present","status":"open"}]}}}}' | cargo run --bin mcp

# Read it back
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_audit_report","arguments":{"domain":"feature","stage":"section","document_id":1}}}' | cargo run --bin mcp

# Mark finding as fixed
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"update_finding_status","arguments":{"report_id":1,"criterion_id":"C1","status":"fixed"}}}' | cargo run --bin mcp
```

Expected: reports are stored to SQLite `semantic_reports` table + filesystem under `docs/reports/<domain>/`. Status transitions persist.

---

### 3.6 — Verify Atomic Report Filesystem Storage

Reports are also written atomically to `docs/reports/<domain>/`:

```bash
ls "docs/reports/feature/"
# Should show:
#   latest.json
#   history/
```

Verify atomic rename (no partial writes survive a crash):

```bash
# The .tmp file should never remain after a write
ls "docs/reports/feature/" 2>/dev/null
# No *.tmp files should exist
```

---

## Phase 4 — AI Integration

Only when Phases 1, 2, 2.5, and 3 pass.

### 4.1 — Claude Code

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

### 4.2 — OpenCode

Configure MCP server in `opencode.json` (project root or global config):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "samgraha": {
      "type": "local",
      "command": ["cargo", "run", "--bin", "mcp"]
    }
  }
}
```

The `cwd` defaults to the workspace root — ensure `.samgraha/knowledge.db` and a registered repo exist there, otherwise the server exits with "Failed to open knowledge registry".

Verify the server is registered:

```bash
opencode mcp list
```

Expected: `samgraha` listed with status.

Test prompts:

- "Use the samgraha tools — how does Knowledge Resolution work?"
- "Use samgraha — search for 'repository registry'"
- "Use samgraha — what documents are available?"

### 4.3 — Codex CLI / future IDE integrations

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
| Sections | `cargo run --bin cli -- sections <semantic_type>` |
| Build MCP binary | `cargo build --bin mcp` |
| Run MCP binary | `cargo run --bin mcp` |
| MCP Inspector | `npx @modelcontextprotocol/inspector cargo run --bin mcp` |
| **MCP semantic audit tools** (25 tools total — 15 core + 10 audit) | |
| `get_documents_by_domain` | `tools/call` with `domain` |
| `get_section` | `tools/call` with `section_id` |
| `get_audit_knowledge` | `tools/call` with `domain`, `section_type` |
| `get_audit_report` | `tools/call` with `domain`, `stage` |
| `get_section_changed` | `tools/call` with `section_id` |
| `check_gate` | `tools/call` with `stage` |
| `store_section_report` | `tools/call` with `report_json` |
| `store_document_report` | `tools/call` with `report_json` |
| `store_cross_domain_report` | `tools/call` with `report_json` |
| `update_finding_status` | `tools/call` with `report_id`, `criterion_id`, `status` |

---

## Architecture Invariants

1. **Search** → `knowledge.db` ONLY. Registry NEVER.
2. **Registry** → `manifest.json` ONLY. `knowledge.db` NEVER.
3. **Resolver** → Never inspects Markdown. Only compiled artifacts.
4. **MCP** → Thin protocol layer over Knowledge Runtime. No business logic.
5. **Semantic Audit Pipeline** → Deterministic → Section → Document → Cross-Domain. Each stage gated by previous.
6. **Audit Knowledge Files** → Source of truth for audit criteria, scoring, and red flags. Engine is criteria-agnostic.
7. **Change Detection** → Section hash (`SHA-256`) compared against `section_audit_hashes` table. Unchanged sections skip LLM calls.
8. **Report Storage** → Atomic write (`latest.json.tmp` → rename) + history rotation. No partial writes survive crash.

---

## Notes

- `resolve` is `registry resolve runtime`, not top-level `resolve`.
- `status` shows all repos (no per-name argument).
- List output shows audit PASS/FAIL, not computed status.
- Test reports auto-saved to `docs/report/manual-audit/` (gitignored) after each test runner run.
- Semantic audit reports persisted to SQLite `semantic_reports` table + filesystem under `docs/reports/<domain>/latest.json`.
- Audit knowledge files live at `docs/raw/audit-standards/` — shared via MCP `get_audit_knowledge` tool.
- P0 knowledge files cover `feature` and `architecture` domains. Remaining ~80 files added incrementally as domains are used.
