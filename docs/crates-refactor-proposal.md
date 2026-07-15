# Proposal: `samgraha init` Enhancements — Document Standards, Auto-Detection, Knowledge System Sync

**Status:** Proposed  
**Date:** 2026-07-15  
**Scope:** `crates/common`, `crates/services`, `crates/cli`, `crates/mcp`

---

## 1. Problem Statement

When a new repository runs `samgraha init` (CLI or MCP), the generated `samgraha.toml` is incomplete in several ways:

### 1.1 No Document Standard Selection

`standard_system` is always `None` in the generated TOML (`crates/common/src/config.rs:445`). The user must manually edit `samgraha.toml` after init to set `[repository.documentation] standard_system`. There is no CLI flag or MCP parameter to specify this at init time.

**Impact:** Without a standard system set, the repo cannot use documentation standards, audit rules, or templates from the knowledge-hub DB. Every audit and compile operation falls back to the bare Rust-native `help`/`standards` builtins (`crates/standards/src/registry.rs:48-75`), which have zero structural requirements.

### 1.2 No Auto-Detection of Directories

`init_repository()` (`crates/services/src/init.rs:24-38`) creates a template with env-var placeholders:
- `root_dir = "${SAMGRAHA_DOCS_DIR}"` — even if `docs/` exists at repo root
- `implementation.dir = "${SAMGRAHA_IMPLEMENTATION_DIR}"` — even if `src/` or `crates/` exists
- `scripts` and `tests` are `None` — even if `scripts/` and `tests/` directories exist

The user must either set `SAMGRAHA_*` env vars in `.env` or manually edit the TOML to replace placeholders with literal paths. For a fresh repo with standard directory layout, this is unnecessary friction.

### 1.3 No Script/Check Overrides at Init

`script_overrides` and `check_overrides` are always empty `HashMap`s (`config.rs:419,425`). There is no way to specify them during init — they can only be set by hand-editing the TOML.

### 1.4 No Knowledge System Sync as Part of Init

After `init`, the repo has no `.samgraha/standards.db`, no help content in `.samgraha/knowledge.db`, and no `.samgraha/scripts/`. The user must separately run `samgraha knowledge pull` (CLI) or `sync_standards` (MCP) to copy these from the global store. This is a separate manual step that is easy to forget and blocks all standards-based functionality.

**The sync logic already exists** in:
- CLI: `commands.rs:1463-1517` (`KnowledgeAction::Pull`)
- MCP: `adapter.rs:1522-1576` (`handle_sync_standards`)

Both do the same thing:
1. Copy `mcp_dir()/standards.db` → `.samgraha/standards.db` (with integrity check)
2. Call `sync_help_into_local(root)` (`builtin.rs:34-64`)
3. Copy `mcp_dir()/scripts/` → `.samgraha/scripts/`

This logic is duplicated and needs to be a single shared function.

### 1.5 Help and Standards Not Available Locally After Init

The comment in `registry.rs:32-47` explains that `help` is the one Rust-native standard that ships without DB dependency. But the actual help *content* (compiled from `docs/raw/product-guide/` into `help.db` at release time) only becomes available after `knowledge pull` / `sync_standards`. Until then, the repo's `knowledge.db` has zero help documents, and `standards.db` doesn't exist at all.

### 1.6 No Ongoing Sync Tracking with Global Store

Documentation standards and help content are published globally (`knowledge publish` → `mcp_dir()/standards.db`). Individual repos pull via `knowledge pull` / `sync_standards`. But there is:
- No tracking of when the local copy was last synced
- No way to know if the local copy is stale vs. the global store
- No `knowledge status` command to inspect sync state
- No `--force` flag on `knowledge pull`

---

## 2. Proposed Solution

### 2.1 Phased Init Flow

Instead of a flat sequence, `init_repository()` is restructured into four clear phases:

```
Phase 1: Configuration
  Build template SamgrahaConfig, apply InitOptions (standard_system, overrides).

Phase 2: Discovery
  Probe filesystem for docs/, src|crates/, tests/, scripts/.
  Set literal paths in template when found.

Phase 3: Synchronization
  If sync_knowledge_system requested, pull from global store.
  Write sync metadata for staleness tracking.

Phase 4: Result
  Return enriched InitResult with sync and detection info.
```

