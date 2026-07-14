# Knowledge Systems: Local-First Registration + Global Sync

## Design Philosophy

Samgraha is no longer coupled to a single documentation methodology.

Instead, repositories explicitly choose a Knowledge System that defines
how documentation is organized, validated, generated, and consumed.

The platform remains generic.

Knowledge Systems provide the domain-specific behavior.

Repositories own the choice of Knowledge System.

Samgraha owns compilation, synchronization, audit, search, planning,
and MCP integration.

---

## What is a Knowledge System?

A Knowledge System is a self-contained package that defines how a repository
documents and manages knowledge.

A Knowledge System may export:

- Documentation Standards
- Audit Standards
- Product Guide
- Templates
- Calculations
- Planning Knowledge
- Scripts

Every repository selects exactly one Knowledge System.

Samgraha loads the selected Knowledge System during compilation.

```
Knowledge System
├── Documentation Standards
├── Audit Standards
├── Product Guide
├── Templates
├── Calculations
├── Plans
└── Scripts
```

Documentation Standards are just one exported capability.

---

## Repository Association

Each repository is associated with exactly one Knowledge System.

Example:

```
Research Repository  --> Research Knowledge System
Frontend Repository  --> Frontend Knowledge System
Backend Repository   --> Backend Knowledge System
```

Samgraha intentionally does not support multiple active Knowledge Systems
within a single repository.

This keeps compilation deterministic and avoids conflicts between standards.

---

## Architecture Overview

```
                    Samgraha Platform
                           |
                           v
                Global Knowledge Registry
                           |
               Register / Publish / Sync
                           |
                           v
                Repository Knowledge DB
                           |
        +------------------+------------------+
        |                  |                  |
     Search             Audit             Planner
        |                  |                  |
        +------------------+------------------+
                           v
                           MCP
```

Where the **Repository Knowledge DB** contains:

```
Knowledge System
├── Documentation Standards
├── Audit Standards
├── Product Guide
├── Templates
├── Calculations
├── Planning Knowledge
├── Scripts
├── Repository Metadata
└── Repository Knowledge
```

---

## Global Knowledge Registry

The Global Knowledge Registry is responsible for:

- Publishing Knowledge Systems
- Repository Registration
- Version Tracking
- Synchronization

It is not used during normal MCP operations.

Repositories synchronize required Knowledge Systems locally.

---

## Repository Database

The Repository Database is the single runtime knowledge source.

All platform capabilities operate exclusively on the repository database.

This includes:

- Search
- Audit
- Planning
- Generation
- Product Guide
- MCP

No runtime component directly queries the Global Knowledge Registry.

---

## Repository Metadata

Repository Metadata is generated during compilation.

It is stored in the Repository Database.

The Product Guide documents Repository Metadata.

The Compiler owns Repository Metadata.

The Product Guide never generates or modifies metadata.

---

## Synchronization

Synchronization is a distribution mechanism.

It prepares a repository for offline operation.

After synchronization, every runtime capability operates entirely
from the local repository database.

The Global Registry is no longer consulted until the next synchronization.

---

## Product Guide

The Product Guide is part of the selected Knowledge System.

It documents:

- Public product capabilities
- CLI
- MCP
- Configuration
- Repository Metadata
- Documentation methodology

It does not own any implementation.

---

## Current State (Problems)

### Build Flow (broken)

```
scripts/build-release.sh
  |
  +--> BUILTIN_SOURCES = { standards: "docs/raw/documentation-standards", help: "docs/raw/product-guide" }
  |      |
  |      +--> "docs/raw/documentation-standards" DOES NOT EXIST  <-- WARNING
  |      +--> "docs/raw/product-guide" compiles to help.db       <-- WORKS
  |
  +--> No standards.db shipped in package
```

### Registration Flow

```
samgraha standards register --path <dir> [--local]
  |
  +--> Default: writes to mcp_dir()/standards.db (global)
  +--> --local: writes to .samgraha/standards.db (local only)
  |
  +--> No mechanism to push local changes to global
```

