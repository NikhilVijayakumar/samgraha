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

**Revision 4 note**: a review of rev. 3 raised three gaps, all resolved
below rather than left open. New §3.15 separates "MCP release version"
from "schema epoch" — they are not the same trigger, and nothing in rev. 3
said so; decided outright that only the epoch drives a wipe, never a plain
version bump, and that the epoch check is automatic (runs on every
`knowledge.db` open, the same call site `run_core_migrations` already
occupies today — verified against `register_standard.rs`), not an operator
step. §7's row 5 disposition is corrected: it previously justified
"no auto-propagation" by claiming samgraha would need new tracking
infrastructure to know which repos have a standard active; verified this
round that `registry.db`'s `repository_cache` table and the existing
`list_repositories` MCP tool already provide exactly that enumeration — the
no-auto-propagation stance is unchanged, but it is now stated correctly as
a deliberate deferral, not a missing-infrastructure blocker.

**Revision 5 note**: five implementation-time hazards flagged against
rev. 4 (all read as "not blocking, worth closing before implementation
starts" — accepted on that basis), now closed: §3.7/§3.9 both switch
their file copy from `copy_dir_recursive` to `copy_dir_atomic` (§1.6) and
add an explicit "delete everything this step just wrote if a later step
fails" rule, so neither the mcp-registry copy nor the per-repo copy can
survive as an orphan with no matching database row; §3.9 gains a new step
3 (`delete_existing`-style row cleanup, reusing `register_standard.rs:179`'s
existing pattern) that runs *before* the seeder, not just tracked as a
side-effect of the epoch reset, which also closes the seeder
double-absolutization question from §3.11's own reasoning (a seeder can
never read back its own prior run's rows if they're deleted first); §3.10
gets one clarifying paragraph confirming `resolve_configured_dir`'s
existing set-but-relative behavior already covers `samgraha_dir`, no new
case; §3.12 gets one paragraph stating outright that a seeder never uses
`--out-dir`, closing the Phase-3-before-Phase-5 ordering question without
reordering the phase graph. §7 rows 11-15 record the disposition of each.

**Revision 6 note**: implemented, then corrected in the same pass on
review of §3.9's own reasoning — "one standard active per repo at a time,
switch by deleting `.samgraha/` and re-registering" makes the local
`standard` table (§2.13, knowledge.db) redundant: it's per-repo catalog
metadata (category/subcategory/extends/version) duplicating what
`standards.db`'s global `standard_registry` already tracks, just at repo
scope. Removed from `knowledge.db` entirely (`CORE_SCHEMA_EPOCH` 1→2);
replaced by a new singleton table, `registry.db`'s `active_standard`
(`schema/registration/01-active_standard.sql`, `REG_V3`) — one row
(`CHECK(id = 1)`, same pattern `_core_schema_epoch` already uses),
written by the per-repo activation call directly from the already-fetched
global `standard_registry` row, no local YAML re-parse needed for those
fields at all. §3.9 step 6 and §3.14 are rewritten below to match; §7
rows 16-17 record the disposition.