Each phase has a single responsibility. Registration is **not** part of init — it is a separate operation (`registry register` / `registry sync`).

### 2.2 New `InitOptions` Struct

**File:** `crates/common/src/config.rs`

```rust
/// Options controlling `init_repository()` behavior.
/// All fields have safe defaults — zeroing InitOptions preserves
/// the exact current behavior of init_repository(root, false).
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    /// Overwrite existing samgraha.toml (vs. backfill missing keys).
    pub force: bool,
    /// Document standard system name to set in
    /// `[repository.documentation] standard_system`.
    /// `None` = leave unset (current default behavior).
    pub standard_system: Option<String>,
    /// Script check overrides: rule_id -> script path.
    pub script_overrides: std::collections::HashMap<String, String>,
    /// Check overrides: check_name -> script path.
    /// Higher priority than script_overrides in the audit resolution chain.
    pub check_overrides: std::collections::HashMap<String, String>,
    /// Probe repo root for docs/, src|crates/, tests/, scripts/ and set
    /// literal paths in the TOML if found. Skip missing dirs.
    pub auto_detect_dirs: bool,
    /// Sync the declared Knowledge System from global store into
    /// the local .samgraha/ after writing samgraha.toml.
    pub sync_knowledge_system: bool,
}
```

**Removed from original proposal:**
- `update_registry` / `clear_registry` — init does not mutate the registry
- `auto_sync_standards` — deferred; users call `knowledge pull` explicitly

**Rationale:** `Default` for `InitOptions` yields all-false/None/empty, which means `init_repository(root, &InitOptions::default())` behaves identically to the current `init_repository(root, false)`. Backward compatible.

**Future consideration:** The `sync_knowledge_system: bool` may evolve into a `SyncPolicy` enum (`Never`, `IfMissing`, `Always`) when use cases demand it. Not implementing now.

### 2.3 Enhanced `InitResult`

**File:** `crates/services/src/init.rs`

```rust
/// Result of a Knowledge System sync from global store to local.
pub struct SyncResult {
    /// Whether standards.db was copied (file existed at source).
    pub standards_synced: bool,
    /// Number of help documents synced into knowledge.db (0 = none).
    pub help_documents_synced: usize,
    /// Number of scripts copied from global scripts/.
    pub scripts_synced: usize,
}

/// Result of initializing (or backfilling) a repository's `samgraha.toml`.
pub struct InitResult {
    pub root: PathBuf,
    pub config: SamgrahaConfig,
    pub status: String,
    pub env_path: PathBuf,
    /// Populated only when `sync_knowledge_system = true`.
    pub sync_result: Option<SyncResult>,
}
```

### 2.4 Rewritten `init_repository()` — Four Phases

**File:** `crates/services/src/init.rs`

```
init_repository(root, options):
  ── Phase 1: Configuration ──
  1. Create .samgraha/ directory
  2. Build template SamgrahaConfig
  3. Set template.documentation.standard_system = options.standard_system
  4. Set template.documentation.script_overrides = options.script_overrides
  5. Set template.documentation.check_overrides = options.check_overrides

  ── Phase 2: Discovery ──
  6. IF options.auto_detect_dirs:
     Probe root for directories, set literal paths in template

  ── Phase 3: Synchronization ──
  7. Write or backfill samgraha.toml (existing merge logic)
  8. Write .env.example (existing logic)
  9. IF options.sync_knowledge_system:
     Call sync_knowledge_system(root) → SyncResult
     Write sync metadata (.samgraha/sync-meta.json)

  ── Phase 4: Result ──
  10. Return InitResult with sync_result
```

### 2.5 `sync_knowledge_system()` — Extracted Single Implementation

**File:** `crates/services/src/init.rs` (new function)

Named to reflect the architecture: this function synchronizes an entire Knowledge System (standards, help, templates, scoring, scripts) — not just "standards".

