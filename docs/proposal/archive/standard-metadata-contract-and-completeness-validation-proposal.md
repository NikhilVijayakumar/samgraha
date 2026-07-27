# Standard Metadata Contract & Completeness Validation Proposal

**Status**: DRAFT — not yet implemented.

**Amends**: `docs/proposal/clean-slate-schema-reset-proposal.md` §3.8 (the
seeder contract — gains a metadata-declaration step before the seeder
runs), §3.9 (`activate_standard` — gains a completeness audit as its final
step), §3.12 (the `artifact`/proposal envelope-reading code in
`run_script_step` — gains schema validation before insert). Nothing in
the existing 17-table `knowledge.db` schema changes (§3.6 below is the
analysis for why not) — this proposal adds a validation layer on top of
it, not new workflow tables.

---

## 1. Verified Current State

### 1.1 `custom_data_tables` population and its reserved-name collision
check only exist on the *old* manifest-parsing path — the new seeder path
has neither

`register_standard()` (`register_standard.rs:144-356`, the pre-seeder
manifest parser, still reachable via `cli/commands.rs:77`) does two
things for a standard's custom tables: rejects any `custom_tables` entry
whose name collides with `RESERVED_TABLE_NAMES` (`register_standard.rs:159-165`,
*before* any `INSERT` — same fail-fast discipline as everything else in
this codebase) and inserts a `custom_data_tables` row per declared table
(`register_standard.rs:336-343`). **`activate_standard`** (`register_standard.rs:431-511`,
the seeder-driven per-repo path every standard registers through now)
does neither. It reads exactly one field from the local manifest copy —
`seeder_script` — and never touches `custom_tables` at all. A standard
registered through the live path today can name a custom table that
collides with a reserved name, and nothing catches it; `custom_data_tables`
stays empty for it regardless of what the seeder actually created.
Verified by grepping `register_standard.rs` for `custom_tables`/
`custom_data_tables` — every hit is inside `register_standard()`, none
inside `activate_standard`.

### 1.2 `custom_data_tables.shape_json` exists and is never read or written

`core_schema.rs:183` — `shape_json TEXT` (nullable, no default). No
`INSERT` anywhere in the codebase sets it (`register_standard.rs:339-343`'s
`INSERT` lists four columns, not five) and no `SELECT` anywhere reads it.
A column that was clearly meant to describe a custom table's expected
shape, entirely unused.

### 1.3 `proposal` is a fully passive table — samgraha validates nothing
about what a script reports as a proposal

`step_execution.rs:142-149`: if `result.proposal` is present in a
script's JSON envelope, samgraha copies `title`/`location` verbatim into
a `proposal` row. No shape check, no reference check (a `usecase_id` is
supplied from the step's own context, not from anything the script
claims), no relationship to `domain`, no phase concept anywhere in the
schema or the code that reads this envelope.

### 1.4 No structural completeness audit exists anywhere in the codebase

Confirmed by search — no function anywhere checks any of: every `domain`
row has at least one `usecase` referencing it, every `usecase` has at
least one `step`, every deterministic `step` has a `step_script` row,
every semantic `step` has a `step_prompt` row, every `script`/`prompt`
row is referenced by at least one step (vs. registered-but-unused), or
every `custom_data_tables` row's `table_name` corresponds to a table that
actually exists in the same `knowledge.db` (a `sqlite_master` check would
answer this; nothing runs it). `FOREIGN KEY` constraints catch a
*dangling reference* at insert time — they do not catch a *missing*
reference, which is the shape of every check just listed.

### 1.5 Every existing "verification" mechanism is either opaque or
explicitly non-blocking

`smoke_test` (`adapter.rs:224-237`) is an exit-code check against a
binary/script the standard itself authors — samgraha runs it and records
`passed`/`failed`, with zero visibility into what it actually checked.
`check_usecase_complete`'s own tool description states outright: *"a
query, not a gate — nothing blocks on its result"* (`main.rs:407`).
`capture_git_state` (`step_execution.rs:26-51`) is provenance capture —
commit SHA, branch, dirty flag, recorded for the record — never compared
against anything, never blocks a run.

