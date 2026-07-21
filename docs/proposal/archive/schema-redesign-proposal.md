# Knowledge-Hub Schema Redesign Proposal

**Trigger**: a real gap found in conversation — the phase/script-level
dependency graph (`system_plans.plan_json`'s `PlanPhase.depends_on`) is a
JSON blob, not a relational schema, unlike the domain-level dependency
graph (`domain_relationships`, real rows, real joins). Asked to (1) fix
that inconsistency with a normalized schema, (2) audit every existing
table for real usage, (3) remove what's vestigial, (4) redesign wherever
it makes the schema better fit the current (capability-script) architecture.

**Status**: PROPOSED — not implemented. Audit findings are verified
against actual Rust source (cited file:line below), not inferred from
doc comments or schema-file intent.

**Rebuild, not migrate** (per `crates-refactor-proposal.md` §7.4,
reaffirmed): `standards.db` holds no permanent data — every change below
is a schema-file edit + `SCHEMA_VERSION` bump (currently `1`, in both
`knowledge-hub-loader.py:33` and `db_reader.rs:10`) + a full re-register of
every system. No `ALTER TABLE`, no in-place upgrade path needed or built.

---

## 1. Audit: What's Actually Used Today

Checked every one of the 22 defined tables against real Rust call sites —
not "the loader writes it" (that's necessary but not sufficient), but
"something reads it back and that something is reachable from a live MCP
tool or CLI command."

| Table | Verdict | Evidence |
|---|---|---|
| `systems`, `standards` | LIVE | Core to every registry bootstrap (`db_reader.rs:285-620`, called from `runtime.rs:58`, `workspace.rs:96`, `adapter.rs:837` — every MCP/runtime startup) |
| `domains`, `section_catalog`, `relationship_types`, `domain_relationships`, `standard_docs` | LIVE | Assembled into every `StandardDefinition`; `standard_docs` backs the live `get_standard_doc` tool (`adapter.rs:1647-1650`) |
| `rules`, `rule_evidence_params`, `script_checks` | LIVE | Read by `providers.rs`'s `DeterministicAuditProvider` and `check_runner.rs`, in the live `audit` tool path (`adapter.rs:313`) |
| `calculation_rules`, `calculation_inputs`, `score_bands` | LIVE | Read by `framework.rs`/`calculation.rs`, feeding the live `audit` tool's scoring |
| `templates` | LIVE | Backs the live `get_audit_knowledge` tool (`adapter.rs:330`) |
| `plan_settings`, `plan_scenarios` | LIVE | Back the live `get_plan_settings`/`get_plan_scenarios` tools (`adapter.rs:1861,1876`) |
| `script_runs`, `system_plans` | LIVE | This session's work — `record_script_run`/`check_phase_prerequisites`, `store_system_plan`/`get_system_plan`, all backing live `run_system_*` tools |
| `script_check_dependencies` | **LOADED-BUT-UNUSED** | Populated by the loader (`knowledge-hub-loader.py:621`), zero Rust references anywhere in `crates/` |
| `validation_rules` | **LOADED-BUT-UNUSED** | Read into `ScoringConfig.validation_rules` (`db_reader.rs:126-141`), but that field has zero readers anywhere else — computed and discarded every startup |
| `workflow_stages` | **LOADED-BUT-UNUSED** | Same shape — `registry.workflow_stages()` getter exists (`registry.rs:229`) but has zero callers in `crates/` |
| `plan_generation_inputs` | **ORPHANED** | Designed in §8.3 of `generic-script-architecture-proposal.md`, but never wired up — no INSERT anywhere (not even the Python loader), no MCP tool writes to it. The semantic-phase write-back path this table exists for was never built. |

**18 of 22 tables are genuinely live.** The other 4 split into two very
different problems: 3 are dead weight (query the DB, throw the result
away), 1 is a real missing feature (a table sitting empty because nothing
ever got built to write to it).

### 1.1 A second axis: which mechanism generation each LIVE table serves

Worth naming explicitly, because it changes what "redesign" should mean
for each one — these aren't all the same kind of table:

- **Registry/catalog (permanent, mechanism-independent)**: `systems`,
  `standards`, `domains`, `relationship_types`, `domain_relationships`,
  `section_catalog`, `standard_docs`. Pure reference data. Keep regardless
  of how validate/calculate/report end up implemented.
- **Gen 1 — DB-driven rule interpretation (live today, architecturally
  destined to shrink)**: `rules`, `rule_evidence_params`, `script_checks`,
  `calculation_rules`, `calculation_inputs`, `score_bands`, `templates`
  (audit-rendering half), `plan_settings`, `plan_scenarios`. This is
  `providers.rs`/`framework.rs` generically interpreting declarative rows
  — not domain-specific Rust code (unlike the 22 deleted pipeline
  modules), but still samgraha doing domain-interpretation work a
  system's own `validate`/`calculate`/`report` script is meant to take
  over. Same phased-removal shape as the pipelines: don't touch until a
  system's script actually replaces what a given row does, then that
  row's table shrinks — but there's no code today making that
  replacement happen (no `run_pipeline()`-style capability-first dispatch
  exists in `providers.rs`/`framework.rs` yet). Flagging this, not fixing
  it here — it's the same pattern as `codebase-refactoring-proposal.md`'s
  Phase 1/4, just one mechanism layer up, and deserves its own proposal
  rather than folding into a schema redesign.
- **Gen 2 — capability-script dispatch (current target, incomplete)**:
  `script_runs`, `system_plans`, and the orphaned `plan_generation_inputs`.
  This redesign's actual scope.

---

## 2. The Main Gap: Phase/Script Graph Isn't Relational

`system_plans.plan_json` stores an entire `InitPlan` (use cases, phases,
`depends_on` arrays, `expiry` rules) as one JSON blob per (standard,
repo). Consequences:

- Can't `SELECT` a phase's dependencies — must parse JSON in Rust first
  (`InitPlan::find_phase`, `capability.rs:282-289`)
- Can't query "which phases depend on X" without loading and scanning
  every plan
- Inconsistent with `domain_relationships`, which solved the *identical*
  structural problem (nodes + typed edges + gating strictness) as real
  rows
- The schema's own stated convention already rejects this shape
  elsewhere: `script_check_dependencies`'s doc comment says "real edges
  instead of a comma-separated or JSON list column" — that's the exact
  principle not applied to `PlanPhase.depends_on`

### 2.1 Proposed schema — normalize, don't duplicate

Replace `system_plans` with three tables. Scoped by `(standard_id,
use_case_id)`, mirroring `domain_relationships`' `(standard_id, from, to,
relationship_type)` shape:

```sql
-- One row per use case a system's init plan declares (§8.4's
-- PlanUseCase, minus the phases array — those are workflow_phases below).
CREATE TABLE workflow_use_cases (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    use_case_id  TEXT    NOT NULL,  -- e.g. "repo_new-case_1_no_documentation"
    label        TEXT    NOT NULL,
    UNIQUE(standard_id, use_case_id)
);