### Sync Flow

```
samgraha standards sync
  |
  +--> Source: mcp_dir()/standards.db (global)
  +--> Target: .samgraha/standards.db (local)
  +--> std::fs::copy (whole file, one-directional only)
```

### Runtime Loading

```
KnowledgeRuntime::new(root, config)
  |
  +--> StandardRegistry::from_standards_db_and_overrides_with_system(
         repo_root,
         config.repository.documentation.standard_system  // None -> is_default=1
       )
  |
  +--> Opens .samgraha/standards.db
  +--> Projects DB rows -> StandardDefinition structs
  +--> Layers .samgraha/standards/ JSON/TOML overrides
```

### Dead Code in Audit Fix Pipeline

`planning_context.rs` reads `.md` source files from disk into `doc_standard_raw` and `audit_standard_raw`, but **these fields are never consumed** by any planner or executor. The full content is already stored in `standards.db` (`standard_docs.content` table). Only `audit_spec_raw` (from `docs/raw/audit/`) is actually used via `check_requirement()`.

This means the hardcoded `docs/knowledge-hub/documentation-standards` path can be **removed entirely** -- the data is in the DB.

**Evidence:**

- `doc_standard_raw` appears 8 times, all in `planning_context.rs`. No other file references it.
- `audit_standard_raw` appears 5 times, all in `planning_context.rs`. No other file references it.
- `PlanningContext.doc_standard` and `PlanningContext.audit_standard` are assigned but never read by any downstream code.
- Only `PlanningContext.audit_spec.raw` is consumed, via `check_requirement()` in `types.rs`.

---

## Proposed Changes

### Design Principles

1. **Local-first**: Knowledge Systems register locally before syncing globally
2. **No hardcoded paths**: Code resolves data from the repository database, not fixed directories
3. **Schema ships empty**: Build provides the schema skeleton; registration populates it
4. **One repository, one system**: Each repository selects exactly one Knowledge System

### What Ships in the Build

```
samgraha/
  bin/
    mcp, cli
    help.db          <-- compiled from docs/raw/product-guide (schema + data)
    knowledge.db     <-- empty schema only, no system rows
  schema/
    knowledge-hub/
      knowledge-hub-loader.py
      00-reset.sql .. 22-plan_scenarios.sql
  samgraha.toml
```

### Registration Flow (new)

```
samgraha standards register --path <dir> [--system NAME] [--no-push]
  |
  1. Run knowledge-hub-loader.py
     --> .samgraha/knowledge.db (local)
  |
  2. Integrity check on local DB
  |
  3. Copy .samgraha/knowledge.db --> mcp_dir()/knowledge.db (global)
     (skipped if --no-push)
```

### Push Flow (new)

```
samgraha standards push
  |
  1. Integrity check on local .samgraha/knowledge.db
  |
  2. Copy .samgraha/knowledge.db --> mcp_dir()/knowledge.db (global)
```

Use cases:
- Register was done with `--no-push`
- Local changes were made manually
- Multiple repos share a knowledge.db and one needs to publish

### Sync Flow (unchanged, now local-first)

```
samgraha standards sync
  |
  +--> Source: mcp_dir()/knowledge.db (global)
  +--> Target: .samgraha/knowledge.db (local)
  +--> std::fs::copy (whole file)
  |
  +--> After sync: all runtime capabilities read from local DB
```

### Default System Selection

- `standards set-default <system>` -- sets `is_default = 1` in `systems` table
- `from_standards_db(conn, system_name)` uses `standard_system` from config, falls back to `is_default = 1`
- First system registered gets `is_default = 1` automatically (loader pass_0)
- Empty schema means first registration creates the system cleanly

### Path Resolution (no hardcoded paths)

```
Before (broken):
  doc_standard_path = repo_root/docs/knowledge-hub/documentation-standards/  <-- hardcoded

After:
  doc_standard_raw = ""  (dead code removed, data comes from DB)
  StandardDoc.content = full .md text from knowledge.db standard_docs table
```

