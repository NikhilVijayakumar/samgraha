# Custom Workflow DB Proposal — Standard-Owned Tables in `knowledge.db`

**Status**: DRAFT — not yet implemented. Follows
`docs/proposal/archive/standard-asset-sync-proposal.md` (scripts/templates
sync, implemented) — this covers the third asset kind that surfaced while
checking it: a standard's own workflow data (e.g. `python_hackathon`'s
team/leaderboard scoring), as opposed to samgraha's own core schema.

**Direction (revised)**: not a separate db file. Custom tables live
*inside* the same `.samgraha/knowledge.db` every repo already has —
`knowledge.db` = samgraha's own core tables (from
`crates/registry/src/migration.rs`) **plus** whatever tables a standard's
own script adds, one file, one connection. Checked first whether that's
safe (§2.1) before committing to it: samgraha's own migrations only ever
`DROP TABLE`/rebuild tables they name explicitly, never the whole file, so
a custom table survives compiles/migrations fine — the one real risk is a
future core table name landing on a name a standard already used, closed
with a mandatory prefix convention (§2.2) rather than a second db file.

**Trigger**: `E:\Python\samgraha\schema\knowledge-hub\*.sql` is the fixed,
versioned schema every standard is loaded into (`systems`, `standards`,
`domains`, `rules`, `calculation_rules`, ... via
`schema/knowledge-hub/knowledge-hub-loader.py`) — that's standard
*definition* data, samgraha-owned, never touched by a standard's own
script. `python_hackathon` needs somewhere to put `standard_participants`/
`standard_domain_scores`/`standard_narratives` for scoring N teams — that's
workflow *data*, a different concern. It already does this today via its
own `hackathon.db`, entirely outside samgraha's schema and outside
`.samgraha/` — but ad hoc, undeclared, and with a path-resolution bug that
will bite the moment it's copied anywhere but its original location.

---

## 1. Problem Statement

### 1.1 Custom DBs already exist, fully outside any schema samgraha knows about

`schema/knowledge-hub/*.sql` and `crates/registry/src/migration.rs`
(`knowledge.db`'s schema — `documents`, `relationships`, `audit_results`,
`graph_nodes`, ...) are samgraha's only two versioned schemas. Neither
mentions `hackathon.db`. It's defined entirely in
`python_hackathon/script/common/db.py:31-60` — three
`CREATE TABLE IF NOT EXISTS` statements the standard's own script owns,
runs, and migrates (by hand, whenever the script changes). Nothing wrong
with a standard needing this — the problem is samgraha has no concept of
it at all: no declaration, no discovery, no path convention.

### 1.2 The path resolution is directory-depth-relative, not root-relative

`common/db.py:15`:

```python
DEFAULT_DB = os.path.join(
    os.path.dirname(__file__), "..", "..", "hackathon.db"
)
```

