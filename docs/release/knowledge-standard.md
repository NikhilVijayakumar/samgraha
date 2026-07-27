# Knowledge Standard

What a knowledge standard is, how it's structured on disk, and how it
moves from a standard author's source tree into a repo's live
`knowledge.db`. Companion doc: `repository-registration.md` covers the
repo-level registration/sync mechanics this document assumes; `build.md`
covers producing the `mcp`/`cli` binaries themselves.

## 1. What a standard is

A knowledge standard is a self-contained package of **workflow
declarations** (usecases, steps, scripts, prompts) plus **catalog
metadata** (custom tables it needs, templates it ships) that samgraha
stores and executes without ever interpreting what any of it *means*.
Samgraha moves envelopes between a standard's own scripts/prompts and
whichever model or operator is driving the MCP client — it never reads a
script's logic, never grades a prompt's output, never judges whether a
custom table's shape makes sense for its stated purpose.

Everything a standard declares lives in `knowledge.db`'s workflow tables,
scoped by a `standard` text column present on every one of them. Nothing
about *how a standard behaves* — how many phases a plan should have, what
"done" means for a usecase, how a script should format its output — is
samgraha's decision. Samgraha enforces *structure* (does every domain
have a usecase, does every table it created get catalogued) and
*shape* (does a declared JSON object match the schema it must match),
never content.

## 2. Structure — what a standard provides, on disk

A standard's source directory (wherever the operator points
`register_standard_globally` at — conventionally inside a separate
standards repo, e.g. `Kriti/samgraha/system/<name>/`) contains:

| File | Required | Purpose |
|---|---|---|
| `standard.yaml` (or `script/schema/standard.yaml`) | Yes | Identity: `name`, `category`, `subcategory`, `extends`, `version`, `description`, `smoke_test`, `seeder_script`. Resolved by `resolve_manifest_path` (`register_standard.rs:399-412`), which tries both locations — the nested form exists for standards like `pcems_2026` whose manifest lives one level below root. |
| A seeder script | If the standard has any workflow to declare | The script named by `seeder_script:` — see §4. |
| `standard.metadata.json` | Only if the standard has custom tables, templates, or generates proposals | Validated against `metadata/standard.metadata.schema.json` — see §5. |
| Whatever scripts/prompts the seeder needs | As needed | Copied along with everything else; the seeder is responsible for pointing at them with paths relative to its own directory. |

`standard.yaml`'s shape has changed across this project's history — the
current, live shape is minimal (identity + `seeder_script`), because
workflow declarations (`usecases`/`scripts`/`prompts`/`domains`) moved
from being **parsed by samgraha out of YAML** to being **inserted
directly by the seeder script's own SQL** (§4). An older, still-present
code path (`register_standard()`, `register_standard.rs:144-360`, used
only via `cli/commands.rs`) still parses the old, larger YAML shape with
inline `scripts:`/`prompts:`/`usecases:`/`custom_tables:`/`domains:`/
`assets:`/`templates:` lists — that path is not what
`register_standard_globally`/`activate_standard` (the MCP tools) use, and
is not the shape to author a new standard against.

## 3. The two-stage lifecycle

A standard's life has two distinct stages, and confusing them is the
most common way to misunderstand this system:

1. **Global registration** (§4) — happens once per standard version, at
   the samgraha-installation level (`mcp_dir()`, e.g. `~/.samgraha/`).
   Copies the standard's files into samgraha's own registry, verifies it
   structurally, records it in `standards.db`'s `standard_registry`. A
   repo is not involved yet.
2. **Per-repo activation** (§6) — happens once per repo that wants to use
   the standard, and again every time that repo wants to re-seed (e.g.
   after the global copy updated). Copies the standard's files from
   samgraha's registry into *this repo's* `.samgraha/<standard-name>/`,
   runs the seeder against *this repo's* `knowledge.db`, and records the
   activation in *this repo's* `registry.db`.

One standard is active per repo `.samgraha/` at a time (§7).

## 4. Global registration — `register_standard_globally`

MCP tool `register_standard_globally`, handled by
`adapter.rs:handle_register_standard_globally`. Given a `path` to the
standard's source directory:

1. **Copy.** `common::fs_sync::copy_dir_atomic` copies `path` into
   `mcp_dir()/registry/<category>/<name>/` — temp-dir-then-rename, so a
   copy failure never corrupts a previous version, and a re-registration
   replaces the directory wholesale rather than merging (stale files from
   an old version can't survive). `<category>` mirrors the real standards
   corpus's own on-disk layout (`Kriti/samgraha/system/<category>/<name>/`)
   — verified directly: no `standard.yaml` in that corpus declares a
   `category:` field at all, so `category` is read from `standard.yaml`
   only as an override; the actual value used is inferred from `path`'s
   own parent directory name when the manifest doesn't declare one, which
   is every real standard on disk today. `subcategory` is **not** a
   directory level anywhere in this — it stays DB-only metadata (§7),
   because the real corpus doesn't have a subcategory folder either
   (`dev/react_dev` vs `dev/fastapi_dev` are distinguished by standard
   name, not a frontend/backend subfolder). Re-registering a name under a
   *different* category than it was previously registered under moves the
   directory — the old category's copy is removed once the new one
   succeeds, never left orphaned.
2. **Structural verify-gate (`smoke_test`).** If `standard.yaml` declares
   `smoke_test:`, that script is run against the copy (not the original
   `path` — proving the copy itself is what will actually be used) and
   its exit code recorded as `verify_status`: `passed`/`failed`/
   `unverified` (no `smoke_test` declared). Samgraha has no idea what the
   smoke test checks — only whether it exited 0.
3. **Metadata contract validation.** If `standard.metadata.json` exists in
   the copy, it's validated against the embedded
   `standard.metadata.schema.json` (§5). This is the one gate samgraha
   *can* meaningfully check without interpreting content — shape, not
   meaning.
4. **Upsert.** `standards.db`'s `standard_registry` row is
   inserted/updated: `name`, `category`, `subcategory`, `extends`,
   `version`, `description`, `verify_status`, plus a catch-all
   `metadata_json` of any manifest keys samgraha doesn't otherwise model.
   `source_path` is set to the **local copy** (`mcp_dir()/registry/<name>/`),
   never the original `path` — from this point on, the standard is
   self-contained under samgraha's own installation, independent of
   whether the original source repo still exists at that path.
5. **Log.** `operation_log` gets a row — `register_globally` for a first
   registration, `update_standard` if this name already existed.

**Any failure at step 2 or 3 removes the directory step 1 copied** before
returning — a failed registration leaves exactly the state it found,
never a half-registered directory with no matching `standard_registry`
row.

## 5. The metadata contract — `standard.metadata.json`

Optional file, validated against `metadata/standard.metadata.schema.json`
(embedded into the binary at compile time — `crates/services/src/metadata_validate.rs`,
`include_str!`, no runtime file lookup). Declares three things:

```json
{
  "custom_tables": [
    { "name": "hackathon_scores", "purpose": "Per-team leaderboard rows",
      "required_columns": ["team_id", "score"] }
  ],
  "templates": [
    { "name": "phase-plan", "purpose": "Renders the phase-wise proposal", "role": "proposal" },
    { "name": "leaderboard-report", "purpose": "Narrative summary", "role": "report" }
  ],
  "proposal_template": "phase-plan"
}
```

- **`custom_tables[]`** — every SQLite table the standard's seeder
  creates for its own use, beyond samgraha's reserved workflow tables.
  `required_columns` (optional) is enforced against the real table's
  `PRAGMA table_info` output at activation time (§6, §8) — not just
  declared, checked.
- **`templates[]`** — catalog entries with a `role`: `proposal`, `report`,
  or `other`. At most one template may declare `role: "proposal"`, and
  `proposal_template` (if set) must name exactly that one — checked by
  `metadata_validate::validate_proposal_template_consistency` in both
  directions (declared with no `proposal_template` pointing at it, or
  `proposal_template` pointing at nothing with that role — both rejected).
- **`proposal_template`** — absent means this standard never generates
  phase-wise proposals; present means every `result.proposal` envelope
  its scripts return gets validated (§9).

## 6. Per-repo activation — `activate_standard`

MCP tool `register_standard` (yes — same tool name the pre-seeder era
used; the per-repo semantics changed underneath it), handled by
`adapter.rs:handle_register_standard` → `register_standard.rs:activate_standard`.
Given a `standard_name` already globally registered (§4) and a target
repo:

1. **Look up the global row.** `standards_db.get_standard(name)` — fails
   loudly if this standard was never globally registered; there is no
   implicit "register it globally for me" fallback.
2. **Copy.** `copy_dir_atomic` copies the global registry's local copy
   into `<samgraha_dir>/<standard_name>/` inside the target repo. Same
   atomic-replace reasoning as §4 step 1 — a re-activation after the
   standard updated globally gets a clean copy, not a merge.
3. **Delete this standard's existing rows first.** `delete_existing`
   clears every `RESERVED_TABLE_NAMES` row scoped to this standard
   (`usecase`/`step`/`step_script`/`step_prompt`/`script`/`prompt`/
   `domain`/`asset_kind`/`standard_asset`/`template_type`/`template`/
   `custom_data_tables`) — **before** the seeder runs, so the seeder can
   never observe its own prior run's rows and echo an already-absolutized
   path back as if it were fresh input. `artifact`/`artifact_type` are
   deliberately **not** cleared here — see §10.
4. **Run the seeder.** The script named by `seeder_script:` in
   `standard.yaml`, invoked via `common::env::run_capability_script`'s
   exact existing signature — `(script_path, repo_root, input_json_path,
   out_dir, timeout_secs)`, same interpreter dispatch every other script
   goes through. The seeder receives an `--in` envelope with two
   underscore-prefixed keys it can't derive itself:
   ```json
   { "_samgraha_dir": "/abs/path/to/.samgraha", "_knowledge_db": "/abs/path/to/.samgraha/knowledge.db" }
   ```
   The seeder is the *only* thing that inserts `usecase`/`domain`/
   `script`/`prompt`/`step`/`step_script`/`step_prompt` rows for its own
   standard — samgraha's Rust code never parses a manifest into these
   rows on this path. It reports `{"status": "ok"}` (or anything else,
   which fails the whole activation).
5. **Absolutize.** Every `script.location`/`standard_asset.location` the
   seeder wrote as a relative path gets prefixed with
   `<samgraha_dir>/<standard>/` — the only two tables with a `location`
   column a seeder can write a relative path into (`template`/`prompt`
   store `content` inline, never a path, so there's nothing of theirs to
   absolutize). A `..` segment in a seeder-written relative path is a
   **rejected contract violation**, never silently resolved — the seeder
   must use an absolute path for anything outside its own copied tree.
   The resulting absolute path is also checked to actually **exist on
   disk** — a seeder that declares a `location` for a file it never
   wrote (typo, forgotten file) is rejected here, at activation time,
   not later as a confusing "file not found" the first time the step
   that uses it actually runs.
6. **Custom-table handling.** If `standard.metadata.json` declares
   `custom_tables`, each one is checked against `RESERVED_TABLE_NAMES`
   (collision = reject), catalogued into `custom_data_tables` with
   `shape_json` populated from a live `PRAGMA table_info` introspection
   of the table the seeder actually created, and — if `required_columns`
   is declared — every named column is confirmed to actually exist.
7. **Layer A structural completeness audit.** The mandatory final step —
   see §8. Any failure at steps 4-7 re-runs step 3's cleanup and removes
   the copied directory before returning the error; a failed activation
   leaves exactly the state it found.
8. **Record activation** (done by the MCP handler, not `activate_standard`
   itself). One row written into the **target repo's own** `registry.db`,
   table `active_standard` — see §7.

## 7. One standard, one repo, at a time — `registry.db`'s `active_standard`

There is no per-repo `standard` table in `knowledge.db` — earlier
revisions of this design had one; it was removed (`CORE_SCHEMA_EPOCH`
1→2) because it duplicated what `standards.db`'s global
`standard_registry` already tracks, at repo scope, for no reason once
"one standard active per repo at a time, switch by deleting `.samgraha/`
and re-registering" became the operating model.

Instead, `registry.db` (the same per-repo SQLite file that also backs
repository registration, `repository-registration.md`) has a singleton
table:

```sql
CREATE TABLE active_standard (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL, category TEXT NOT NULL DEFAULT '',
    subcategory TEXT, extends TEXT, version TEXT NOT NULL DEFAULT '0.0.0',
    metadata_json TEXT NOT NULL DEFAULT '{}', activated_at TEXT NOT NULL
);
```

One row (`id = 1`, same `CHECK` pattern `knowledge.db`'s own
`_core_schema_epoch` uses), written by the MCP handler straight from the
global `standard_registry` row it already fetched in §6 step 1 — no
manifest re-parse for `category`/`version`/etc. Overwritten, not
accumulated, on the next activation of a *different* standard.

## 8. The Layer A structural completeness audit

Pure SQL against the rows a seeder just wrote — no declared file needed,
because every check is implied by rows that should exist. Runs as
`activate_standard`'s mandatory last step
(`crates/services/src/layer_a_audit.rs`):

1. Every `domain` row is referenced by ≥1 `usecase`.
2. Every `usecase` has ≥1 `step`.
3. Every deterministic `step` has exactly one `step_script` row.
4. Every semantic `step` has exactly one `step_prompt` row.
5. Every `script` row is referenced by ≥1 `step_script` (catches
   declared-but-unused scripts).
6. Every `prompt` row is referenced by ≥1 `step_prompt` (same, prompts).
7. Every `custom_data_tables.table_name` actually exists in
   `sqlite_master`.
8. **Bidirectional**: every non-reserved table that exists in
   `sqlite_master` — regardless of row count, including empty tables —
   and has no matching `custom_data_tables` row for this standard is
   flagged. Catches a seeder that creates a table without declaring it.

Any failure fails the whole activation (§6 step 7's cleanup).

## 9. Using a standard — deterministic vs. semantic steps

Once activated, a usecase's steps run through three MCP tools:

- **`run_script_step`** (deterministic steps) — runs the mapped script
  via `run_capability_script`, records an `execution` row (with git
  provenance, §11), and reads the script's JSON envelope for
  `proposal`/`artifacts[]` (§10 for the proposal case).
- **`prepare_semantic_step`** — stages a semantic step's mapped prompt
  content for the calling agent. Records nothing yet; samgraha reads the
  prompt for bytes only, never for meaning.
- **`complete_semantic_step`** — records that the agent's reasoning
  finished (with a status). Persisting the agent's actual output is the
  job of the *next* deterministic step in the usecase's sequence, run
  normally through `run_script_step` with the agent's result as input —
  samgraha never persists a semantic result itself. There is no separate
  "pre/semantic/post" concept; a step is the atomic unit, and a
  pre → semantic → post triad is simply three consecutive steps.

`check_usecase_complete` is a fourth tool, but a query, not a gate — it
runs a usecase's declared `verify_script` and reports the exit code;
nothing blocks on the result. `smoke_test` (§4) and this are the only two
places a standard can inject its own arbitrary validation logic; samgraha
never substitutes for either.

## 10. Proposal generation — opt-in, always phase-per-domain when used

If a standard declares `proposal_template` (§5), every `result.proposal`
envelope a script returns is validated against
`metadata/proposal.schema.json` (also compile-time embedded) **and**
cross-checked against real rows before being inserted:

```json
{
  "title": "Grading pipeline plan",
  "phases": [
    { "domain": "grading", "phase_number": 1, "usecases": ["ingest"],
      "steps": [12], "rationale": "linear pipeline, one phase",
      "git": { "commit_sha": "...", "branch": "main", "dirty": false } }
  ]
}
```

`phases` is required and non-empty — this is what makes "a proposal is
always a phase-wise plan" an enforced contract, not a convention a
standard can silently skip. One domain may span multiple phase entries;
how many phases a domain needs for its complexity is entirely the
standard's own judgment — samgraha never scores it.

Cross-checks, all against the repo's live `knowledge.db`/git state, not
just schema shape:
- every `domain` named exists for this standard
- every `usecases` entry exists and belongs to the phase's declared domain
- every `steps` id exists and belongs to one of that phase's usecases
- if a `git` block is present, its `commit_sha` must exist in the repo
  **and** be an ancestor of `HEAD` (`git cat-file -t` +
  `git merge-base --is-ancestor`)

A validation failure means no `proposal` row is inserted — the
`execution`/`git_detail` rows for the run that produced it still stand
(they document what happened), but the proposal itself is rejected
outright, not stored with a bad reference.

If a standard doesn't declare `proposal_template`, none of this runs —
`result.proposal` (if present) is stored as-is, same as it always was.

## 11. Git provenance

Every `execution` row (deterministic or semantic-completion) captures
`commit_sha`/`branch`/`dirty` via `capture_git_state` (best-effort — a
non-git repo or missing `git` binary just means no `git_detail` row, not
a failure) and stores it in `git_detail`, linked via
`execution.git_detail_id`. This is provenance, recorded for the record —
it never blocks or gates a run on its own; the only place git state is
*checked against a claim* is §10's proposal cross-check.

## 12. `samgraha.toml` fields relevant to standards

See `docs/proposal/samgraha-toml-configuration-contract-proposal.md` for
the full contract; the fields that matter here:

- **`[knowledge] standards`** — plain policy list (§2.2 of that
  proposal), names matched against `standards.db`'s global
  `standard_registry`. Declares intent; doesn't itself trigger
  activation — that's still an explicit `register_standard` MCP call.
- **`[repository] samgraha_dir`** — where `.samgraha/<standard-name>/`
  lands. `${SAMGRAHA_DIR}` pattern, falls back to `<repo-root>/.samgraha`.

## 13. Staleness and updates

Re-running `register_standard_globally` for an already-registered name
deletes and re-copies `mcp_dir()/registry/<category>/<name>/` wholesale
(never merged), re-runs the verify-gate, and logs `update_standard`
instead of `register_globally` in `operation_log` — so "was this fresh
or a re-registration" is answerable from the log, not inferred.

Samgraha does **not** auto-propagate a global update into every
consuming repo — a repo's own `register_standard` call has to be run
again, deliberately. "Is this repo stale?" is answered live, by
comparison, never a stored flag that could drift:

```rust
let stale = active_standard.version != global_registry_row.version;
```

(`active_standard` in the repo's own `registry.db`, §7; `standard_registry`
the global one in `standards.db`.) This needs the global row reachable —
a repo working fully offline reports "unknown," not "stale" or "current."

## 14. Standard Author Checklist

What to have in place before running `register_standard_globally`, in
the order a seeder actually has to satisfy it (parents before children —
every FK below is `NOT NULL`, so getting the order wrong fails loud, not
silently):

1. **`standard.yaml`** has `name:` (the only truly required field) and,
   if this standard has any workflow at all, `seeder_script:` naming a
   real file in the same tree. `category:`/`subcategory:` are optional —
   if omitted, `category` is inferred from the source directory's own
   parent folder name (§4), matching how the real standards corpus
   already works; don't rely on this if you need a specific category
   distinct from where the directory happens to sit.
2. **Every script/prompt file the seeder will reference by path** exists
   in the standard's own tree *before* registration — `resolve_location`
   canonicalizes and requires real existence; a path that doesn't resolve
   fails the seeder run, not silently.
3. **The seeder inserts rows in dependency order**: `domain` before any
   `usecase` that references it; `usecase` before its `step`s; `script`/
   `prompt` before the `step_script`/`step_prompt` rows that reference
   them; if this standard uses custom tables or ships templates,
   `asset_kind`/`template_type` before the `standard_asset`/`template`
   rows that reference them (same declare-then-reference discipline as
   `domain`, §5) — every one of these is a `NOT NULL REFERENCES`, so
   inserting a child before its parent is a hard SQLite error, not a
   silent gap.
4. **Every `script.location`/`standard_asset.location` the seeder writes
   is either absolute, or relative-with-no-`..`-and-actually-exists on
   disk** (§6 step 5) — checked at activation time now, not just at
   whatever later moment a step tries to run the file.
5. **`standard.metadata.json` (if this standard has custom tables,
   templates, or generates proposals) declares every custom table the
   seeder creates**, with `required_columns` for any column another part
   of the standard depends on existing — the bidirectional Layer A check
   (§8, point 8) rejects a table the seeder created that metadata never
   declared, and vice versa.
6. **Every `domain` has ≥1 `usecase`, every `usecase` has ≥1 `step`,
   every deterministic `step` maps to exactly one `script`, every
   semantic `step` maps to exactly one `prompt`, every `script`/`prompt`
   row is referenced by at least one step** — Layer A (§8) checks all of
   this after the seeder runs; a standard that fails any of it never
   completes activation, so it's worth checking locally before
   registering rather than discovering it through a rejected activation.
7. **If this standard generates proposals**, exactly one `templates[]`
   entry in `standard.metadata.json` declares `role: "proposal"` and
   `proposal_template` names it (§5) — checked in both directions, a
   mismatch either way is rejected. Every `result.proposal` envelope a
   script returns must then be a non-empty `phases[]` array, each phase
   naming a real `domain`/`usecases`/`steps` that actually belong to each
   other (§10) — validate this locally against
   `metadata/proposal.schema.json` before assuming a generation step
   will succeed against a real repo.

**Known asymmetry, not yet closed**: `standard.metadata.json`'s
`custom_tables[]` gets full round-trip checking (declared, created,
required columns present, and — bidirectionally — nothing undeclared).
`templates[]` only gets checked for the one `proposal_template` role
relationship (§5, §7 above); there's no equivalent "every `template` row
the seeder inserts was declared, and vice versa" check the way there is
for custom tables, and `asset_kind`/`standard_asset` have no declarative
section in `standard.metadata.json` at all — the seeder can create as
many as it wants, freely, with only the `NOT NULL REFERENCES` FK
enforcing that a `standard_asset` row's `kind_id` points at *some*
`asset_kind` row, never that the *set* of kinds matches anything
declared. Closing this symmetrically is a real, scoped follow-on (not
attempted here) — it needs its own small design decision about what an
`asset_kind` declaration in `standard.metadata.json` should even look
like, not a mechanical copy of the `custom_tables[]` pattern.