```rust
/// Synchronize the declared Knowledge System from the global store
/// (mcp_dir()) into a repo's local .samgraha/. Single implementation
/// shared by CLI `knowledge pull`, MCP `sync_standards`, and `init_repository()`.
pub fn sync_knowledge_system(root: &Path) -> Result<SyncResult> {
    let mcp_dir = common::env::mcp_dir();
    let local_db = root.join(".samgraha").join("standards.db");
    let source_db = mcp_dir.join("standards.db");

    // 1. Copy standards.db (with integrity + schema version check)
    let standards_synced = if source_db.exists() {
        let check_conn = rusqlite::Connection::open(&source_db)?;
        let ok: String = check_conn.query_row(
            "PRAGMA integrity_check", [], |row| row.get(0)
        )?;
        if ok != "ok" {
            bail!("Standards DB integrity check failed: {}", ok);
        }
        standards::check_schema_version(&check_conn)?;
        if let Some(parent) = local_db.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&source_db, &local_db)?;
        true
    } else {
        false
    };

    // 2. Sync help docs into knowledge.db
    let help_documents_synced = builtin::sync_help_into_local(root)?;

    // 3. Sync scripts/
    let source_scripts = mcp_dir.join("scripts");
    let mut scripts_synced = 0usize;
    if source_scripts.exists() {
        let local_scripts = root.join(".samgraha").join("scripts");
        std::fs::create_dir_all(&local_scripts)?;
        for entry in std::fs::read_dir(&source_scripts)? {
            let entry = entry?;
            std::fs::copy(entry.path(), local_scripts.join(entry.file_name()))?;
            scripts_synced += 1;
        }
    }

    Ok(SyncResult { standards_synced, help_documents_synced, scripts_synced })
}
```

**Why extract:** Both CLI and MCP have identical sync logic copy-pasted. A single function eliminates drift. `init_repository()` becomes the third caller without triplication.

### 2.6 Auto-Detection Algorithm

```
probe_directories(root) -> DetectedDirs:
  detected = DetectedDirs::default()

  IF root.join("docs").is_dir():
    detected.root_dir = Some("docs")
  ELIF root.join("documentation").is_dir():
    detected.root_dir = Some("documentation")

  IF root.join("crates").is_dir():
    detected.implementation_dir = Some("crates")
  ELIF root.join("src").is_dir():
    detected.implementation_dir = Some("src")
  ELIF root.join("lib").is_dir():
    detected.implementation_dir = Some("lib")

  IF root.join("tests").is_dir():
    detected.tests_dir = Some("tests")
  ELIF root.join("test").is_dir():
    detected.tests_dir = Some("test")

  IF root.join("scripts").is_dir():
    detected.scripts_dir = Some("scripts")

  return detected
```

**Rules:**
- Probe only, never create directories
- Set literal paths (not env placeholders) when found
- Prefer `crates/` over `src/` for Rust workspaces
- Skip missing dirs silently — user adds manually
- Applied to both fresh init and backfill mode (only fills unset values)

### 2.7 Sync Metadata

**File:** `crates/services/src/init.rs`

After a successful `sync_knowledge_system()`, write lightweight metadata to `.samgraha/sync-meta.json`:

```json
{
  "system": "samgraha-documentation",
  "version": "1.0.0",
  "synced_at": "2026-07-15T12:00:00Z"
}
```

**No hashes.** Version comparison is the correct abstraction for local-to-local sync. The `standards` table already has a `version` column (`schema/knowledge-hub/02-standards.sql:10`). We read it during sync and store it in the metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncMeta {
    pub system: String,
    pub version: String,
    pub synced_at: String,
}
```

### 2.8 Staleness Detection

**File:** `crates/services/src/init.rs`

```rust
pub enum StalenessStatus {
    /// No sync-meta.json — never synced.
    NeverSynced,
    /// No local standards.db — needs initial sync.
    MissingLocal,
    /// Local version matches global — up to date.
    UpToDate { version: String },
    /// Local version differs from global — stale.
    Stale { local_version: String, global_version: String },
    /// Global source DB doesn't exist.
    SourceMissing,
}