Resolved relative to the script's own file location, not repo root. In
Kriti today that's `script/common/../../hackathon.db` →
`python_hackathon/hackathon.db` (matches what's on disk). Copied into a
consuming repo via the asset-sync mechanism (`.samgraha/scripts/common/
db.py`), the same two-`..` walk lands at `.samgraha/scripts/common/../../
hackathon.db` → `.samgraha/hackathon.db` — different depth, happens to
still resolve to *somewhere* only because the current nesting is
coincidentally two levels either way. Confirmed nothing at
`E:\Python\Heimdall\.samgraha\hackathon.db` yet (only `knowledge.db`,
`registry.db`, `standards.db` exist there) — it hasn't been exercised
post-sync, so this hasn't been hit yet, but the moment asset-sync's
`script_include` (existing proposal §3.4) pulls a subset with different
nesting, or `system.yaml`'s `assets.scripts` dir name changes, this walk
silently points somewhere else with no error — it'll just create a new
empty db in the wrong place and every prior score row will look missing.

### 1.3 No declaration means no discoverability, no backup convention, no
collision detection

Nothing records that `python_hackathon` owns a file called `hackathon.db`,
which script creates it, or where it's supposed to live per consuming
repo. Two standards both wanting a `data.db` would collide with zero
warning, same shape as the pre-fix flat script store
(`standard-asset-sync-proposal.md` §1.5).

---

## 2. Proposal

### 2.1 Verified this is safe: samgraha's own migrations never touch the
whole file, only tables they name

`crates/registry/src/migration.rs` has 7 `DROP TABLE IF EXISTS`
statements (`graph_edges`, `graph_nodes`, `relationships`,
`audit_results`, `glossary`, `enrichment`, `search_index`) — all
versioned rebuilds of tables samgraha itself created, targeted by exact
name, never a blanket reset. `RegistryDb::open` (`registry_db.rs:22`)
never deletes the file. `--force` compile (`compilation.rs:96`) only
skips incremental hash-based skipping, doesn't recreate `knowledge.db`.
So a custom table added to the same file is untouched by any of
samgraha's own migration/compile paths — the file is additive-only from
samgraha's side.

The one real risk: samgraha's reserved table-name surface is large and
growing — grepped `migration.rs`'s `CREATE TABLE` list: 60+ names,
including one `*_reports` table per documentation domain
(`engineering_reports`, `architecture_reports`, `design_reports`, ...),
and that list grows every time a domain is added. A standard's custom
table landing on a name samgraha claims later is the only real collision
vector — closed by a naming convention (§2.2), not a second file.

### 2.2 Same file, mandatory table prefix, declared in `system.yaml`

```yaml
name: python_hackathon
assets:
  scripts: script
  templates: templates
  data_tables:
    owner_script: common/db.py   # creates/migrates these tables; samgraha never touches their schema
    prefix: hackathon_            # every table this script creates must start with this
```

`hackathon_participants`, `hackathon_domain_scores`,
`hackathon_narratives` instead of today's unprefixed
`standard_participants`/`standard_domain_scores`/`standard_narratives` —
`standard_*` is exactly the shape samgraha's own reserved names already
take (`standard_audit_runs` exists in `migration.rs` today), so the
current unprefixed names are already one bad rename away from a
collision. **Enforced at `register_standard` time, not merely
documented**: the check rejects the registration outright (non-zero exit,
no rows written) if `data_tables.prefix` is empty or matches any name in
samgraha's own reserved list — a cheap check, samgraha already knows that
list completely, it's the process compiling the db. A convention a
standard author can silently skip isn't a convention, it's a suggestion;
this has to be a hard gate at the one point samgraha already controls
(registration), not a lint someone has to remember to run. Absent
`data_tables` block = standard has no custom tables, no change from
today. Reuses the same `assets:` block the asset-sync proposal already
added to `system.yaml` — one manifest, three asset kinds.

### 2.3 Fix the path to be root-relative, not `__file__`-relative

`db.py:15` currently derives its db path from `os.path.dirname(__file__)`
two levels up — works by coincidence today (both Kriti's and Heimdall's
current nesting happen to be 2 deep), breaks the moment asset-sync copies
it somewhere else. Point it at `.samgraha/knowledge.db` directly, resolved
from a root samgraha hands it — same contract check-scripts already use
(`common::env::run_check_script` passes `--repo-root`;
`run_workflow_script` sets `cmd.current_dir(cwd)`). `db.py` takes
`--repo-root` like every other script contract in this codebase, builds
`os.path.join(repo_root, ".samgraha", "knowledge.db")`, and does its
`CREATE TABLE IF NOT EXISTS hackathon_participants (...)` (etc.) against
that shared file instead of a private one it invents.

### 2.4 What samgraha does NOT own

- No migration tooling for the custom tables' internal schema — that
  stays `owner_script`'s `CREATE TABLE IF NOT EXISTS` responsibility, same
  as today.
- No cross-standard schema compatibility guarantees — two standards'
  custom tables can have wildly different shapes, samgraha doesn't care,
  as long as prefixes don't collide.
- No backup/versioning beyond what `knowledge.db` already gets — it's
  already per-repo, already part of whatever backup story `.samgraha/`
  has today; nothing new needed here.
- **No transaction boundary across `owner_script`'s writes.** If a
  standard's script crashes mid-run after writing some but not all of a
  batch (e.g. half the teams scored), that partial state is the
  standard's own problem to detect and recover from (re-run idempotently,
  check a status column, whatever it chooses) — same reasoning as the
  rest of this section: samgraha doesn't own the schema, so it doesn't
  own crash-consistency for writes into it either. Explicit here so it
  doesn't come up as an unstated assumption later: a standard wanting
  atomicity wraps its own writes in a `BEGIN`/`COMMIT` inside
  `owner_script` — samgraha provides the shared file, not a transaction
  boundary around a script it doesn't control the internals of.