**Revision 7 note**: `standard_asset.kind`, `template.type`, and
`artifact.type` were free `TEXT` columns — samgraha stored whatever
string a standard wrote, no relational integrity. Corrected per direction
this round: each gets its own per-standard lookup table (`asset_kind`,
`template_type`, `artifact_type`), same shape and `UNIQUE(standard, name)`
pattern the existing `domain`/`usecase.domain_id` relation already uses,
with a `*_id` foreign key replacing the free-text column
(`CORE_SCHEMA_EPOCH` 2→3). `asset_kind`/`template_type` are declared by a
standard's own seeder before it references them (same discipline as
`domain`). `artifact_type` is different — artifact rows come from a
script's *runtime* output, whose vocabulary can't be predicted at
registration time, so samgraha itself find-or-creates the type row
(`get_or_create_lookup`) rather than requiring a seeder to pre-declare
every possible output type. New §3.16 below; §7 rows 18-20 record the
disposition.

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
   `copy_dir_atomic` (§1.6) — full tree: scripts, prompts, templates,
   the manifest, and the seeder/verify/smoke-test scripts §3.8 introduces.
   **`copy_dir_atomic`, not `copy_dir_recursive`**: this call site can
   target a directory that already holds a *previous* copy of the same
   standard (§3.13's update path re-runs step 1), and `copy_dir_recursive`
   only overwrites — it never deletes a file present in the old copy but
   absent from the new `path`, so a file removed from a standard's source
   between versions would otherwise survive indefinitely as an orphan
   under `mcp_dir()/registry/<standard-name>/`. `copy_dir_atomic` (§1.6,
   `fs_sync.rs:91-128`) copies into a sibling temp dir first, then
   `remove_dir_all`s the old target and `rename`s the temp dir over it —
   the old tree is *fully replaced*, never merged, and a failure during
   the copy itself leaves the previous target untouched (verified against
   `fs_sync.rs`'s own `atomic_copy_leaves_old_tree_on_failure` test).
2. Run the structural verify-gate (`smoke_test`, unchanged from the
   earlier proposal's §2.4.1) against **this copy**, not the original path
   — proves the copy is what actually gets used from here on, catching a
   copy-step bug immediately rather than only on first real per-repo use.
3. Upsert `standard_registry` with `source_path` now pointing at
   `mcp_dir()/registry/<standard-name>/` — the **local** copy. The
   original `path` argument is used once, for the copy, and never stored.

**Cleanup on failure after step 1**: `copy_dir_atomic` only guarantees
step 1 itself is all-or-nothing — it says nothing about steps 2 or 3
failing *after* a good copy lands. If the verify-gate (step 2) fails, or
the upsert (step 3) fails for any reason, `mcp_dir()/registry/<standard-name>/`
now holds a fully-copied tree with no matching `standard_registry` row —
a live directory samgraha itself created but has no record of, invisible
to `list_standards` yet still occupying disk under a path this proposal
otherwise treats as fully samgraha-owned. **Decided**: any failure at
step 2 or step 3 removes the directory `copy_dir_atomic` just wrote
(`remove_dir_all(mcp_dir()/registry/<standard-name>/)`) before returning
the error — a failed `register_standard_globally` call leaves exactly the
state it found (nothing, or the previous version's copy if this was a
§3.13 update), never a half-registered directory. Same "fail fast, no
partial state" discipline `register_standard.rs` already applies before
writing any row (its reserved-table-name collision check runs before any
`INSERT`, lines 155-165) — this extends that discipline across a step
boundary that writes to the filesystem, not just across statements in one
transaction.

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
   proposal's own previous draft left open, closed below). **Rev. 7**: if
   it writes any `standard_asset` or `template` row, it must first
   declare that row's `kind`/`type` into `asset_kind`/`template_type`
   (§3.16) — `kind_id`/`type_id` are foreign keys now, not free text, and
   there is no other writer of those two catalog tables. It never inserts
   into `artifact_type` — that one is samgraha's own responsibility
   (§3.16), not the seeder's.
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
   `copy_dir_atomic` (§1.6) — same reasoning as §3.7 step 1: a repo
   re-registering after a standard update (§3.13) already has a local
   copy of the *previous* version under this path, and only a full
   replace-not-merge prunes files the new version dropped.
3. **Delete this standard's existing rows first**: run the same
   `WHERE standard = ?1` cleanup `register_standard.rs`'s existing
   `delete_existing` already performs (before any `INSERT`) —
   across `usecase`/`step`/`step_script`/`step_prompt`/`script`/`prompt`/
   `domain`/`asset_kind`/`standard_asset`/`template_type`/`template`/
   `custom_data_tables` (children before the parent they reference — Rev.
   7's `asset_kind`/`template_type` deleted *after* the `standard_asset`/
   `template` rows that FK into them). **Not** `artifact`/`artifact_type`
   — those are a historical output record, deliberately left out of this
   cleanup (§3.16) so re-registering a standard doesn't erase what it
   already generated.
   **Why this step is new, not implied**: a seeder is arbitrary code that
   can, in principle, read the very rows it's about to write (nothing
   stops it querying `knowledge_db` first). Deleting this standard's rows
   *before* the seeder runs means there is nothing left for it to read
   back — it always starts from a standard-scoped-clean database, so a
   re-registration's seeder run can never observe its own prior run's
   already-absolutized `location` values and echo them back as-if-fresh.
   This closes the ambiguity §3.11 otherwise leaves about a seeder's
   *own* consistency across re-runs (the `NOT LIKE '/%'` guard still
   protects against double-prefixing regardless, but this step means it
   is never exercised by a standard's own re-registration in the first
   place — only ever by a seeder that hardcodes an absolute path on
   purpose, its own stated design choice).
4. Call `run_seeder(repo_root, <local-copy>/<seeder_script>, samgraha_dir,
   knowledge_db, timeout)` (§3.8) — `samgraha_dir`/`knowledge_db` resolved
   from `RepositoryConfig.samgraha_dir` (§3.10), not assumed.
5. Run §3.11's absolutize pass — rejects (fails the whole registration,
   no partial state) if any `location` the seeder wrote contains a `..`
   segment.
6. **Rev. 6, rewritten**: no local `standard` row is written into
   `knowledge.db` at all anymore (§3.14 below explains why the table is
   gone). Instead, once steps 2-5 succeed, the caller (not
   `activate_standard` itself — it doesn't hold a `registry.db` handle)
   writes one row into `registry.db`'s `active_standard` (singleton,
   `REG_V3`) — `name`/`category`/`subcategory`/`extends`/`version`/
   `metadata_json` sourced straight from the global `standard_registry`
   row already fetched at the top of this call, no manifest re-read.
   `activate_standard` itself reads the local manifest copy for exactly
   one field now: `seeder_script` — the path to the script it has to run.
   Category/version/etc. never touch YAML on the per-repo path at all.

**Cleanup on failure after step 2**: steps 3-5 write to a database that
step 3 already emptied for this standard — if step 4's seeder exits
non-`ok`, or step 5's absolutize pass rejects a `..`-containing path,
whatever rows the seeder *did* manage to insert before failing (step 4 is
not one transaction — the seeder is an external process making its own
`INSERT`s) are still sitting in `knowledge.db`, and
`repo_root/.samgraha/<standard-name>/` (step 2) still holds the copied
files. **Decided**: any failure at step 3, 4, or 5 re-runs step 3's
`delete_existing`-style cleanup and removes
`repo_root/.samgraha/<standard-name>/` before returning the error — the
same "leave exactly the state you found" guarantee §3.7 now states for
the global side, applied here to the per-repo side. Because step 6 (the
`registry.db` write) only ever runs *after* steps 2-5 have already
succeeded, there is no step-6 failure mode to clean up after — the
repo's `active_standard` row simply never gets touched if anything
upstream failed, which is a stronger guarantee than the old local
`standard`-row design had (that row lived in the same transactionless
sequence as everything else it could fail alongside).

`RESERVED_TABLE_NAMES`'s reset (§3.2, rev. 1) already handles the "re-run
this and get a clean slate" case (all these tables get dropped and
recreated on an epoch bump); step 2's per-repo file copy and step 3's
per-standard row cleanup are both idempotent by construction, so
re-running steps 1-6 after either kind of reset is exactly "re-register,"
no special case needed.

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