-- One row per phase (§8.4's PlanPhase, minus depends_on — real edges below,
-- not an embedded array).
CREATE TABLE workflow_phases (
    id                INTEGER PRIMARY KEY,
    use_case_id       INTEGER NOT NULL REFERENCES workflow_use_cases(id) ON DELETE CASCADE,
    phase_id          TEXT    NOT NULL,  -- e.g. "tier1-generate"
    kind              TEXT    NOT NULL CHECK (kind IN ('semantic','script')),
    description       TEXT,
    script_path       TEXT,              -- NULL for kind='semantic'
    pre_script        TEXT,
    post_script       TEXT,
    expiry_rule_json  TEXT,              -- same shape as script_runs.expiry_rule_json; NULL = never expires
    UNIQUE(use_case_id, phase_id)
);

-- Real edges, not a JSON array — same pattern domain_relationships
-- already established, applied at phase granularity.
CREATE TABLE workflow_phase_dependencies (
    id                    INTEGER PRIMARY KEY,
    phase_id              INTEGER NOT NULL REFERENCES workflow_phases(id) ON DELETE CASCADE,
    depends_on_phase_id   INTEGER NOT NULL REFERENCES workflow_phases(id) ON DELETE CASCADE,
    UNIQUE(phase_id, depends_on_phase_id)
);
```

`check_phase_prerequisites` (`capability.rs:315-380`) changes from
"parse JSON, find phase, read its `depends_on` array" to a plain SQL
join against `workflow_phase_dependencies` — same semantics, real rows.
`store_system_plan`/`get_system_plan` MCP tools become inserts/selects
against these three tables instead of one JSON blob column.

**On performance, explicitly, so it doesn't get re-litigated later**:
this isn't a speed argument. Typical cardinality is tiny — `base_dev`'s
real plan (verified last session) is 4 use cases × 8 tiers × 3 stages,
24 phases per use case, well under 100 rows total. A JSON blob parse at
that size is already fast; the join isn't meaningfully faster. The
reason to normalize is queryability and consistency with
`domain_relationships`, not throughput — nobody should propose "just
index the JSON instead" as a competing fix, because the problem being
solved isn't a performance one.

### 2.2 What this does *not* touch

`domain_relationships` stays exactly as-is — it's live, tested, working,
and already the correct shape. This redesign adds the missing sibling for
phase-level graphs; it doesn't rework what already works. (A fully unified
polymorphic node/edge schema covering both domain- and phase-level graphs
in one set of tables was considered and rejected here — it would touch
the live, working `domain_relationships` mechanism for a benefit this
proposal doesn't need. Revisit only if a real case for cross-referencing
a domain node and a phase node in the same query shows up.)

### 2.3 Script registry — the other half of "track purpose, when to run"

`workflow_phases.script_path`/`description` covers "what does this script
do, which phase does it belong to" for phases already declared in a plan.
But there's no table of "what capability scripts does this system have,
period" independent of any specific use case — e.g. `calculate` is one
script but appears in zero or many use cases' phase lists. Given the
audit found no evidence this is needed yet (no code asks "list all
scripts a system has" independent of a plan), **not** proposing a
separate `script_registry` table now — `workflow_phases` already answers
"which script for which phase," and capability discovery itself is
filesystem-based (`resolve_capability`'s tiers), not DB-based, by design
(§2.4 of the architecture proposal: discovery stays generic and doesn't
need a database at all). Flagging the option, not building it — YAGNI
until something actually needs to enumerate a system's scripts via SQL.

---

## 3. Fixing the Orphaned Table: `plan_generation_inputs`

This isn't a table to remove — it's a *feature that was designed and
never built*. §7.2/§8.3's semantic-determination step is supposed to
write here; nothing does. **Committing to a concrete spec here, not
deferring it again** — it's small enough (one INSERT tool, symmetric to
the already-built `store_system_plan`) that "flag as follow-up" is the
wrong call; that's exactly how an orphaned table stays orphaned for
another cycle.

### 3.0 Schema bug found while checking this, before the tool — fix first

Re-read `10-plan_generation_inputs.sql`'s actual `UNIQUE` constraint
before writing the `ON CONFLICT` clause below, since the constraint has
to genuinely exist for that to work. It does exist syntactically —
`UNIQUE(standard_id, repo_fingerprint, workflow_id, domain_key,
instance_key)` — but `domain_key`/`instance_key` are nullable and used
raw, not coalesced. SQLite's `UNIQUE` treats `NULL != NULL`, so for the
table's own documented "plan-level" case (both columns `NULL`, per the
table's own comment: *"NULL = plan-level, not domain-specific"*), the
constraint never catches a duplicate — `ON CONFLICT` would silently never
fire for that case, and every re-run would insert a fresh row instead of
upserting. This is the exact bug `rules.sql`'s own comment already
documents finding and fixing elsewhere in this schema (*"SQL NULL is
never equal to NULL... found by actually re-running the loader twice and
diffing row counts — not a hypothetical"*) — `templates.sql` hit and
fixed the same thing with `audit_bucket_key`. `plan_generation_inputs`
never got the equivalent treatment. Fix, same pattern:

```sql
CREATE TABLE plan_generation_inputs (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id          INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint     TEXT    NOT NULL,
    workflow_id          TEXT    NOT NULL,
    domain_key           TEXT,
    instance_key         TEXT,
    input_json           TEXT    NOT NULL,
    previous_input_json  TEXT,
    created_at           TEXT    NOT NULL DEFAULT (datetime('now')),
    domain_key_key    TEXT GENERATED ALWAYS AS (COALESCE(domain_key, '')) VIRTUAL,
    instance_key_key  TEXT GENERATED ALWAYS AS (COALESCE(instance_key, '')) VIRTUAL,
    UNIQUE(standard_id, repo_fingerprint, workflow_id, domain_key_key, instance_key_key)
);
```
The `ON CONFLICT` target in the tool below uses the coalesced columns,
not the raw nullable ones. Only found because the review question forced
checking the actual file instead of trusting the earlier read.

**New MCP tool — `store_plan_generation_input`**:
```
Params: system_name (string), workflow_id (string),
        domain_key (string, optional), instance_key (string, optional),
        input_json (string — the semantic determination's own output)

Handler (mirrors handle_store_system_plan's shape in adapter.rs):
  standard_id = resolve_standard_id(&conn, Some(&system_name))?
  repo_fingerprint = common::env::repo_fingerprint(&repo_root)
  INSERT INTO plan_generation_inputs
      (standard_id, repo_fingerprint, workflow_id, domain_key,
       instance_key, input_json, created_at)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
  -- Targets the coalesced virtual columns (§3.0's fix), not the raw
  -- nullable domain_key/instance_key — SQLite can't match a NULL-bearing
  -- ON CONFLICT target against the raw columns.
  ON CONFLICT (standard_id, repo_fingerprint, workflow_id, domain_key_key, instance_key_key)
  DO UPDATE SET previous_input_json = plan_generation_inputs.input_json,
                input_json = excluded.input_json,
                created_at = excluded.created_at
```
Symmetric `get_plan_generation_input` tool for the read side (same
resolve pattern as `handle_get_system_plan`). `plan-generation`'s script
`--in` payload (§8.2) gets assembled by reading this table server-side,
not left to the caller to hand-assemble.

**Recommend building this in the same pass as §2's rebuild**, not as a
separately-scheduled follow-up — same rollout step (§5), same
`cargo build`/test cycle, no reason to split it into its own session
given how small it is.

---

## 4. Fixing the Two Dead Tables

### 4.1 `workflow_stages` — remove

Query runs every startup (`db_reader.rs:193`), result discarded (zero
callers on the getter). Its job — "a standard's declared stage list, in
order" — is already covered by `plan_scenarios` (repo-state × doc-state ×
tier × step, actually consumed) and, going forward, `workflow_phases`
(§2.1). No migration needed for existing data — nothing reads it, so
there's nothing to carry forward. Delete the table, delete
`load_workflow_stages()`/`registry.workflow_stages()`/`set_workflow_stages()`
in `db_reader.rs`/`registry.rs`, remove the loader pass that populates it.

### 4.2 `script_check_dependencies` — remove

Same shape: real edges (correctly modeled, per its own doc comment), zero
consumers. `script_checks` itself stays live (backs `list_script_checks`)
— only the unused dependency-ordering sub-table goes. If check-ordering
within the Gen-1 rule-interpretation mechanism ever becomes a real
requirement, `workflow_phase_dependencies`' pattern (§2.1) is right there
to reuse rather than reviving this table.

### 4.3 `validation_rules` — keep, but not as samgraha-executed logic

Different from the other two: this one has an actually good idea behind
it (lint a standard's own `calculation_rules`/`score_bands` for internal
consistency — weight sums, score bounds, domain counts). "Wire it up in
`register_standard`" (my first pass) was vague and, on closer look,
wrong — the table's own doc comment already settles this: `rule` is
prose, "documentation for a human/LLM reading the check... not something
this schema or any Rust code parses as an expression language." That's
the same no-samgraha-side-vocabulary principle every other table here
follows (`calculation_rules.bucket` has no `CHECK` constraint for the
identical reason). Writing a Rust or Python function that *executes*
these rules would be exactly the kind of samgraha-encodes-domain-logic
mistake this whole architecture pivot exists to avoid — and a hardcoded
"weight sums to 100" check is no different in kind from a hardcoded
Rust pipeline, just smaller.

**Concrete answer, not a hand-wave**: no new samgraha code at all. The
table stays exactly as populated today, queryable by a system's own
script through the already-sanctioned direct-DB-access path
(`knowledge-system-author-guide.md`'s FAQ: scripts can open
`.samgraha/knowledge.db` directly). A system's `calculate` or `validate`
script reads its own `validation_rules` rows and self-checks against
them — same pattern as everything else moving toward system-owned logic.
This earns the table's keep without adding a single line of
samgraha-side interpretation.

---

## 5. Rollout

Per §7.4's rebuild convention (reaffirmed at top):

1. Edit `01-systems.sql`... `24-workflow_stages.sql` etc. directly —
   delete `08-script_check_dependencies.sql` and `24-workflow_stages.sql`
   entirely, replace `25-system_plans.sql` with the three new tables from
   §2.1 (renumber if needed — `25-`, `26-`, `27-`)
2. Fix `10-plan_generation_inputs.sql`'s `UNIQUE` constraint (§3.0) — add
   `domain_key_key`/`instance_key_key` virtual columns, same pass, since
   it's touching schema files anyway
3. Add `store_plan_generation_input`/`get_plan_generation_input` MCP
   tools (§3) — same pass, not a separate follow-up session
4. Bump `SCHEMA_VERSION` (loader) / `EXPECTED_SCHEMA_VERSION`
   (`db_reader.rs:10`) from `1` to `2`
5. Remove the now-dead `load_workflow_stages`/`registry.workflow_stages()`
   Rust code (§4.1) — actual dead-code deletion, not just a schema change
6. Rewrite `check_phase_prerequisites`/`store_system_plan`/
   `get_system_plan` to use the new relational tables instead of
   `system_plans.plan_json` (§2.1)
7. Full re-register of every system (dev-class + academic + hackathon) —
   coordinated single session, matches the existing rebuild convention,
   no partial-migration window

## 6. Summary Table

| Table | Action |
|---|---|
| `workflow_stages` | **Remove** — dead, superseded by `plan_scenarios`/`workflow_phases` |
| `script_check_dependencies` | **Remove** — dead, zero consumers |
| `validation_rules` | **Keep, no samgraha code** — good idea, but the fix is a system's own script reading it directly, not new Rust/Python logic here |
| `plan_generation_inputs` | **Keep, fix + wire up now** — real `UNIQUE`-constraint NULL bug found and fixed (§3.0), then `store_plan_generation_input`/`get_plan_generation_input` tools (§3), same pass as §2's rebuild, not deferred |
| `system_plans` | **Replace** with `workflow_use_cases` + `workflow_phases` + `workflow_phase_dependencies` (§2.1) — this is the actual fix for the reported gap |
| `domain_relationships` and everything else LIVE | **No change** — already correct, already working |

---

## 7. Tracked, Not Scoped Here: Gen-1 Schema Debt

§1.1 named the real issue and stopped short of a plan — fixing that.
`rules`/`rule_evidence_params`/`script_checks`/`calculation_rules`/
`calculation_inputs`/`score_bands`/`templates`(audit half)/`plan_settings`/
`plan_scenarios` are all live today, all genuinely used, and all destined
to become the *next* version of exactly the problem this session already
solved once — samgraha interpreting a system's domain logic itself
instead of dispatching to that system's own script.

**Not scoping the removal here** — it depends on work this proposal
doesn't control (systems actually writing `validate`/`calculate` scripts,
same prerequisite `codebase-refactoring-proposal.md`'s Phase 4 had for
the 22 Rust pipelines). But leaving it as a paragraph risks it being
forgotten the way `plan_generation_inputs` almost was. Concretely:

- **Tracking doc**: `docs/gen1-schema-debt-proposal.md`, to be written
  once at least one system has a real `calculate` or `validate` script
  proven against real repo state (mirrors this session's own
  `calculate`/`rust_dev` pilot recommendation from
  `generic-script-architecture-proposal.md` §5)
- **Precedent to follow when that doc gets written**: the exact phased
  pattern `codebase-refactoring-proposal.md` §10 Phase 1→4 already used
  for the Rust pipelines — rewire the read path to try the capability
  script first, prove it per table/domain, only then drop that table's
  rows or the table itself. Not a blanket "delete these 9 tables" — the
  same one-domain-at-a-time discipline, applied one layer up.
- **Trigger condition, so this doesn't sit forever waiting for a
  "someday"**: the first time a system's `calculate.py` or `validate.py`
  script actually ships and gets proven, that's the signal to write the
  tracking doc — not before (nothing to phase out yet) and not
  indefinitely after (once one script exists, the pattern for the rest
  is already known and shouldn't wait).

---

## 8. Implementation Plan

Phase-wise execution order. Each phase is independently committable and
the codebase compiles after each phase (no half-broken intermediate
states). Phases 1–2 are pure deletion. Phases 3–4 are rewrites that
depend on phase 1's schema. Phase 5 is additive (new tools). Phase 6 is
a version bump. Phase 7 is verification.

### Phase 1: Schema file changes

**Files changed:**
- DELETE `schema/knowledge-hub/08-script_check_dependencies.sql`
- DELETE `schema/knowledge-hub/24-workflow_stages.sql`
- CREATE `schema/knowledge-hub/25-workflow_use_cases.sql` (§2.1)
- CREATE `schema/knowledge-hub/26-workflow_phases.sql` (§2.1)
- CREATE `schema/knowledge-hub/27-workflow_phase_dependencies.sql` (§2.1)
- EDIT `schema/knowledge-hub/10-plan_generation_inputs.sql` — add
  `domain_key_key`/`instance_key_key` COALESCE virtual columns, repoint
  UNIQUE constraint (§3.0)
- EDIT `schema/knowledge-hub/00-reset.sql` — add missing DROP statements
  for `plan_generation_inputs`, `validation_rules`, `script_runs`,
  `workflow_stages`, `system_plans` (currently absent from reset); add
  drops for new tables in reverse dependency order

**Verification:** `00-reset.sql` + all remaining `.sql` files load
cleanly into an empty SQLite database with no errors.

### Phase 2: Remove dead Rust code

**Files changed:**
- EDIT `crates/standards/src/db_reader.rs` — delete
  `load_workflow_stages` (lines 190–221), delete the call site that
  invokes it and `set_workflow_stages` (lines 592–598), remove
  `WorkflowStage` from the import at line 5
- EDIT `crates/standards/src/registry.rs` — remove `workflow_stages`
  field from `StandardRegistry` (line 17), remove `workflow_stages()`
  getter (lines 229–231), remove `set_workflow_stages()` setter (lines
  233–235), remove `WorkflowStage` from the import at line 4
- EDIT `crates/schemas/src/standard.rs` — delete `WorkflowStage` struct
  (lines 190–199)

**Verification:** `cargo build` succeeds. No remaining references to
`WorkflowStage`, `load_workflow_stages`, `set_workflow_stages`, or
`workflow_stages()` outside of dead-code warnings (which are the point).

### Phase 3: Rewrite store/get system plan for normalized tables

**Files changed:**
- EDIT `crates/mcp/src/adapter.rs` — rewrite `handle_store_system_plan`
  (lines 2121–2156) to:
  1. Parse the incoming `plan_json` into `InitPlan` (same validation
     as today)
  2. For each `PlanUseCase` in `plan.use_cases`: INSERT into
     `workflow_use_cases` (ON CONFLICT DO UPDATE label)
  3. For each `PlanPhase` in `use_case.phases`: INSERT into
     `workflow_phases` (ON CONFLICT DO UPDATE all mutable fields);
     serialize `PlanPhase.depends_on` into dependency rows
  4. For each `(phase_id, dep_id)` in `PlanPhase.depends_on`: INSERT
     into `workflow_phase_dependencies` (ON CONFLICT DO NOTHING)
  5. Return same JSON shape as today (`{stored, system_name,
     use_cases, total_phases}`) for backward compat

- EDIT `crates/mcp/src/adapter.rs` — rewrite `handle_get_system_plan`
  (lines 2159–2189) to:
  1. SELECT from `workflow_use_cases` + `workflow_phases` +
     `workflow_phase_dependencies` to reconstruct `InitPlan`
  2. Reconstruct the `InitPlan` JSON from normalized rows (use_cases
     → phases → depends_on arrays)
  3. Return same JSON shape as today (`{system_name, plan}`)

**Verification:** `cargo build` succeeds. `store_system_plan` followed
by `get_system_plan` round-trips identically to the old JSON-blob path.

### Phase 4: Rewrite prerequisite gate + check_phase_prerequisites

**Files changed:**
- EDIT `crates/audit/src/capability.rs` — rewrite
  `check_phase_prerequisites` (lines 315–380) to:
  1. Accept `phase_id: &str` (the phase's string ID, not `&PlanPhase`)
     and `use_case_id: Option<&str>` (optional scoping)
  2. Query `workflow_phase_dependencies` + `workflow_phases` to get
     dependency phase IDs for the given phase
  3. For each dependency, run the same `script_runs` lookup + expiry
     evaluation as today (lines 330–375 unchanged)
  4. Return `PrerequisiteCheck` as before

- EDIT `crates/mcp/src/adapter.rs` — rewrite the prerequisite gate
  block in `run_capability_for` (lines 2005–2043) to:
  1. Instead of querying `system_plans.plan_json`, query
     `workflow_use_cases` + `workflow_phases` to find the phase by
     `phase_id` string
  2. Extract `expiry_rule_json` from `workflow_phases` and deserialize
     into `Option<ExpiryRule>` (replaces `gated_phase: Option<PlanPhase>`
     with `gated_expiry: Option<ExpiryRule>`)
  3. Call the rewritten `check_phase_prerequisites` with the phase ID
  4. Downstream `record_script_run` (line 2087–2098) uses
     `gated_expiry` instead of `gated_phase.as_ref().and_then(|p|
     p.expiry.as_ref())`

- EDIT `crates/audit/src/capability.rs` — `InitPlan.find_phase` (line
  284) becomes unused after this change. Delete the method and the
  `impl InitPlan` block (lines 282–290) — `InitPlan` struct itself
  stays (still used for serialization in store/get handlers).

**Verification:** `cargo build` succeeds. A capability dispatch with
`phase_id` param still gates correctly against the new normalized
tables.

### Phase 5: Add store/get plan generation input MCP tools

**Files changed:**
- EDIT `crates/mcp/src/adapter.rs` — add two new handler methods:
  - `handle_store_plan_generation_input`: params
    (`system_name`, `workflow_id`, `domain_key?`, `instance_key?`,
    `input_json`); INSERT into `plan_generation_inputs` with
    `ON CONFLICT` using the COALESCE virtual columns (§3.0); returns
    `{stored: true, ...}`
  - `handle_get_plan_generation_input`: params (`system_name`,
    `workflow_id`, `domain_key?`, `instance_key?`); SELECT from
    `plan_generation_inputs` using the COALESCE virtual columns;
    returns `{input_json, previous_input_json, ...}` or `{input: null}`

- EDIT `crates/mcp/src/adapter.rs` — register both tools in the
  capability methods list (around line 191–192) and add dispatch arms
  in the match block (around line 389–390)

- EDIT `crates/mcp/src/main.rs` — add tool definitions in the tools
  list (around lines 1093–1107) and dispatch arms

**Verification:** `cargo build` succeeds. Both tools are listed in the
MCP tool catalog.

### Phase 6: Bump schema version

**Files changed:**
- EDIT `schema/knowledge-hub/knowledge-hub-loader.py` line 33 —
  `SCHEMA_VERSION = 1` → `SCHEMA_VERSION = 2`
- EDIT `crates/standards/src/db_reader.rs` line 10 —
  `EXPECTED_SCHEMA_VERSION: i64 = 1` → `EXPECTED_SCHEMA_VERSION: i64 = 2`

**Verification:** Existing `standards.db` with version 1 correctly
rejects (version mismatch error). Freshly registered DB has version 2.

### Phase 7: Build and verify

**Commands:**
```
cargo build 2>&1
cargo test 2>&1  (if tests exist)
```

**Full verification checklist:**
1. `cargo build` — zero errors, zero new warnings (dead-code warnings
   for removed items are expected and correct)
2. Schema files load cleanly: `python knowledge-hub-loader.py` on an
   empty DB produces version 2 with all expected tables
3. `workflow_stages`, `script_check_dependencies`, `system_plans` — no
   longer exist in the schema
4. `workflow_use_cases`, `workflow_phases`,
   `workflow_phase_dependencies` — exist and accept data
5. `plan_generation_inputs` — COALESCE virtual columns handle the
   NULL/NULL case correctly (upsert on re-run)
6. `store_system_plan` → `get_system_plan` round-trips correctly
7. `store_plan_generation_input` → `get_plan_generation_input`
   round-trips correctly
8. Capability dispatch with `phase_id` gates against normalized tables