Rejected alternative: samgraha-owned migration framework (Rails/Django-
style) for arbitrary standard schemas. Not needed for hackathon-shaped
workflow data — heavier machinery than the problem calls for; revisit only
if a future standard needs actual schema evolution over time rather than
one create-if-missing script.

---

## 3. Migration

- `python_hackathon`: add `assets.data_tables` to a new `system.yaml` (it
  currently has none — confirmed absent from `Kriti/samgraha/system/
  python_hackathon/`, unlike `fastapi_dev`/`react_dev`/etc. which all have
  one). Rename `standard_participants`/`standard_domain_scores`/
  `standard_narratives` → `hackathon_*` and fix `db.py:15` to take
  `--repo-root` and target `.samgraha/knowledge.db` instead of a private
  file it locates by walking its own directory depth.
- **Live data question, needs a decision before this ships**: Kriti's
  existing `python_hackathon/hackathon.db` has real team-scoring rows in
  it right now. Moving to shared `knowledge.db` means either (a) a
  one-time import of those rows into the renamed/prefixed tables in
  whichever repo's `knowledge.db` should own them, or (b) treating this as
  a clean cutover and re-running the hackathon's scoring pipeline fresh.
  Don't silently pick one — confirm which with whoever owns the current
  hackathon run before implementing.

## 4. Testing

- Unit: path-resolution fix in `db.py` — assert it targets
  `<repo-root>/.samgraha/knowledge.db` regardless of the script's own
  directory depth.
- Unit: prefix-collision check — assert a `data_tables.prefix` matching
  any name in samgraha's reserved table list is rejected at
  register-standard time, not silently allowed through.
- Integration: register `python_hackathon` with the new `data_tables`
  manifest entry into a scratch repo, run its scoring script against that
  repo's `knowledge.db`, then run a normal samgraha `compile`/migration
  pass and confirm the `hackathon_*` rows are still there afterward —
  this is the actual claim (§2.1) that needs proving, not just asserting.

---

## 5. Custom-table discoverability — samgraha-owned catalog

Ask: samgraha should be aware of *what* custom tables a standard has, not
just tolerate them existing. Nothing like this exists today — genuinely
new, unlike §6/§7 below. Same shape as `script_checks` (already the
pattern for "standard declares X, samgraha catalogs it queryably"):

```sql
CREATE TABLE custom_data_tables (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    table_name   TEXT NOT NULL,   -- must start with data_tables.prefix (§2.2)
    purpose      TEXT NOT NULL,
    owner_script TEXT NOT NULL,
    UNIQUE(standard_id, table_name)
);
```

Populated at `register_standard` from `system.yaml`'s `data_tables` block
(§2.2), queryable by MCP the same way `script_checks` already is
(`get_standard`-style introspection).

**Why `knowledge.db`, not `standards.db`, even though this is
declarative standard metadata same as `script_checks`/`rules`**: because
the tables it catalogs physically live in `knowledge.db` (§2.1's
decision), not `standards.db`. `standards.db` is a periodically-refreshed,
effectively read-only snapshot — wholesale-copied from the global store on
`register_standard`/`sync_standards` (per the asset-sync proposal), so it
can be temporarily stale or absent mid-sync. `knowledge.db` is the repo's
own always-present live file. Keeping the catalog in the same file as the
data it describes means one connection answers "what custom tables does
this repo actually have" self-containedly — no cross-file join against a
snapshot that might not match what's really in `knowledge.db` right now.
If a future need arose to introspect a standard's *declared* custom
tables independent of any specific repo (before it's ever registered
anywhere), that's a `standards.db` question and would want its own
row there — different question from "what does this repo's file contain
today," which is what `custom_data_tables` in `knowledge.db` answers.

## 6. Usecase registry and script dispatch — already built, needs wiring,
not new schema

Checked before proposing anything new: `crates/audit/src/capability.rs:
253-299` (`PlanPhase`/`PlanUseCase`/`InitPlan`) plus `workflow_use_cases`/
`workflow_phases`/`workflow_phase_dependencies` (`schema/knowledge-hub/
25-27`) and the `store_system_plan`/`get_system_plan` MCP tools
(`adapter.rs:2562,2684`) already model exactly this:

- `kind`: `"semantic"` (LLM step, no script) or `"script"` (samgraha
  dispatches) — the deterministic/semantic split.
- `script`: relative path, for `kind: "script"` phases — which script for
  which usecase.