---

## 2. What this proposal adds, and what it deliberately does not

**Adds**: a declared, versioned, machine-checkable contract — JSON Schema
files under `metadata/` at the samgraha repo root — that a standard's own
declarations (which custom tables it needs and why, which templates it
ships and what role each plays, what shape its generated proposals must
have) get validated against, plus a structural completeness audit that
runs against the *actual seeded rows* after a seeder finishes, plus
schema + cross-reference validation of a proposal's phase-per-domain
content before it's ever inserted.

**Does not add**: any new samgraha-owned table for phase data (§3.6), any
interpretation of what a standard's tables/prompts/scripts *mean*
(samgraha validates *shape* and *referential completeness*, never
content correctness — same boundary every prior proposal in this line has
held), and no change to the file-copy / seeder-invocation mechanics
`clean-slate-schema-reset-proposal.md` already specifies.

---

## 3. Design

### 3.1 Two validation layers — they check different things and run at
different times

**Layer A — structural completeness audit.** Pure SQL against
`knowledge.db`, no new declared file needed, because everything it checks
is already implied by rows that exist (or should exist) after seeding:

- every `domain` row (`WHERE standard = ?`) is referenced by ≥1 `usecase.domain_id`
- every `usecase` has ≥1 `step`
- every `step` with `kind = 'deterministic'` has exactly one `step_script` row
- every `step` with `kind = 'semantic'` has exactly one `step_prompt` row
- every `script` row is referenced by ≥1 `step_script` (catches declared-but-unused)
- every `prompt` row is referenced by ≥1 `step_prompt` (same, for prompts)
- every `custom_data_tables.table_name` exists in `sqlite_master` for this
  same connection (catches declared-but-never-created — closes §1.1's gap)

**Layer B — declared-metadata schema validation.** Checks things Layer A
structurally cannot, because they're not implied by row existence alone —
a custom table's *purpose* and *required shape* (which columns it must
have, not just that the table exists), a template's *role* (is this the
one that renders proposals, or just a report template), and the shape any
generated proposal's phase-per-domain content must take. This needs a
standard to declare it somewhere samgraha can check against — §3.2-3.3.

### 3.2 `metadata/` — samgraha's own JSON Schema definitions, not
per-repo runtime data

New top-level directory, `E:\Python\samgraha\metadata\` (samgraha's own
repo, versioned with its source — same tier as `schema/knowledge/`,
`schema/registration/`, `schema/standards/`, just JSON Schema instead of
SQL DDL, because what it's describing is a document shape, not a table
shape):

```
metadata/
  standard.metadata.schema.json   — what a standard's own metadata.json must look like
  proposal.schema.json            — what a generated proposal's phase-per-domain content must look like
  README.md                       — what validates against what, and when
