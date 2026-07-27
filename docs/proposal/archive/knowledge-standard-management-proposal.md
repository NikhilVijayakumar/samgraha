# Knowledge Standard Management Proposal — Registry, Category/Subcategory, Verify-Gate, Templates/Artifacts, Proposal Generation, Tracing

**Status**: DRAFT — not yet implemented. Every claim below is verified
against current code (file:line) or current on-disk state
(`Kriti/samgraha/system/`), not assumed.

**Revision note (rev. 3)**: rev. 2 was a design-level document — correct
about *what* to build, silent on the concrete code changes needed to build
it safely inside this crate tree. A third review checked every mechanism
against the actual current code (`McpAdapter`'s real fields, `CORE_MIGRATIONS`'s
real shape, `resolve_location`'s real base-path argument) and found one
genuine correctness bug rev. 2 would have shipped
(`resolve_location`/`register_standard`'s manifest-relative-path resolution
breaks for any manifest not at a standard's root — exactly `pcems_2026`'s
real location) plus a second one this round's own verification surfaced
that the reviewer didn't name: `CORE_MIGRATIONS` has no version gate today
— it's re-run in full on every `register_standard` call, safe only because
every existing statement is `CREATE TABLE/INDEX IF NOT EXISTS`; the moment
this proposal's `ALTER TABLE ... ADD COLUMN` statements join it, the second
call in the same repo crashes on "duplicate column" without a real
`_schema_version` gate. §8 (new) is the concrete implementation-level
companion to §2/§4 — code and DDL, not just shape — covering both of these
plus every other item the third review raised.

**Revision note (rev. 2)**: rev. 1 corrected a factual error (§1.4 —
claimed no standard on disk has a `standard.yaml`) by reading
`pcems_2026`'s manifest in full, and in the process dropped the global
`template`/`proposal` tables entirely on the grounds that `pcems_2026`
already solves both problems itself (`academic_templates`,
`academic_proposals`). That was scope drift in the other direction: this
proposal's job is samgraha/MCP-side generic infrastructure, not a verdict
on how any one knowledge standard should be built. `pcems_2026` stays
read-only evidence throughout this document — nothing here requires
changing anything under `Kriti/`. §2.7/§2.8 are rewritten again: `template`,
`proposal`, and `artifact` are back as **optional, generic, samgraha-owned**
schema — infrastructure a standard may use instead of building its own
(most of the six not-yet-authored standards have nothing yet; `pcems_2026`
is free to keep using its own richer tables and never touch these). §2.15
is new: `samgraha.toml`'s actual requirements for a knowledge-authoring
repo vs. a normal consuming repo, verified against `config.rs` and both
`samgraha.toml`/Kriti's `samgraha.toml` directly.

**Trigger**: `Bodha/docs/errorlist/samgraha/2026-07-27-standard-management-gaps.md`
plus a full read of `Kriti/samgraha/system/pcems_2026/script/schema/standard.yaml`
(143 lines → parsed: 37 scripts, 33 prompts, 22 custom tables, 73
usecases) and its supporting `plan/core/loop.yaml`, `script/verify/`,
`script/smoke_test.py`, and `schema/14-academic_templates.sql` /
`18-academic_report_history.sql` / `22-academic_proposals.sql`.

---

## 0. Executive Summary

`commit ba001b8` replaced samgraha's old domain/audit/generate model
(`system.yaml`-based, archived under `schema/archive/knowledge-hub/` and
`archive/crates/mcp-legacy/`) with a smaller one: a standard declares
**usecases** of ordered **steps**, `deterministic` (samgraha runs a script)
or `semantic` (samgraha stages a prompt, the calling agent reasons). This
engine (`crates/services/src/register_standard.rs`,
`crates/services/src/step_execution.rs`, `schema/knowledge/*.sql`) is
well-built, well-tested, and — corrected from this proposal's first draft —
**already proven against one real standard**: `pcems_2026` ships a fully
authored `standard.yaml` (not at its root — at `script/schema/standard.yaml`,
which is why the first draft's directory listing missed it) that
`register_standard` can parse today, unmodified.

What's actually missing, confirmed by reading that manifest in full rather
than just confirming its existence:

1. **Discovery**: no global registry, no `list_standards`, no category
   tagging. Still true, still the core of this proposal (§2.1-2.3).
2. **Path assumption**: `register_standard(path)` assumes `<path>/standard.yaml`.
   `pcems_2026`'s lives three directories deeper. Needs a resolution rule,
   not a mandate that every standard move its manifest (§2.14).
3. **Extensibility**: `pcems_2026`'s manifest has a top-level `classify_repo:`
   block `StandardManifest` doesn't declare — silently dropped by
   `serde_yaml`, not rejected. Needs a catch-all field (§2.12).
4. **Two different "usecase" shapes, conflated in the first draft**: 7 of
   `pcems_2026`'s 73 usecases (`schema-init`, `classify-repo`, the four
   `propose-*` usecases, `approve-proposal`) have real `steps:` and run
   through the existing dispatcher exactly as designed. The other 66 have
   `steps: []` by design — driven by `plan/core/loop.yaml`'s tier-gate/
   fix-loop logic externally, using the same `scripts:`/`prompts:` catalog
   entries but sequenced by the standard's own orchestrator, not by
   samgraha's step dispatch. The first draft's `bootstrap: true` idea
   assumed every usecase either has steps or doesn't matter; both halves
   need modeling, differently (§2.5, §2.11).
5. **Domains have no home.** `system.yaml`'s `domains:` (with `sort_order`)
   and `loop.yaml`'s per-tier domain lists are the actual runtime-consulted
   structure — "what does this standard score, in what order" — and
   neither the manifest nor any schema table carries them today (§2.11).
6. **`plan/` and `guide/` are real, load-bearing, and invisible.**
   `loop.yaml` (104 lines: tiers, relationships, proposal gates, fix-loop
   thresholds) is the orchestrator's actual brain; `guide/` (12
   subdirectories: writing guides, checklists, reviewer expectations)
   is read during semantic steps. Neither has a table or manifest field.
   The first draft's `template` table didn't cover either, and would have
   been the wrong shape even if it tried (§2.9).
7. **Verify is two unrelated things, and the first draft designed for only
   one, incorrectly.** `script/smoke_test.py` is a structural,
   registration-time validator (schemas load, manifest parses, prompts
   non-empty) — this is what a registration gate should run. `script/verify/`
   (59 generated files, all delegating to `_common.py:verify_main`) is a
   **runtime, per-usecase, DB-state completion check** (`--db-path
   --paper-id`, not the `--repo-root --in --out` script contract at all) —
   this answers "has usecase X finished for paper Y," consumed by the
   orchestrator's tier-gate, never by registration (§2.4).
8. **`template`/`proposal`/`artifact` belong in samgraha as generic,
   optional schema — not a mandate on any one standard.** `pcems_2026`
   already ships richer, working equivalents (`academic_templates`,
   `academic_report_history`, `academic_proposals`) and keeps using them
   unmodified; nothing here asks it to change. The gap this proposal
   actually closes is that **every other standard** (five `dev`-category
   standards not yet authored, `python_hackathon` with no manifest at all,
   any future standard) has nowhere to put this without inventing its own
   `academic_*`-style tables from scratch. samgraha ships one small,
   generic, opt-in schema for exactly that (§2.7, §2.8, §2.9).
9. **`samgraha.toml` has no working way to declare "this repo needs
   standard X"**, and `RepositoryKind::Knowledge` vs. `Repository` — the
   one config distinction that could carry authoring-repo-vs-consuming-repo
   semantics — is declared (`config.rs:289-298`) and read nowhere else in
   the codebase (§2.15).