- `depends_on: Vec<String>` — real dependency edges, not a forced linear
  order; a phase runs once its deps are satisfied, not "when its turn in a
  numbered list comes up."
- `description` — what a phase is for.

`python_hackathon/plan/core/loop.yaml`'s 8 stages already carry this
shape informally (`script: run_hackathon.py --deterministic-only` vs.
`script: agent-driven sessions` is exactly `kind: "script"` vs.
`kind: "semantic"`; `rule: always runs FIRST per team, before 2b` is
exactly a `depends_on` edge) — but nothing has ever converted it and
called `store_system_plan` for this standard. **The gap is a missing
conversion step, not missing schema.** Task: turn `loop.yaml`'s 8 stages
into an `InitPlan` JSON, call `store_system_plan("python_hackathon", ...)`
once at register time (or author the JSON directly going forward and
retire `loop.yaml` as the source of truth).

Separately: a generic "catalog every script" table is unneeded —
`PlanPhase.script` + `.description` already is that, at the level MCP
actually needs (dispatch: which script for which usecase, and why).
Helper modules not bound to a single phase (`common/db.py`) don't need
cataloging — MCP never calls them directly, only entry-point scripts a
phase names.

## 7. `audit/` and `analysis/` are inputs, not outputs — seed-once, not
sync-always

Checked: zero Rust code references `.samgraha/audit/` or
`.samgraha/analysis/` as literal paths — grepped the whole `crates/`
tree. Samgraha's own core audit engine (`semantic_reports`/
`pipeline_reports`/`report_findings` in `knowledge.db`) has no
involvement in either folder; they're pure `python_hackathon` convention.

Read the actual content, not assumed from folder names:

- `audit/semantic/document/*.yaml` — rubric *definitions* (domain,
  required ensemble models, evaluation criteria). Standard-definition
  content, same category as `domains/*.md`/`calculation/*.yaml`, meant for
  `knowledge-hub-loader.py`'s ingest into `standards.db`.
- `analysis/00-leaderboard.md` — a **prompt template**, not output. Its own
  text says "Not parsed by a script — this is a prompt" and lists
  `standard_domain_scores` DB table as its input. It's the spec an
  MCP-driving agent reads to perform the semantic Analysis phase.
- Actual results (scores, narratives) live only in the DB tables from §1-2
  (`hackathon_domain_scores`/`hackathon_narratives`) — never in these
  folders.

Diffed Kriti's source against Heimdall's copy:
`audit/semantic/document/01-infrastructure.yaml` is 33 lines in Kriti, 128
in Heimdall — genuinely different content, not accidental sync drift. This
competition's rubric was deliberately customized after seeding. That's a
different behavior contract than scripts/templates, which should always
stay in lockstep with the standard.

**Proposal**: `audit/` and `analysis/` are a distinct asset behavior —
seed-once, not sync-always:

- On first `init`, copy from the standard's declared dirs (`system.yaml`'s
  `assets:` block, same manifest scripts/templates already use).
- On re-sync, **never overwrite** a file that already exists locally —
  identical to the rule `init_repository` already applies to
  `samgraha.toml` itself (`init.rs:125-158`, "backfill missing keys, never
  touch existing ones"). Not new behavior, the existing pattern applied to
  a new asset kind.
- Results never live here — always the prefixed DB tables (§1-2), full
  stop.

**Config representation — don't paint this into a boolean corner.**
`AssetSyncConfig` (`common/config.rs:467-479`, already shipped) has
`scripts: bool`/`templates: bool` — fine as-is, since "always resync" is
genuinely their only mode and changing already-shipped fields isn't worth
it here. But `audit`/`analysis` need a third mode (`seed_once`) that a
plain bool can't express alongside `scripts`/`templates`'s "off vs.
always." So the new fields go in as a string-enum from the start, not
`bool`:

```toml
[repository.documentation.asset_sync]
scripts = true
templates = true
audit = "seed_once"      # "off" | "seed_once" — never "always": these aren't meant to stay in lockstep
analysis = "seed_once"
```

Adding a fourth sync mode later (say, `scripts` growing a `seed_once`
option too) means widening this same enum, not reworking a boolean into
something else — the representation already has room for it. Concretely:
`enum AssetSyncMode { Off, SeedOnce }` today (bool stays for `scripts`/
`templates` since neither needs a third state yet), with the door open to
promote them to the same enum if that ever changes.

## 8. Phase execution — one generic semantic-run primitive, audit and
generate unified, samgraha stays opaque to which one it is

**Revised ask, tighter than the original framing below**: samgraha should
not know or care whether a semantic phase is "auditing" or "generating."
Both are the same shape from samgraha's side — pre-script (deterministic
prep) → semantic step (LLM reasoning over prepared data + instruction
content, the one part that can't be a script) → post-script (deterministic
persist). The *meaning* (score-and-report vs. write-new-content) lives
entirely in what the instruction content says and what `post_script` does
with the result — samgraha's own code should never branch on "is this
audit or generate," only ever run the same three steps.

