# MCP Execution Substrate Proposal — Standard-Defined Usecases, Samgraha-Owned Dispatch

**Status**: DRAFT — not yet implemented.

**Supersedes**: `docs/proposal/archive/custom-workflow-db-proposal.md`
(archived). That doc's §1-§7 investigation and decisions survive intact —
nothing in them was wrong, they just weren't the final shape. §8 (phase
execution) is superseded outright by §5 below, which generalizes it
further than that doc's own final revision did. This doc is the single
place to build from; don't read the archived one for anything but
historical "why."

**What survives from the archived doc, unchanged**:
- Custom workflow data lives inside `.samgraha/knowledge.db`, not a
  separate file (§2.1 of the old doc — verified safe: samgraha's own
  migrations only `DROP TABLE`/rebuild tables they name explicitly).
- Mandatory table-name prefix, enforced at `register_standard` time
  against `RESERVED_TABLE_NAMES` (already shipped: `adapter.rs:2105-2136`,
  `migration.rs`'s `RESERVED_TABLE_NAMES` const).
- `custom_data_tables` catalog table exists (already shipped: V34,
  `migration.rs:1056-1071`) — extended below, not replaced.
- Scripts/templates asset-sync (already shipped, from
  `standard-asset-sync-proposal.md`) and seed-once `audit_analysis` sync
  (already shipped: `adapter.rs:2204-2228`, `init.rs`'s seed-once block) —
  both stay exactly as built.
- The "no transaction boundary, no migration framework, no cross-standard
  schema guarantees" rejections (old doc §2.4) still hold.

**What's new here**: the core principle validated over the last several
turns — samgraha is an MCP execution substrate. A knowledge standard
declares *usecases* (workflows); each usecase is a graph of steps; each
step is either **deterministic** (samgraha runs a script directly) or
**semantic** (samgraha stages data + instructions and hands reasoning to
whichever model is driving the calling MCP client). Samgraha owns
orchestration and the calling convention. The standard owns everything
else — meaning, data shape, storage, scripts, prompts. This doc designs
the consolidated data model and execution engine for that principle, then
maps `python_hackathon` onto it as the worked example.

---

## 1. The two things samgraha can execute — exhaustive, not just adequate

In an LLM-driven MCP substrate there are exactly two categories of work:
something samgraha can run itself (deterministic — a script, a fixed
input/output contract, no judgment involved), and something it can't
(semantic — requires reasoning, has to go to whichever model is calling
samgraha over MCP right now). `workflow_phases.kind CHECK IN
('semantic','script')` already encodes this and needs no change — it was
already correct, only the execution engine on top of it was missing.

Confirmed (again) that samgraha never calls an LLM itself anywhere in this
codebase: `AiConfigSection`'s `provider`/`ollama`/`openai` fields
(`common/config.rs:666`) are read by `providers::config::create_provider`
(`providers/src/config.rs:6-17`), and every branch except `"rule-based"`
just logs a warning and falls back to `RuleBasedProvider` — there is no
outbound API call to any model provider in this codebase. Every semantic
step is, by construction, executed by whatever model is on the other end
of the MCP connection — Claude in Claude Code, Gemini in Antigravity,
whatever OpenCode binds to. Samgraha has no opinion and no code path to
have one.

## 2. Samgraha's one non-negotiable opinion: the script I/O envelope

"Standard defines everything" applies to meaning, data shape, and
storage — not to the calling convention. `run_check_script`/
`run_capability_script` (`common/env.rs`) already fix this contract:
`--repo-root`, `--in`, `--out`, a JSON envelope in and out, interpreter
dispatch by file extension (`script_command`). Every deterministic step
and every pre/post script in this design uses this exact contract,
unchanged. This is the one thing samgraha imposes; everything else is the
standard's call.

**Validation is just another script** — this replaces an earlier, wronger
idea of mine (a samgraha-owned `result_schema`/JSON-Schema validation
step before persisting a semantic step's output). No new samgraha
capability needed: `post_script` (or a `validate_script` chained before
it, standard's choice) receives the agent's raw output as `--in` through
the same contract and decides for itself whether it's acceptable. Simpler,
and it doesn't add a second thing samgraha has to interpret the meaning
of.

## 3. Samgraha owns orchestration, standard owns content — draw the line
precisely

- **Samgraha's job** (scheduling, not meaning): enforce
  `workflow_phase_dependencies` edges before allowing a phase to run;
  check staleness/expiry (`script_runs`, `expiry_rule_json`) so a phase
  doesn't re-run needlessly; run the script/pre_script/post_script
  contract; move data between steps without interpreting it.
- **Standard's job** (meaning): what each script computes, what a
  semantic step's instruction says, what shape results take, where
  they're stored, what "done" means for a given usecase.

This split is what makes the earlier "is this valid?" question answer
yes — samgraha's code never has to branch on which standard, or on
audit-vs-generate, because none of that is samgraha's concern by design,
only by the accident of `handle_audit`/`handle_generate` being written
before this principle was made explicit.

## 4. Consolidated data model

### 4.1 `standard_assets` — one catalog, not per-thing tables

Two turns ago I dismissed a general script catalog as redundant with
`PlanPhase.script`. Wrong: `PlanPhase` only surfaces phase-bound entries.
A validator reused across three usecases, or a prompt not tied to any
single phase, has nowhere to be discoverable. One table, both kinds,
simpler than the two-table version first sketched:

```sql
-- V36 (knowledge.db): catalog of every named script/prompt asset a
-- standard ships, independent of which phase(s) reference it. Lets an
-- LLM (or samgraha's dispatcher) resolve a name to a path and read the
-- one-line purpose without inspecting the filesystem.
CREATE TABLE standard_assets (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,   -- e.g. "audit-python", "leaderboard-prompt"
    kind     TEXT    NOT NULL CHECK (kind IN ('script','prompt')),
    path     TEXT    NOT NULL,   -- relative to .samgraha/scripts/ or /templates/ or /audit_analysis/
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);
```

`PlanPhase.script`/`pre_script`/`post_script` become **names** resolved
against `standard_assets` (`kind='script'`) at dispatch time, not raw
paths embedded per-phase. New `PlanPhase.instruction: Option<String>`
field, also a name, resolved against `standard_assets` (`kind='prompt'`)
— the file handed to the calling agent verbatim for a semantic step.

**Decided: rename `workflow_phases.script_path` → `script_name`.** A
column called `script_path` holding a catalog name instead of a path is
actively misleading to anyone reading the schema later — the rename is
low-cost (this table has no external consumers besides samgraha itself;
`store_system_plan`/`get_system_plan` are the only read/write paths, both
already touched by this refactor) versus the permanent confusion of
leaving a stale name in place. Same migration that adds `standard_assets`
(V36) carries the rename.

**Dispatch-time resolution risk, called out explicitly**: every dispatch
now needs a lookup (`SELECT path FROM standard_assets WHERE standard = ?
AND name = ? AND kind = ?`) instead of reading an embedded literal. Two
consequences to design for, not just note:
- A typo in a phase's `script`/`instruction` name is now a **runtime**
  catalog-miss instead of a **load-time** file-not-found. `run_semantic_
  phase` (§5) must fail this loudly and immediately — "no asset named X
  for standard Y" — never silently skip or fall back.
- `standard_assets` must already be populated before any dispatch is
  attempted, which makes §4.3's propagation fix a hard prerequisite for
  §5, not a parallel-track nice-to-have. Sequencing note carried into §7.

### 4.2 `custom_data_tables` — add auto-introspected shape

Extend the already-shipped V34 table (new migration, additive, matches
this codebase's existing "never edit a past `Vn`, always add the next
one" discipline):

```sql
-- V35 (knowledge.db): adds shape_json to the already-shipped
-- custom_data_tables (V34). Populated by samgraha running
-- `PRAGMA table_info(<table>)` against the actual table after
-- owner_script has created it — never hand-authored by the standard,
-- so it can't drift from what the table really looks like (the same
-- class of drift bug this whole line of proposals keeps finding and
-- fixing for scripts/templates/table names).
ALTER TABLE custom_data_tables ADD COLUMN shape_json TEXT;
```

Populated opportunistically: the table this describes may not exist yet
at first registration (before `owner_script` has ever run) — `shape_json`
stays `NULL` until the first successful introspection, never assumed
available immediately.

**Staleness, noted not solved**: if a table is altered outside
`owner_script` (a manual `ALTER TABLE` someone runs by hand against
`knowledge.db`), `shape_json` goes stale with no signal — it only
refreshes on the next `register_standard`/introspection pass. Same risk
class as any cached metadata; not fixing this now (no mechanism watches
`knowledge.db` for out-of-band schema changes, and building one is out of
proportion to the problem), just naming it so it isn't rediscovered later
as a surprise.

### 4.3 Close the author-only propagation gap — for real, on both new
tables

Found and flagged two turns ago, still true today: `custom_data_tables`
only gets populated in `handle_register_standard` (`adapter.rs:2230`),
which writes into whichever repo *calls* `register_standard` — the
standard's authoring repo (Kriti), never a consuming repo (Heimdall) via
its normal `init`/`sync_standards` path. `system.yaml` itself is never
copied into `mcp_dir()/systems/<name>/`, so the consumer-side sync
(`init.rs`'s `sync_knowledge_system`) has no way to even learn the
declared `data_tables`/`standard_assets` entries today.

**Fix, required for both `standard_assets` and `custom_data_tables`, not
optional**: push `system.yaml` (or the parsed `assets:` block as JSON)
alongside scripts/templates/audit_analysis into
`mcp_dir()/systems/<name>/`, and add the identical catalog-populate logic
to `sync_knowledge_system` (`init.rs`) that `register_standard`
(`adapter.rs`) already has. One shared function, called from both push
and pull paths — same discipline `copy_dir_recursive` already established
for scripts/templates, applied here to catalog rows instead of files.

**Implementation note**: the shared function must take the target
`knowledge.db` path as an explicit parameter, not assume `root.join(
".samgraha").join("knowledge.db")` internally. `adapter.rs` calls it with
the authoring repo's path (`self.runtime.context.repository_root`),
`init.rs` calls it with whatever `root` it was passed — same function,
caller supplies the path, no hidden assumption about which repo it's
running against.

## 5. Execution engine — `prepare_semantic_phase` + `complete_semantic_phase`,
one shape, no audit/generate branch

Design only, per the earlier ask — not implemented yet.

**Protocol correction from the previous draft**: MCP is request-response,
not a mechanism that can block mid-call while an external LLM reasons and
then resume the same call with a result. A single tool that "prepares
data, hands off, waits, then runs post_script" isn't expressible as one
MCP round-trip. Splitting into two tools, matching how `audit`'s
`semantic_review.tasks` → `store_section_report` hand-off already works in
practice:

1. **`prepare_semantic_phase(phase_id, repo_root)`** — resolves and runs
   `pre_script` if set (`run_capability_script`), resolves `instruction`
   via `standard_assets`, returns `{prepared_data, instruction_content}`
   to the calling agent. Samgraha's involvement ends here for this call.
2. Calling agent reasons **off-MCP, in its own context** — this is not a
   samgraha concern at all, just the gap where the requesting model does
   its job.
3. **`complete_semantic_phase(phase_id, repo_root, result)`** — resolves
   and runs `post_script` with `result` as `--in`, persists however the
   standard's script decides, returns the script's own status envelope.

`kind: "script"` phases don't need this split — one call
(`run_script_phase(phase_id, repo_root)`, or a `kind` branch inside a
shared entry tool that only ever takes the one-call path for `"script"`)
resolves the name and runs it via `run_capability_script` directly, no
hand-off involved. Naming above is illustrative — three tools
(`run_script_phase`/`prepare_semantic_phase`/`complete_semantic_phase`) or
one entry tool that internally returns "call `complete_*` next" are both
consistent with this design; pick at implementation time, not here.

Mapped onto the two-call split — `kind: "semantic"`, meaning-agnostic
throughout:

1. `prepare_semantic_phase` — if `pre_script` set, resolve + run it (same
   contract), writes prepared data to a JSON envelope. Samgraha doesn't
   interpret it.
2. `prepare_semantic_phase` returns the prepared data plus the resolved
   `instruction` file's raw content. This is where "audit" or "generate"
   gets decided — entirely by what that content says, never by
   samgraha's own code. For `python_hackathon`, `instruction` points at
   either an `audit/semantic/document/*.yaml` rubric (scoring) or an
   `analysis/*.md` prompt (narrative writing) — samgraha reads neither for
   meaning.
3. Calling agent reasons off-MCP, produces a result.
4. `complete_semantic_phase` runs `post_script` with the agent's result as
   `--in` — persists however the standard's script decides (a
   `custom_data_tables` row, a generated file, anything), and does its own
   validation of the agent's output as part of that same script (§2 — no
   separate samgraha-owned validation step).

**Multi-repo dispatch**: both `prepare_semantic_phase` and
`complete_semantic_phase` take an explicit `repo_root`/`repo_path`
parameter at call time, same pattern several existing tools already use
(`main.rs`'s "Pass 'repo_path' to target a different local repository"
convention). One `workflow_phases` row is a *definition*; invoking it once
per participant repo (`python_hackathon` scoring N teams) is the caller's
loop, not a schema change — this reconciles the "N-repo vs. self-audit"
shape mismatch flagged earlier without needing per-team rows in
`workflow_phases` at all.

**`handle_audit`/`handle_generate` are untouched.** They're correct and
working for samgraha's own single-repo document-audit/content-generation
model — and, worth noting now that the protocol's been split into two
calls, they already follow exactly this shape (`audit`'s
`semantic_review.tasks` is `prepare_*`'s equivalent, `store_section_report`
is `complete_*`'s). `prepare_semantic_phase`/`complete_semantic_phase` are
a separate, new, genuinely generic pair for anything declared via
`workflow_use_cases`/`workflow_phases` — not a replacement, the thing that
was missing alongside them, built on the same proven two-call shape.

**Trust boundary, named not solved**: this dispatcher runs
standard-authored scripts automatically once a usecase is registered —
more automatic than today's manual `check_overrides`/`script_overrides`
invocation. Same accepted risk model as those (arbitrary code execution
with the repo's own permissions), just more load-bearing now that it's
wired to auto-dispatch rather than requiring a human to invoke a specific
override.

---

## 6. `python_hackathon` — the worked example, still not done

Confirmed still true as of this doc: `Kriti/samgraha/system/
python_hackathon/` has **no `system.yaml`**. Nothing in §4/§5 fires for
this standard until one is authored. Concretely needed:

```yaml
name: python_hackathon
assets:
  scripts: script
  templates: templates
  audit_analysis: [audit, analysis]   # Decided: Vec<String>, not Option<String>.
                                        # No system.yaml ships for python_hackathon
                                        # today (confirmed above), so there's no
                                        # existing file to migrate — zero cost to
                                        # widen the field now versus adding a
                                        # backward-compat union type later for a
                                        # format nothing has used yet.
  data_tables:
    owner_script: common/db.py
    prefix: hackathon_
    purpose: "Per-team scoring, domain breakdowns, and narrative analysis for the hackathon competition."
  catalog:                       # populates standard_assets (§4.1)
    - { name: audit-python, kind: script, path: usecase-2a-det-audit/audit_python.py, purpose: "Deterministic Python static-analysis checks (radon/mypy/bandit config detection)." }
    - { name: leaderboard-prompt, kind: prompt, path: analysis/00-leaderboard.md, purpose: "Competition-wide summary narrative, read after all teams scored." }
    # ... one entry per script/prompt worth naming; helper-only modules
    # (common/db.py itself) don't need a catalog row unless referenced
    # directly by a phase's script/pre_script/post_script/instruction.
```

Plus: convert `plan/core/loop.yaml`'s 8 stages into an `InitPlan` and call
`store_system_plan("python_hackathon", ...)` once (still unimplemented,
carried forward from the archived doc's §6 — the conversion step itself
doesn't change under this design, only what the phases' `script`/
`instruction` fields point at: catalog names, not raw paths).

**Live-data decision still open, still not mine to silently make**:
Kriti's `python_hackathon/hackathon.db` has real team-scoring rows today.
Moving to prefixed tables inside `knowledge.db` means either importing
those rows or a clean cutover — confirm with whoever owns the current
hackathon run before this ships.

---

## 7. Migration plan — what's shipped, what changes, what's net-new

**Keep as-is** (already shipped, no changes needed):
- `copy_dir_recursive`, `copy_dir_atomic`, `DEFAULT_EXCLUDES`
  (`common/fs_sync.rs`).
- Scripts/templates sync, both push (`adapter.rs`) and pull (`init.rs`).
- Seed-once `audit_analysis` sync, both directions.
- `RESERVED_TABLE_NAMES` + prefix-collision gate at `register_standard`.
- `custom_data_tables` (V34) as a table — extended, not replaced (§4.2).

**Change** (already shipped, needs rework):
- `custom_data_tables` population: currently `adapter.rs`-only (author
  side) — needs the identical logic added to `init.rs` (consumer side),
  per §4.3. This is the single most important fix in this doc — without
  it, every consumer repo (Heimdall) never gets a populated catalog,
  which defeats the stated purpose of having one.
- `StandardAssets` struct (`adapter.rs:101-121`) gains a `catalog: Vec<
  CatalogEntry>` field (§4.1/§6) alongside existing `data_tables`/
  `audit_analysis`.
- **Decided**: `audit_analysis: Option<String>` → `Vec<String>` (§6).
  Straight type change, no compat shim — no `system.yaml` exists yet for
  any standard that would need migrating (`python_hackathon` has none;
  it's the only standard this field was designed for), so there's nothing
  to preserve backward compatibility with.

**Net-new** (nothing exists yet):
- V35 migration (`shape_json` column).
- V36 migration (`standard_assets` table).
- PRAGMA-based shape auto-introspection, run after `owner_script`
  executes (opportunistic, not required at register time).
- `PlanPhase.instruction` field; `script`/`pre_script`/`post_script`
  semantics change from raw path to catalog name (resolved via
  `standard_assets` at dispatch time).
- `run_semantic_phase` MCP tool and its dispatcher (§5) — the actual
  execution engine, still entirely unbuilt.
- `python_hackathon/system.yaml` (doesn't exist) and the `loop.yaml` →
  `InitPlan` conversion (still unimplemented).

## 8. Testing

- Unit: `standard_assets` name resolution — a `PlanPhase.script`/
  `instruction` value that doesn't match any cataloged name fails loudly
  at dispatch time, not silently.
- Unit: `shape_json` introspection — create a table via a fixture
  `owner_script`, run introspection, assert the JSON matches
  `PRAGMA table_info`'s actual columns; assert it stays `NULL` if the
  table doesn't exist yet.
- Unit: catalog propagation parity — register a standard with
  `register_standard`, then sync it into a second scratch repo via
  `sync_knowledge_system`/`init`; assert `standard_assets` and
  `custom_data_tables` rows exist identically in *both* repos'
  `knowledge.db` files. This is the regression test for §4.3's gap
  specifically — must fail before the fix and pass after.
- Integration: register `python_hackathon` (once its `system.yaml` and
  `loop.yaml`-derived plan exist) into a scratch repo, call
  `prepare_semantic_phase`/`complete_semantic_phase` for both a scoring
  phase and a narrative phase, confirm the same two tools/same code path
  handle both without any audit/generate branch being hit — the actual
  claim of §5, provable by code-coverage showing zero standard-specific or
  audit/generate-specific branches taken in either dispatcher.
- Unit: stale catalog reference — register a standard with a catalog
  entry named `X`, store a phase referencing `X`, re-register the standard
  *without* `X` in its `system.yaml` (simulating the standard author
  removing/renaming an asset), then dispatch that phase again — assert it
  fails loudly with a catalog-miss error, never silently runs against
  whatever `X` used to resolve to or skips the step.
- Integration: concurrent dispatch of the same phase in the same repo —
  two simultaneous `prepare_semantic_phase`/`run_script_phase` calls for
  identical `(phase_id, repo_root)`. `script_runs`'s existing `UNIQUE
  (standard_id, repo_fingerprint, capability, phase_or_check_key)`
  constraint plus `check_phase_prerequisites`/the run-recording path
  should already make this safe — this test proves it under the new
  dispatcher specifically rather than assuming the existing mechanism
  covers a code path it's never been exercised through.

---

## 9. Implementation Plan — Ordered Phases with Dependencies

### Phase 1: Schema Changes (foundation — everything depends on this)

**1a. `schema/knowledge-hub/26-workflow_phases.sql`**
- Rename `script_path` → `script_name`
- Add `instruction TEXT` column (nullable, for semantic steps — catalog
  name of the prompt file, resolved against `standard_assets` at dispatch)
- Update header comment to reflect catalog-name semantics

**1b. `schema/knowledge-hub/29-standard_assets.sql`** (new file)
- `CREATE TABLE standard_assets`:
  `id INTEGER PRIMARY KEY, standard_id INTEGER NOT NULL REFERENCES
  standards(id) ON DELETE CASCADE, name TEXT NOT NULL, kind TEXT NOT NULL
  CHECK (kind IN ('script','prompt')), path TEXT NOT NULL, purpose TEXT
  NOT NULL DEFAULT '', UNIQUE(standard_id, name)`
- Index on `(standard_id, kind)`

**1c. `crates/registry/src/migration.rs`** (knowledge.db side)
- V35: `ALTER TABLE custom_data_tables ADD COLUMN shape_json TEXT;`
- V36: `CREATE TABLE IF NOT EXISTS standard_assets (...)` — mirrors
  29-standard_assets.sql so knowledge.db also has the table for local
  queries by samgraha's own code
- Update `KNOWLEDGE_MIGRATIONS` array (add V35, V36)
- Update `RESERVED_TABLE_NAMES` (add `"standard_assets"`)

### Phase 2: Config Struct + Audit/Analysis Vec Fix

**2a. Extend `StandardAssets`** (`adapter.rs`)
- Add `CatalogEntry` struct: `name: String`, `kind: String`
  (`"script"` | `"prompt"`), `path: String`, `purpose: String`
- Add `catalog: Vec<CatalogEntry>` field (serde default empty vec)
- Change `audit_analysis: Option<String>` → `audit_analysis: Vec<String>`
  (serde default empty vec, straight type change, no compat shim needed)

**2b. Fix audit/analysis copy loop** (`adapter.rs` + `init.rs`)
- `handle_register_standard`: iterate `assets.audit_analysis` vec instead
  of single-dir lookup
- `sync_knowledge_system` seed-once block: same — iterate vec of source
  dirs from system.yaml

### Phase 3: Shared Catalog Populate + Propagation Fix (§4.3)

**3a. Shared function** (new `crates/services/src/catalog.rs`, or inline
in `init.rs`)
- Signature: `populate_standard_catalogs(
    knowledge_db: &Path,
    standard_name: &str,
    assets: &StandardAssets,
  ) -> Result<CatalogPopulateResult>`
- Opens `knowledge.db` via `RegistryStore::open` (runs V35/V36)
- DELETEs existing rows for this standard from `standard_assets` and
  `custom_data_tables`
- INSERTs `standard_assets` rows from `assets.catalog`
- INSERTs `custom_data_tables` row from `assets.data_tables` (if present)
- Returns counts

**3b. Call from `handle_register_standard`** (`adapter.rs`)
- Replace current inline catalog populate code with call to shared fn
- Pass `self.runtime.context.repository_root.join(".samgraha")
  .join("knowledge.db")`

**3c. Propagation: copy assets config to global store** (`adapter.rs`)
- After parsing `system.yaml`, serialize the full `assets` block as JSON
  and write to `mcp_dir()/systems/<name>/system-assets.json`
- In `sync_knowledge_system` (`init.rs`): read `system-assets.json` from
  `sys_dir`, deserialize into `StandardAssets`, call
  `populate_standard_catalogs` with the local `knowledge.db`

### Phase 4: PRAGMA Shape Introspection (§4.2)

**4a. New MCP tool or post-script hook**
- After `owner_script` executes successfully, query `sqlite_master` for
  tables whose name starts with the declared prefix
- Run `PRAGMA table_info(<table>)` on each, serialize as JSON
- Update `custom_data_tables.shape_json`
- Opportunistic: skip silently if table doesn't exist yet (shape_json
  stays NULL)

**4b. Trigger point**
- Standalone MCP tool `introspect_custom_tables(standard, repo_root?)`
  — caller invokes after running owner_script, before dispatching phases
  that depend on the data. Cleaner than embedding in the dispatch flow.

### Phase 5: PlanPhase Updates + store/get Handler Changes

**5a. `PlanPhase` struct** (`capability.rs`)
- Add `instruction: Option<String>` field
- `script`/`pre_script`/`post_script` hold catalog names (semantic
  change, no type change)

**5b. `handle_store_system_plan`** (`adapter.rs`)
- Write `instruction` column to `workflow_phases`
- Write `script_name` column (was `script_path`)

**5c. `handle_get_system_plan`** (`adapter.rs`)
- Read `script_name` and `instruction` columns → map to
  `PlanPhase.script` and `PlanPhase.instruction`

**5d. `check_phase_prerequisites` / `record_script_run`** (`capability.rs`)
- No changes needed — they key on `phase_id` (string), not script paths

### Phase 6: Execution Engine — Two MCP Tools + Script Dispatcher

**6a. `prepare_semantic_phase(phase_id, repo_root, repo_path?)`**
- Resolve `phase_id` → `workflow_phases` row
- Check prerequisites via `check_phase_prerequisites` → blocked if so
- If `pre_script` set: resolve catalog name via `standard_assets`, run
  via `run_capability_script`, capture output
- Resolve `instruction` catalog name via `standard_assets` (`kind=
  'prompt'`), read file content
- Return `{ blocked: false, prepared_data, instruction, phase_meta }`

**6b. `complete_semantic_phase(phase_id, repo_root, repo_path?, result)`**
- Resolve `phase_id` → `workflow_phases` row
- If `post_script` set: resolve catalog name via `standard_assets`, run
  via `run_capability_script` with `result` as `--in`
- Record script run via `record_script_run`
- Return `{ success, post_script_output }`

**6c. `run_script_phase(phase_id, repo_root, repo_path?, input?)`**
  (for `kind: "script"`)
- Resolve `phase_id`, check prerequisites
- Resolve `script_name` via `standard_assets`, run via
  `run_capability_script`
- Record script run, return result

**6d. Register tools** in `main.rs` tool definitions

### Phase 7: Tests

- Unit: catalog name resolution — valid name resolves, missing name fails
  loudly
- Unit: stale catalog reference — re-register without entry → dispatch
  fails
- Unit: `shape_json` introspection — create table, introspect, assert
  JSON matches; NULL when table absent
- Unit: catalog propagation parity — register in repo A, sync to repo B,
  assert identical rows in both knowledge.db files
- Unit: concurrent dispatch — two `prepare_semantic_phase` calls for same
  phase, `complete` upserts correctly via UNIQUE constraint
- Integration: same two tools handle scoring + narrative without
  audit/generate branch (coverage-provable)

### Dependency Graph

```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 6
Phase 1 ──→ Phase 5 ──→ Phase 6
Phase 4 (standalone) ──→ Phase 7
Phase 3 + Phase 6 ──→ Phase 7
```

Phase 1 is the foundation. Phases 2/4/5 can proceed in parallel after
Phase 1. Phase 3 depends on Phase 2. Phase 6 depends on Phases 3 and 5.
Phase 7 depends on everything.
