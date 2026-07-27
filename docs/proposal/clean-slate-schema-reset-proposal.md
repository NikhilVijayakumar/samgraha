# Clean-Slate Schema Reset Proposal — No In-Place Migrations, File-Copied Standards, Seeder-Driven Registration

**Status**: DRAFT — not yet implemented.

**Amends**: `docs/proposal/knowledge-standard-management-proposal.md` §2.1
(global registry — `source_path` no longer points at a standard's external
repo, §3.7 below), §2.7 (`template` as inline-content-only, now also
file-copied, §3.7), and `register_standard`'s Rust-side manifest-to-SQL
translation for workflow shape (§3.8 replaces it with a standard-provided
seeder script for `usecase`/`domain`/`script`/`prompt`/`template` rows —
`custom_tables:`/`smoke_test`/`category`/`extends` stay manifest fields,
they're identity/gating metadata, not workflow shape). Nothing in the
earlier proposal's schema (the 15 `RESERVED_TABLE_NAMES` tables) changes —
only *how* rows get into them.

**Revision 2 note**: rev. 1 designed the wipe-and-reregister mechanism
around a single trigger (an MCP schema-epoch bump). This revision adds a
second trigger — a standard's own content changing — and a materially
different registration architecture underneath both: standards.db stops
pointing at a standard's external source repo and instead holds a local
copy of its files (§3.7); a standard provides an executable **seeder
script** that inserts its own workflow rows directly, rather than samgraha
parsing a declarative manifest into SQL (§3.8); files land in a
configurable, explicit `.samgraha` directory (§3.10) with paths
absolutized after seeding (§3.11); generated output lands in a fixed
`.samgraha/output/` folder (§3.12). §1.6-1.9 are the new verified findings
behind this; §2 states the one deliberate reversal plainly, the same way
earlier rounds of this conversation have flagged reversals rather than
burying them.

**Revision 3 note**: a review of rev. 2 found the seeder contract (§3.8)
was justified by pointing at `pcems_2026`/`base_academic`'s existing
`init_schema.py` as though it already did most of the job — it doesn't
(verified: neither inserts a `usecase`/`script`/`prompt`/`step` row
anywhere). Per direction this round: **the spec is defined from samgraha's
own side, grounded in samgraha's own existing mechanisms
(`run_capability_script`'s real signature, verified directly against
`env.rs` this round) — not derived from, or justified by, what any
standard in `Kriti/` currently happens to contain.** §3.8 is rewritten on
that basis. Three open technical gaps from the review are resolved, not
left as options: `samgraha_dir` discovery is an `--in`-envelope key
(§3.8), a `..` segment in a seeder-written path is a rejected contract
violation, never silently resolved (§3.11), and `run_seeder` is a real
function with its own envelope-construction and status-validation
responsibility, not a 4-line pass-through (§3.8). §3.14 (new) answers the
staleness-visibility gap by comparison, not a new stored flag. §7 (new)
is the full disposition of every point the review raised.

---

## 1. Verified Current State

### 1.1-1.5 — carried over from rev. 1, unchanged

Three migration systems exist today (`core_schema.rs`, `migration.rs`,
`standards_db.rs`), all version-array-and-gate shaped (§1.1). `RESERVED_TABLE_NAMES`
already lists every samgraha-owned `knowledge.db` table (§1.2). Standard-owned
custom data lives in the same physical `knowledge.db` file as samgraha's
own tables — verified via `schema/knowledge/README.md` and the archived
mcp-execution-substrate proposal (§1.3). `delete_existing`'s row-scoped,
`WHERE standard = ?1` cleanup is already the safe per-standard pattern
(§1.4). `schema/knowledge/` is missing 7 files (`09`-`15`) that exist only
as a Rust string constant (§1.5) — closing this is still Phase 1, unchanged.

### 1.6 File-copy primitives already exist, unused by the live path

`crates/common/src/fs_sync.rs`: `copy_dir_recursive`, `copy_dir_atomic`,
`DEFAULT_EXCLUDES` (`&["**/__pycache__/**", "**/*.pyc"]`). Built for the
archived asset-sync era (`standard-asset-sync-proposal.md`), never removed,
never wired into the current `register_standard`/`register_standard_globally`
path — `init.rs`'s own comment (verified in the earlier proposal, §1.2) says
plainly there's "nothing to copy into a fresh repo anymore" under the
current model. §3.7/§3.9 reuse these functions as-is rather than writing
new copy logic — they're already correct, already tested (implied by
their continued presence and the standard-asset-sync-proposal's own
verification work), just currently dead code.