pub fn check_knowledge_staleness(root: &Path) -> Result<StalenessStatus> {
    let meta_path = root.join(".samgraha").join("sync-meta.json");
    if !meta_path.exists() {
        return Ok(StalenessStatus::NeverSynced);
    }
    let meta: SyncMeta = serde_json::from_str(&std::fs::read_to_string(&meta_path)?)?;

    let source_db = common::env::mcp_dir().join("standards.db");
    if !source_db.exists() {
        return Ok(StalenessStatus::SourceMissing);
    }

    let local_db = root.join(".samgraha").join("standards.db");
    if !local_db.exists() {
        return Ok(StalenessStatus::MissingLocal);
    }

    // Read current version from global DB
    let conn = rusqlite::Connection::open(&source_db)?;
    let global_version: String = conn.query_row(
        "SELECT version FROM standards WHERE name = 'documentation-standards' LIMIT 1",
        [], |row| row.get(0),
    ).unwrap_or_default();

    if global_version == meta.version {
        Ok(StalenessStatus::UpToDate { version: meta.version })
    } else {
        Ok(StalenessStatus::Stale {
            local_version: meta.version,
            global_version,
        })
    }
}
```

**Why version not hash:** Hashs are for corruption detection against untrusted sources. Here we sync between two local databases we control. Version comparison tells us if the knowledge system has been updated globally. Simple, fast, correct.

### 2.9 CLI Changes

**File:** `crates/cli/src/commands.rs`

#### Init command — new args

Current definition at line 141-148:
```rust
Commands::Init {
    path: Option<PathBuf>,
    #[arg(long = "force")]
    force: bool,
}
```

New definition:
```rust
Commands::Init {
    path: Option<PathBuf>,
    #[arg(long = "force", help = "Overwrite existing configuration")]
    force: bool,
    #[arg(long = "standard-system", help = "Document standard system name (e.g. 'samgraha-documentation')")]
    standard_system: Option<String>,
    #[arg(long = "auto-detect", help = "Auto-detect docs/src/tests/scripts directories")]
    auto_detect: bool,
    #[arg(long = "sync", help = "Sync Knowledge System from global store after init")]
    sync: bool,
}
```

Updated dispatch at line 406:
```rust
Commands::Init { path, force, standard_system, auto_detect, sync }
    => self.execute_init(
        path.as_ref(), *force, standard_system.as_deref(),
        *auto_detect, *sync, &format
    ),