```

### 3.3 A standard gains one more file: `standard.metadata.json`

Alongside `standard.yaml` and its seeder script, copied into
`mcp_dir()/registry/<name>/` the same way everything else is
(`copy_dir_atomic`, unchanged mechanism). Validated against
`metadata/standard.metadata.schema.json` at `register_standard_globally`
time — same verify-gate step `smoke_test` already occupies
(`adapter.rs:224-237`), same failure semantics (reject, clean up the
just-copied directory, log to `operation_log`).

Shape (illustrative — final field list is implementation-time detail, not
load-bearing here):

```json
{
  "custom_tables": [
    { "name": "hackathon_scores", "purpose": "Per-team leaderboard rows",
      "required_columns": ["team_id", "score", "recorded_at"] }
  ],
  "templates": [
    { "name": "leaderboard-report", "purpose": "Narrative summary", "role": "report" },
    { "name": "phase-plan", "purpose": "Renders the phase-wise proposal", "role": "proposal" }
  ],
  "proposal_template": "phase-plan"
}
```

Exactly one template may declare `"role": "proposal"`, and
`proposal_template` must name it — this is what makes "a proposal is
always a phase-wise plan" an enforced contract rather than a convention
someone can silently skip: if a standard's seeder produces a `result.proposal`
envelope at all, that role assignment is what tells samgraha which
schema (§3.5) to hold it to.

### 3.4 `activate_standard` gains three things

1. **Restores custom-table handling for the seeder path** (closes §1.1):
   after the seeder runs (§3.8/§3.9 of the base proposal, unchanged),
   read `standard.metadata.json`'s `custom_tables` list, run the same
   `RESERVED_TABLE_NAMES` collision check `register_standard()` already
   has, and insert one `custom_data_tables` row per declared table
   (`shape_json` finally populated — §1.2's dead column gets a writer).
2. **Metadata schema validation** (Layer B) — already done once at global
   registration (§3.3); re-checked here only if the metadata file
   changed independent of a global re-registration is out of scope (a
   standard's metadata travels with its copied tree, same as everything
   else — no separate versioning axis).
3. **Structural completeness audit** (Layer A) as the new final step —
   runs after the absolutize pass, before `activate_standard` returns
   success. **Mandatory, not opt-in**: a standard that fails the audit
   fails the whole registration, same cleanup-on-failure discipline
   `clean-slate-schema-reset-proposal.md` §3.9 already established for
   every other step in this function (delete the rows this run inserted,
   remove the copied directory, return the error).

### 3.5 The proposal envelope contract — validated and cross-checked, not
just recorded

When `run_script_step` sees `result.proposal` (`step_execution.rs:142-149`)
and the step's usecase belongs to a standard whose `standard.metadata.json`
declares a `proposal_template`, the envelope is held to
`metadata/proposal.schema.json` before anything is inserted:

```json
{
  "title": "...",
  "location": "...",
  "phases": [
    {
      "domain": "grading",
      "phase_number": 1,
      "usecases": ["ingest-submissions"],
      "steps": [12, 13],
      "rationale": "Single phase — ingest is one linear pipeline, no parallelizable sub-tasks",
      "git": { "commit_sha": "...", "branch": "...", "dirty": false }
    },
    {
      "domain": "grading",
      "phase_number": 2,
      "usecases": ["score-submissions", "flag-outliers"],
      "steps": [14, 15, 16],
      "rationale": "Split into a second phase — scoring and outlier-flagging can run independently once ingest is done"
    }
  ]
}
```

`phases` is required and non-empty (this is the enforcement mechanism for
"always a phase-wise plan," not a convention). One domain may appear in
multiple phase entries (`grading` above spans two) — the schema doesn't
cap phases-per-domain, matching "one or multiple phases based on task
complexity" directly: complexity is the *standard's own* judgment,
samgraha only requires the judgment be expressed in this shape, never
scores or second-guesses it.

Beyond schema shape, samgraha cross-checks references before insert —
this is where "fetch corresponding usecase/step data and verify" from the
original ask is actually satisfied:

- every `domain` named exists in `domain` for this standard
- every `usecases` entry exists and its `domain_id` resolves to the
  `domain` named in the same phase entry (catches a phase claiming a
  usecase for a domain it doesn't actually belong to)
- every `steps` id exists and belongs to one of that phase's `usecases`
- if a `git` block is present, it must match the `git_detail` row this
  same `run_script_step` call just captured (`git_state`, already in
  scope at `step_execution.rs:106`) — this is "considering git commit and
  current state": the proposal is rejected if it claims a commit that
  isn't the one samgraha actually observed for this run, closing the gap
  where a script could report a stale or fabricated git state.

A validation failure here means the whole `run_script_step` call fails —
`execution`/`git_detail` rows already recorded for this run stay
(they document what happened), but no `proposal` row is inserted for
content that doesn't check out.

### 3.6 No new `phase` table — analysis

Considered and rejected, for three reasons converging on the same answer:

1. **Consistency with `proposal`'s own existing design.** `proposal`
   already deliberately keeps its actual content *out* of the database —
   `location` points at a file, `template.content` is the only place this
   schema stores generated text inline, and `proposal`'s own header
   comment (`schema/knowledge/15-proposal.sql:3-4`) says outright
   *"samgraha never interprets proposal content — it only records the
   lifecycle."* A `phase` table would mean samgraha starts storing
   structured proposal content as rows — a reversal of an already-stated
   design stance, not an extension of it.
2. **Nothing needs to query a phase in isolation.** The original ask was
   for *verification* ("check to verify all these"), not for runtime
   phase-tracking (no "list phases," "mark phase 2 done," or
   "which usecases belong to phase 3" capability was requested). A
   validator that parses the envelope JSON, extracts phase→domain→usecase/
   step references, and cross-queries the existing tables (§3.5) answers
   every verification requirement without a new table to keep in sync.
3. **A table would force a schema decision the content doesn't need
   settled yet.** `proposal.usecase_id` is `NOT NULL` — one proposal, one
   anchoring usecase (the one whose step triggered generation, per
   `step_execution.rs:134-140`'s existing lookup). A phase-plan spanning
   multiple domains/usecases doesn't fit that FK without either loosening
   it (weakens an existing constraint for every *other* proposal that
   doesn't need multi-domain phases) or adding a join table on top of a
   new `phase` table (two new tables, another `CORE_SCHEMA_EPOCH` bump,
   another `RESERVED_TABLE_NAMES` entry, another `delete_existing` case)
   — for a need the content-shape approach already meets.

If a future need genuinely requires querying phases as first-class rows
(cross-standard phase reporting, a dashboard, something that reads phases
*without* first locating and parsing a proposal's file) — that's new
scope, evaluated against a real requirement then, not speculated into
this proposal now.

### 3.7 Validator implementation — Rust, integrated, one new dependency

Per direction: validation logic lives inside the MCP flow itself, not a
standalone script. JSON Schema is a real specification (types, `required`,
`enum`, `pattern`, `oneOf`, `$ref` — hand-rolling a subset would be the
wrong kind of lazy, the kind that silently under-validates); pull in an
existing, maintained `jsonschema` crate rather than writing one. One new
workspace dependency, same tier as `serde_yaml`/`tera` already are.

```rust
// crates/services/src/metadata_validate.rs
pub fn validate_against_schema(instance: &serde_json::Value, schema_path: &Path) -> Result<()> {
    let schema_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(schema_path)?)?;
    let compiled = jsonschema::JSONSchema::compile(&schema_json)
        .map_err(|e| anyhow::anyhow!("invalid schema at {}: {e}", schema_path.display()))?;
    let errors: Vec<String> = compiled.validate(instance)
        .map(|_| Vec::new())
        .unwrap_or_else(|errs| errs.map(|e| e.to_string()).collect());
    if !errors.is_empty() {
        bail!("metadata validation failed against {}: {}", schema_path.display(), errors.join("; "));
    }
    Ok(())
}
```

Hook points (all mandatory gates, per direction — none of these are
"query, not a gate" like `check_usecase_complete`):

1. `register_standard_globally` (`adapter.rs:191-309`) — validates
   `standard.metadata.json` against `metadata/standard.metadata.schema.json`,
   inserted right after the `smoke_test` step, same failure/cleanup path.
2. `activate_standard` (`register_standard.rs:431-511`) — restores
   custom-table handling (§3.4.1), runs the Layer A completeness audit
   as its new final step (§3.4.3).
3. `run_script_step`'s proposal-envelope reader (`step_execution.rs:142-149`)
   — validates against `metadata/proposal.schema.json` plus the
   reference/git cross-checks (§3.5), before the `INSERT INTO proposal`.

Also exposed as a standalone MCP tool, `validate_standard_metadata`, for
checking a standard's `standard.metadata.json` *before* attempting
registration — explicitly a query, not a gate (mirrors
`check_usecase_complete`'s framing), useful for a standard author
iterating locally without round-tripping through global registration each
time.

### 3.8 Failure discipline — nothing new, same rule extended

Every gate added by this proposal (§3.4.1's collision check, §3.4.3's
audit, §3.5's envelope validation) fails loud and leaves no partial
state, reusing the cleanup-on-failure pattern
`clean-slate-schema-reset-proposal.md` §3.7/§3.9 already established:
delete whatever this call inserted for the standard, remove any directory
it copied, return the error, log it to `operation_log`. No new failure
philosophy — this proposal only adds more things that can fail loudly
inside a discipline that already exists.

---

## 4. Implementation Plan

### Phase 1 — `metadata/` schemas + `jsonschema` dependency
Write `standard.metadata.schema.json` and `proposal.schema.json`, add the
`jsonschema` crate, `services::metadata_validate::validate_against_schema`.

### Phase 2 — Global registration gains metadata validation
`register_standard_globally` reads and validates `standard.metadata.json`
after `smoke_test`, before upsert (§3.4 point 2, §3.7 point 1).

### Phase 3 — `activate_standard` gains custom-table handling + Layer A audit
Restores §1.1's gap (custom-table collision check + `custom_data_tables`
population, now sourced from `standard.metadata.json` instead of the old
`standard.yaml.custom_tables`) and adds the structural completeness audit
as the function's new final, mandatory step (§3.4 points 1 and 3).

### Phase 4 — Proposal envelope validation
`run_script_step`'s proposal reader gains schema validation and the
domain/usecase/step/git cross-checks (§3.5), gated on the triggering
standard having declared a `proposal_template`.

### Phase 5 — `validate_standard_metadata` MCP tool
Standalone, non-gating check for standard authors (§3.7, last paragraph).

### Dependency graph
```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4
                                  └──────→ Phase 5