**`SAMGRAHA_DIR` set but relative — already handled, not a new case to
design**: verified against `resolve_configured_dir`'s existing body
(`config.rs:242-256`) — if the variable is set, its value is joined onto
`root` when not itself absolute (`config.rs:248-252`, `if p.is_absolute() { p } else { root.join(p) }`),
exactly the same rule already applied to every other `${VAR}`-style field
in `samgraha.toml` today. `samgraha_dir` gets no special case: set-and-
absolute is used as-is, set-and-relative resolves under `root`, unset
falls back to `root.join(".samgraha")` — one function, three inputs,
already written.

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

**Rev. 7**: `artifact.type` (the string a script's `artifacts[]` envelope
entry reports, e.g. `image`/`diagram`/`dataset`/`model`) is now a
foreign key into `artifact_type`, not free text — see §3.16 for why this
one is samgraha-populated rather than seeder-declared.

**A seeder never needs `--out-dir`, so Phase 3 landing before Phase 5
(§4) is not a bootstrap-ordering problem the way §1.9 was**: `run_seeder`
(§3.8) writes rows directly into `knowledge.db` — it has no `artifacts[]`
envelope field to act on and §3.8 point 4 already scopes a seeder's
output to `operation_log`, never `execution`/`artifact`. `--out-dir` is
part of `run_script_step`'s per-usecase-step contract (§3.12, this
section), which Phase 5 adds; `run_seeder` calls the same underlying
`run_capability_script` (§3.8 point 1) but never passes or expects an
out-dir argument, so extending that function's contract in Phase 5 has
nothing to retrofit on the Phase-3-landed seeder path.

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
**Rev. 6**: the local side of this comparison moved — `registry.db`'s
`active_standard` (singleton, §3.9 step 6) now stores the `version` this
repo last activated, replacing the knowledge.db `standard` table the
original design used for the same purpose (removed, §3.9/§3.14 both
rewritten this round). The global `standard_registry` (§2.1) still stores
the current one, unchanged. `get_standard_info`/`repository_status`
(existing tools) compute staleness live, at read time:

```rust
let stale = active_standard.version != global_registry_row.version;
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

### 3.15 MCP release version vs. schema epoch — two different triggers, not one

**Decided**: a plain MCP release (a `Cargo.toml` version bump with no
`RESERVED_TABLE_NAMES`-shape change) does **not** wipe anything.
`CORE_SCHEMA_EPOCH` (§3.3, rev. 1) is the *only* trigger for a
`knowledge.db` reset — bumped by hand, in code, exactly when the reserved
tables' shape actually changes. Most releases won't touch that shape, and
wiping every repo's standard-catalog data on every release for no schema
reason would force needless reseeding across every consuming repo with no
corresponding benefit — rejected outright, not an oversight left for later.
No new "MCP version" field is added to `standards.db` or `registry.db` for
this purpose; there is exactly one stored version concept per axis this
proposal cares about: `CORE_SCHEMA_EPOCH` for "has samgraha's own table
shape changed" (§3.3), and `standard_registry.version` / `registry.db`'s
`active_standard.version` for "has this standard's content changed"
(§3.14, Rev. 6) — a
third, release-level version tracked independently of both would be a
value that could drift out of sync with the thing it's meant to gate,
the same failure mode §3.14 already rejected a stored staleness flag for.

**Decided**: the epoch check is automatic, not an operator step. It runs
inside the same call site `run_core_migrations` already occupies today —
verified this round: `register_standard.rs`'s `register_standard` opens
`knowledge_db_path` and calls `registry::core_schema::run_core_migrations(&conn)`
before touching any row. Phase 2's `ensure_current_schema` takes over that
exact call site: on every `knowledge.db` open, it compares the file's
recorded epoch to `CORE_SCHEMA_EPOCH`, and on mismatch wipes and recreates
`RESERVED_TABLE_NAMES` tables before anything else runs — the same
lazy-on-open shape `run_core_migrations` already has, not a separate
`reinit` tool call an operator has to remember to invoke. This is the one
structural difference from §3.13's standard-update trigger, which stays
manual by design (it targets one named standard, not samgraha's own global
schema, so there's no "next open" to hook — an operator, or a future
automated caller per §7 row 5's corrected disposition, has to actually
invoke `register_standard_globally`/`register_standard` again).

### 3.16 Relational asset/template/artifact types — not free text

**Rev. 7.** `standard_asset.kind`, `template.type`, and `artifact.type`
were plain `TEXT` columns in every revision through rev. 6 — samgraha
stored whatever string a standard wrote and never checked it against
anything. Corrected: each gets its own per-standard lookup table, same
shape and `UNIQUE(standard, name)` constraint `domain` already uses for
`usecase.domain_id` — `(id, standard, name, description)` — and the
column each was on becomes a foreign key (`kind_id`/`type_id`) into it.
`CORE_SCHEMA_EPOCH` 2→3 for this (a column type/name change, same
full-wipe treatment §3.15's rev. 6 bump got, not an in-place `ALTER`).

**Three tables, two different population rules — because the two
concepts aren't symmetric:**

- **`asset_kind`, `template_type`** — declared by the standard's own
  seeder, *before* it inserts the `standard_asset`/`template` row that
  references it. This is safe because it's the same script, one run,
  declaring then referencing in whatever order it writes its own SQL —
  no ordering hazard, same responsibility §3.8 already gives the seeder
  for `domain`. A seeder that references a `kind`/`type` it never
  declared gets a plain FK-violation error (`NOT NULL REFERENCES`,
  `PRAGMA foreign_keys = ON` already set on every `knowledge.db`
  connection) — loud, not silently accepted as free text used to be.
- **`artifact_type`** — *not* seeder-declared. Artifact rows come from a
  script's runtime output — `run_script_step`'s `result.artifacts[]`
  envelope, read long after registration, on every deterministic step
  run. A script's own output vocabulary can't be enumerated up front the
  way a seeder's declared assets/templates can (a script might report a
  type nobody thought to pre-declare), and failing a whole step's
  execution over a missing catalog row would be a worse failure mode
  than growing the catalog on demand. So samgraha itself find-or-creates
  the `artifact_type` row at insert time:

  ```rust
  /// Find-or-create a row in a per-standard lookup table (`asset_kind`,
  /// `template_type`, `artifact_type` — all three share the shape
  /// `(id, standard, name, description)` with `UNIQUE(standard, name)`)
  /// and return its id.
  pub fn get_or_create_lookup(conn: &Connection, table: &str, standard: &str, name: &str) -> Result<i64> {
      conn.execute(
          &format!("INSERT INTO {table} (standard, name) VALUES (?1, ?2) ON CONFLICT(standard, name) DO NOTHING"),
          rusqlite::params![standard, name],
      )?;
      let id: i64 = conn.query_row(
          &format!("SELECT id FROM {table} WHERE standard = ?1 AND name = ?2"),
          rusqlite::params![standard, name],
          |r| r.get(0),
      )?;
      Ok(id)
  }
  ```

  Used from both the seeder-adjacent path (the old manifest-parsing
  `register_standard()`, still reachable via the CLI, calls it for
  `asset_kind`/`template_type` too, since that function constructs the
  `INSERT`s itself rather than delegating to a seeder script) and from
  `run_script_step`'s artifact-envelope reader, for `artifact_type`.

**Lifecycle on re-registration, not symmetric either** — extends §3.9
step 3's cleanup: `asset_kind`/`template_type` are deleted alongside
`standard_asset`/`template` (children first, since the FK points from
`standard_asset`/`template` at them) — they're declared content, replaced
fresh every reseed, same as everything else `delete_existing` scopes.
`artifact`/`artifact_type` are **not** deleted — artifacts are a
historical output record (what did a run of this standard actually
produce), and erasing them just because the standard re-registered would
throw away exactly the record §3.12's `.samgraha/output/` convention
exists to keep.

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
- Unit: §3.7/§3.9 failure cleanup — force step 2 (verify-gate / seeder) to
  fail on a fixture standard; assert the directory step 1 copied
  (`mcp_dir()/registry/<name>/` or `repo_root/.samgraha/<name>/`) no
  longer exists afterward, and (per-repo case) no row for that standard
  remains in any `RESERVED_TABLE_NAMES` table.
- Unit: `copy_dir_atomic` prunes stale files — register a fixture standard
  version with files `{a, b}`, then re-register (§3.13) a version with
  files `{a, c}`; assert `b` is gone from the copied tree, not merged
  alongside `c`.
- Unit: §3.9 step 3 — seed a standard, re-register it with a seeder
  fixture that (deliberately, to probe the ordering) queries `script`/
  `standard_asset` for its own standard's rows before inserting; assert
  the query returns zero rows at seeder-start time on the second run,
  proving step 3's cleanup runs before step 4's seeder invocation.
- Unit (Rev. 6): per-repo activation writes `registry.db`'s
  `active_standard` singleton from the already-fetched global row —
  assert `category`/`version`/etc. match `standard_registry` exactly
  without ever reading the local manifest copy for those fields, and
  that activating a second standard overwrites the one row rather than
  accumulating a second.
- Unit (Rev. 7): `get_or_create_lookup` — same `(standard, name)` called
  twice returns the same id, not a duplicate row; the same `name` under a
  different `standard` returns a distinct id (proves per-standard
  scoping, not a shared global vocabulary).
- Unit (Rev. 7): register a standard with an asset/template through the
  real per-repo path — assert `standard_asset.kind_id`/`template.type_id`
  resolve, via join, to the exact `name` the manifest/seeder declared.
- Unit (Rev. 7): re-register a standard whose seeder no longer declares a
  `kind`/`type` it used to — assert the old `asset_kind`/`template_type`
  row is gone (§3.9 step 3's extended cleanup), not orphaned.
- Unit (Rev. 7): run two `run_script_step` calls reporting artifacts of
  the same `type` string — assert both artifact rows resolve to the same
  `artifact_type.id` (find-or-create dedupes), and that re-registering
  the standard in between does *not* delete either the `artifact` rows
  or the `artifact_type` row they reference.

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
| 5 | §3.13 — no auto-propagation of global standard updates to consuming repos is an operational burden at scale | **Accepted as designed, rationale corrected this round** — unchanged from rev. 2's stance (consistent with rev. 1 §3.5's "no auto-rediscovery" position), but the *reason* given previously was wrong: it claimed propagation would require new tracking of which repos have which standard active. Verified this round it wouldn't — `registry.db`'s `repository_cache` table and the existing `list_repositories` MCP tool already enumerate every registered repo's root and `knowledge_db` path; §3.14's version-comparison logic, run per repo in that loop, is all "auto-heal" would need. The design still defers this deliberately (cost of opening every repo's `knowledge_db` per check; an offline repo never gets checked at all) — but as a choice, not because the infrastructure is missing. |
| 6 | `run_seeder` was a 4-line pass-through — question whether it should exist as a named function at all | **Fixed by giving it real work**: envelope construction (`_samgraha_dir`/`_knowledge_db`) and status validation both moved into the function itself, out of the caller — it now does something a bare `run_capability_script` call doesn't. |
| 7 | No state machine / visibility for "this repo hasn't re-seeded since the standard updated" | **Fixed, by comparison rather than a stored flag** (§3.14) — `registry.db`'s `active_standard.version` (Rev. 6; was knowledge.db's local `standard.version` through rev. 5) vs. global `standard_registry.version`, computed live by existing tools, no new column to keep in sync. |
| 8 | (This round's own direction) Define the standard spec from samgraha's side; stop grounding requirements in what `Kriti/` currently contains | **Applied throughout §3.8** — the seeder contract is now stated as samgraha's own requirement, verified only against `env.rs`'s real `run_capability_script` signature, with an explicit note that zero standards on disk implement it yet and that authoring one is separate, later work. |
| 9 | §3.3's epoch trigger and an MCP release version were never distinguished — could be read as the same thing | **Fixed, decided** (§3.15) — only `CORE_SCHEMA_EPOCH` triggers a wipe; a plain release version bump never does; no third stored version is added. |
| 10 | Rev. 3 never states whether the epoch check runs automatically or needs an explicit operator step | **Fixed, decided** (§3.15) — automatic, on every `knowledge.db` open, same call site `run_core_migrations` already occupies (verified against `register_standard.rs`). |
| 11 | §3.7/§3.9 — copy→verify→upsert (and copy→seed→absolutize→write) isn't atomic; a failure after the copy leaves an orphaned directory with no matching database row | **Fixed** — both sections now state explicitly: any failure after the copy step deletes the directory that step just wrote before returning the error. A failed call leaves exactly the state it found. |
| 12 | §3.9 step 2 — `copy_dir_recursive` overwrites but never prunes a file present in an old version and absent from the new one; re-registration can leave orphaned files from a prior standard version | **Fixed** — §3.7 step 1 and §3.9 step 2 both switch to `copy_dir_atomic` (§1.6), which replaces the target directory wholesale (temp-copy, then `remove_dir_all` + `rename`), never merges. |
| 13 | §3.8 — `run_seeder`'s temp `--in` file is written to `std::env::temp_dir()`; if `run_capability_script` errors, does cleanup get skipped? | **Checked, not a bug** — the code in §3.8 already calls `std::fs::remove_file(&in_path)` *before* propagating the result via `?` (the cleanup line sits ahead of the `result?` line), so cleanup runs on both the success and error path. No change needed. |
| 14 | §3.10 — `${SAMGRAHA_DIR}` syntax implies env-var interpolation, but does an operator setting it to a *relative* path get silently mishandled? | **Fixed, clarified** — verified against `resolve_configured_dir` (`config.rs:242-256`): a set-but-relative value is already joined onto `root`, the same rule every other `${VAR}` field in `samgraha.toml` follows today. §3.10 now states this outright rather than leaving it to be inferred. |
| 15 | §4 dependency graph — Phase 5 (`--out-dir`) lands after Phase 3 (the seeder, which already calls `run_capability_script`); does the seeder need `--out-dir` before Phase 5 ships it? | **Fixed, clarified** — §3.12 now states outright that a seeder never uses `--out-dir`: it writes rows directly to `knowledge.db`, not artifacts, and its output is scoped to `operation_log` (§3.8 point 4), never the `artifact` table `--out-dir` exists for. Nothing on the Phase-3 path needs retrofitting when Phase 5 lands. |
| 16 | (Post-Phase-6 implementation review) Local `standard` table (§2.13) duplicates what `standard_registry` already tracks globally, now that one standard is active per repo at a time | **Fixed, relocated** (Rev. 6, §3.9 step 6, §3.14) — removed from `knowledge.db` (`CORE_SCHEMA_EPOCH` 1→2); replaced by `registry.db`'s singleton `active_standard` table (`REG_V3`), written from the already-fetched global row, no local YAML re-parse. |
| 17 | §3.9's `activate_standard`, as first implemented, still read `category`/`subcategory`/`extends`/`version` from the local manifest copy — contradicting this document's own "no YAML touching" claim for the per-repo path | **Fixed** — those fields are no longer read by `activate_standard` at all; the caller sources them from the global `standard_registry` row it already has. Only `seeder_script` is still read from the local manifest copy. |
| 18 | `standard_asset.kind`, `template.type`, `artifact.type` stored as free `TEXT` — no relational integrity, any string accepted | **Fixed** (Rev. 7, §3.16) — `asset_kind`/`template_type`/`artifact_type` lookup tables added, same shape as `domain`; `kind_id`/`type_id` foreign keys replace the free-text columns. `CORE_SCHEMA_EPOCH` 2→3. |
| 19 | If lookup rows are required, does a seeder have to pre-declare every possible artifact output type before it can ever run a step? | **Resolved by design split** (§3.16) — `asset_kind`/`template_type` are seeder-declared (safe, one script controls both the declare and the reference); `artifact_type` is samgraha-populated via find-or-create (`get_or_create_lookup`) at artifact-insert time, so a script's output vocabulary is never a pre-registration requirement. |
| 20 | Does re-registering a standard wipe its historical `artifact` output along with its declared `standard_asset`/`template` rows? | **No, by design** — `delete_existing`'s cleanup (§3.9 step 3, extended for Rev. 7) deletes `asset_kind`/`standard_asset`/`template_type`/`template` (declared content, replaced fresh every reseed) but deliberately excludes `artifact`/`artifact_type` (historical output record, survives re-registration — §3.16). |