```

Updated `execute_init()` at line 1014-1029:
```rust
fn execute_init(
    &self,
    path: Option<&PathBuf>,
    force: bool,
    standard_system: Option<&str>,
    auto_detect: bool,
    sync: bool,
    format: &OutputFormat,
) -> Result<ExitCode> {
    let root = path.cloned()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    let options = services::init::InitOptions {
        force,
        standard_system: standard_system.map(|s| s.to_string()),
        auto_detect_dirs: auto_detect,
        sync_knowledge_system: sync,
        ..Default::default()
    };
    let result = services::init::init_repository(&root, &options)?;

    println!("{}", result.status);

    if let Some(ref sync) = result.sync_result {
        println!("Standards synced: {}",
            if sync.standards_synced { "yes" } else { "no (source not found)" });
        println!("Help documents synced: {}", sync.help_documents_synced);
        println!("Scripts synced: {}", sync.scripts_synced);
    }

    println!("Generated {}", result.env_path.display());
    println!("{}", format_output(&result.config, format));
    Ok(ExitCode::Success)
}
```

#### Knowledge command — new subcommands

Current `KnowledgeAction` at line 263-287:
```rust
enum KnowledgeAction {
    Publish { ... },
    Pull,
}
```

New definition:
```rust
enum KnowledgeAction {
    Publish { ... },
    Pull {
        #[arg(long = "force", help = "Force re-sync even if local copy appears current")]
        force: bool,
    },
    Status,
}
```

`Pull` with `--force`:
```rust
KnowledgeAction::Pull { force } => {
    if !force {
        match services::init::check_knowledge_staleness(&root) {
            Ok(StalenessStatus::UpToDate { version }) => {
                println!("Knowledge System v{} is up to date. Use --force to re-sync.", version);
                return Ok(ExitCode::Success);
            }
            Ok(status) => tracing::info!("Staleness check: {:?}", status),
            Err(e) => tracing::warn!("Staleness check failed: {}", e),
        }
    }
    // ... existing pull logic, but calling sync_knowledge_system() ...
    let result = services::init::sync_knowledge_system(&root)?;
    // ... print results ...
}
```

`Status`:
```rust
KnowledgeAction::Status => {
    let meta_path = root.join(".samgraha").join("sync-meta.json");
    if !meta_path.exists() {
        println!("Never synced — run `samgraha knowledge pull` to sync from global.");
        return Ok(ExitCode::Success);
    }
    let meta: services::init::SyncMeta = serde_json::from_str(
        &std::fs::read_to_string(&meta_path)?
    )?;
    println!("Knowledge System: {}", meta.system);
    println!("Version:          {}", meta.version);
    println!("Last sync:        {}", meta.synced_at);

    match services::init::check_knowledge_staleness(&root)? {
        StalenessStatus::UpToDate { .. } => println!("Status:           Up to date"),
        StalenessStatus::Stale { local_version, global_version } => {
            println!("Status:           STALE (local v{}, global v{})", local_version, global_version);
            println!("                  Run `samgraha knowledge pull` to update.");
        }
        StalenessStatus::SourceMissing => println!("Status:           Source DB missing"),
        _ => println!("Status:           Unknown"),
    }
}
```

### 2.10 MCP Tool Schema Changes

**File:** `crates/mcp/src/main.rs`

#### Updated `init` tool definition

```json
{
    "name": "init",
    "description": "Initialize samgraha.toml and .samgraha/ for a repository. Optionally select a document standard system, auto-detect directories, and sync the Knowledge System from global — all in one pass.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "force": { "type": "boolean", "description": "Overwrite existing samgraha.toml with a fresh template instead of backfilling missing keys" },
            "repo_path": { "type": "string", "description": "Absolute path to the repository to initialize" },
            "standard_system": { "type": "string", "description": "Document standard system name (e.g. 'samgraha-documentation')" },
            "script_overrides": { "type": "object", "description": "Map of rule_id -> script path for [repository.documentation.script_overrides]" },
            "check_overrides": { "type": "object", "description": "Map of check_name -> script path for [repository.documentation.check_overrides]" },
            "auto_detect": { "type": "boolean", "description": "Probe repo for docs/, src|crates/, tests/, scripts/ and set literal paths if found" },
            "sync": { "type": "boolean", "description": "Sync Knowledge System from global store into .samgraha/ after init" }
        }
    }
}
```

#### Updated `sync_standards` tool — add `force`

```json
{
    "name": "sync_standards",
    "description": "Pull the Knowledge System from global store into this repo's local .samgraha/. Checks staleness by default; pass force=true to re-sync unconditionally.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "force": { "type": "boolean", "description": "Force re-sync even if local copy appears current" },
            "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
        }
    }
}
```

#### New `check_knowledge_staleness` tool

```json
{
    "name": "check_knowledge_staleness",
    "description": "Check whether this repo's local Knowledge System is up-to-date vs the global store. Returns staleness status without syncing.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "repo_path": { "type": "string", "description": "Absolute path to a different local repository to target" }
        }
    }
}
```

### 2.11 MCP Adapter Changes

**File:** `crates/mcp/src/adapter.rs`

Updated `handle_init()` at line 397-414:

```rust
fn handle_init(&self, req: &McpRequest) -> Result<serde_json::Value> {
    let force = req.params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    let standard_system = req.params.get("standard_system")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let script_overrides: std::collections::HashMap<String, String> =
        req.params.get("script_overrides")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
    let check_overrides: std::collections::HashMap<String, String> =
        req.params.get("check_overrides")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
    let auto_detect = req.params.get("auto_detect")
        .and_then(|v| v.as_bool()).unwrap_or(false);
    let sync = req.params.get("sync")
        .and_then(|v| v.as_bool()).unwrap_or(false);

    let owned_root;
    let root: &Path = match req.params.get("repo_path").and_then(|v| v.as_str()) {
        Some(p) => { owned_root = std::path::PathBuf::from(p); &owned_root }
        None => &self.runtime.context.repository_root,
    };

    let options = services::init::InitOptions {
        force,
        standard_system,
        script_overrides,
        check_overrides,
        auto_detect_dirs: auto_detect,
        sync_knowledge_system: sync,
    };
    let result = services::init::init_repository(root, &options)?;

    Ok(serde_json::json!({
        "status": result.status,
        "root": result.root.display().to_string(),
        "env_path": result.env_path.display().to_string(),
        "config": result.config,
        "sync_result": result.sync_result.as_ref().map(|s| serde_json::json!({
            "standards_synced": s.standards_synced,
            "help_documents_synced": s.help_documents_synced,
            "scripts_synced": s.scripts_synced,
        })),
    }))
}
```

Updated `handle_sync_standards()` — add staleness check + force:

```rust
fn handle_sync_standards(&self, req: &McpRequest) -> Result<serde_json::Value> {
    let force = req.params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    let owned_root;
    let root: &Path = match req.params.get("repo_path").and_then(|v| v.as_str()) {
        Some(p) => { owned_root = std::path::PathBuf::from(p); &owned_root }
        None => &self.runtime.context.repository_root,
    };

    if !force {
        match services::init::check_knowledge_staleness(root)? {
            StalenessStatus::UpToDate { version } => {
                return Ok(serde_json::json!({
                    "synced": false,
                    "reason": "up_to_date",
                    "version": version,
                }));
            }
            _ => {}
        }
    }

    let result = services::init::sync_knowledge_system(root)?;
    // ... write sync-meta.json ...
    Ok(serde_json::json!({
        "synced": true,
        "standards_synced": result.standards_synced,
        "help_documents_synced": result.help_documents_synced,
        "scripts_synced": result.scripts_synced,
    }))
}
```

New `handle_check_staleness()`:

```rust
fn handle_check_staleness(&self, req: &McpRequest) -> Result<serde_json::Value> {
    let owned_root;
    let root: &Path = match req.params.get("repo_path").and_then(|v| v.as_str()) {
        Some(p) => { owned_root = std::path::PathBuf::from(p); &owned_root }
        None => &self.runtime.context.repository_root,
    };
    let status = services::init::check_knowledge_staleness(root)?;
    Ok(serde_json::json!({
        "status": match &status {
            StalenessStatus::NeverSynced => "never_synced",
            StalenessStatus::MissingLocal => "missing_local",
            StalenessStatus::UpToDate { .. } => "up_to_date",
            StalenessStatus::Stale { .. } => "stale",
            StalenessStatus::SourceMissing => "source_missing",
        },
        "detail": format!("{:?}", status),
    }))
}
```

---

## 3. File Change Summary

| # | File | Lines Changed | What |
|---|------|--------------|------|
| 1 | `crates/common/src/config.rs` | +15 | Add `InitOptions` struct with `Default` impl |
| 2 | `crates/services/src/init.rs` | +220, ~30 modified | Add `SyncMeta`, `SyncResult`, `StalenessStatus`, `check_knowledge_staleness()`, `sync_knowledge_system()`, `probe_directories()`, rewrite `init_repository()` |
| 3 | `crates/cli/src/commands.rs` | +55, ~15 modified | Add init CLI args, `KnowledgeAction::Status`, `Pull { force }`, update dispatch and `execute_init()` |
| 4 | `crates/mcp/src/main.rs` | ~30 modified | Update `init` tool, add `check_knowledge_staleness` tool, update `sync_standards` with `force` |
| 5 | `crates/mcp/src/adapter.rs` | +40, ~10 modified | Update `handle_init()`, add `handle_check_staleness()`, update `handle_sync_standards()` |

**Total:** ~360 lines added, ~55 lines modified across 5 files.

**Removed from original proposal:**
- `update_registry` / `clear_registry` from `InitOptions` (init ≠ registration)
- `auto_sync_standards` config option (deferred)
- SHA-256 / `file_sha256()` / hash-based staleness (version comparison instead)
- Complex `sync-meta.json` structure (3 fields only)
- `DocumentsSyncInfo`, `DbSyncInfo`, `ScriptsSyncInfo` structs (single `SyncMeta` struct)

---

## 4. Backward Compatibility

### 4.1 Default Behavior Preserved

`InitOptions::default()` yields:
```rust
InitOptions {
    force: false,
    standard_system: None,
    script_overrides: HashMap::new(),
    check_overrides: HashMap::new(),
    auto_detect_dirs: false,
    sync_knowledge_system: false,
}
```

Any caller that passes `&InitOptions::default()` gets the exact same behavior as `init_repository(root, false)` today.

### 4.2 Existing `samgraha.toml` Files Unaffected

Backfill mode (`merge_missing_keys`) only adds keys that are missing. `standard_system`, `script_overrides`, and `check_overrides` already exist in the schema with `#[serde(default)]` — backfill won't touch them unless literally absent.