---

## Implementation Plan

### Phase 1: Remove Dead Code + Hardcoded Paths

**Goal**: Clean up hardcoded paths and dead code.

#### 1a. `crates/audit/src/fix/planning_context.rs`

- **Remove** hardcoded `docs/knowledge-hub/documentation-standards` from `doc_standard_path()` (line 163)
- **Remove** hardcoded `docs/raw/audit-standards` from `audit_standard_path()` (line 156)
- Since `doc_standard_raw` and `audit_standard_raw` are dead data, make both methods return empty gracefully
- Simplest fix: change `doc_standard_path` to look in `.samgraha/` (local synced location), and `audit_standard_path` similarly. Both already handled by `read_file_optional` returning empty string.

#### 1b. `crates/compiler/src/discovery.rs`

- Revert test path at line 187 to a neutral value, or keep as-is (the test validates `infer_standard` logic, not path existence)

#### 1c. `crates/registry/src/store.rs`

- Update comment at line 3201 to reference DB content, not disk path

#### 1d. `crates/standards/src/registry.rs`

- **Keep** the "standards" builtin in `with_builtins()` -- needed for `validate_config` when compiling standards locally
- **Keep** the test update (expects 2 builtins)

#### 1e. `scripts/build-release.sh`

- Remove `[standards]` from `BUILTIN_SOURCES` (already done)

#### 1f. `scripts/build-release.ps1`

- Remove standards entry from `$builtinSources` (already done)

### Phase 2: Build Ships Empty Schema + Loader

**Goal**: Package includes empty `knowledge.db` (schema only) and the loader/schema files.

#### 2a. `scripts/build-release.sh`

Add after binary packaging:

```bash
# Create empty knowledge.db with schema (no system rows)
SCHEMA_DIR="$ROOT_DIR/schema/knowledge-hub"
if [[ -f "$SCHEMA_DIR/knowledge-hub-loader.py" ]]; then
    python3 "$SCHEMA_DIR/knowledge-hub-loader.py" \
        --db "$PKG_DIR/bin/knowledge.db" \
        --knowledge-hub "$SCHEMA_DIR" \
        --reset --dry-run 2>/dev/null || {
        # Fallback: create DB with raw SQL schema
        python3 -c "
import sqlite3, glob, os
conn = sqlite3.connect('$PKG_DIR/bin/knowledge.db')
conn.execute('PRAGMA user_version = 1')
for f in sorted(glob.glob('$SCHEMA_DIR/*.sql')):
    conn.executescript(open(f).read())
conn.close()
"
    }
fi

# Ship schema + loader for registration
mkdir -p "$PKG_DIR/schema/knowledge-hub"
cp "$SCHEMA_DIR"/*.sql "$PKG_DIR/schema/knowledge-hub/"
cp "$SCHEMA_DIR/knowledge-hub-loader.py" "$PKG_DIR/schema/knowledge-hub/"
```

#### 2b. `scripts/build-release.ps1`

Same changes for PowerShell.

#### 2c. Updated `builtin_stores` in `runtime.rs`

Update the info reporter to distinguish between "empty schema" and "has data":

```rust
// Line 1775-1781: Update to check if knowledge.db has system rows
builtin_stores: {
    let knowledge_path = common::env::mcp_dir().join("knowledge.db");
    let knowledge_status = if knowledge_path.exists() {
        match rusqlite::Connection::open(&knowledge_path) {
            Ok(conn) => {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM systems", [], |r| r.get(0)
                ).unwrap_or(0);
                if count > 0 { "available" } else { "empty (register a system)" }
            }
            Err(_) => "corrupt",
        }
    } else {
        "not shipped"
    };
    // ... same for help.db
}
```

### Phase 3: Register Writes Locally First, Then Pushes

**Goal**: `standards register` creates local DB, then syncs to global.

#### 3a. `crates/cli/src/commands.rs` -- `StandardsAction::Register`

New logic:

```rust
StandardsAction::Register { path, system, layout, no_push, dry_run } => {
    // 1. Always write to local first
    let local_db = root.join(".samgraha").join("knowledge.db");
    run_loader(&local_db, &path, system, layout, dry_run)?;

    if !dry_run && !no_push {
        // 2. Push to global
        let global_db = common::env::mcp_dir().join("knowledge.db");
        if let Some(parent) = global_db.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Integrity check before overwriting global
        {
            let check_conn = rusqlite::Connection::open(&local_db)?;
            let ok: String = check_conn.query_row(
                "PRAGMA integrity_check", [], |row| row.get(0)
            )?;
            if ok != "ok" {
                anyhow::bail!("Local knowledge DB failed integrity check: {}", ok);
            }
            standards::check_schema_version(&check_conn)?;
        }
        std::fs::copy(&local_db, &global_db)?;
        println!("Pushed to {}", global_db.display());
    }
}
```

Add `--no-push` flag to the CLI args:

```rust
#[arg(long, help = "Register locally only, do not push to global knowledge.db")]
no_push: bool,
```

#### 3b. `crates/cli/src/commands.rs` -- New `StandardsAction::Push`

```rust
StandardsAction::Push => {
    let local_db = root.join(".samgraha").join("knowledge.db");
    if !local_db.exists() {
        anyhow::bail!("No local .samgraha/knowledge.db -- register a system first");
    }
    let global_db = common::env::mcp_dir().join("knowledge.db");
    if let Some(parent) = global_db.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Integrity check
    {
        let check_conn = rusqlite::Connection::open(&local_db)?;
        let ok: String = check_conn.query_row(
            "PRAGMA integrity_check", [], |row| row.get(0)
        )?;
        if ok != "ok" {
            anyhow::bail!("Local knowledge DB failed integrity check: {}", ok);
        }
        standards::check_schema_version(&check_conn)?;
    }
    std::fs::copy(&local_db, &global_db)?;
    println!("Pushed {} to {}", local_db.display(), global_db.display());
}
```

#### 3c. `crates/mcp/src/adapter.rs` -- `handle_register_standard`

Same local-first logic as 3a.

Add MCP tool `push_standards`:

```rust
fn handle_push_standards(&self, _req: &McpRequest) -> Result<serde_json::Value> {
    let local_db = self.runtime.context.repository_root.join(".samgraha").join("knowledge.db");
    // ... same logic as CLI Push ...
}
```

#### 3d. `crates/cli/src/commands.rs` -- Update `StandardsAction` enum

```rust
#[derive(Subcommand)]
pub enum StandardsAction {
    List,
    Show { domain: String, version: Option<String> },
    ShowDoc { domain: String },
    Register {
        #[arg(long = "path")]
        path: PathBuf,
        #[arg(long)]
        system: Option<String>,
        #[arg(long)]
        layout: Option<PathBuf>,
        #[arg(long)]
        no_push: bool,
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    Push,
    SetDefault { system: String },
    Sync,
    Remove { domain: String },
}
```

### Phase 4: Documentation Updates

**Goal**: Update product guide and config comments to reflect the Knowledge System model.

#### 4a. `docs/raw/product-guide/build-guide/overview.md`

Update step 3 to describe empty schema:

```
3. Creates an empty `knowledge.db` (schema only) and copies the
   knowledge-hub loader + schema files for Knowledge System registration.
```

#### 4b. `docs/raw/product-guide/build-guide/distribution.md`

Update Built-in Knowledge Distribution section:

```
### Built-in Knowledge Distribution

`help.db` is compiled during the release build (from `docs/raw/product-guide`)
and placed in the package root. `knowledge.db` ships with an empty schema --
no system rows, no domains, no rules. Users register their Knowledge System
with `standards register`, which creates local data and syncs to the
global store.

At runtime, `cli.exe` and `mcp.exe` look for these files next to whichever
binary is running (via `std::env::current_exe()?.parent()`).
```

#### 4c. `samgraha.toml` comments

Update the comment about built-in knowledge:

```toml
# help is built-in knowledge compiled separately by the release script
# (--path docs/raw/product-guide). knowledge.db ships as an empty schema --
# users register their Knowledge System with `standards register` which
# populates the local DB and syncs to the global store.
```

---

## Verification Checklist

| # | Test | Expected Result |
|---|------|----------------|
| 1 | `cargo test --release -p compiler -p standards -p audit` | All tests pass (42 compiler + 4 standards + 215 audit) |
| 2 | Build script produces package | Package contains `help.db` (data) + `knowledge.db` (empty schema) + `schema/knowledge-hub/` |
| 3 | `standards list` (fresh install) | No systems registered |
| 4 | `standards register --path docs/knowledge-hub` | Creates local `.samgraha/knowledge.db`, pushes to global |
| 5 | `standards list` (after register) | Shows "samgraha-documentation" system with 16 domains |
| 6 | `standards register --path other-system --no-push` | Creates local only, global unchanged |
| 7 | `standards push` | Copies local to global |
| 8 | `standards sync` (in another repo) | Pulls global to local |
| 9 | `standards set-default other-system` | Switches default system |
| 10 | `samgraha info` | Shows correct builtin store status |
| 11 | No hardcoded `docs/knowledge-hub` in Rust source | `grep -r "docs/knowledge-hub" crates/` returns empty |
| 12 | No hardcoded `docs/raw/documentation-standards` in Rust source | `grep -r "docs/raw/documentation-standards" crates/` returns empty |

---

## Files Changed Summary

| File | Phase | Change |
|------|-------|--------|
| `crates/audit/src/fix/planning_context.rs` | 1 | Remove hardcoded paths, use `.samgraha/` or empty fallback |
| `crates/compiler/src/discovery.rs` | 1 | Revert/neutralize test path |
| `crates/registry/src/store.rs` | 1 | Update comment |
| `crates/standards/src/registry.rs` | 1 | Keep "standards" builtin + test |
| `scripts/build-release.sh` | 1+2 | Remove standards from BUILTIN_SOURCES, add empty schema creation + schema shipping |
| `scripts/build-release.ps1` | 1+2 | Same |
| `crates/cli/src/commands.rs` | 3 | Register writes local-first + push; add Push command; add --no-push flag |
| `crates/mcp/src/adapter.rs` | 3 | Register writes local-first + push; add push_standards tool |
| `crates/services/src/runtime/runtime.rs` | 2 | Update builtin_stores info reporter |
| `docs/raw/product-guide/build-guide/overview.md` | 4 | Update build steps description |
| `docs/raw/product-guide/build-guide/distribution.md` | 4 | Update distribution description |
| `samgraha.toml` | 4 | Update comments |

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Python loader not available at register time | Build ships loader + schema; `SAMGRAHA_KNOWLEDGE_HUB_LOADER` env override exists |
| Empty knowledge.db breaks existing repos | `from_standards_db_and_overrides_with_system` falls back to `with_builtins_and_overrides` when no system found |
| Schema version mismatch after upgrade | `check_schema_version` rejects mismatched DBs; re-register fixes it |
| Concurrent register from multiple repos | SQLite WAL mode + whole-file copy minimizes window; not a typical workflow |
| `--no-push` leaves global stale | User must explicitly `standards push`; `standards list` shows local vs global state |

---

## What We Are Not Introducing

- Multiple active Knowledge Systems per repository
- Knowledge overlays
- Layered standards
- Runtime merging
- Priority rules
- Plugin inheritance

These would add complexity without solving a current problem.

---

## Summary

This proposal represents a key architectural milestone in Samgraha's evolution.

The platform becomes **Knowledge System driven**:

- Samgraha provides the engine.
- Knowledge Systems provide the domain-specific behavior.
- Repositories explicitly choose one Knowledge System.
- The Global Registry distributes and synchronizes knowledge.
- The Repository Database becomes the single source of runtime truth.

That is a clean, cohesive, and extensible architecture without introducing
unnecessary complexity.