```

---

## 5. Testing

- Unit: a `standard.metadata.json` missing `proposal_template` while
  declaring a template with `"role": "proposal"` (or vice versa) fails
  schema validation with a clear message identifying which side is wrong.
- Unit: `activate_standard` against a fixture standard whose seeder
  creates a custom table matching a `RESERVED_TABLE_NAMES` entry — assert
  rejection before any row lands, same cleanup-on-failure as every other
  step in this function.
- Unit: Layer A audit — seed a fixture with a `domain` row that no
  `usecase` references; assert the audit reports it and `activate_standard`
  fails the whole registration.
- Unit: Layer A audit — seed a fixture where `custom_data_tables` catalogs
  a table the seeder never actually `CREATE TABLE`d; assert the audit
  catches the mismatch via `sqlite_master`.
- Unit: proposal envelope validation — a `result.proposal.phases[]` entry
  naming a `usecases` value that exists but belongs to a different
  `domain` than the one declared on that phase entry; assert rejection,
  not a silently-accepted mismatch.
- Unit: proposal envelope's `git` block deliberately set to a commit SHA
  that doesn't match `capture_git_state`'s result for the same call;
  assert rejection.
- Integration: full flow — global register (with valid
  `standard.metadata.json`) → per-repo activate (custom tables + audit
  pass) → `run_script_step` on a usecase whose result includes a
  multi-phase, multi-domain proposal — assert the `proposal` row lands
  and every phase's referenced usecases/steps/domains resolve correctly
  when read back.

---

## 6. Explicitly out of scope

- **No new `phase` database table** — analyzed and rejected in §3.6;
  revisit only against a concrete new requirement, not speculatively here.
- **No scoring or second-guessing of a standard's complexity judgment.**
  Samgraha verifies the phase-per-domain *shape* and every reference in
  it; it never evaluates whether two phases were the *right* number for a
  given domain's actual complexity — that judgment call stays the
  standard's own, exactly as `smoke_test`/`verify_script` content always
  has.
- **No retroactive validation of standards already registered before this
  lands.** A standard's `standard.metadata.json` is validated at
  registration time going forward; this proposal doesn't add a "re-audit
  every already-registered standard" sweep. If that's wanted, it's a
  small follow-on (the same Layer A audit function, run manually or via a
  new tool, against every row in `standard_registry`) — not designed
  here.
- **`jsonschema` crate selection** (exact crates.io package/version) is an
  implementation detail for whoever picks up Phase 1 — any maintained,
  spec-compliant JSON Schema Draft 7+ validator satisfies this design.