### 1.7 `.samgraha` directory location is hardcoded, no config override

Exactly two call sites construct it: `init.rs:40` (`root.join(".samgraha")`)
and `adapter.rs:85` (`self.target_root(req).join(".samgraha").join("knowledge.db")`).
No `samgraha.toml` field controls this today — §3.10 adds one.

### 1.8 `registry.db` already has its `.sql` mirror; `standards.db` still doesn't

`schema/registration/00-repository_cache.sql` exists, mirrors `registry.db`
exactly, and its own header comment says so explicitly (*"mirrored here as
the canonical reference copy"*). `standards.db`'s schema
(`standard_registry`, `operation_log`, from `standards_db.rs`'s `STD_V1`)
has **no** `.sql` mirror anywhere in `schema/` — a second instance of the
same drift class §1.5 already found for `knowledge.db`'s `CORE_V2`. Both
gaps close together in Phase 1 (§4).

### 1.9 A seeder script can't run through `run_script_step` — a real bootstrap-ordering problem

`run_script_step(knowledge_db_path, step_id, ...)` (`step_execution.rs`)
starts with `SELECT kind FROM step WHERE id = ?1` — it requires the `step`
row to already exist. A seeder script's entire job (§3.8) is to *create*
`usecase`/`step`/`script`/`prompt`/`domain`/`template` rows in the first
place — there is no `step_id` to pass in before it's run once. This isn't
a detail to patch later; it means the seeder needs its **own** invocation
path, structurally separate from `run_script_step`, from the start (§3.8).

---

## 2. The reversal, stated plainly

`init.rs`'s own comment (quoted in the earlier proposal, §1.2) documents a
deliberate decision made during the "pivot to MCP execution substrate":
*"No knowledge-system sync step ... there's nothing to copy into a fresh
repo anymore."* This proposal reverses that decision — files get copied
again, twice (into an mcp-owned registry, then into each consuming repo's
own `.samgraha/`), because the new requirement (standards.db stores no
external pointers, a standard's files must survive a wipe-and-reregister
cycle without depending on the standard's original repo still being
reachable at the same path) can't be met any other way. Naming this
outright rather than silently reintroducing the exact mechanism a past
decision removed: the earlier decision was right for the model it was
made under (samgraha only ever recorded absolute paths, so nothing needed
copying); it's wrong for this one (samgraha now needs its own durable copy,
independent of the standard's source repo's continued existence at a fixed
path).

---

## 3. Design

### 3.1-3.6 — carried over from rev. 1, with one amendment

Table-scoped reset for `knowledge.db` via `RESERVED_TABLE_NAMES` (§3.2),
single-`CORE_SCHEMA`-plus-epoch instead of a versioned array (§3.3),
`schema/knowledge/` made authoritative (§3.4, extended by §1.8 to also
cover `standards.db`'s new mirror), the survives/destroyed table (§3.6) —
all unchanged in mechanism. **Amendment to §3.1**: `standards.db` is no
longer described as storing "nothing but metadata pointing elsewhere" — it
now also indirectly owns a file tree (§3.7). A full wipe of `standards.db`
must be paired with deleting `mcp_dir()/registry/` too, not just the SQLite
file — both are addressed together in §3.7's reset step, not left as two
separate, easy-to-forget operations.

### 3.7 The MCP registry — files, not a pointer

`register_standard_globally(path)` changes from "record `path` as
`source_path`" to:

1. Copy `path` (the standard's source directory, wherever the operator
   points it — still typically inside `Kriti`) into
   `mcp_dir()/registry/<standard-name>/`, via the existing
   `copy_dir_recursive` (§1.6) — full tree: scripts, prompts, templates,
   the manifest, and the seeder/verify/smoke-test scripts §3.8 introduces.
2. Run the structural verify-gate (`smoke_test`, unchanged from the
   earlier proposal's §2.4.1) against **this copy**, not the original path
   — proves the copy is what actually gets used from here on, catching a
   copy-step bug immediately rather than only on first real per-repo use.
3. Upsert `standard_registry` with `source_path` now pointing at
   `mcp_dir()/registry/<standard-name>/` — the **local** copy. The
   original `path` argument is used once, for the copy, and never stored.

This is what "we do not need a db for standard as it stores nothing"
means concretely: `standard_registry` still holds a database row, but
that row is pure metadata (name/category/subcategory/version/verify_status/
timestamps) pointing at a file tree samgraha itself owns and controls —
never a live pointer into an external repo that might move, get deleted,
or belong to a machine this MCP instance can't reach. A registered
standard is fully self-contained under `mcp_dir()` from this point on.

**Re-registering the same standard** (content changed, not an MCP version
bump) follows §3.13, not a plain re-copy over the old files — the two
triggers ("MCP updated" vs. "this standard updated") both end in the same
place (empty, then reseeded) but for different reasons, and both need
naming so an operator knows which one they're looking at in `operation_log`.

### 3.8 The Seeder Contract — defined from samgraha's side, not read off `Kriti/`

**Scope correction from the previous round**: this section previously
justified the seeder idea by pointing at `pcems_2026`/`base_academic`'s
existing `init_schema.py` as if it already did most of this job. On
inspection that framing overstated it — those scripts seed the standard's
*own* `academic_*` tables (domains, templates, visualization types,
calculation dependencies); neither one inserts a `usecase`, `script`,
`prompt`, `step`, `step_script`, or `step_prompt` row anywhere — those are
still parsed from `standard.yaml` by `register_standard.rs` today
(lines 152-260 of the version in this repo right now). Calling the seeder
contract below "a reuse of existing behavior" was wrong; it's a **new**
contract. No standard on disk implements it. This document defines it
samgraha-side — as a spec any future standard must satisfy to register —
without needing (or claiming) that `Kriti/`'s current content already
does. Whether/when any specific standard is authored or rewritten against
it is separate work, out of scope here, same stance this whole line of
proposals has taken toward `Kriti/` since the "read-only evidence" scoping
correction two rounds ago.

**The contract, in full — grounded in samgraha's own existing mechanisms,
not in any standard's current file layout:**

1. **Invocation.** Exactly `common::env::run_capability_script`'s existing
   signature (`crates/common/src/env.rs:400-404`, verified directly this
   round) — `(script_path, repo_root, input_json_path, timeout_secs)`,
   dispatched through the same `script_command` interpreter-by-extension
   table every other script already goes through (`.py`→Python, `.sh`→shell,
   `.ps1`→pwsh, `env.rs:119-151`). No new CLI flags, no new interpreter
   dispatch. This *is* the reuse — the calling convention, not any
   standard's specific script content.
2. **Input** (`--in` JSON payload) — carries exactly two keys the seeder
   needs and can't derive from `--repo-root` alone (§3.10 is why one of
   these is now required rather than inferred):
   ```json
   { "_samgraha_dir": "/abs/path/to/.samgraha", "_knowledge_db": "/abs/path/to/.samgraha/knowledge.db" }
   ```
   **Decided** (resolves the samgraha_dir-discovery question directly):
   these are injected into the `--in` envelope by the Rust caller before
   invocation — the same mechanism the earlier proposal's §2.10 already
   established for `_git` (commit_sha/branch/dirty). One consistent answer
   for "how does a script learn something about its environment it can't
   compute itself": an underscore-prefixed key in the envelope it already
   reads, never a new flag, never a config file the script parses itself.
   This is why a seeder must **not** hardcode
   `repo_root / ".samgraha" / "knowledge.db"` the way today's
   `_adapter.py:parse_step_args` does — that helper's convention predates
   §3.10's configurable `samgraha_dir` and is now the *wrong* thing for a
   seeder to copy. (Every other existing script that does hardcode this is
   unaffected — they don't change under this proposal, only the seeder
   contract requires the explicit key.)
3. **Responsibilities** — the seeder is the *only* thing that inserts rows
   into `usecase`, `domain`, `script`, `prompt`, `template`, `step`,
   `step_script`, `step_prompt` for its own standard (`custom_tables:` stays
   a declarative manifest field — pure catalog metadata, no rows to write,
   unchanged). Every `location` value it writes for `script`/`standard_asset`
   rows **must** be relative to `_samgraha_dir` and **must not** contain a
   `..` segment — §3.11's absolutize pass enforces this by rejecting, not
   resolving, any path that violates it (a stricter rule than this
   proposal's own previous draft left open, closed below).
4. **Output** — same JSON-envelope-out convention as every other script
   (`{"status": "ok" | anything-else}`); a seeder is only ever run once per
   registration (bootstrap, not per-step), so its own output is logged to
   `operation_log`, never fed into `execution` (that table stays scoped to
   actual usecase steps, unchanged).

**Bootstrap invocation — a real function, not a thin wrapper.** Constructs
the enriched envelope itself (not left to the caller) and validates the
returned status, matching every other execution entry point's own
discipline:

```rust
/// Runs a standard's seeder script once, against a target repo. Unlike
/// run_script_step, this needs no pre-existing step_id — the seeder is
/// what creates step/usecase rows in the first place, so it can't be
/// resolved through a step lookup the way every other dispatch is.
pub fn run_seeder(
    repo_root: &Path,
    seeder_script_path: &Path,
    samgraha_dir: &Path,
    knowledge_db: &Path,
    timeout_secs: Option<u64>,
) -> Result<serde_json::Value> {
    let payload = serde_json::json!({
        "_samgraha_dir": samgraha_dir.display().to_string(),
        "_knowledge_db": knowledge_db.display().to_string(),
    });
    let in_path = std::env::temp_dir().join(format!("samgraha-seed-in-{}.json", uuid::Uuid::new_v4()));
    std::fs::write(&in_path, serde_json::to_string(&payload)?)?;
    let result = common::env::run_capability_script(seeder_script_path, repo_root, &in_path, timeout_secs);
    let _ = std::fs::remove_file(&in_path);
    let result = result?;
    let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("ok");
    if status != "ok" {
        bail!("seeder script for reported non-ok status: {status}");
    }
    Ok(result)
}
```

### 3.9 Per-repo copy — `.samgraha/<standard-name>/`, then run the seeder locally

`register_standard(standard_name, repo_root)` (per-repo call, renamed
conceptually from "parse this path's manifest" to "activate this
already-globally-registered standard here") becomes:

1. Look up `standard_registry` (global) by name → get `source_path`
   (the mcp-registry copy, §3.7).
2. Copy `source_path` into `repo_root/.samgraha/<standard-name>/` via
   `copy_dir_recursive` (§1.6) — same function, same exclude list, second
   use.
3. Call `run_seeder(repo_root, <local-copy>/<seeder_script>, samgraha_dir,
   knowledge_db, timeout)` (§3.8) — `samgraha_dir`/`knowledge_db` resolved
   from `RepositoryConfig.samgraha_dir` (§3.10), not assumed.
4. Run §3.11's absolutize pass — rejects (fails the whole registration,
   no partial state) if any `location` the seeder wrote contains a `..`
   segment.
5. Write the local `standard` row (unchanged from the earlier proposal's
   §2.13) — `category`/`subcategory`/`extends`/`version` now read from the
   global `standard_registry` row (already parsed once, at §3.7's global
   registration time) rather than re-parsing a manifest here — the local
   `register_standard` call no longer touches YAML at all.

`RESERVED_TABLE_NAMES`'s reset (§3.2, rev. 1) already handles the "re-run
this and get a clean slate" case (all these tables get dropped and
recreated on an epoch bump); step 2's per-repo file copy is idempotent by
construction (`copy_dir_recursive` overwrites) so re-running steps 1-4
after a reset is exactly "re-register," no special case needed.

### 3.10 `samgraha.toml` — explicit, absolute `.samgraha` location

New field on `RepositoryConfig` (`crates/common/src/config.rs`):

```toml
[repository]
samgraha_dir = "${SAMGRAHA_DIR}"   # absolute path; unset falls back to
                                    # <repo-root>/.samgraha (today's only
                                    # behavior, unchanged default)
```

Resolved the same way every other `${VAR}`-style field in this file
already is (`resolve_configured_dir`, referenced throughout the existing
`samgraha.toml` comments) — no new resolution mechanism, one more field
using the one that exists. `init.rs:40` and `adapter.rs:85`'s two hardcoded
`root.join(".samgraha")` call sites both become
`resolve_configured_dir(&config.repository.samgraha_dir, root)`-equivalent
lookups instead of a literal join — small, mechanical, contained to two
call sites (§1.7 already confirmed there are only two).

### 3.11 Absolutize pass — the seeder writes relative, the DB stores absolute; `..` is rejected, never resolved

**Closed, not left open**: a seeder-written path containing a `..` segment
(the kind of cross-tree reference a hand-authored `standard.yaml` has today
— not a concern for a *new* seeder contract, since §3.8 states outright
that every `location` must be relative to `_samgraha_dir` with no `..`
allowed) is a **contract violation**, rejected loudly, not silently walked
and resolved. A seeder that needs to reference something outside its own
copied tree writes an absolute path instead (the `NOT LIKE '/%'` guard
below already treats absolute as "leave alone") — the contract has exactly
two valid shapes for a location value: relative-with-no-`..`, or absolute.
Nothing in between.

```rust
fn absolutize_paths(conn: &Connection, standard: &str, samgraha_dir: &Path) -> Result<()> {
    let prefix = samgraha_dir.join(standard).display().to_string();
    for table in ["script", "standard_asset"] {
        let mut stmt = conn.prepare(&format!(
            "SELECT id, location FROM {table} WHERE standard = ?1 AND location NOT LIKE '/%'"
        ))?;
        let rows: Vec<(i64, String)> = stmt.query_map(rusqlite::params![standard], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(|r| r.ok()).collect();
        for (id, location) in rows {
            if location.split('/').any(|seg| seg == "..") {
                bail!("seeder for '{standard}' wrote a '..'-containing relative location ('{location}') in {table} — not allowed; use an absolute path for anything outside the standard's own copied tree");
            }
            conn.execute(
                &format!("UPDATE {table} SET location = ?1 || '/' || location WHERE id = ?2"),
                rusqlite::params![prefix, id],
            )?;
        }
    }
    Ok(())
}
```

Only `script.location` and `standard_asset.location` need this — `prompt.content`
and `template.content` are inline text (read once, no path stored, unchanged
from the earlier proposal's §2.7 design), never relative paths to begin
with. `NOT LIKE '/%'` (in the `SELECT`, now, not the `UPDATE`) still guards
against double-prefixing a value the seeder already wrote absolute.

### 3.12 Artifacts land in `.samgraha/output/`

`artifact.location` (rows written via `run_script_step`'s existing
`result.artifacts[]` envelope read, earlier proposal §2.9) is expected to
be a path under `<samgraha_dir>/output/` — not enforced by a `CHECK`
constraint (samgraha doesn't validate a script's own output path choices,
consistent with never interpreting a standard's meaning), but every
script contract (`run_capability_script`) gains `--out-dir` alongside the
existing `--repo-root/--in/--out`, pointing at `<samgraha_dir>/output/`
(created if missing, once, at repo init time) — so a script that wants to
report an artifact has a conventional place to write it without inventing
its own path, and `.samgraha/output/` becomes the one place an operator
looks for "what did this repo's standards actually generate," across every
standard registered in it.

### 3.13 Standard update — same reset principle, applied to one standard instead of an epoch

Extends §3.2/§3.3 (rev. 1) with a second, narrower trigger. Re-running
`register_standard_globally(path)` for a standard already in
`standard_registry`:

1. Deletes `mcp_dir()/registry/<standard-name>/` entirely (not merged/
   diffed — a fresh copy, same reasoning as an epoch reset: no incremental
   migration, ever, for this either).
2. Re-copies `path` fresh (§3.7, step 1).
3. Re-runs the structural verify-gate against the new copy.
4. Every repo that had this standard active must re-run §3.9's per-repo
   flow — samgraha does not auto-propagate a global update into every
   consuming repo (same "no auto-rediscovery" stance as rev. 1's §3.5;
   a repo's own `register_standard(standard_name, repo_root)` call, run
   again, deletes and recopies its own `.samgraha/<standard-name>/` and
   reseeds — the row-scoped `RESERVED_TABLE_NAMES`-table cleanup already
   scoped `WHERE standard = ?1` (rev. 1 §1.4) handles the DB side; the file
   side (`.samgraha/<standard-name>/`) gets the same delete-then-recopy
   treatment as step 1, not a diff).
5. `operation_log` (earlier proposal §2.10/§8.1) gets a new `operation`
   value, `"update_standard"`, distinct from `"register_globally"` — so
   "was this a fresh registration or a re-registration after a content
   change" is answerable from the log, not inferred from a upsert's
   side-effects.

### 3.14 "Is this repo stale?" — answered by comparison, not a stored flag

No new column, no `reseed_required` flag to remember to set and clear.
The local `standard` table (earlier proposal §2.13) already stores the
`version` it last seeded at; the global `standard_registry` (§2.1) already
stores the current one. `get_standard_info`/`repository_status` (existing
tools) compute staleness live, at read time:

```rust
let stale = local_standard.version != global_registry_row.version;
```

Same reasoning as the earlier proposal's git-provenance design (§2.10's
"latest commit" answered by a query against `git_detail`, not a second
"latest" table) — a value derived by comparing two things that already
exist can't drift out of sync with itself the way a third, independently-
set flag could (forget to clear it, clear it too early, clear it on the
wrong repo). This needs the global lookup to be reachable at query time
(a repo working fully offline from `standards.db` sees no comparison,
reports "unknown" rather than "stale" or "current") — named as the one
limitation of not storing the flag locally, accepted because it only
affects a read-time nicety, never registration or seeding correctness.

---

## 4. Implementation Plan

### Phase 1 — Close both drift gaps first (rev. 1 §3.4 + §1.8 here)
`schema/knowledge/09-15*.sql` (rev. 1) **and** a new `schema/standards/`
(or `schema/registry/` — name TBD at implementation time, not load-bearing
here) directory mirroring `standards_db.rs`'s `STD_V1`. Same
byte-for-byte consistency test/`build.rs` generation as rev. 1 proposed,
applied to both.

### Phase 2 — Migration collapse (rev. 1, unchanged)
`CORE_SCHEMA_EPOCH`/`CORE_SCHEMA`/`ensure_current_schema`/
`reset_samgraha_tables` for `knowledge.db`; full-file-safe variants for
`registry.db`/`standards.db` — extended here so the `standards.db` variant
also deletes `mcp_dir()/registry/` (§3.1's amendment), not just the SQLite
file.

### Phase 3 — MCP registry + seeder bootstrap (new, this revision)
- `register_standard_globally`: copy-then-verify-then-upsert (§3.7).
- `run_seeder` (new function, `crates/services/src/`) — bootstrap
  invocation, no `step_id` required (§3.8, §1.9).
- `StandardManifest` shrinks: remove `scripts`/`prompts`/`usecases`/
  `domains`/`assets`/`templates` fields, add `seeder_script: String`. This
  is a breaking change to the manifest shape — every standard authored
  against the *previous* (register_standard.rs's Rust-parsed) shape needs
  its workflow declarations moved from YAML into an actual seeder script.
  Scoped identically to the earlier proposal's §3 (`pcems_2026` is
  evidence, not a repo this proposal edits) — nothing here requires
  touching `Kriti/` as part of landing this phase; it only changes what
  samgraha *expects* the next time a standard registers.

### Phase 4 — Per-repo copy + absolutize + `samgraha.toml` field
`register_standard(standard_name, repo_root)`'s new copy-then-seed-then-
absolutize shape (§3.9, §3.11). `RepositoryConfig.samgraha_dir` (§3.10),
both hardcoded call sites updated.

### Phase 5 — Output folder convention
`--out-dir` added to the script contract (§3.12), `.samgraha/output/`
created at repo init time.

### Phase 6 — Standard-update trigger (§3.13)
`"update_standard"` `operation_log` value; delete-then-recopy for both the
mcp-registry tree and every affected repo's local tree.

### Dependency graph
```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4 ──→ Phase 6
                                  └──────────→ Phase 5
```

---

## 5. Testing

- Unit: `register_standard_globally` copies `path`'s full tree into
  `mcp_dir()/registry/<name>/`, `standard_registry.source_path` points at
  the copy, not the original `path` — assert by deleting the original
  `path` after registration and confirming a subsequent per-repo
  `register_standard` call still succeeds (proves independence from the
  source repo's continued existence, the actual point of §3.7).
- Unit: `run_seeder` against a fixture seeder script that inserts one
  `usecase`/`script`/`domain` row directly via SQL — asserts the rows
  exist afterward, with no `step_id` ever passed in (proves §1.9's
  bootstrap-ordering fix).
- Unit: §3.11's absolutize pass — seed a `script.location` as a relative
  path, run the pass, assert it's now `<samgraha_dir>/<standard>/<relative>`;
  run it a second time (idempotency), assert no double-prefixing
  (`NOT LIKE '/%'` guard).
- Unit: §3.13 — re-run `register_standard_globally` for an already-registered
  standard with modified fixture content; assert the old
  `mcp_dir()/registry/<name>/` content is gone (not merged), the new
  content is there, and `operation_log` has an `"update_standard"` row
  distinct from the original `"register_globally"` row.
- Integration: full per-repo flow (§3.9) against a fixture standard with a
  real seeder script — global register, per-repo register, confirm
  `.samgraha/<standard>/` exists in the target repo with absolutized
  `script.location` values that actually resolve to real, executable files.

---

## 6. Explicitly out of scope

- **No sandboxing of seeder scripts beyond the existing script-execution
  trust model.** A seeder script runs with the same "arbitrary code
  execution under the repo's own permissions" trust already accepted for
  every other deterministic step (named explicitly in the archived
  mcp-execution-substrate proposal's own trust-boundary note) — this
  proposal doesn't add new isolation, it just adds one more thing that
  runs under the same accepted model.
- **No dedup/caching of identical standard files across multiple consuming
  repos.** Each repo's `register_standard` call does its own full copy
  (§3.9) — N repos registering the same standard means N copies on disk.
  Real disk-space cost for a large standard registered widely; not
  designed around here since no standard on disk today is large enough
  for this to matter, and a shared-cache layer is exactly the kind of
  speculative infrastructure this whole line of proposals has been
  removing, not adding.
- **Not mandating a seeder script's implementation language.** Every
  existing example (`pcems_2026`, `base_academic`) is Python; nothing in
  this design requires that — `run_capability_script`'s interpreter
  dispatch-by-extension already handles any language it's configured for.
- **`--out-dir`'s exact plumbing into `run_capability_script`'s existing
  four-argument contract** is a signature change to a function used
  everywhere — the convention (§3.12) is decided here; the precise
  parameter-threading is an implementation detail for whoever picks up
  Phase 5, not pinned down further in this document.

---

## 7. Disposition of review findings

| # | Finding | Disposition |
|---|---|---|
| 1 | §1.6-1.9, §1.4 factual claims | **Confirmed accurate** — no change needed. |
| 2 | §3.8's "`pcems_2026`/`base_academic` already ship a seeder" overstated — neither inserts `usecase`/`script`/`prompt`/`step` rows | **Fixed, and scope-corrected further per this round's own direction**: §3.8 rewritten to define the contract from samgraha's own mechanisms only, explicitly *not* claiming any standard already implements it. |
| 3 | §3.8's `run_seeder` doesn't say how it discovers `samgraha_dir`; conflicts with §3.10's configurable path | **Fixed, decided** — `_samgraha_dir`/`_knowledge_db` injected into the `--in` envelope (same mechanism as the earlier proposal's `_git` injection), not a new flag, not a config-parsing dependency added to standard scripts. |
| 4 | §3.11's absolutize pass mishandles `..`-containing relative paths (fragile, works by accident on Unix) | **Fixed, closed rather than left open** — a `..` segment in a seeder-written location is now a rejected contract violation (registration fails loudly), never silently walked/resolved. |
| 5 | §3.13 — no auto-propagation of global standard updates to consuming repos is an operational burden at scale | **Accepted as designed**, unchanged from rev. 2's stance — consistent with rev. 1 §3.5's "no auto-rediscovery" position; building propagation would require samgraha to track which repos have which standard active, which the whole design deliberately avoids doing. |
| 6 | `run_seeder` was a 4-line pass-through — question whether it should exist as a named function at all | **Fixed by giving it real work**: envelope construction (`_samgraha_dir`/`_knowledge_db`) and status validation both moved into the function itself, out of the caller — it now does something a bare `run_capability_script` call doesn't. |
| 7 | No state machine / visibility for "this repo hasn't re-seeded since the standard updated" | **Fixed, by comparison rather than a stored flag** (§3.14) — local `standard.version` vs. global `standard_registry.version`, computed live by existing tools, no new column to keep in sync. |
| 8 | (This round's own direction) Define the standard spec from samgraha's side; stop grounding requirements in what `Kriti/` currently contains | **Applied throughout §3.8** — the seeder contract is now stated as samgraha's own requirement, verified only against `env.rs`'s real `run_capability_script` signature, with an explicit note that zero standards on disk implement it yet and that authoring one is separate, later work. |
