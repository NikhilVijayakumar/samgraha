# Proposal: Repo Script Standardization — `scripts/` as System Overrides + Rust Implementations

**Status:** Phase 1 implemented.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  System defaults (docs/knowledge-hub/script/)       │
│  18 checks × 2 platforms (ubuntu/windows)           │
│  Schemas, manifests, mapping.yaml, policy.yaml      │
│  Part of the knowledge system — do NOT modify       │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│  Repo overrides (scripts/)                          │
│  This repo's check implementations                  │
│  Follows the same interface contract                │
│  Can override any system check                      │
│  Can add repo-specific checks                       │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│  Rust native (crates/audit/src/checks/)             │
│  Built-in implementations for common checks         │
│  No shell dependency, cross-platform                │
│  Fastest execution path                             │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│  samgraha.toml [repository.documentation]           │
│  check_overrides = { check-name = "scripts/x.sh" } │
│  Wires repo scripts into the audit engine           │
└─────────────────────────────────────────────────────┘
```

**Resolution order** (first hit wins):

1. `check_overrides[check_name]` in `samgraha.toml` — explicit repo override
2. `scripts/<check_name>.{sh,ps1}` — repo-local script (this repo's convention)
3. `.samgraha/scripts/<check_name>.{sh,ps1}` — synced from global
4. `~/.samgraha/scripts/<check_name>.{sh,ps1}` — global system default
5. Rust native implementation — built into the binary
6. `docs/knowledge-hub/script/{ubuntu,windows}/<domain>/<check_name>.{sh,ps1}` — schema fallback

## Problem

1. **`scripts/` has 6 pairs of .ps1/.sh with duplicated logic.** ~2400 lines of
   shell across both platforms doing the same work. `lib/report.sh` and
   `lib/report.ps1` duplicate the same template engine and metrics tracking.

2. **`scripts/` doesn't follow the knowledge-hub script interface.** The system
   defines a standard contract (`--repo-root`, `--repo-fingerprint`, `--out <json>`,
   JSON schema validation). The repo's scripts use their own ad-hoc patterns.

3. **No wiring from `samgraha.toml` to the audit engine for check overrides.**
   `script_overrides` exists per rule ID (`config.rs:386`) but there's no
   `check_overrides` per check name. The repo can't say "use my `build-succeeds`
   instead of the system default" and have it work through the audit pipeline.

4. **`scripts/` contains both "audit checks" and "build/deploy tooling."**
   `build-release`, `test-coverage`, `run-tests` are development workflow tools
   that happen to overlap with audit checks. They should be standardized as
   checks so the audit engine can invoke them.

## Goal

1. `scripts/` follows the knowledge-hub script interface — same args, same
   JSON output schema, same manifest/metadata pattern.

2. Core logic moves to Rust. PS1/SH become thin wrappers calling the Rust
   binary. No more duplicated logic between platforms.

3. `samgraha.toml` gains `check_overrides` that wires repo scripts into the
   audit engine. The repo's build/test workflow aligns with its audit system.

## Current State

### `scripts/` inventory

| Script | Purpose | Lines (sh+ps1) | Knowledge-hub check overlap |
|--------|---------|----------------|----------------------------|
| `build-release` | Build + package + launcher + checksums | ~400 | `build-succeeds` (14-build) |
| `test-coverage` | Run `cargo llvm-cov`, write TestRunReport JSON | ~200 | `unit-test-coverage` (12-qa) |
| `run-tests` | E2E test runner: CLI + MCP integration | ~600 | None (new check) |
| `mcp-discover` | MCP server discovery: 10-phase validation | ~800 | None (new check) |
| `audit-phase1` | Manual config audit: backup/restore | ~200 | None (workflow tool) |
| `demo-dependency` | Create temp fixture, test resolution | ~150 | None (demo helper) |
| `lib/report` | Shared: template engine, metrics, trends | ~400 | None (shared util) |

### `docs/knowledge-hub/script/` system checks (18 checks, do NOT modify)

All follow the standardized interface:

```
--repo-root <path>           # repository root
--repo-fingerprint <hash>    # cache invalidation key
--out <path>                 # JSON output file
--<check-specific>           # e.g. --build-command, --docs-root
```

Output JSON:
```json
{
  "check": "<name>",
  "domain": "<domain>",
  "category": "A|B|C",
  "status": "pass|fail|error|not_applicable",
  "metrics": { ... },
  "evidence": ["string", ...],
  "executed_at": "ISO-8601",
  "repo_fingerprint": "string"
}
```

### `samgraha.toml` current config

```toml
[repository.scripts]
dir = "${SAMGRAHA_SCRIPTS_DIR}"

[repository.documentation]
script_overrides = {}  # per rule ID, not per check name

[pipelines.build]
command = ["cargo", "build", "--release", "-p", "cli"]
artifacts = ["${PROJECT_ROOT}/target/release/cli.exe"]