### 4.3 CLI Without New Flags

`samgraha init` with no flags = `InitOptions::default()` = current behavior. All new flags are opt-in.

### 4.4 MCP Without New Params

`init` tool call with no new params = same as above. Existing MCP clients continue working.

---

## 5. Edge Cases & Error Handling

| Scenario | Behavior |
|----------|----------|
| `auto_detect` but no `docs/` found | Skip `root_dir` — stays as `${SAMGRAHA_DOCS_DIR}` placeholder |
| `auto_detect` and both `src/` and `crates/` exist | Prefer `crates/` (Rust workspace convention) |
| `sync` but `mcp_dir()/standards.db` doesn't exist | `standards_synced = false`, no error. Help sync still runs. |
| `sync` but standards.db fails integrity check | **Error returned** — partial sync is worse than no sync |
| `standard_system` set to nonexistent system | Written to TOML. Error surfaces at compile/audit time. |
| `force` + `auto_detect` | Fresh TOML with detected dirs. Old TOML overwritten. |
| Existing TOML + `auto_detect` (backfill) | Only fills in missing dirs — never overwrites user-set values. |
| `knowledge pull` when already up-to-date | Skip copy, print "Already up to date" |
| `knowledge pull --force` when up-to-date | Re-copy anyway, update sync metadata |
| `check_staleness` with no `sync-meta.json` | Returns `NeverSynced` |
| `check_staleness` with no local `standards.db` | Returns `MissingLocal` |
| `check_staleness` when global source deleted | Returns `SourceMissing` |
| Version comparison failure | Treat as stale, sync with warning |