§1 is the corrected inventory. §2 is the revised design — scoped
throughout to samgraha/MCP-side code and schema only; every `pcems_2026`
reference is evidence for *why* a mechanism is shaped the way it is, never
a requirement to change anything in `Kriti/`. §3 is the migration path
(short — `pcems_2026` needs zero changes to prove the core engine; the
other standards' own authoring is separate, deferred work). §4-§6 are
implementation, testing, and explicit scope cuts. §7 is the point-by-point
disposition of every item the first review raised.

---

## 1. Verified Current State

### 1.1 What ships today (live, working, tested)

| Piece | Location | Note |
|---|---|---|
| Core schema | `schema/knowledge/01..08-*.sql` | `usecase`, `script`, `prompt`, `step`, `step_script`, `step_prompt`, `execution`, `custom_data_tables`. Mirrored in `crates/registry/src/core_schema.rs`'s `CORE_MIGRATIONS`, applied per-repo to `.samgraha/knowledge.db` on open. |
| `register_standard(path, knowledge_db)` | `crates/services/src/register_standard.rs:89` | Parses `<path>/standard.yaml` into `StandardManifest` (line 16-26), writes rows, delete-then-reinsert on re-registration (line 236). 5 unit tests, all passing against fixture manifests — but never yet run against a real standard's manifest (§1.4 fixes this gap in verification, not in code). |
| Step execution | `crates/services/src/step_execution.rs` | `run_script_step` (deterministic), `prepare_semantic_step`/`complete_semantic_step` (semantic, two-call split). |
| MCP tool surface | `crates/mcp/src/adapter.rs:48-56` | 9 tools: `init`, `register_repository`, `unregister_repository`, `list_repositories`, `repository_status`, `register_standard`, `run_script_step`, `prepare_semantic_step`, `complete_semantic_step`. |
| Repo registration | `crates/registry/src/registry_db.rs` | `.samgraha/registry.db`'s `repository_cache` — a cross-repo dependency cache, unrelated to standards. |

### 1.2 What's declared in config but dead

`crates/common/src/config.rs`: `KnowledgeConfig` (261-285, no `standards`
field), `AssetSyncConfig` (467-484, referenced by nothing outside its own
definition/tests), `InitOptions.sync_knowledge_system` (509, set by test
fixtures only, confirmed dead by `init.rs:35-37`'s own doc comment).
Flagged for removal, not touched by this proposal (§6).

### 1.3 What the archived model had that the new one doesn't

- **Abstract-standard soft-warning** (`archive/crates/mcp-legacy/adapter.rs:2108-2116`):
  reads `system.yaml`'s `abstract`, warns, doesn't reject. Gone entirely
  from the live handler.
- **A separate global `standards.db`** — named directly in the legacy
  handler's doc comment (`mcp_dir()`-adjacent). No such file/table/function
  exists live.
- **`standard_assets` catalog** (`schema/archive/knowledge-hub/29-standard_assets.sql`):
  `(standard_id, name, kind[script|prompt], path, purpose)`. Closest
  archived precedent to §2.9's revised design.
- **`{"type": "head_commit"}` expiry** (`docs/proposal/archive/knowledge-system-author-guide.md:539`):
  no live replacement — `execution` (`schema/knowledge/07`) has no commit
  column.
- **`systems`/`standards` two-tier registry** (`schema/archive/knowledge-hub/01-systems.sql`,
  `02-standards.sql`): closest precedent to "category" (§2.2).

### 1.4 `Kriti/samgraha/system/` — corrected, verified in full

```
system/
├── dev/        base_dev (abstract: true), fastapi_dev (extends: base_dev,
│               drops: [06-design, 09-feature-design]), rust_dev,
│               electron_dev, react_dev
├── academic/   base_academic (abstract: true), pcems_2026, eswa_journal
└── hackathon/  python_hackathon (no system.yaml, no standard.yaml at all)
```

**Category = the directory one level under `system/`** — a filesystem
convention, invisible to samgraha today (§2.2 makes it explicit).
**`abstract: true`** marks base-only standards (`base_dev`, `base_academic`
— both say so in their own file's prose). **`extends`+`drops`** (dev
category) is a real, working, author-intended inheritance convention,
resolved historically by filesystem fallback. **Academic category rejected
that same fallback mechanism deliberately** — `pcems_2026/system.yaml`'s
own comment: *"Originally inherited base_academic's shared schema/scripts
by directory-fallback convention; now a self-contained fork ... so no
standard reaches into another's files."* Load-bearing precedent for §6:
the standard authors already tried implicit fallback and moved away from
it for the harder case.

**Corrected finding — `pcems_2026` already has a `standard.yaml`, at
`script/schema/standard.yaml`, not at the standard's root.** Its own header
comment explains why: *"Paths are relative to this file's location
(script/schema/)"* — every `scripts:`/`prompts:` entry's `location:` is a
relative path resolved from *there*, not from the standard's root. Parsed
directly (`python3 -c "import yaml; ..."`) to get exact counts:

| Field | Count |
|---|---|
| `scripts` | 37 |
| `prompts` | 33 |
| `custom_tables` | 22 |
| `usecases` | 73 (7 with real `steps:`, 66 with `steps: []`) |

The 7 step-bearing usecases: `schema-init` (1 step), `classify-repo`
(1 step), `propose-generation`/`propose-audit`/`propose-fix` (4 steps
each: gather → semantic draft → persist → render), `propose-report`
(3 steps, no semantic draft — auto-built from score+report-kind list),
`approve-proposal` (1 step, human decision). **These already work,
unmodified, through the existing engine** — no new samgraha capability is
needed for any of them; §2.8 confirms this rather than adding anything.

The other 66 (`novelty-analysis`, `gap-analysis`, the 6-domains × 9-stages
generate/cite/enrich/budget/audit/humanize sequence, `calculate`,
`render-paper`, etc.) declare `steps: []` **by design** — their component
scripts/prompts exist in the manifest's flat `scripts:`/`prompts:` catalog
(e.g. `discover-modules`, `persist-domain-semantic-score`), but sequencing,
branching (Path A generate vs. Path B audit→fix), and gating (tier-gate,
proposal-gate) all live in `plan/core/loop.yaml`, read by an external
orchestrator — not by samgraha's step dispatcher. `loop.yaml` (104 lines)
declares: 5 tiers (`introduction` → `methodology` → `findings` →
`{conclusion, title-and-metadata}` → `references`), a relationship graph
(`guides`/`validates`/`informs` edges between domains), `path_selection`
(generate-from-scratch vs. audit-then-fix), a `proposal_gate` (*"generation/
audit/report may not begin until an `academic_proposals` row for (phase,
commit_sha) has status=approved"* — commit-keyed, confirming §2.9's git
provenance is not a nice-to-have, it's already load-bearing to how this
standard actually gates work), and `fix_loop` (max 5 iterations, falls back
to human review).

`classify_repo: { min_doc_words: 200 }` — a top-level manifest key
`StandardManifest` does not declare. `serde_yaml::from_str` does not reject
unknown fields by default; this value is silently discarded on every
`register_standard` call today, with no error, no warning (§2.12).

`script/verify/` — 59 files, 58 of them one-line generated wrappers
(*"generated by script/schema/generate_per_domain_usecases.py, do not
hand-edit"*) that all call `_common.py:verify_main(usecase_name)`, which:

```python
def verify_main(usecase_name):
    args = parse_args(description=f"Verify {usecase_name} completion")
    conn = academic_schema.get_conn(args.db_path)
    complete, detail = academic_schema.usecase_status(conn, args.paper_id, usecase_name)
    sys.exit(0 if complete else 1)
```

`parse_args` here is a **different** function from `parse_step_args`
(`script/common/_adapter.py:18`, the real `--repo-root --in --out` script
contract every deterministic step uses) — it takes `--db-path`/`--paper-id`,
no JSON envelope out, exit code only. This is a **per-usecase, runtime,
DB-state completion check**, not a structural registration gate.

`script/smoke_test.py` **is** the structural, registration-time validator
the first draft's verify-gate should have been designed around: its own
docstring lists exactly what it checks — SQL schemas load, `standard.yaml`
parses, prompts are non-empty, HTML templates render, renderer scripts
import cleanly, the bibliography parser works. Runs once, standalone,
`python script/smoke_test.py --repo-root <path>`, no DB/paper context
needed (§2.4).

`schema/14-academic_templates.sql` — the standard's own template catalog,
already shipped: `(template_kind IN ('prompt','scaffold'), scope, name,
file_path)`, `scope` deliberately **not** a fixed enum (*"a hand-maintained
enum would only ever lag"* — its own comment), populated by `init-schema`'s
`seed_templates()` walking `prompt/` and `templates/`. `schema/18-academic_report_history.sql` —
per-render-run tracking, `report_kind`/`format`/`is_latest`, three report
kinds. `schema/22-academic_proposals.sql` — `status` lifecycle
`pending → approved|rejected|superseded` (terminal states plus a
stale-redraft state the first draft's `draft/final/archived` never
modeled), `commit_sha`-keyed, `is_latest` per `(paper, phase, scope_domain)`.
All three are correct, working, richer than what a first-pass global table
would offer (§2.9).

### 1.5 Net assessment (revised)

The Bodha errorlist's gaps are real: no registry, no listing, no category
tagging, no per-standard versioning display. What the first draft got
wrong was assuming those gaps meant "no standard is written against the
new format" — one is, fully, and it's a better proof of the model than
this proposal could have invented from scratch. The actual remaining work
is narrower and more precise than the first draft's phase-0 framing: fix
path resolution (§2.14), add a catch-all for standard-specific manifest
extensions (§2.12), model domains and usecase-to-domain mapping (§2.11) and
the plan/guide content the orchestrator actually reads (§2.9), and split
"verify" into the two genuinely different things `pcems_2026` proves they
are (§2.4) — rather than inventing global tables that would compete with
`academic_templates`/`academic_report_history`/`academic_proposals`, which
already do this job correctly.

---

## 2. Design

### 2.1 Global standard registry — new `standards.db`

Unchanged from the first draft in shape, one column added:

```sql
CREATE TABLE IF NOT EXISTS standard_registry (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT    NOT NULL UNIQUE,
    category       TEXT    NOT NULL,
    subcategory    TEXT,
    source_path    TEXT    NOT NULL,   -- the *directory* passed to register_standard_globally,
                                        -- not necessarily where standard.yaml itself lives (§2.14)
    is_abstract    INTEGER NOT NULL DEFAULT 0,
    extends        TEXT,
    version        TEXT    NOT NULL DEFAULT '0.0.0',
    description    TEXT    NOT NULL DEFAULT '',
    metadata_json  TEXT    NOT NULL DEFAULT '{}',   -- §2.12 catch-all (e.g. pcems_2026's classify_repo:)
    verify_status  TEXT    NOT NULL DEFAULT 'unverified'
                   CHECK (verify_status IN ('unverified','passed','failed')),
    verified_at    TEXT,
    registered_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_standard_registry_category ON standard_registry(category, subcategory);
```

`register_standard_globally(path)` parses the manifest (resolved per
§2.14), runs the structural verify-gate (§2.4), upserts this row. Does not
duplicate the standard's scripts/prompts/usecases — that's still
`register_standard`'s per-repo job, unchanged.

### 2.2 Category / subcategory — unchanged from first draft

Declared in the manifest (`category: dev`, optional `subcategory: null`),
not inferred from directory depth — `pcems_2026`'s own fork-away-from-
directory-convention precedent (§1.4) is exactly why inference is the
wrong call here too.

### 2.3 Abstract-standard gate — unchanged from first draft

`register_standard_globally` allows `abstract: true`. `register_standard`
(per-repo) rejects a standard whose global registry row has
`is_abstract = 1`, naming every row with matching `extends` as the
alternative. Reviewed at #10 in §7 and kept as originally designed —
intentional, no change needed.

### 2.4 Verify — two mechanisms, matching what `pcems_2026` actually does

Corrected from a single-entrypoint design to two, because `pcems_2026`
proves they're genuinely different concerns with different contracts
(§1.4):

**2.4.1 Structural gate, at registration time — models `smoke_test.py`.**
New optional manifest field: `smoke_test: script/smoke_test.py` (a plain
path, run relative to the standard's root — `pcems_2026`'s own lives one
level above `script/schema/`, i.e. at `script/smoke_test.py`, resolved the
same way every other `location:` is, `register_standard.rs:264-275`'s
`resolve_location`, unchanged). `register_standard_globally` runs it as a
plain subprocess (`std::process::Command`, not the `--repo-root/--in/--out`
envelope contract — `smoke_test.py` takes `--repo-root` only, no JSON
in/out, per its own docstring, §1.4), checks exit code. `0` →
`verify_status = 'passed'`; nonzero → `'failed'`, registration rejected, no
row written (verify runs before upsert). No `smoke_test` declared →
`'unverified'`, registration proceeds — opt-in, matching §2.4 of the first
draft's reasoning, just now pointed at the right script.

**2.4.2 Runtime, per-usecase completion check — models `script/verify/`.**
Not a registration gate at all. New optional field on `UsecaseDecl`:
`verify_script: script/verify/uc5_audit_det_findings.py` (or, generalized
to any standard's own convention: any script satisfying "exits 0 if this
usecase is complete for the given repo/paper context, nonzero otherwise" —
samgraha does not mandate `_common.py`'s specific `--db-path --paper-id`
shape, only that it's a plain subprocess with a meaningful exit code, since
that's the one thing every verify script here actually has in common).
New MCP tool `check_usecase_complete(usecase, repo_root, extra_args?)` —
resolves the usecase's `verify_script`, runs it with `extra_args` appended
verbatim (samgraha doesn't know or care that `pcems_2026` wants
`--paper-id`; it's the caller's job to know what a given standard's verify
scripts expect, exactly as `run_script_step`'s `--in` payload is already
opaque to samgraha), returns `{"complete": exit_code == 0}`. This is a
**query**, not a gate — nothing in samgraha blocks on its result; the
standard's own orchestrator (`loop.yaml`'s tier-gate) is free to call it
and act on the answer, same as it does today by shelling the script
directly. The only thing this adds is a discoverable, uniform entrypoint
instead of "go read which `script/verify/*.py` file matches this usecase's
name."

### 2.5 Usecase sequencing metadata — `driver` + `depends_on`, replacing `bootstrap: true`

Corrected: the first draft's single `bootstrap: true` boolean only fit
`schema-init`. `pcems_2026` proves two real categories of usecase exist
(§1.4) — step-bearing (samgraha dispatches) and orchestrator-driven
(the standard's own `loop.yaml` dispatches, using the manifest only as a
script/prompt catalog). Two new optional fields on `UsecaseDecl`:

```yaml
usecases:
  - name: schema-init
    driver: samgraha        # default; has steps:, dispatched via run_script_step
    depends_on: []
    steps: [...]
  - name: classify-repo
    driver: samgraha
    depends_on: [schema-init]
    steps: [...]
  - name: novelty-analysis
    driver: external         # steps: [] is valid and expected; samgraha
    depends_on: [classify-repo]   # never dispatches this usecase itself
    steps: []
```

`register_standard` **stops requiring** every usecase to have non-empty
`steps` (it never actually enforced this — confirmed by `pcems_2026`
registering-in-principle fine today — but `driver: external` makes the
absence of steps an explicit, intentional declaration instead of an
unremarked-on empty list). `depends_on` is a flat list of prerequisite
usecase names, validated the same way `step`'s `script`/`prompt`
cross-references already are (`register_standard.rs:183-188`'s "unknown
script" pattern) — unknown name in `depends_on` fails registration loudly.
`seed_standard` (renamed from the first draft's narrower version) walks
`depends_on` transitively for any requested usecase and runs every
`driver: samgraha` prerequisite's steps in dependency order before the
target — a generic topological walk, not a `bootstrap`-flag special case.
`driver: external` usecases are skipped by `seed_standard` (nothing to
run) but still satisfy dependents that name them, once
`check_usecase_complete` (§2.4.2) confirms they're done — for a usecase
with no `verify_script` declared, `depends_on` on it is accepted at
registration time but can't be automatically confirmed-complete; that's
named as a limitation, not silently ignored.

### 2.6 New MCP tools (revised list)

| Tool | Purpose |
|---|---|
| `register_standard_globally(path)` | §2.1, §2.3, §2.4.1. |
| `list_standards(category?, subcategory?)` | Queries `standard_registry`. |
| `get_standard_info(name)` | Registry row plus local `standard` table (§2.13) if this repo has it. |
| `get_standard_usecases(name)` | `usecase`/`step` rows, each usecase's `driver`, `depends_on`, resolved `domain` (§2.11). |
| `get_standard_scripts(name)` / `get_standard_prompts(name)` | Flat catalog lists. |
| `get_standard_assets(name, kind?)` | §2.9 — plan/guide/config content, filterable by `kind`. |
| `check_usecase_complete(usecase, repo_root, extra_args?)` | §2.4.2. |
| `seed_standard(standard, repo_root, usecase)` | §2.5. |

### 2.7 `template` — generic, optional, samgraha-owned rendering-template catalog

Rev. 2 reinstates this, reframed. Not a replacement for
`academic_templates` — `pcems_2026` already has a working, better-fitted
catalog (directory-scoped `scope`, `template_kind IN ('prompt','scaffold')`,
§1.4) and has no reason to touch this. This table exists for every standard
that *doesn't* have one yet — the five unauthored `dev`-category standards,
`python_hackathon`, any new standard started tomorrow — so "declare a
rendering template" doesn't require inventing a bespoke schema on day one:

```sql
-- 13-template.sql — a named, typed rendering template a standard ships,
-- for standards that choose to use samgraha's generic catalog instead of
-- building their own (a standard is always free to do what pcems_2026
-- does instead — this table doesn't require adoption, it just exists so
-- "I have no template catalog yet" isn't a blocker).
CREATE TABLE IF NOT EXISTS template (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    type     TEXT    NOT NULL,   -- 'html' | 'markdown' | 'json' | ... — open
                                  -- vocabulary, same reasoning as
                                  -- academic_templates.scope not being a
                                  -- fixed enum (§1.4): a closed list lags
                                  -- what standards actually ship.
    content  TEXT    NOT NULL,   -- read once at register time, same
                                  -- discipline as prompt.content (03-prompt.sql)
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, name)
);
```

`StandardManifest` gains an optional `templates: Vec<TemplateDecl>` field
(`name`, `location`, `purpose`, `type` — same shape as `PromptDecl`,
populated the same read-once-at-register-time way, `register_standard.rs:139-149`).
A standard that doesn't declare any never gets a row here — this is purely
additive, opt-in per standard.

### 2.8 `proposal` — generic, optional, links a usecase's run to its rendered output and the commit it ran against

Rev. 2 reinstates this too, same framing as §2.7: `pcems_2026`'s
`academic_proposals` (four-state lifecycle including `superseded`,
`commit_sha`-keyed, §1.4) is richer and stays exactly as it is — this table
is for the standards that don't have anything like it yet, and it's the
direct answer to "generate a proposal based on a usecase and save it using
a template, with everything backtracked to the commit it ran against":

```sql
-- 14-proposal.sql — one row per generated proposal: which usecase produced
-- it, which template rendered it, which execution ran it, so the commit
-- state is one join away via execution.git_detail_id (§2.10) — no need to
-- duplicate git_detail_id here. Deliberately a simpler 3-state lifecycle
-- than academic_proposals' 4-state one (no 'superseded') — a standard that
-- needs that nuance builds its own table, the way pcems_2026 already does;
-- this is the default for one that doesn't.
CREATE TABLE IF NOT EXISTS proposal (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    standard     TEXT    NOT NULL,
    usecase_id   INTEGER NOT NULL REFERENCES usecase(id) ON DELETE CASCADE,
    template_id  INTEGER REFERENCES template(id) ON DELETE SET NULL,
    execution_id INTEGER REFERENCES execution(id) ON DELETE SET NULL,
    title        TEXT    NOT NULL,
    status       TEXT    NOT NULL DEFAULT 'draft'
                 CHECK (status IN ('draft','final','archived')),
    location     TEXT,    -- rendered file path, nullable until rendering finishes
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
```

No new MCP tool or dispatcher — a "propose" usecase is just a usecase
(exactly as `pcems_2026`'s four `propose-*` usecases already prove, §1.4:
fully step-declared, running through the existing, unmodified engine
today). The only new mechanism: `run_script_step` (§2.9's `artifacts[]`
envelope read, extended one field further) also reads an optional
`result.proposal: {usecase_id, template_id, title, location}` key from a
step's output envelope and inserts one `proposal` row, `execution_id` set
to the row it just recorded — same one-line addition to an already-existing
read, not a new code path. A standard that never reports `result.proposal`
never gets a row here; `pcems_2026`'s own `persist-proposal`/`render-proposal`
steps have no reason to start (their existing `academic_proposals` writes
keep working exactly as they do today).

### 2.9 `standard_asset` — a thin, kind-open catalog for what has no table anywhere: plan and guide content, plus generic `artifact` tracking

`plan/core/loop.yaml` and `guide/`'s 12 subdirectories are real,
load-bearing, currently invisible to any query, for *any* standard that
ships this kind of content — this isn't something even `pcems_2026`'s own
schema catalogs today (`academic_templates` covers `prompt`/`scaffold`
content only, §1.4; plan/guide content has no table anywhere, not even a
standard-owned one). One table, kind-open (matching
`academic_templates.scope`'s own "not a fixed enum, would only lag"
reasoning, §1.4, applied one level up):

```sql
-- 09-standard_asset.sql — catalog of standard-shipped content that isn't
-- a script, a prompt, or a standard-owned custom table: orchestrator plan
-- files, author-facing guides, lint/style config, anything else a standard
-- wants discoverable by name without samgraha inventing a dedicated table
-- per content type. kind is an open string, not a CHECK-constrained enum —
-- same reasoning as academic_templates.scope: a fixed vocabulary lags what
-- standards actually ship.
CREATE TABLE IF NOT EXISTS standard_asset (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    standard TEXT    NOT NULL,
    kind     TEXT    NOT NULL,   -- 'plan' | 'guide' | 'config' | anything else
    name     TEXT    NOT NULL,
    location TEXT    NOT NULL,   -- resolved same as script/prompt locations
    purpose  TEXT    NOT NULL DEFAULT '',
    UNIQUE(standard, kind, name)
);
```

`StandardManifest` gains `assets: Vec<AssetDecl>` (`name`, `kind`,
`location`, `purpose` — same shape as `ScriptDecl`/`PromptDecl`), read-once
at register time like prompts (content is **not** inlined — `plan/core/
loop.yaml` and guide files are read by the standard's own orchestrator/
agent context, not by samgraha, so only the path is stored, matching how
`script.location` already works, not how `prompt.content` does).

`artifact` — produced output, execution-linked, same "optional, generic,
for a standard that doesn't have `academic_report_history`-style tracking
yet" framing as `template`/`proposal` (§2.7, §2.8):

```sql
-- 15-artifact.sql — a produced output: something an execution actually
-- generated (a rendered proposal doc, a chart image, a PDF). Distinct from
-- template (the input) and standard_asset (reference content samgraha
-- never wrote) — one row per produced file, traceable to the execution
-- that made it, and from there to the commit it ran against
-- (execution.git_detail_id, §2.10) without a denormalized copy here.
CREATE TABLE IF NOT EXISTS artifact (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    standard     TEXT    NOT NULL,
    execution_id INTEGER REFERENCES execution(id) ON DELETE SET NULL,
    name         TEXT    NOT NULL,
    type         TEXT    NOT NULL,   -- 'document' | 'image' | 'model' | 'file' | ...
    location     TEXT    NOT NULL,
    purpose      TEXT    NOT NULL DEFAULT '',
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_artifact_execution ON artifact(execution_id);
```

Populated the same way as `proposal` (§2.8): `run_script_step` reads an
optional `result.artifacts: [{name, type, location, purpose}]` array from a
step's output envelope and inserts one row per entry. A standard with its
own tracking (`pcems_2026`) never reports this key and never gets rows
here.

### 2.10 Git detail — provenance, fed forward, never conflated with a standard's own commit column

`loop.yaml`'s `proposal_gate` keys on `(phase, commit_sha)` and
`academic_proposals`/other tables already carry their own `commit_sha`
column (§1.4) — confirming git-state tracking is load-bearing, not a
nice-to-have, and that standards today must be sourcing `commit_sha`
themselves (shelling `git` in their own Python, presumably in
`script/common`). Kept from the first draft, with one addition: samgraha
captures it once per execution and **passes it forward** instead of only
storing it silently, so a standard no longer needs its own git-shelling
code to populate columns like `academic_proposals.commit_sha`:

```sql
-- 10-git_detail.sql
CREATE TABLE IF NOT EXISTS git_detail (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_root   TEXT    NOT NULL,
    commit_sha  TEXT    NOT NULL,
    branch      TEXT    NOT NULL DEFAULT '',
    dirty       INTEGER NOT NULL DEFAULT 0,
    captured_at TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(repo_root, commit_sha, dirty)
);
ALTER TABLE execution ADD COLUMN git_detail_id INTEGER REFERENCES git_detail(id);
```

`record_execution` (`step_execution.rs:120-126`) upserts `git_detail`
before its existing `INSERT` (`git rev-parse HEAD` /
`--abbrev-ref HEAD` / `status --porcelain`, best-effort, `NULL` if not a
git repo — never blocking). `run_script_step` additionally injects
`{"_git": {"commit_sha", "branch", "dirty"}}` into the JSON payload it
already writes to `--in` before invoking the script (additive key, next to
whatever the step's own `input_json` already carries) — a standard's
`persist-proposal`-equivalent script can read `_git.commit_sha` straight
from its own input envelope instead of shelling `git` itself. Deliberately
**not** adding `merge_base` (raised in review, §7 #14) — no standard on
disk references a merge-base anywhere; `branch` plus `commit_sha` covers
every real use found. Add it later against an actual need, not
speculatively.

`git_detail_id` is infrastructure provenance only — "what was HEAD when
samgraha dispatched this" — and is never written into or read from a
standard's own business columns (`academic_proposals.commit_sha` stays
exactly as it is, populated by the standard's own script, now sourced from
`_git.commit_sha` instead of its own `git` subprocess call if the standard
chooses to simplify — that migration is the standard author's call, not
this proposal's).

**"Latest commit" and full backtracking, both answered by query, not a
second table.** `git_detail` is an append-only log — one row per distinct
`(repo_root, commit_sha, dirty)` ever seen, not one row per execution.
"What's the latest commit this repo ran against" is
`SELECT * FROM git_detail WHERE repo_root = ? ORDER BY captured_at DESC LIMIT 1`
— no dedicated "latest" table needed, and no risk of a second table
drifting from the log it would be summarizing. Every execution-linked row
this proposal adds (`proposal.execution_id`, `artifact.execution_id`)
reaches its commit the same way: one join to `execution`, one join from
`execution.git_detail_id` to `git_detail` — "was this proposal/artifact
generated against the current commit, or a stale one" is answerable end to
end without any table needing its own copy of the commit SHA.

### 2.11 Domains — a thin discovery mirror, not a replacement for a standard's own domain table

Both categories with real content build their own richer domain table
today — `academic_domains` (pcems_2026, referenced by `init-schema`'s
`seed_domains()`) and `hackathon_domains`
(`schema/02-hackathon_domains.sql`: `key`, `display_name`, `sort_order`,
`weight`, `det_weight`, `sem_weight` — business fields no generic table
should try to hold). What's missing isn't a replacement for either — it's
that **samgraha itself has no way to answer "what domains does standard X
cover" without opening that standard's own custom table**, which only
exists after `init-schema` has actually run once. A thin, register-time
mirror closes this:

```sql
-- 11-domain.sql — discovery-only mirror of a standard's declared domains.
-- Not authoritative for scoring/weights (the standard's own custom table,
-- e.g. academic_domains/hackathon_domains, stays the source of truth for
-- those) — exists so "what domains, what order" is answerable immediately
-- after register_standard, before any usecase has ever run.
CREATE TABLE IF NOT EXISTS domain (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    standard   TEXT    NOT NULL,
    key        TEXT    NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    description TEXT   NOT NULL DEFAULT '',
    UNIQUE(standard, key)
);
```

`StandardManifest` gains `domains: Vec<DomainDecl>` (`key`, `sort_order`,
`description`) populated the same way `system.yaml`'s existing `domains:`
list already reads for `pcems_2026` (§1.4) — this is a straight carry-over
of data the standard already declares, not new authoring burden.

**Usecase-to-domain mapping** (the explicit ask this review closed with):
`usecase` (`schema/knowledge/01-usecase.sql`) gains one nullable column:

```sql
ALTER TABLE usecase ADD COLUMN domain_id INTEGER REFERENCES domain(id);
```

`UsecaseDecl` gains optional `domain: Option<String>` (a `domain.key`),
resolved to `domain_id` at register time the same way a step's `script`/
`prompt` name resolves to an id today (`register_standard.rs:183-188`
pattern, reused). `pcems_2026`'s per-domain usecases
(`deterministic-audit-title-and-metadata`, etc.) currently encode the
domain in the usecase's *name* only — this makes it a real, queryable
foreign key instead of a naming convention, without requiring the standard
to rename anything (`get_standard_usecases` can then answer "every usecase
for domain `findings`" as a `WHERE` clause, not a name-prefix guess).

### 2.12 Manifest extensibility — catch-all, not a rigid struct

`classify_repo: { min_doc_words: 200 }` (§1.4) is silently dropped today.
Fix: `StandardManifest` gains

```rust
#[serde(flatten)]
pub extra: serde_json::Map<String, serde_json::Value>,
```

Every top-level key not otherwise named in `StandardManifest` lands here
instead of vanishing. `register_standard_globally` serializes it into
`standard_registry.metadata_json` (§2.1); `register_standard` (per-repo)
serializes it into the new local `standard.metadata_json` (§2.13). Neither
samgraha crate interprets any key inside it — same "never interpret the
standard's own meaning" discipline as everywhere else in this codebase;
it's stored and returned via `get_standard_info`, nothing more.

### 2.13 A local `standard` table — closing a real gap the review surfaced indirectly

Today, "standard" is only a `TEXT` column scattered across `usecase`,
`script`, `prompt`, `custom_data_tables` — there is no single row
representing the standard itself inside a consuming repo's own
`knowledge.db`. That's fine for the tables that existed before this
proposal, but §2.11's `domains` and §2.12's `metadata_json` both need a
per-repo home that doesn't depend on the global `standards.db` always
being reachable (a repo should be able to answer "what standard is this,
what category, what metadata" from its own `knowledge.db` alone, offline).

```sql
-- 12-standard.sql — one row per standard registered *into this repo*.
-- Local mirror of the fields standard_registry (§2.1) tracks globally,
-- so a repo's own knowledge.db is self-describing without a live
-- standards.db lookup.
CREATE TABLE IF NOT EXISTS standard (
    name          TEXT PRIMARY KEY,
    category      TEXT    NOT NULL,
    subcategory   TEXT,
    extends       TEXT,
    version       TEXT    NOT NULL DEFAULT '0.0.0',
    metadata_json TEXT    NOT NULL DEFAULT '{}',
    registered_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
```

`register_standard` (per-repo) writes/replaces this row alongside its
existing script/prompt/usecase writes — same delete-then-reinsert
discipline (`register_standard.rs:236`'s `delete_existing`, extended by one
more table).

### 2.14 Manifest path resolution — the actual fix for the first draft's core error

`register_standard`/`register_standard_globally` currently assume
`<path>/standard.yaml`. `pcems_2026` disproves that assumption outright.
Fix: accept either a direct file path ending in `.yaml`/`.yml`, used as-is,
or a directory, in which case check exactly two candidate locations, in
order — **not** an open-ended multi-directory search (that would trade one
hardcoded assumption for an unbounded one; two locations is the one real
convention observed across all eight standards on disk):

1. `<path>/standard.yaml` (the location every archived-doc example and the
   existing unit-test fixtures use).
2. `<path>/script/schema/standard.yaml` (`pcems_2026`'s actual location,
   confirmed §1.4 — and, per its own header comment about self-containment,
   plausibly the pattern any future academic-category standard will also
   use, since it's *generated* by `script/schema/generate_per_domain_usecases.py`
   living right next to it).

Neither found → error names both paths checked, so a standard author
immediately knows where samgraha looked, not just "no manifest found."

### 2.15 `samgraha.toml` — what a knowledge-authoring repo needs vs. a normal repo, verified

Checked directly against `crates/common/src/config.rs`, `samgraha`'s own
`samgraha.toml`, and Kriti's `samgraha.toml` (the one real
`kind = "knowledge"` repo on this machine):

- **`RepositoryKind` (`config.rs:287-298`: `Repository` | `Knowledge`) is
  declared and read nowhere else in the codebase** — `grep -rn
  RepositoryKind crates/` outside `config.rs` itself returns nothing. Kriti
  sets `kind = "knowledge"` in its `samgraha.toml` today; it has zero effect
  on samgraha's behavior. This is a real, verified dead distinction, not a
  hypothetical gap.
- **`KnowledgeConfig` (`config.rs:261-271`) has `root`, `dependencies`,
  `interests` — no `standards` field.** A repo cannot declare "I need
  standard X" in its own `samgraha.toml` today (this is the Bodha
  errorlist's original P1 ask, and it was previously undesignable — there
  was no global registry for a name to resolve against. §2.1 makes it
  designable: a name resolves to `standard_registry.source_path`).
- **`root` (`KnowledgeConfig.root`, default `"system"`) is documented as
  "the root directory containing Knowledge Systems (used when kind =
  knowledge")** — i.e. this field already exists specifically for
  authoring repos, confirmed by Kriti's own `samgraha.toml`
  (`[knowledge] root = "samgraha/system"`, matching its actual
  `samgraha/system/{dev,academic,hackathon}/` layout) — but nothing reads
  it as anything other than a plain config value; no code walks it to
  discover standards to register.

**Fix — give the existing `kind` field real meaning, tied to this
proposal's registry, and add the one missing config key:**

- `KnowledgeConfig` gains `standards: Vec<String>` — names to resolve
  against `standard_registry` (§2.1) and register locally. `init_repository`
  (`crates/services/src/init.rs`), after writing `samgraha.toml`, resolves
  each name via `standard_registry.source_path` and calls the existing
  `register_standard(source_path, local_knowledge_db)` — this is the exact
  "automatic copying/registration from global to a repo's `.samgraha/`"
  the original Bodha errorlist asked for (its item 3), now implementable
  because §2.1 gives a name something to resolve against. Unset/empty
  `standards` (today's universal default) is unchanged behavior — nothing
  breaks for a repo that never declares any.
- `register_standard_globally(path)` (§2.1) reads the calling repo's own
  `samgraha.toml` `kind` — **warns** (not a hard error; most repos won't
  bother setting `kind`, and rejecting on a config field that's been dead
  until this proposal would be a surprising new failure mode) if `kind !=
  "knowledge"`, since a normal consuming repo authoring/registering a
  standard globally is almost always a mistake, not an intended workflow.
  `kind = "knowledge"` repos (like Kriti) register cleanly, silently.
- `KnowledgeConfig.root` keeps its existing meaning (the directory a
  `kind = "knowledge"` repo's standards live under) — this proposal adds no
  new field for it, since `register_standard_globally(path)` already takes
  an explicit path per standard; `root` remains documentation/convention
  for where an author points that path *from*, not something samgraha
  walks automatically (consistent with §2.14's "two known locations, not an
  open-ended search" discipline — walking `root` for every subdirectory
  that might be a standard is exactly the unbounded search §2.14 already
  rejected doing at the manifest level).

---

## 3. Standard migration — out of this proposal's scope; `pcems_2026` is read-only evidence

This proposal is samgraha/MCP-side only — schema, service code, and MCP
tools inside this repo. It does not ask anyone to change anything under
`Kriti/samgraha/system/`, and every design decision above is checked
against that content only to confirm it *works unmodified*, never as a
prerequisite the standard must first meet.

Concretely: `pcems_2026` already registers successfully today, unmodified,
once §2.14's path fix lands — every field this proposal adds
(`category`, `domains`, `smoke_test`, `driver`/`depends_on`, `assets`,
`templates`) is **optional** on `StandardManifest`, defaulting to empty/
absent. `pcems_2026` not declaring any of them is not a blocker; it simply
means `standard_registry`/`domain`/`standard_asset`/`template` stay empty
for it while its 37 scripts, 33 prompts, 22 custom tables, and 7
step-bearing usecases register and run exactly as they do today (§1.4).

Whether `pcems_2026` — or `base_dev`, `base_academic`, the other three
`dev`-category standards, or `python_hackathon` (no manifest at all) —
ever adopts `category`/`domains`/`smoke_test`/the generic `template`/
`proposal` tables is entirely each standard author's own call, made
against their own repository (`Kriti`), on their own timeline, not a
deliverable of this document. Phase 5 (§4) proves every new mechanism
against `pcems_2026` **as it exists on disk right now** — no edits, no
fixture standing in for it — precisely so this proposal's own claims stay
checked against real content without asking that content's owner to change
anything for samgraha's sake.

---

## 4. Implementation Plan

### Phase 1 — `standards.db` + registry
`standard_registry` (§2.1, with `metadata_json`), `operation_log` (`scope`
column added per §7 #11), global-DB wrapper mirroring `RegistryDb::open`,
`register_standard_globally` service function, the `kind`-aware warning
(§2.15).

### Phase 2 — Manifest fixes (the corrected core of this proposal)
- §2.14 path resolution — **land first**, it's what makes every other
  phase testable against `pcems_2026` instead of only synthetic fixtures.
- §2.12 `#[serde(flatten)] extra` field.
- `StandardManifest`: `category`, `subcategory`, `extends`, `smoke_test`
  (§2.4.1), `domains: Vec<DomainDecl>` (§2.11), `assets: Vec<AssetDecl>`
  (§2.9).
- `UsecaseDecl`: `driver`, `depends_on`, `domain`, `verify_script` (§2.4.2,
  §2.5, §2.11).
- Abstract-gate lookup in `register_standard` (§2.3).
- `KnowledgeConfig.standards: Vec<String>` (§2.15), `init_repository`
  resolve-and-register loop.

### Phase 3 — Schema (knowledge.db)
**Land the `CORE_MIGRATIONS` version-gate (§8.8) first, in its own
sub-step, before any of the `ALTER TABLE` statements below exist** — the
gate is what makes those statements safe to add at all. Then:
`09-standard_asset.sql`, `10-git_detail.sql` (+ `execution.git_detail_id`),
`11-domain.sql` (+ `usecase.domain_id`), `12-standard.sql`, `13-template.sql`,
`14-proposal.sql`, `15-artifact.sql` — all seven additive, all optional
(populated only for a standard that declares the corresponding manifest
field or reports the corresponding envelope key; a standard that does
neither leaves every one of these tables empty for its own name, forever,
with no error).

### Phase 4 — MCP tool surface
`list_standards`, `get_standard_info`, `get_standard_usecases`,
`get_standard_scripts`, `get_standard_prompts`, `get_standard_assets`,
`check_usecase_complete`, `register_standard_globally`, `seed_standard`
(revised per §2.5) — nine new entries in `adapter.rs`'s dispatch table. No
new tool for `template`/`proposal`/`artifact` (§2.7-§2.9) — they're rows in
`knowledge.db`, read the same way `custom_data_tables` rows already are,
by any script that wants them; adding a dedicated getter for each would be
exactly the kind of unrequested tool-surface growth this proposal
otherwise avoids.

### Phase 5 — Proof, entirely read-only against `Kriti/`
Every bullet below runs against `Kriti/samgraha/system/academic/pcems_2026`
**exactly as it exists on disk today** — zero edits to that repo, per §3:

- `register_standard_globally(pcems_2026's directory)` — §2.14 resolves
  the manifest at `script/schema/standard.yaml`; assert success,
  `standard_registry` row has `category`/`domains`/every new optional
  column empty/default (since the on-disk manifest declares none of them),
  `verify_status = 'unverified'` (no `smoke_test` declared).
- `register_standard(pcems_2026, scratch_repo)` — assert the existing 37
  scripts/33 prompts/22 custom tables/73 usecases register exactly as they
  do today (regression guard against §2's additions breaking the baseline
  parse), plus one new `standard` row, with `domain`/`standard_asset`/
  `template` empty (correctly — nothing declares them).
- `seed_standard(pcems_2026, scratch_repo, "classify-repo")` — `schema-init`
  and `classify-repo` are the two real step-bearing prerequisites already
  in the manifest; assert both run in the right order **without** any
  `depends_on` field present (falls back to "no declared dependency, just
  run the requested usecase's own steps" — `depends_on` is optional, this
  proves the optional-and-absent path, not just the declared-present one).
- `check_usecase_complete("deterministic-audit-findings", scratch_repo,
  ["--paper-id", "1"])` against the real generated
  `script/verify/uc5_audit_det_findings.py` (no `verify_script` manifest
  field needed for this test — the tool takes an explicit script path
  argument too, resolved directly, so it's provable against `pcems_2026`'s
  real file without requiring the manifest to name it first).

Every mechanism that needs a manifest field `pcems_2026` doesn't declare
(`category`, `smoke_test`, `domains`, `driver`/`depends_on`, `assets`,
`templates`, `proposal`/`artifact` reporting) is proven with samgraha's own
test fixtures (§5) — small, synthetic `standard.yaml`s inside this repo's
own test suite, same as `register_standard.rs`'s existing five unit tests.
That split — real standard for the baseline engine, fixtures for new
optional fields — is what keeps this phase honest about not requiring
`Kriti/` changes while still proving every new mechanism works.

### Dependency graph
```
Phase 1 ──→ Phase 2 ──→ Phase 5
Phase 1 ──→ Phase 3 ──→ Phase 5
Phase 2 + Phase 3 ──→ Phase 4 ──→ Phase 5
```

---

## 5. Testing

- Unit: §2.14 path resolution — directory with root-level `standard.yaml`
  resolves there; directory with only `script/schema/standard.yaml`
  resolves there; directory with neither errors naming both paths.
- Unit: §2.12 — a manifest with an unrecognized top-level key round-trips
  it into `metadata_json` unchanged (parse `classify_repo:` fixture,
  assert `metadata_json` contains it verbatim).
- Unit: §2.4.1 — `smoke_test` script exiting nonzero rejects registration,
  no `standard_registry` row written; exiting 0 sets `verify_status =
  'passed'`; undeclared leaves `'unverified'` and still registers.
- Unit: §2.4.2 — `check_usecase_complete` against a fixture verify script
  that exits 0/1 on a flag, confirm both outcomes surface correctly, with
  arbitrary `extra_args` passed through unmodified.
- Unit: §2.5 — `depends_on` naming an unknown usecase fails registration
  loudly (mirrors the existing "unknown script" test,
  `register_standard.rs:361-378`); `seed_standard` on a 3-deep `depends_on`
  chain runs every `driver: samgraha` link in order exactly once.
- Unit: §2.11 — `usecase.domain` resolves to the right `domain_id`;
  unknown domain name fails registration loudly, same pattern.
- Unit: §2.10 — git capture inside/outside a real git fixture repo,
  dirty-vs-clean produces distinct `git_detail` rows (§2.10's `UNIQUE`),
  `run_script_step`'s `--in` payload contains `_git.commit_sha` matching
  `git rev-parse HEAD`.
- Unit: §2.3 abstract-gate — unchanged from first draft's plan.
- Unit: §2.7/§2.8/§2.9 — a fixture manifest declaring `templates:` and a
  usecase whose step reports `result.proposal`/`result.artifacts` in its
  output envelope; assert `template`/`proposal`/`artifact` rows are
  written correctly. A fixture manifest declaring **none** of these keys
  registers with zero rows in any of the three tables — proves the
  opt-in framing isn't just documentation.
- Unit: §2.15 — `KnowledgeConfig.standards = ["fixture-standard"]` resolves
  against a fixture `standard_registry` row's `source_path` and calls
  `register_standard` during `init_repository`; a repo with `kind !=
  "knowledge"` calling `register_standard_globally` logs the warning but
  still succeeds (soft gate, not a hard rejection, per §2.15).
- Integration: register `pcems_2026` end-to-end (Phase 5's bullets)
  against a real checkout of `Kriti/samgraha/system/academic/pcems_2026`,
  **unmodified** — no edits to that repo — this is the regression test
  that would have caught rev. 1's core error, so it must run against the
  actual directory as it exists today, not a trimmed-down or edited
  stand-in.

---

## 6. Explicitly out of scope

- No fallback file-resolution engine for `extends` (§1.4's `pcems_2026`
  precedent).
- No subcategory hierarchy — one flat optional string.
- No version-range resolution/pinning beyond a display string.
- Not removing `AssetSyncConfig`/`sync_knowledge_system` here — separate
  small cleanup.
- Not migrating `python_hackathon`, or authoring/editing `standard.yaml`
  for `base_dev`/`base_academic`/`fastapi_dev`/`rust_dev`/`electron_dev`/
  `react_dev`/`eswa_journal` — none of that is samgraha/MCP-side work, and
  none of it is this proposal's to schedule (§3). Whether and when any of
  them adopts `category`/`domains`/`smoke_test`/the generic `template`/
  `proposal`/`artifact` tables is entirely each standard's own author's
  call.
- Not standardizing domain *weights*/business fields across standards —
  `domain` (§2.11) is a discovery mirror only; `academic_domains`/
  `hackathon_domains`'s own weight/tier fields stay wherever each standard
  already keeps them. Unifying that is a much larger, cross-standard schema
  change with no demonstrated need yet (both existing implementations work
  fine independently) — not this proposal's problem to solve speculatively.
- Not making `KnowledgeConfig.standards`/`register_standard_globally`'s
  `kind` check a hard gate (§2.15) — soft warning only, since `kind` has
  had zero effect until now and a repo that never set it shouldn't suddenly
  start failing a call it made successfully yesterday.

---

## 7. Disposition of review findings

Findings 1-15 are from the first review (rev. 1). Findings 16-18 are from
the second (rev. 2), which caught rev. 1 over-correcting into out-of-scope
territory.

| # | Finding | Disposition |
|---|---|---|
| 1 | §1.4 wrong — `pcems_2026` has a `standard.yaml` | **Fixed.** §1.4 rewritten, exact counts verified by parsing. |
| 2 | serde silently drops `classify_repo:` | **Fixed.** §2.12, `#[serde(flatten)]`. |
| 3 | 63(66)/67(73) usecases have `steps: []`, model assumed otherwise | **Fixed.** §2.5, `driver`/`depends_on`. |
| 4 | domains not in registry/manifest | **Fixed.** §2.11, `domain` table + manifest field. |
| 5 | `plan/` invisible | **Fixed.** §2.9, `standard_asset` catalog. |
| 6 | `guide/` unindexed | **Fixed.** §2.9, same table, `kind: guide`. |
| 7 | verify-gate doesn't match real verify scripts | **Fixed.** §2.4 split into 2.4.1 (`smoke_test`) / 2.4.2 (`script/verify/`). |
| 8 | `template`/`artifact` duplicate `academic_templates`/`academic_report_history`/`academic_proposals` | **Fixed, then revised again at #16** — rev. 1 dropped the global tables outright; rev. 2 reinstated them as generic/optional (§2.7-§2.9), correctly scoped to standards that have nothing of their own, per finding #16 below. |
| 9 | `bootstrap: true` too narrow | **Fixed.** §2.5, `depends_on` generalizes to any prerequisite. |
| 10 | `extends` metadata-only loses inheritance semantics | **Accepted as designed** — §1.4's `pcems_2026` precedent is the reason, unchanged. |
| 11 | `operation_log` conflates global/per-repo grain | **Fixed.** `scope` column added (§4 Phase 1). |
| 12 | manifest path hardcoded to root | **Fixed.** §2.14 — the fix that made every other correction verifiable. |
| 13 | manifest needs `#[serde(flatten)]` | **Fixed.** Same as #2. |
| 14 | `git_detail` should capture merge-base | **Rejected, explicitly** — §2.10, no standard on disk references one; add when a real need shows up. `branch` was kept. |
| 15 | `smoke_test.py` should route through verify-gate | **Fixed.** §2.4.1 — it *is* the verify-gate now, not a coexisting separate mechanism. |
| 16 | This proposal's scope is samgraha/MCP-side only; rev. 1's `pcems_2026`-derived commentary had drifted into implicitly telling `Kriti/` what to change | **Fixed.** §3 rewritten to read-only evidence framing; §2.7/§2.8 reinstated as generic/optional infra rather than staying dropped; Phase 5 (§4) proves everything against `pcems_2026` unmodified. |
| 17 | `proposal`/`artifact`/`template` schema wanted, linked to usecase/plan, with domain mapped to usecase | **Fixed.** §2.7 (`template`), §2.8 (`proposal`, links `usecase_id`+`template_id`+`execution_id`), §2.9 (`artifact`), §2.11 (`domain` + `usecase.domain_id`) — all additive to existing `schema/knowledge` tables, all optional. |
| 18 | Need a commit table so execution can be backtracked to what commit it ran against; check `samgraha.toml` needs for a knowledge standard repo vs. a normal repo | **Fixed.** §2.10 clarifies `git_detail` + `execution.git_detail_id` already is that table (no second "latest" table needed — a query, not new schema) and extends backtracking to `proposal`/`artifact` via their shared `execution_id`. §2.15 (new) — `RepositoryKind` confirmed dead code, `KnowledgeConfig.standards` field added, `kind`-aware soft gate on `register_standard_globally`. |

---

## 8. Implementation Notes — verified against current code, not just shape

Every subsection below was checked against the actual file it touches
(`crates/mcp/src/adapter.rs`'s real struct, `crates/registry/src/core_schema.rs`'s
real `CORE_MIGRATIONS`, `crates/mcp/src/main.rs`'s real init sequence) —
this section is the concrete companion to §2/§4, code and DDL where §2/§4
only gave shape.

### 8.1 `operation_log` DDL (Phase 1)

```sql
CREATE TABLE IF NOT EXISTS operation_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    operation   TEXT    NOT NULL,   -- 'register_globally' | 'verify' | 'seed' | 'register_repo'
    standard    TEXT    NOT NULL,
    repo_root   TEXT,               -- NULL for global-only ops
    scope       TEXT    NOT NULL DEFAULT 'global' CHECK (scope IN ('global','repo')),
    status      TEXT    NOT NULL,
    detail_json TEXT    NOT NULL DEFAULT '{}',
    occurred_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
```

Lives in `standards.db` (§2.1, §8.2), alongside `standard_registry` — every
`register_*`/`check_usecase_complete`/`seed_standard` call writes exactly
one row on completion, `scope` set by the caller (`'global'` for
`register_standard_globally`, `'repo'` for `register_standard`/`seed_standard`/
`check_usecase_complete`), never inferred from `repo_root`'s nullness.

### 8.2 `standards.db` wrapper — new module, mirrors `RegistryDb` exactly

Verified: `RegistryDb` (`crates/registry/src/registry_db.rs:15-36`) is the
only existing precedent for a global-ish SQLite wrapper (it's actually
per-repo, at `.samgraha/registry.db`, but its `open`/migration-versioning
shape is the one to copy). New file `crates/registry/src/standards_db.rs`:

```rust
pub struct StandardsDb {
    conn: Mutex<Connection>,
}

impl StandardsDb {
    pub fn open() -> Result<Self> {
        let path = common::env::mcp_dir().join("standards.db");
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = Self { conn: Mutex::new(conn) };
        store.run_migrations()?;   // same version-gated pattern as
        Ok(store)                  // RegistryDb::run_registry_migrations,
    }                               // §8.8 explains why this matters
}
```

`common::env::mcp_dir()` (`crates/common/src/env.rs:78`) already exists and
is exactly the location the archived legacy handler's doc comment named
for this purpose (§1.3) — no new path-resolution logic needed, reuse as-is.

`McpAdapter` (`crates/mcp/src/adapter.rs:20-23`) currently has exactly two
fields:

```rust
pub struct McpAdapter {
    repository_root: PathBuf,
    registry: Arc<dyn RegistryClient>,
}
```

Gains a third: `standards_db: Arc<StandardsDb>`. `McpAdapter::new` (line
26) gains a matching third parameter. `crates/mcp/src/main.rs:44-46`:

```rust
let root = discover_root()?;
let registry = Arc::new(FileRegistryClient::new(&root));
let standards_db = Arc::new(registry::standards_db::StandardsDb::open()?);
let adapter = McpAdapter::new(root, registry, standards_db);
```

One `Arc`, opened once per MCP process, shared across every handler call —
same lifetime pattern `registry: Arc<dyn RegistryClient>` already uses.

**Global vs. repo-scoped handlers**: `target_root`/`knowledge_db_path`
(`adapter.rs:66-74`) resolve a *repo's* `knowledge.db` from the request's
`repo_path` (or `self.repository_root`) — correct, unchanged, for every
existing tool plus `register_standard`/`seed_standard`/`check_usecase_complete`
(§2.6, all repo-scoped). `list_standards`/`get_standard_info`/
`register_standard_globally` do not call `knowledge_db_path` at all — they
go straight to `self.standards_db`, ignoring `repository_root` entirely.
No change to `target_root`/`knowledge_db_path` themselves; the new handlers
simply don't use them, same as `handle_list_repositories` (`adapter.rs:116`)
already doesn't use `knowledge_db_path` today.

### 8.3 `resolve_location`'s base path — the one change that actually matters

**Confirmed the single most important code fix in this proposal.**
`register_standard` (`register_standard.rs:89-97`) today:

```rust
pub fn register_standard(standard_path: &Path, knowledge_db_path: &Path) -> Result<RegisterStandardResult> {
    let manifest_path = standard_path.join("standard.yaml");
    if !manifest_path.is_file() { bail!("No standard.yaml at {}", manifest_path.display()); }
    ...
```

and `resolve_location` (`register_standard.rs:264-275`):

```rust
fn resolve_location(standard_path: &Path, location: &str) -> Result<String> {
    let candidate = Path::new(location);
    let resolved = if candidate.is_absolute() { candidate.to_path_buf() }
                   else { standard_path.join(candidate) };
    ...
```

Every `scripts:`/`prompts:` `location:` resolves against `standard_path` —
the directory passed in, today always the standard's root. `pcems_2026`'s
manifest lives at `script/schema/standard.yaml` and its own header comment
is explicit that its relative paths (`../../script/schema-init/init_schema.py`)
resolve **from `script/schema/`, not from the standard's root**. Passing
`pcems_2026`'s root as `standard_path` while the manifest is found via
§2.14's two-location check at `script/schema/standard.yaml` would resolve
`../../script/schema-init/init_schema.py` against the wrong base and either
miss the file entirely or (worse) resolve to a wrong-but-existing path one
directory off. **Fix**: rename the parameter and use the manifest's own
parent directory, not the directory `register_standard` was called with:

```rust
pub fn register_standard(standard_path: &Path, knowledge_db_path: &Path) -> Result<RegisterStandardResult> {
    let manifest_path = resolve_manifest_path(standard_path)?;  // §2.14's two-location check
    let manifest_dir = manifest_path.parent().unwrap_or(standard_path);
    let manifest_content = std::fs::read_to_string(&manifest_path)...;
    let manifest: StandardManifest = serde_yaml::from_str(&manifest_content)...;
    // every existing call resolve_location(standard_path, ...) becomes:
    // resolve_location(manifest_dir, ...)
    ...
}

fn resolve_location(manifest_dir: &Path, location: &str) -> Result<String> {
    let candidate = Path::new(location);
    let resolved = if candidate.is_absolute() { candidate.to_path_buf() }
                   else { manifest_dir.join(candidate) };
    ...
}
```

`standard_path` (the original argument) is kept only for `standard_registry.source_path`/
`standard.name`-adjacent bookkeeping (§2.1, §2.13) — never for path
resolution again. Only caller is `register_standard` itself
(confirmed — `grep -rn resolve_location crates/` has exactly one call
site outside its own tests), so this is a contained rename, not a
cross-crate breaking change. `register_standard_globally` (§2.1) must use
the identical `resolve_manifest_path`/`manifest_dir` logic, not a
second, divergent implementation — one function, both callers.

### 8.4 Manifest catch-all — diff against known fields, not `#[serde(flatten)]`

Rev. 2's `#[serde(flatten)] extra: serde_json::Map<String, serde_json::Value>`
is riskier than it looks: `serde(flatten)` support under `serde_yaml`
depends on `serde_yaml`'s own `Deserializer` correctly implementing serde's
map-flattening protocol, which has had version-dependent rough edges
historically — betting a silent-data-loss fix on that interaction adds a
new fragility instead of removing one. **Simpler, more robust, avoids the
flatten mechanism entirely**: parse the manifest twice — once into the
typed `StandardManifest`, once into a bare `serde_yaml::Mapping` — and
diff the second against the first struct's known field names:

```rust
const KNOWN_FIELDS: &[&str] = &[
    "name", "category", "subcategory", "extends", "smoke_test",
    "scripts", "prompts", "usecases", "custom_tables", "domains", "assets", "templates",
];

let raw: serde_yaml::Mapping = serde_yaml::from_str(&manifest_content)?;
let extra: serde_json::Map<String, serde_json::Value> = raw.iter()
    .filter_map(|(k, v)| {
        let key = k.as_str()?.to_string();
        if KNOWN_FIELDS.contains(&key.as_str()) { return None; }
        let json_v = serde_json::to_value(v).ok()?;   // Value -> Value via
        Some((key, json_v))                            // re-serialize through
    })                                                  // serde_json, sidesteps
    .collect();                                         // any direct-Deserialize risk
```

One extra parse of a small YAML file, once per registration — negligible
cost, zero new dependency, and no reliance on cross-crate `flatten`
compatibility. `classify_repo: { min_doc_words: 200 }` round-trips through
`serde_yaml::Value → serde_json::Value` cleanly for every case this
codebase's manifests actually use (scalars, nested maps, lists) — the only
YAML constructs that don't map 1:1 to JSON (multi-document streams, non-
string map keys) don't appear in any manifest on disk today, and aren't
worth defending against speculatively.

### 8.5 `depends_on` validation — a second pass, after every usecase name is known

`register_standard`'s existing validation (`register_standard.rs:159-215`)
checks a step's `script`/`prompt` reference *while* inserting that step —
valid there because `script_ids`/`prompt_ids` (built in an earlier pass,
lines 129-149) are fully populated before any usecase loop starts.
`depends_on` names *other usecases*, which aren't known until the full
`usecases:` list has been walked at least once. **Fix**: build a
`usecase_names: HashSet<&str>` in the same first pass that inserts
`usecase` rows (before any `step`/`domain_id` resolution), then a second
loop over `manifest.usecases` validates each `depends_on` entry against
that set — unknown name bails with the same "usecase 'X' depends_on
unknown usecase 'Y'" phrasing as the existing unknown-script/unknown-prompt
errors (`register_standard.rs:183-188`, `201-206`), before any `step` rows
for that usecase are written (fail-fast, no partial state, same discipline
as everywhere else in this function).

### 8.6 `domain`/`usecase.domain_id` — two passes, domains first

`domain.id` must exist before any `usecase.domain_id` can reference it.
Current single-loop-over-usecases shape (`register_standard.rs:152-215`)
inserts `usecase` rows one at a time, interleaved with their `step` rows.
**Fix**: insert every `domain` row *before* that loop starts (a new pass
right after the existing script/prompt-id passes, §8.5's `usecase_names`
pass can run in the same loop), building a `domain_ids: HashMap<String, i64>`
the usecase-insertion loop then consults when a `UsecaseDecl.domain` field
is present — exactly the same `HashMap<String, i64>` pattern
`script_ids`/`prompt_ids` already use (lines 129, 139), applied one level
earlier in the function.

### 8.7 `standard` table — one more line in `delete_existing`

`delete_existing` (`register_standard.rs:236-262`) currently issues 6
`DELETE`s in dependency order (`step_script` → `step_prompt` → `step` →
`usecase` → `custom_data_tables` → `prompt` → `script` — 6 statements
total against 6 tables). Adding the local `standard` table (§2.13) means
one more line, same place, same order-doesn't-matter-here reasoning
(no FK from `standard` to anything else being deleted in this function):

```rust
conn.execute("DELETE FROM standard WHERE name = ?1", rusqlite::params![standard])?;
```

Placed anywhere in the existing function body (no ordering constraint —
unlike the FK-cascading deletes above it, `standard` has no dependents
being cleaned up in the same call), followed by the `standard` row's own
`INSERT` alongside the function's existing script/prompt/usecase inserts.

### 8.8 `CORE_MIGRATIONS` — needs real version-gating, not just a longer array

Verified `CORE_MIGRATIONS` (`crates/registry/src/core_schema.rs:9`) is
`&[CORE_V1]` — one entry — and has exactly two call sites
(`register_standard.rs:119`, `step_execution.rs:135`'s test setup), both:

```rust
for m in registry::core_schema::CORE_MIGRATIONS {
    conn.execute_batch(m)?;
}
```

**No version check, ever** — every statement in `CORE_V1` re-runs on
*every single call* to `register_standard`/`run_script_step`'s test setup.
This has been silently safe only because every statement in `CORE_V1` is
`CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF NOT EXISTS` — idempotent by
construction. **This breaks the moment `CORE_V2` adds `ALTER TABLE
execution ADD COLUMN git_detail_id ...` (§2.10) or `ALTER TABLE usecase ADD
COLUMN domain_id ...` (§2.11)** — SQLite's `ALTER TABLE ADD COLUMN` has no
`IF NOT EXISTS` form, so the *second* `register_standard` call against an
already-migrated `knowledge.db` fails outright with "duplicate column
name." This is a real bug this proposal would introduce, not merely a
style gap the first review's "grow the array" framing implied. **Required
fix, beyond what was asked**: give `knowledge.db`'s migration runner the
same version-gate `RegistryDb::run_registry_migrations`
(`registry_db.rs:50-70`) already has for `registry.db` — read
`SELECT COALESCE(MAX(version),0) FROM _schema_version` (the table `CORE_V1`
already creates but nothing reads today), apply only `CORE_MIGRATIONS`
entries past that version, `INSERT INTO _schema_version` after each. Both
call sites (`register_standard.rs`, `step_execution.rs`'s test helper)
switch from the current unconditional loop to this gated one — a small,
mechanical change, but one that must land *before* Phase 3's `ALTER TABLE`
statements are added, not alongside them, so there's a version between
"idempotent-by-luck" and "idempotent-by-construction" to bisect against if
anything goes wrong.

### 8.9 Git capture — a separate helper, not inlined into `record_execution`

`record_execution` (`step_execution.rs:120-126`) is currently one
`INSERT`, no I/O. Keep it that way — extract git capture into its own
function, called just before:

```rust
fn capture_git_detail(conn: &Connection, repo_root: &Path) -> Option<i64> {
    let sha = run_git(repo_root, &["rev-parse", "HEAD"])?;
    let branch = run_git(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    let dirty = !run_git(repo_root, &["status", "--porcelain"])?.trim().is_empty();
    upsert_git_detail(conn, repo_root, &sha, &branch, dirty).ok()
}
```

`record_execution` gains one line (`let git_detail_id =
capture_git_detail(conn, repo_root);`) feeding the new column — the
function's own shape and its existing unit tests
(`step_execution.rs:170-228`) stay otherwise unchanged. `run_git` returns
`None` on any failure (not a git repo, `git` not on `PATH`) — best-effort,
never a hard error, per §2.10.

### 8.10 `_git` envelope injection — exact diff to `run_script_step`

`run_script_step` (`step_execution.rs:56-57`) currently:

```rust
let in_path = std::env::temp_dir().join(format!("samgraha-step-in-{}.json", uuid::Uuid::new_v4()));
std::fs::write(&in_path, serde_json::to_string(input_json)?)?;
```

Becomes:

```rust
let mut enriched = input_json.clone();
if let (Some(obj), Some(git)) = (enriched.as_object_mut(), git_detail_for(repo_root)) {
    obj.insert("_git".into(), serde_json::json!({
        "commit_sha": git.commit_sha, "branch": git.branch, "dirty": git.dirty,
    }));
}
let in_path = std::env::temp_dir().join(format!("samgraha-step-in-{}.json", uuid::Uuid::new_v4()));
std::fs::write(&in_path, serde_json::to_string(&enriched)?)?;
```

Noted plainly: this changes the envelope every deterministic step's script
sees, for every standard, unconditionally — an additive key, not a
breaking one (`json.load(...)["_git"]` is simply unused by any script that
doesn't ask for it, and Python's `json.load` never errors on unrecognized
keys), but worth stating outright rather than leaving implicit, since it's
the one change in this proposal that touches every script contract at
once rather than being opt-in per standard.

### 8.11-8.13 MCP tool surface — nine new handlers, one new service function, one non-standard subprocess runner

Nine new dispatch-table entries (`adapter.rs:48-56` currently has 9; this
doubles it) — six are thin read-only queries against `standards_db`/local
`knowledge.db` tables, unremarkable relative to the existing 9. Two need
their own new logic, called out specifically:

- **`seed_standard`** — new service function (`crates/services/src/`, not
  just an adapter handler — same "adapter handlers stay thin, logic lives
  in `services`" split every existing handler already follows). Algorithm:
  resolve the target usecase's `depends_on` transitively (plain DFS,
  `pcems_2026`'s real graph is at most 2 deep — `classify-repo` depends on
  `schema-init`, nothing depends on more than one hop today, so a full
  general-purpose topological sort is more machinery than the evidence
  calls for; DFS with a visited-set cycle check is the right-sized
  algorithm here), filter the walk to `driver: samgraha` entries only,
  fail loudly on a cycle (visited-but-not-yet-finished node reached again),
  then call the existing `run_script_step` once per resolved usecase's
  steps, in dependency order. No new execution primitive — this is
  sequencing logic on top of what already exists.
- **`check_usecase_complete`** — the one tool in this proposal that
  doesn't use `run_capability_script`'s `--repo-root/--in/--out` contract
  (§2.4.2 already says this; confirmed here as an implementation
  consequence, not just a design note): it needs its own
  `std::process::Command` invocation, checking exit code only, with
  caller-supplied `extra_args` appended verbatim to the resolved script
  path. Kept deliberately separate from `run_capability_script` rather than
  bent to fit it — forcing `script/verify/`'s real `--db-path --paper-id`
  convention through the JSON-envelope contract would require changing
  `_common.py:verify_main` (out of scope, §3/§6) or building a
  contract-translation shim (more code than a second, honestly-different
  subprocess runner).

### 8.14 Test path — env var, not a hardcoded absolute path

Phase 5's integration test (§4, §5) needs `Kriti/samgraha/system/academic/pcems_2026`'s
real path, which isn't portable across machines/CI:

```rust
let pcems_path = std::env::var("SAMGRAHA_TEST_PCEMS_2026_PATH")
    .map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from("/home/dell/PycharmProjects/Kriti/samgraha/system/academic/pcems_2026"));
if !pcems_path.exists() {
    eprintln!("skipping pcems_2026 integration test: {} not found (set SAMGRAHA_TEST_PCEMS_2026_PATH)", pcems_path.display());
    return;
}
```

Skips (not fails) when the path isn't present — this test depends on a
sibling repo (`Kriti`) that a CI runner or a different developer's machine
may not have checked out at all; failing the whole suite over a missing
sibling repo would be a worse outcome than a visibly-skipped test naming
exactly which env var fixes it.