[pipelines.test]
command = ["pwsh", "-File", "scripts/test-coverage.ps1"]
artifacts = ["docs/report/test-results.json"]
```

## Proposed Changes

### 1. `samgraha.toml` — add `check_overrides`

```toml
[repository.documentation]
# Override system checks with repo-specific implementations.
# Key = check name, Value = script path relative to repo root.
check_overrides = {
    build-succeeds = "scripts/build-succeeds.sh",
    unit-test-coverage = "scripts/test-coverage.sh",
    test-run = "scripts/run-tests.sh",
    mcp-discover = "scripts/mcp-discover.sh",
}
```

### 2. `config.rs` — add `check_overrides` field

Add to `DocumentationConfig`:
```rust
pub check_overrides: std::collections::HashMap<String, String>,
```

### 3. `providers.rs` — wire `check_overrides` into resolution

When a rule has `evidence_type = 'script_result'`:
1. Check `check_overrides[check_name]` first
2. Fall back to existing `script_overrides[rule_id]`
3. Fall back to knowledge-hub schema default

### 4. `scripts/` — refactor to match knowledge-hub interface

Each script becomes:
- Same `--repo-root`, `--repo-fingerprint`, `--out` interface
- Same JSON output format matching the relevant schema
- Thin wrapper calling Rust when possible, full implementation otherwise

| Script | Refactor approach |
|--------|------------------|
| `build-release` | Split: `build-succeeds` check → Rust native. Packaging → stays shell. |
| `test-coverage` | `unit-test-coverage` check → Rust native (parse `cargo llvm-cov` JSON) |
| `run-tests` | New check `test-run` → stays shell (too complex to port), but standardize interface |
| `mcp-discover` | New check `mcp-discover` → stays shell, standardize interface |
| `audit-phase1` | Workflow tool, not a check → stays as-is (or become `samgraha audit workflow`) |
| `demo-dependency` | Demo helper → stays as-is |
| `lib/report` | Template engine → Rust (`samgraha report generate`) |

### 5. Rust-native checks (new `crates/audit/src/checks/`)

Implement in Rust to eliminate shell duplication:

| Check | What it does | Why Rust |
|-------|-------------|----------|
| `build-succeeds` | Spawn build command, measure time, capture exit code | Already have `pipelines.build` contract |
| `unit-test-coverage` | Parse `cargo llvm-cov` JSON, apply threshold | Pure data processing |
| `lint-pass` | Spawn lint command, check exit code | Same as build-succeeds |
| `folder-structure` | Walk directory, compare against expected | Pure file ops |
| `dependency-manifest` | Check for Cargo.toml/package.json | Trivial file check |
| `artifact-exists` | Check if artifact path exists | Trivial file check |
| `secret-scan` | Regex scan for secrets | No external deps |
| `traceability-refs-exist` | Parse markdown, resolve refs | Complex but pure Rust |
| `feature-family-mapping` | Cross-reference 3 doc types | Pure data processing |

Checks staying as shell (need external tools):
- `mock-api-runs` — needs server process
- `module-boundary-diff` — needs cargo metadata
- `dependency-reachable` — needs cargo tree
- `dependency-vuln-scan` — needs cargo audit / npm audit
- `run-tests` — orchestration, not a single check
- `mcp-discover` — orchestration, not a single check

## Phases

### Phase 1 — Config + check runner ✅

**Goal:** `check_overrides` config + `samgraha check <name>` CLI subcommand.

1. ✅ Added `check_overrides: HashMap<String, String>` to `DocumentationConfig`
2. ✅ Added `check` subcommand to CLI (`samgraha check <name>`)
3. ✅ Implemented resolution chain in `audit/src/check_runner.rs`
4. ✅ MCP tool: `run_check`
5. ✅ Wired `check_overrides` into `providers.rs` script resolution
6. ✅ Added commented-out `check_overrides` section to `samgraha.toml`

**Files:**
- `crates/common/src/config.rs`
- `crates/cli/src/commands.rs`
- `crates/audit/src/check_runner.rs` (new)
- `crates/audit/src/providers.rs`
- `crates/mcp/src/adapter.rs`
- `crates/mcp/Cargo.toml`

### Phase 2 — Rust-native checks (5 checks)

**Goal:** `build-succeeds`, `lint-pass`, `folder-structure`, `dependency-manifest`, `artifact-exists` in Rust.

**Files:**
- `crates/audit/src/checks/` (new directory)
- `crates/audit/src/check_runner.rs` — wire native checks

### Phase 3 — Port 5 more + refactor `scripts/`

**Goal:** `secret-scan`, `traceability-refs-exist`, `feature-family-mapping`, `unit-test-coverage`, `dependency-vuln-scan` in Rust. `scripts/` becomes thin wrappers.

**Files:**
- `crates/audit/src/checks/` — 5 new modules
- `scripts/*.sh`, `scripts/*.ps1` — rewrite
- `scripts/lib/report.sh`, `scripts/lib/report.ps1` → delete
- `crates/services/src/reporting/template.rs` — Rust template engine

### Phase 4 — Audit engine wiring

**Goal:** `mapping.yaml`/`policy.yaml`/manifests connected to Rust audit engine. Script checks become first-class audit rules with caching.

**Files:**
- `crates/audit/src/providers.rs`
- `crates/registry/src/store.rs` — script_cache
- `docs/knowledge-hub/script/mapping.yaml` — real rule_ids

## Verification

After each phase:
- `cargo check` passes
- `cargo test` passes (216/217 — pre-existing failure on `docs/raw/audit`)
- `samgraha check build-succeeds --repo-root . --out /tmp/test.json` produces valid JSON
- `scripts/build-release.sh` still works (thin wrapper)
- Audit engine resolves repo overrides before system defaults