Checked this against what already exists, twice — once for the narrower
original framing, once for whether it actually forces the generalization:

- `PlanPhase.kind` (`capability.rs:267-272`) is already just
  `"semantic"` vs. `"script"` — it was never "audit" vs. "generate" in the
  first place. The schema already doesn't distinguish the two; only the
  *current tools* (`audit`'s `semantic_review` builder, `generate`'s task
  builder) hardcode that distinction, each as separate Rust code with its
  own data shape (`document_sections`/`domains` for audit,
  `generation_granularity`/`content_kind` for generate — confirmed
  previous turn). The schema was already generic; the tools built on top
  of it weren't.
- `pre_script`/`post_script` fields and `workflow_phases` columns
  (`schema/knowledge-hub/26-workflow_phases.sql:16-17`) already exist —
  same finding as before, still true.
- `run_check_script`/`run_capability_script` (`common/env.rs`) already
  define a symmetric, meaning-agnostic script contract: `--repo-root`,
  `--in`, `--out`, JSON envelope in and out. Neither knows or cares what
  the script computes — exactly the property needed here. `post_script`
  gets the agent's output as `--in` and decides what "persist" means
  (write findings, write generated files, write DB rows) entirely on its
  own — samgraha just moves the envelope.
- Still true: no handler executes a phase at all.
  `store_system_plan`/`get_system_plan` only persist/read the plan shape.

**Proposal** (design only, still not implemented): one MCP tool —
`run_semantic_phase` (or fold into a generalized `run_phase` covering both
`kind`s) — with exactly one execution shape for `kind: "semantic"`,
no audit/generate branch anywhere in it:

1. If `pre_script` set, run it via `run_capability_script`
   (`--repo-root`, `--out`) — deterministic, writes prepared data to a
   JSON envelope. Samgraha doesn't interpret this data, just relays it.
2. Return to the calling agent: the prepared data (from step 1) plus the
   phase's `instruction` content, verbatim — this is where "audit" or
   "generate" is decided, entirely by what that content says. For
   `python_hackathon`, `instruction` would point at either an
   `audit/semantic/document/*.yaml` rubric (scoring) or an
   `analysis/*.md` prompt (narrative generation) — samgraha reads neither
   for meaning, only for bytes to hand over. Same tool, same three steps,
   either case.
3. Calling agent reasons over data + instruction, produces a result of
   whatever shape the instruction implied, reports it back over MCP.
4. `post_script` runs via `run_capability_script` (`--in` = agent's
   result, `--repo-root`, `--out` = status), persists wherever the
   standard's own script decides — a custom table (§1-2), a generated
   file, anything. Samgraha never inspects what got written.

`PlanPhase` needs one new field to carry this: `instruction_path` (or
reuse `description` if it's ever just a file path rather than free text —
cleaner as its own field so `description` stays human-readable metadata
and `instruction_path` stays "what to hand the agent"). Points at a file
under the already-seeded `.samgraha/audit_analysis/` (§7) or
`.samgraha/templates/` — either way, content the standard already ships,
never samgraha-authored.

Net effect: `audit`/`generate` as they exist today (`handle_audit`,
`handle_generate`) stay as-is for samgraha's own single-repo document
model — no reason to touch working code. `run_semantic_phase` is the new,
separate, genuinely generic primitive for anything declared via
`workflow_phases` — standard-agnostic and audit/generate-agnostic by
construction, not by convention. `python_hackathon`'s two semantic flows
(score-and-report, write-narrative) become two *instances* of this one
mechanism, distinguished only by which `instruction_path` and which
pre/post scripts their `system.yaml`-declared phases point at — never by
a branch in samgraha's own code.

Not building this now — flagging it as the concrete next task once §1-7
land, since it's the piece that actually makes a declared semantic phase
runnable instead of just describable.