---

## 6. Testing Plan

### 6.1 Unit Tests (in `crates/services/src/init.rs`)

| Test | Validates |
|------|-----------|
| `init_options_default_matches_old_behavior` | `InitOptions::default()` produces same TOML as old `init_repository(root, false)` |
| `auto_detect_docs_directory` | Temp dir with `docs/` → TOML has `root_dir = "docs"` |
| `auto_detect_crates_over_src` | Temp dir with both `crates/` and `src/` → TOML has `dir = "crates"` |
| `auto_detect_tests_and_scripts` | Temp dir with `tests/` and `scripts/` → TOML has those sections |
| `auto_detect_skips_missing` | Temp dir with only `docs/` → no `tests` or `scripts` sections |
| `standard_system_propagation` | `standard_system: Some("test-sys")` → TOML has it |
| `check_overrides_propagation` | `check_overrides: {"build": "scripts/build.sh"}` → TOML has it |
| `sync_knowledge_system_copies_db` | Mock `mcp_dir()` with fake `standards.db` → local copy exists |
| `sync_knowledge_system_handles_missing_source` | No `standards.db` → `standards_synced = false`, no error |
| `sync_meta_written_after_sync` | After sync, `.samgraha/sync-meta.json` has system/version/timestamp |
| `staleness_detection_up_to_date` | Same version → `UpToDate` |
| `staleness_detection_stale` | Different version → `Stale` |
| `staleness_detection_never_synced` | No `sync-meta.json` → `NeverSynced` |
| `backfill_does_not_overwrite_existing_dirs` | Existing TOML + `auto_detect` → stays as-is |

### 6.2 Integration Tests

| Test | Validates |
|------|-----------|
| `full_init_with_all_options` | Fresh init with `standard_system`, `auto_detect`, `sync` on temp repo |
| `knowledge_pull_force_re_syncs` | `--force` re-copies even when version matches |
| `knowledge_pull_skips_when_current` | Without `--force`, skips when version matches |
| `knowledge_status_shows_sync_info` | `knowledge status` prints correct system/version/timestamp |

---

## 7. Implementation Order

1. **`crates/common/src/config.rs`** — Add `InitOptions` struct
2. **`crates/services/src/init.rs`** — Add `SyncMeta`, `SyncResult`, `StalenessStatus`, `check_knowledge_staleness()`, `sync_knowledge_system()`, `probe_directories()`, rewrite `init_repository()` with four phases
3. **`crates/cli/src/commands.rs`** — Add init CLI args, `KnowledgeAction::Status`, `Pull { force }`, update dispatch and `execute_init()`
4. **`crates/mcp/src/main.rs`** — Update `init` tool definition, add `check_knowledge_staleness` tool, update `sync_standards` with `force`
5. **`crates/mcp/src/adapter.rs`** — Update `handle_init()`, add `handle_check_staleness()`, update `handle_sync_standards()` with staleness check
6. **Run `cargo build`** — Verify compilation
7. **Run `cargo test`** — Verify all existing + new tests pass
8. **Run `cargo clippy`** — Lint check
