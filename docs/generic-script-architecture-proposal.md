# Generic Script-Dispatch Architecture Proposal

**Supersedes**: `docs/crates-refactor-proposal.md` §2-12 (the schema-driven,
DB-table-shaped inheritance model). See §7 below for exactly what survives
from that doc and what doesn't.

**Status**: SAMGRAHA-SIDE COMPLETE, verified against the code (not just the
implementer's own report). Phases A→D built the shape; an independent
review pass (Phase E, §10) found the gating mechanism's write path was
missing entirely — `check_phase_prerequisites` could read `script_runs`,
but nothing ever wrote to it, so a gated phase could never actually unblock.
Two `standard_id` stubs and one `current_head` stub were also hardcoded
(marked `// TODO`, not hidden). All fixed and covered by new tests
exercising the write-then-read round trip directly — 264 audit tests + 8
mcp tests pass, `cargo build --workspace` clean. System-side scripts
(`calculate.py`, `scaffold.py`, etc.) are out of scope — provided by each
system's owner per §8.1/§8.2 envelope specs.

See §7 for what survives from the inheritance proposal and what doesn't.

---

## 1. Problem Statement

### 1.1 The concrete trigger: academic-class inheritance saves almost nothing

Comparing `eswa_journal` and `pcems_2026` (both `academic` class, both meant
to share a `base_academic`) file-by-file:

| File | Shared? |
|---|---|
| `calculation/summary/score_bands.yaml`, `trend.yaml` | Identical |
| `calculation/summary/final_score.yaml`, `calculation/validation/scoring_validation.yaml` | 99% identical — only a system-name string differs |
| `calculation/semantic/document.yaml` | Identical structure (lines 1-9), fully different domain lists / quality checks / examples (lines 11-22) |
| `plan/core/loop.yaml` | Identical skeleton (thresholds, scoring/validate/tier_gate blocks), fully different `within_tier_ordering`, `evidence`, `generate` blocks |
| `plan/core/tiers.yaml` | **Fully different** — confirmed by `diff`: zero shared lines beyond YAML key names. eswa's tiers are `[problem-definition, related-work]`/`[methodology]`/...; pcems's are `[introduction, methodology]`/`[findings]`/... — different domain vocabularies entirely |
| `00-domain-relationships.md` | **Fully different** — different domains, different edges, same template shape only |

Two files are genuinely shareable as-is. Everything else is either
"same shape, different values" (needs templating, which file-override
inheritance doesn't do) or "no shared shape at all." A `base_academic`
abstract base — as `crates-refactor-proposal.md` §8.2 proposed — would save
2 files out of 9+ and add inheritance-chain complexity for that. Not worth it.

### 1.2 The deeper problem this exposes

The dev class (`base_dev`, `electron_dev`, `fastapi_dev`, `react_dev`,
`rust_dev`) *does* share real content — that's why inheritance helped there.
Academic class doesn't, because **academic systems don't share content, they
share a role** (generate paper sections, audit them semantically, no
competition). Forcing both classes through the same "base tree + file
override" mechanism assumed all systems within a class look like the dev
class. They don't.

Zoom out further: `schema/knowledge-hub/knowledge-hub-loader.py` is 11 fixed
passes, each hardcoding how to parse a specific shape — `domains`,
`section_catalog`, `rules`, `calculation_rules`, `plan_settings`, etc. That
shape *is* the documentation-audit domain model, written directly into
samgraha's own Python source. A system for a genuinely different domain —
film-production tracking, game-design docs, anything that isn't
"tiered documents with deterministic+semantic audit and a scoring formula" —
doesn't produce these 11 shapes at all. Building that system today would
mean changing samgraha's loader and schema, not just adding data. That's
backwards for a tool whose whole premise is "systems are data, samgraha is
generic infrastructure."

### 1.3 What this means for the inheritance work

Inheritance (§13.7 of the other doc: `extends`/`drops`, arbitrary depth) is
still useful — real, provably: the dev class had byte-identical files across
5 systems. It's a mechanism for **deduplicating close siblings**, not a
mechanism for **generalizing across domains**. Keep it, scoped to that job.
Don't build `base_academic` on the strength of "academic is a class too" —
build it only if/when two academic systems actually share enough to be worth
a base (right now, that's 2 files, not worth an abstract system).

---

## 2. Proposed Architecture

### 2.1 Core principle

samgraha's own source code contains **zero domain-shape knowledge**. It
exposes generic MCP tools — `scaffold`, `validate`, `calculate`, `report`,
`plan-generation` — each of which dispatches to a script the *system*
provides. samgraha's job: discover the right script, run it under a fixed
contract, store/relay whatever comes back. The system's job: know what a
"domain," a "tier," a "score," a "report" mean for itself, and implement
that as an executable script, not as rows in a schema samgraha had to be
taught to parse.

### 2.2 This mechanism already exists — for one capability

`crates/audit/src/check_runner.rs::resolve_check()` already does exactly
this, today, for deterministic checks:

- 5-tier discovery (`check_runner.rs:49-101`): `samgraha.toml` override →
  repo-level `scripts/` → local synced `.samgraha/scripts/` → global
  system-default scripts → Rust-native placeholder
- Fixed CLI contract (`common/src/env.rs:266-291`,
  `run_check_script()`): script gets `--repo-root`, `--repo-fingerprint`,
  `--out <path>`; writes a JSON result to `--out`
- Fixed result shape samgraha parses back (`check_runner.rs:174-201`):
  `status: pass|fail|not_applicable`, `evidence: [...]`, `metrics: {...}`

samgraha doesn't know or care *how* a check script decides pass/fail — it
just runs it and reads the envelope. That's the entire principle this
proposal generalizes to the other four capabilities. Nothing here is
speculative; it's "do what `check_runner.rs` already does, for `calculate`,
`report`, `scaffold`, and `plan-generation` too."

**Script language is already unconstrained, per-script, not tied to repo or
standard language.** `script_command()` (`common/src/env.rs:90-120`)
dispatches by file extension: `.ps1` → PowerShell, `.sh` → sh/bash, `.py` →
python, `.js` → node, anything else (or no extension) → run directly as an
executable (covers a compiled Rust/Go/etc. binary). A `rust_dev` system's
scripts don't have to be Rust — pick whatever's convenient per script, mix
languages freely across a system's own script set. `probe_script()`
(`check_runner.rs:103-134`) also tries the platform-native shell first
(`.ps1` on Windows, `.sh` elsewhere) then falls back to the other, so a
system can ship both for cross-platform coverage, or just use `.py`/`.js`
since those need no per-platform variant at all. Not covered today: `.ts`
— no dispatch case exists for it (would need requiring pre-compiled `.js`,
or wiring in `deno`/`bun`/`ts-node`).

### 2.3 The five other capabilities, same contract shape

| Capability | Today (DB-schema-shaped) | Proposed (script-shaped) |
|---|---|---|
| `validate` | Already script-based (§2.2) — no change | — |
| `calculate` | `calculation_rules`/`calculation_inputs`/`score_bands` rows; Rust reads them and computes | System's own `calculate` script reads raw `audit_results` (still stored generically, see §2.4) via `--repo-root`/`--out`-equivalent args, returns a score JSON |
| `report` | `templates` rows + Rust template-fill logic | System's own `report` script reads stored audit data + its own template files, writes the rendered report (JSON and/or Markdown) to `--out` |
| `scaffold` | Nothing today (§1.2's `templates/generation/*.md` are static, not executed) | System's own `scaffold` script creates the file/section skeleton for a document or domain |
| `plan-generation` | `plan_scenarios`/`plan_settings` rows drive a fixed state machine | System's own script takes current repo/doc state as input JSON, returns "here's what needs generating" as a JSON task list — the semantic (LLM) fill-in step consumes that list per section, one section at a time |
| `init` | Nothing today | System's own `init` script — self-describing discovery, not registration (§2.6). Distinct from Pass 0 |

### 2.6 `init` — a system describing itself, separate from registering itself

Pass 0 (`register_standard`/`sync_standards`) stays as-is — that's
mechanical registration (get this system's rows into `standards.db`), and
nothing about it assumes documentation-audit shape today, so it doesn't
need replacing. What's missing is a *different* thing: a way for a system
to tell the user (and MCP) **what it supports and how to use it**, once
it's registered. That's a new `init` script every system provides:

- Enumerates which use cases/workflows the system supports (§7.1's four,
  or whatever a given system defines)
- Returns a **phase-wise plan** for running them — not just a task list,
  an ordered sequence of phases, each of which can declare its own
  pre-script and post-script hooks
- Marks which phases are **semantic** — the LLM has to do the work for that
  phase, driven through MCP tool calls, not a deterministic script the
  system can just run unattended — vs. which phases are plain script steps
- This phase-wise plan is exactly what §7.3's "run this first" mechanism
  points at: when MCP blocks an operation on a missing precondition, it's
  naming a phase (or its pre-script) from this same plan, not inventing a
  separate instruction each time.

### 2.4 What stays generic in samgraha (genuinely domain-agnostic today, keep as-is)

- `systems`/`standards` registry tables — name, description, `is_default`.
  No domain knowledge here now; none needed.
- Raw result storage (`audit_results`-equivalent, `script_cache.result_json`)
  — the README's own design rule already says this stays JSON "because its
  shape is inherently per-rule/per-check variant" — that principle just
  needed to be applied to more than one capability. Store: timestamp,
  system, capability, input reference, output JSON. Never interpret the
  JSON's internal shape samgraha-side.
- `resolve_check`'s 5-tier script discovery — reuse verbatim for the other
  4 capabilities, it's already capability-agnostic in design (just named
  for "checks" today).
- Inheritance (`system.yaml`/`extends`/`drops`, scoped per §1.3) — for
  deduplicating near-identical sibling systems' *scripts and config*, same
  as it deduplicated files today. A system's `script/calculate.py` can be
  inherited/overridden exactly like `plan/core/loop.yaml` is today.

### 2.5 What moves out of samgraha, into each system

- All 11 of the current loader's pass-specific parsing logic
  (`knowledge-hub-loader.py`'s domain/section/rule/calculation/plan-setting
  shape knowledge) — becomes each system's own script logic, not samgraha
  Python source.
- The loader itself shrinks to: Pass 0 (systems/standards registry — stays,
  §2.4) plus a generic "run this system's `register` script, store whatever
  JSON it returns as opaque metadata" hook. Whether the other 10 passes get
  deleted outright or phased out gradually is an implementation-order
  question, not an architecture one — see §5.

---

## 3. Worked example: `calculate` for `rust_dev`

Today: `calculation_rules`/`calculation_inputs`/`score_bands` rows, computed
by Rust code in `crates/audit/src/calculation.rs` that has to know the
generic "deterministic × semantic × document × section, weighted sum,
banded rating" shape.

Proposed: `rust_dev/script/calculate.py` (or `.ps1`/`.sh` — same
multi-shell probing `probe_script()` already does at `check_runner.rs:103`)
gets invoked with `--repo-root`, `--out <path>`, plus a new
`--audit-results <path-or-inline-json>` arg carrying whatever's already
stored for this repo. The script does whatever math `rust_dev` wants —
weighted sum, something else entirely for a future system — and writes
`{"final_score": 82, "band": "Good", "breakdown": {...}}` to `--out`.
samgraha stores that JSON, shows it, never re-derives it. If `rust_dev` and
`fastapi_dev` want the identical formula, `fastapi_dev`'s `system.yaml`
`extends: rust_dev`'s script the same way it extends its `loop.yaml` today
— dev-class-style dedup, unchanged mechanism, just pointed at a script file
instead of a YAML file.

---

## 4. Open Questions (flagging, not answering — resolve before implementation)

1. ~~Exact envelope schema per capability~~ — **CONFIRMED direction**: one
   generic, samgraha-owned schema per capability (same pattern as
   `validate`'s `status`/`evidence`/`metrics`), containing no domain
   knowledge — every system's script for that capability must conform to
   it, none of them get to vary the envelope shape. Still need to actually
   write the 4 remaining envelope specs (`calculate`/`report`/`scaffold`/
   `plan-generation`) — that part's unstarted, just the *approach* is settled.
2. ~~Semantic-fill ownership~~ — **RESOLVED**: samgraha never runs an LLM
   itself. It exposes MCP tools (task list, where to write the result);
   whatever's on the other end of the MCP session — Claude Code, opencode,
   any other client — does the actual semantic work with whatever model
   it's using. samgraha has no visibility into or control over which model
   answers a "semantic phase" (Claude vs. opencode, Opus vs. Sonnet), so
   content quality on any semantic step is a property of the calling
   client, not something samgraha's contract can normalize or judge. Corollary
   for §2.6/§7: a phase marked "semantic" is just a signal to the caller
   "this step needs an LLM to fill it in," and samgraha's job stops at
   accepting whatever the caller writes back — same passive role for the
   §7.7 report-commentary step and any other semantic sub-step, not just
   the main `plan-generation` task list.
3. ~~Migration order~~ — **RESOLVED**: no migration, no parallel-run
   transition period. `standards.db` holds no permanent data — it's a
   rebuildable cache only (`crates-refactor-proposal.md` §7.4, reaffirmed
   here). Converting a system to scripts means ripping out its
   DB-schema-driven rows for that capability immediately and rebuilding the
   cache from the new script path — no coexistence window needed.
4. ~~Pass 0's fate~~ — **PARTIALLY RESOLVED**: Pass 0 (register/sync) stays
   as-is, unreplaced — it's mechanical registration, already generic. What's
   new isn't a replacement for it, it's an addition: each system provides
   its own `init` script (§2.6), separate from registration, that tells the
   user what use cases it supports and returns a phase-wise plan (with
   pre/post-script hooks per phase, and semantic-vs-scripted phases marked)
   for running them. Still open: the exact shape of that phase-wise plan
   JSON, and how `init` composes with #6 below (the "run this first" MCP
   response should be pointing at a phase from `init`'s plan, not a
   separately-invented instruction — needs one shared design, not two).
   Also still open, per §7.2: the DB table shape for the semantic
   plan-generation input (moved off the filesystem, onto DB — see §7.2).
5. ~~Per-domain multi-parent inheritance~~ — **RETRACTED, not a gap**.
   Clarified: the build-tool example is `dev → build_tool → rust`, a plain
   2-hop single-parent chain (`rust` extends `build_tool` only;
   `build_tool` extends `dev` only, and only if a build-tool base is even
   worth having). Not multi-parent at all — already covered by the
   depth-agnostic single-`extends:`-per-node mechanism
   (`crates-refactor-proposal.md` §3.1/§13.7), which already supports
   arbitrary chain depth. No new design needed here.
6. ~~The "run this first" MCP contract~~ — **RESOLVED, with one new
   deliverable spun out below.** Workflow/step/order data is already in DB
   (the existing `plan_settings`/`plan_scenarios` shape, read by `init`'s
   plan per #4) — so MCP isn't inventing "what should happen next," it
   already has that in DB and just checks it. The plan itself is rendered
   to the user as Markdown the same way any other document is: template
   (DB) + workflow (DB) + script, no new rendering mechanism needed. And
   this isn't special to plan-generation — it's universal: *any* step that
   depends on a prescript having already run gets the same check, because
   the workflow definition already states step order/dependencies, so MCP
   has what it needs to flag "run X first" at any point, not just at the
   start.

   **New deliverable this surfaces**: script outputs can go **stale**, and
   that's not uniform across scripts. Example: an implementation-audit
   prescript ran 2 days ago — its output exists, but the repo may have
   changed since (new commits, edits), so that old output may no longer be
   trustworthy. Needs:
   - A **run-tracking record** per script execution: which script, which
     repo, when it ran.
   - An **expiry rule** per script — time-based (e.g. "valid 24h"),
     event-based (e.g. "valid until HEAD changes"), or none (never expires).
     **The system defines this rule, not samgraha** — consistent with every
     other resolved item: samgraha tracks generically (when did X run, is
     it still valid per the rule it was given), it doesn't decide what
     "valid" means for any particular script.
   - MCP's "run this first" check now has two failure reasons to
     distinguish: the step never ran at all, or it ran but its output has
     expired — both should produce the same kind of blocked-response,
     naming which case it is.

   Still undefined: the run-tracking table's shape, and the concrete syntax
   a system uses to declare a script's expiry rule (part of `init`'s plan
   definition, most likely, but not spec'd).
7. ~~Domain cardinality~~ — **RESOLVED**: not samgraha's concern, no field
   needed anywhere in samgraha's domain model. It's purely a workflow
   composition choice each system makes for itself, using the same two
   building blocks as everything else: a semantic phase, then a script
   phase. For "feature," the system's own workflow runs a semantic-analysis
   phase first (LLM, per #2 — reads the vision/PRD, decides how many
   features and what they're called), saves that result to DB, then a
   script phase reads it back from DB and fans out — runs the standard
   §7.4 4-step pipeline once per resolved instance. "Vision" just has no
   such phase in its workflow, because it's always exactly one. Singular
   vs. multi-instance is a difference in which phases a system's `init`
   plan (#4) declares for that domain, not a property samgraha tracks.
8. ~~Backfill/dependency-check logic ownership~~ — **RESOLVED**: the
   system's own scripts own this, not samgraha. Scripts must align with
   samgraha's generic schema/contract (the envelope shapes from §2.6), but
   checking "has my prerequisite run yet, and if not, trigger it" is the
   data system's responsibility to implement and validate — samgraha
   doesn't provide a shared dependency-resolution helper. Consistent with
   §1.2's principle: samgraha stays out of anything that requires knowing
   what a step's output *means*, and "is this prerequisite satisfied" is
   already a domain-specific judgment call per system, not a generic one.
9. ~~One MCP tool or five?~~ — **RESOLVED**: both. The 5 named capabilities
   (`validate`/`calculate`/`report`/`scaffold`/`plan-generation`) each get
   their own dedicated MCP tool — clearer, more discoverable for clients.
   Alongside them, one generic `run_system_script(system, capability,
   params)` tool exists too, for whatever capability gets invented later.
   New capability types go through the generic tool without needing a new
   dedicated MCP tool added (and a version bump to the MCP surface) every
   time — the dedicated 5 are a convenience layer over the same underlying
   dispatch, not a different mechanism.
10. ~~Archive/rotation ownership~~ — **RESOLVED**: the `report` script
    itself owns this, provided by the standard, same as everything else —
    not a separate samgraha capability or a samgraha-provided default. If a
    standard's `report` script doesn't implement archive-rotation, there's
    no fallback behavior from samgraha — the new report simply overwrites
    the old "latest" one in place. Archiving is opt-in, per-standard, and
    entirely inside the script's own logic.

---

## 5. Suggested Piloting Order (not a full phase plan — write one once §4 is answered)

**Split responsibility — this isn't all samgraha-side work.** samgraha
builds the dispatch plumbing (unavoidable — no data system can build generic
infra for itself). The actual `calculate.py` script content is the data
system's job (whoever owns that system), per §1.2/§2.1's core principle.
samgraha writing that script too would defeat the pilot's real purpose: the
thing actually worth proving is whether §8.1's envelope spec is clear enough
for someone *outside* samgraha to implement against without extra
hand-holding — not whether samgraha can write a script for itself.

1. Pick **one** capability (`calculate` is the smallest surface — see §3)
   and **one** system (`rust_dev` — already has inheritance shipped, small
   blast radius).
2. **samgraha side**: wire a generic `run_system_script(system, capability,
   params)` MCP tool that reuses `resolve_check`'s discovery-tier logic
   under a new name (or a `capability` parameter added to the existing
   resolver — check which reads cleaner against the current `CheckSource`
   enum before deciding). **Data-system side**: hand the system owner the
   §8.1 spec; `rust_dev/script/calculate.py` gets written there, not here.
3. Prove it end-to-end against `rust_dev`, compare output to the current
   DB-computed score for the same repo state.
4. Recommended order after that (not committed as a full rollout plan —
   just the sequencing rationale): **`scaffold` next** — pure filesystem
   operations, no template or DB-schema dependency, lowest-risk second
   proof point. **`report` third** — depends on templates and the
   archive-rotation decision (§4 item 10). **`plan-generation` last** — the
   most complex capability (multi-stage, semantic-dependent, needs the
   DB-schema plan-input table from §7.2/§4 item 4 designed first), and
   benefits most from the other three patterns already being proven out.

**Cost callout, stated plainly rather than glossed over**: `calculate.py`
for `rust_dev` (§3) is genuinely small. `report`/`plan-generation` scripts
for a system with 16 domains are not — today, a 16-domain report is
whatever generic Rust template-fill code + declarative YAML rows already
produce "for free"; under this proposal, that same rendering logic has to
be written *per system*, in a script. That's a real per-system cost that
scales with domain count, not something the piloting order makes free —
piloting only proves the *pattern* works before committing to writing it
16 times. Mitigation worth building alongside the pilot: a small shared
script-helper library (e.g. a `samgraha_script_helpers` package for
whichever languages get used — Python first, given `calculation.rs`'s
current logic is the most likely first port) that handles common
boilerplate (fetching `--in` JSON, writing the §8.1/§8.2 envelope, basic
template-fill from a Jinja-style template + DB rows). Contains no domain
knowledge — same category as `check_runner.rs`/`env.rs` already being
reusable plumbing every script benefits from without samgraha needing to
know what a script computes.

---

## 6. What This Does *Not* Solve

Doesn't remove the need for *some* fixed contract systems must follow — a
system still has to provide scripts with the right CLI args and JSON
envelope shape, same as it has to provide `system.yaml` today. Genericity
here means "samgraha doesn't know what's in a domain," not "a system needs
zero structure." Film/game systems still need to write real scripts; this
proposal just stops samgraha's own source from needing to know what those
scripts compute.

---

## 7. Workflow-Driven Generation & Audit Pipeline

Captures the fuller model described in session discussion: each capability
in §2.3 isn't a single script call, it's a small pipeline, and systems don't
just provide scripts — they provide a **workflow** (how those scripts chain
together per use case) alongside them. Understood parts below; genuinely
open parts moved to §4's list at the end.

### 7.1 Workflow = which use case a repo is in, drives what needs generating

Four base use cases per system, matching what already exists as
`plan_scenarios`' repo-state × doc-state × tier × step matrix (no new
concept here — confirms this table's shape survives the pivot, just gets
*read by* scripts instead of *interpreted by* Rust code):

1. New repo (no code, no docs)
2. New repo with some existing documentation
3. Existing repo, no documentation
4. Existing repo, implementation only

A workflow is data (template + use-case definition), same as today. What
changes: instead of Rust code walking `plan_scenarios` rows to decide what
happens next, a script reads the workflow + template rows and drives the
actual generation.

### 7.2 Plan generation — two stages, one of them durable DB state

1. **Semantic determination** — "what needs generating for this repo, in
   this workflow" is itself a semantic (LLM) step, not deterministic. Its
   output is saved to **DB, not a repo file** (decided — reversed from an
   earlier file-based sketch): same rationale as everything else stored
   generically in samgraha's schema (§2.4) — a script can query it directly
   rather than needing to locate and parse a repo-local file path, and it's
   durable/inspectable the same way DB rows already are. Needs its own
   table (shape unspecified — see §4).
2. **Plan rendering** — a script combines {plan template (DB), workflow
   definition (DB), the semantic input from step 1 (DB)} and produces the
   actual plan Markdown file in the repo — the *rendered output* is a repo
   file, same as any other generated document; only the semantic
   intermediate result moved to DB.

### 7.3 Hard precondition: MCP must be able to say "run this first"

If the semantic input from §7.2 step 1 doesn't exist yet, the standard
can't be used at all — this isn't a soft warning, it's a blocking
precondition. MCP's response for an operation blocked this way needs a
distinct shape (not a bare error, not silent partial success) that names
the exact script/command the user needs to run and why. This is a new
protocol-level contract, not just a script-contract question — see §4.

### 7.4 Document generation — a 4-step pipeline, order-tolerant

Same shape for every document-producing domain (worked example: Vision):

1. Scaffold the file/folder skeleton
2. Add section headings inside the file (a *separate* step from file
   scaffolding — the file exists but sections are still empty stubs)
3. Semantically generate each section's content, section by section, saved
   to DB
4. Assemble the final document from the DB's stored section content

**Order-tolerant by design, not by accident**: running step 3 without step
2 having run, or step 4 without step 3, must not fail outright. Each step
checks whether its prerequisite's data already exists; if not, it triggers
that prerequisite first rather than erroring. The "optimized" path (DB data
first, then render) is one valid order among several — the system has to
handle all of them without breaking, by treating each step as
idempotent-and-backfilling rather than assuming strict pipeline order. This
is a property each capability script needs to implement (check-then-act),
not something the MCP contract can enforce for them.

### 7.5 Singular vs. multi-instance domains

Vision is always exactly one document. Feature-like domains aren't — the
count isn't known upfront. For those, an **enumeration step** runs first:
read dependent/upstream data (e.g. Vision's or a PRD's content) to identify
how many instances exist and what each is called, *then* run the §7.4
4-step pipeline once per instance. This is a real fork in the domain model:
a domain needs a declared cardinality (singular vs. multi-instance), and
multi-instance domains need an enumeration script that runs before the
per-instance pipeline, not instead of it.

### 7.6 Audit pipeline

Once a domain's document data exists in DB (from §7.4):

- Semantic audit and deterministic audit each run per their own scope
  (whole-document, section-by-section, or cross-section — already the
  `rules.scope` shape today, unchanged) and save raw results to DB.
- Separate report templates and separate scripts for deterministic vs.
  semantic audit reports.
- Once a domain's audit is complete, a script pulls all of that domain's
  JSON results from DB and renders the full report from the report
  template — report rendering is its own step, downstream of and separate
  from running the checks themselves (confirms §2.3's `report` capability
  is genuinely distinct from `validate`, not a mode of it).

### 7.7 Calculation, reporting, and archive rotation

- A `calculate` script computes the score/rating from stored data +
  template (§2.3, unchanged).
- Report rendering can include a small amount of semantic content too — a
  short LLM-authored commentary blurb — which also gets stored in DB, not
  just embedded in the rendered file. So `report` isn't purely
  deterministic rendering; it has its own small semantic-input step,
  mirroring §7.2's pattern at a smaller scale.
- **Archive/latest rotation**: when a new report is generated, the current
  "latest" report moves to an archive/history location before the new one
  takes its place. This isn't hypothetical — this exact pattern
  (`history/<rev>-rev<rev>.json` / `.md`) already exists as live output
  elsewhere in this repo's own test fixtures, so the convention has
  precedent worth reusing rather than inventing fresh.

---

## 8. Concrete Specs (drafted for implementation verification)

Everything below is a **draft**, not a final contract — written so an
implementation has something concrete to check itself against, not so it
can't change. Each spec reuses `check_runner.rs`/`run_check_script`'s
existing conventions (`--repo-root`, `--repo-fingerprint`, `--out`, JSON
envelope) wherever it fits, rather than inventing a new shape per capability.

### 8.1 Envelope specs — `calculate`, `scaffold` (value-returning)

Same CLI args as `validate` today (`common/src/env.rs:266-291`):
`--repo-root <path> --repo-fingerprint <str> --out <path>`, plus one new arg
carrying whatever input samgraha assembled for the script:
`--in <path>` (a JSON file samgraha writes before invoking the script).

**`calculate`** — `--in` contents: raw stored audit results for this
standard+repo. `--out` contents:
```json
{
  "status": "ok",
  "final_score": 82,
  "band": "Good",
  "breakdown": { "deterministic_document": 90, "semantic_section": 74 }
}
```
On failure: `{"status": "error", "message": "<why>"}`. `breakdown`'s
internal shape is deliberately unconstrained — samgraha stores it, never
interprets it (§2.4).

**`scaffold`** — `--in` contents: `{"domain": "feature", "instance": "auth",
"sections": ["overview", "acceptance-criteria"], "target_path":
"docs/features/auth.md"}` (instance is null for singular domains, §4 item 7).
`--out` contents:
```json
{
  "status": "ok",
  "created": ["docs/features/auth.md"],
  "skipped": ["docs/features/auth.md#overview (already exists)"],
  "message": null
}
```
`skipped` (not overwritten) is required, not optional — it's how a scaffold
script stays idempotent under §7.4's order-tolerant rule: re-running it must
never clobber content a later step already filled in.

### 8.2 Envelope specs — `report`, `plan-generation` (document-rendering)

**Revised to converge on one parsing convention, not two.** Originally
specced as exit-code+file (no JSON envelope at all) — dropped that in favor
of reusing `scaffold`'s existing shape (§8.1): do the real file-system work
directly, and *also* always write a small JSON status envelope to `--out`.
Every capability now parses the same way — check `--out` for JSON, always
— which is the actual fix for "two conventions means two code paths," not
embedding full document content as a JSON string (which would trade one
problem for another: escaping overhead and an unreadable `--out` file for
anything beyond a trivial-length document).

Args: `--repo-root --repo-fingerprint --in <path> --target <path> --out
<path>`. `--target` is where the real rendered file goes (the actual repo
artifact — e.g. `docs/plan/PLAN.md`); the script writes it directly, same
as `scaffold` writes its files directly. `--out` gets the status envelope:
```json
{ "status": "ok", "written": ["docs/plan/PLAN.md"], "message": null }
```
On failure: `{"status": "error", "message": "<why>", "written": []}` —
`written` stays present (possibly empty) even on failure, so a partial
write is always visible to the caller, not just inferred from a missing file.

- **`report`**'s `--in`: assembled audit results + `calculate`'s last
  output + any stored semantic commentary (§7.7) for this domain.
- **`plan-generation`**'s `--in`: plan template (DB) + workflow definition
  (DB) + the semantic plan-input row (§7.2/§8.3) for this repo+workflow.
  Note: this script covers §7.2 **stage 2 only** (rendering). Stage 1
  (semantic determination) is not a script at all — it's an MCP-mediated
  LLM step (§4 item 2), with no CLI/JSON-file envelope, because samgraha
  never runs the LLM itself.

### 8.3 DB table — plan-generation semantic input (§7.2, §4 item 4)

```sql
CREATE TABLE plan_generation_inputs (
    id               INTEGER PRIMARY KEY,
    standard_id      INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint TEXT NOT NULL,   -- same fingerprint convention as
                                      -- check_runner.rs:170's `{check_name}-{repo_root}`
    workflow_id      TEXT NOT NULL,  -- matches an id in init's phase plan, §8.4
    domain_key       TEXT,           -- NULL = plan-level, not domain-specific
    instance_key     TEXT,           -- NULL unless domain is multi-instance (§4 item 7)
    input_json       TEXT NOT NULL,  -- the semantic determination's own output, opaque to samgraha
    previous_input_json TEXT,        -- prior input_json, shifted here by the upsert (one generation back, cheap hedge — see below)
    created_at       TEXT NOT NULL,  -- ISO8601
    UNIQUE(standard_id, repo_fingerprint, workflow_id, domain_key, instance_key)
);
```
Re-running the semantic-determination step for the same key upserts (matches
the idempotent/backfill principle in §7.4). Full history/versioning isn't
built in — but a one-generation-back hedge is: the upsert shifts the
current `input_json` into `previous_input_json` before overwriting
(`ON CONFLICT DO UPDATE SET previous_input_json = plan_generation_inputs.input_json,
input_json = excluded.input_json`). Costs one column, gives "what did the
LLM decide last time vs. this time" during iteration — the exact case
where full history would otherwise get requested almost immediately. If
that's still not enough later, upgrading to a real history table is easy
(cache data, no migration constraint per §7.4).

### 8.4 `init`'s phase-wise plan — JSON shape (§2.6, §4 item 4)

```json
{
  "system": "rust_dev",
  "use_cases": [
    {
      "id": "new-repo",
      "label": "New repo, no code, no docs",
      "phases": [
        {
          "id": "vision-plan-input",
          "kind": "semantic",
          "description": "Determine what Vision needs, given repo state",
          "depends_on": [],
          "pre_script": null,
          "post_script": null
        },
        {
          "id": "vision-plan-render",
          "kind": "script",
          "script": "script/plan_render.py",
          "depends_on": ["vision-plan-input"],
          "pre_script": null,
          "post_script": null,
          "expiry": null
        }
      ]
    }
  ]
}
```
`kind: "semantic"` phases have no `script` field — nothing to invoke besides
an MCP tool call (§4 item 2). `kind: "script"` phases name the script
samgraha dispatches to. `pre_script`/`post_script` are optional hook scripts
run immediately before/after the phase's main action — same envelope family
as whichever capability the phase belongs to. `expiry` (script phases only)
declares the phase's own staleness rule, consumed by §8.5's table — `null`
means never expires.

**This JSON is a static plan, not per-repo state — the two combine, they
don't merge.** `init`'s output is the same for every repo running this
system (a template/description of what's possible). "Where is *this* repo
right now" is a separate, computed question: for each phase in the plan,
check `script_runs` (§8.5) for a matching row and evaluate its validity.
That check is mechanical — it only needs "has phase X run for this repo,
and is it still valid per its declared rule" — no domain knowledge required,
so it belongs in samgraha's generic dispatch layer (whatever runs before any
capability actually executes), not in a system's script. This is consistent
with, not a contradiction of, §4 item 8 ("system owns backfill logic"):
item 8 is about *domain-specific* sufficiency judgments ("is this data good
enough for my calculation"); phase-gating against an explicitly-declared
`depends_on` list is pure bookkeeping samgraha can do itself. Nothing new
gets stored for this — it's a computed view over §8.4 (static) + §8.5
(per-repo history), recomputed on demand, not a third table.

### 8.5 Run-tracking + expiry — DB table (§4 item 6)

```sql
CREATE TABLE script_runs (
    id                  INTEGER PRIMARY KEY,
    standard_id         INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint    TEXT NOT NULL,
    capability          TEXT NOT NULL,  -- validate|calculate|report|scaffold|plan-generation|init|<future>
    phase_or_check_key  TEXT NOT NULL,  -- init's phase id, or an existing check_name
    ran_at              TEXT NOT NULL,  -- ISO8601
    expiry_rule_json    TEXT,           -- NULL = never expires; else e.g.
                                         -- {"type":"ttl","seconds":86400}
                                         -- or {"type":"head_commit"}
    expires_at          TEXT,           -- precomputed for ttl-type rules; NULL otherwise
    head_commit_at_run  TEXT,           -- git HEAD sha at run time, for head_commit-type rules
    UNIQUE(standard_id, repo_fingerprint, capability, phase_or_check_key)
);
```
Validity check (generic, samgraha-side, never interprets *why* a rule
exists — just evaluates it): `expiry_rule_json` NULL → always valid;
`type: "ttl"` → valid while `now < expires_at`; `type: "head_commit"` →
valid while current repo HEAD equals `head_commit_at_run`. The rule itself
comes from the system's `init` plan (§8.4's `expiry` field) — samgraha only
stores and evaluates it.

**Recommended default: `ttl`, not `head_commit`.** `head_commit` sounds
more precise ("re-run only when code actually changes") but is too coarse
in practice: a rebase, amend, or squash changes HEAD's sha even when the
underlying content is unchanged, causing unnecessary re-runs. Over-eager
invalidation is safe but wasteful. `ttl` is simpler and more predictable,
even though it can go the other way (content changes mid-window, a stale
result gets reused briefly). Systems should default their phases to `ttl`
and reach for `head_commit` only for checks that specifically need
commit-granularity freshness (e.g. an implementation-audit tied tightly to
code state, where a stale result is worse than an extra re-run).

### 8.6 Blocked-precondition MCP response (§4 item 6, §7.3)

```json
{
  "blocked": true,
  "reason": "missing_precondition",
  "system": "rust_dev",
  "phase_id": "vision-plan-input",
  "phase_kind": "semantic",
  "message": "Vision plan input hasn't been generated yet for this repo.",
  "how_to_run": {
    "tool": "plan_generation",
    "args": { "system": "rust_dev", "phase": "vision-plan-input" }
  }
}
```
`reason` is `"missing_precondition"` (phase never ran) or
`"expired_precondition"` (ran, but §8.5's validity check failed) — same
shape either way, callers branch on `reason` if they care why, not just
whether. `how_to_run` always names an MCP tool + args (never a raw shell
command) — consistent with §4 item 9's decision that every capability,
including future ones, goes through MCP dispatch (dedicated tool or the
generic `run_system_script` fallback), not direct script invocation by the
caller.

---

## 10. Implementation Plan — Remaining Samgraha-Side Work

### Phase A: Schema alignment (§8.3 + §8.5)

**Goal**: DB schemas match the spec exactly, ready for Rust code to query.

| Deliverable | File | Spec ref |
|---|---|---|
| Update `script_runs` to §8.5 spec | `schema/knowledge-hub/09-script_runs.sql` | §8.5 |
| New `plan_generation_inputs` table | `schema/knowledge-hub/10-plan_generation_inputs.sql` | §8.3 |
| New `system_plans` table (stores init output) | `schema/knowledge-hub/25-system_plans.sql` | §8.4 |

**Key changes to `script_runs`**:
- `system_id` → `standard_id REFERENCES standards(id)` (spec uses standards, not systems)
- `repo_root` → `repo_fingerprint TEXT NOT NULL` (matches check_runner convention)
- Add `phase_or_check_key TEXT NOT NULL` (phase id from init plan, or check name)
- Add `expiry_rule_json TEXT`, `expires_at TEXT`, `head_commit_at_run TEXT`
- Drop `staleness` column (replaced by expiry fields)
- Drop `script_path`, `message`, `written_json`, `output_json`, `duration_ms` (output goes through capability result envelope, not stored in script_runs)
- Update unique constraint: `UNIQUE(standard_id, repo_fingerprint, capability, phase_or_check_key)`

**Verification**: schemas parse as valid SQL; column names/types match §8.3/§8.5 exactly.

---

### Phase B: Init plan storage (§8.4)

**Goal**: samgraha can store and retrieve init output, making the dependency graph queryable.

| Deliverable | File | Spec ref |
|---|---|---|
| `store_system_plan` MCP tool | `crates/mcp/src/adapter.rs` | §8.4 |
| `get_system_plan` MCP tool | `crates/mcp/src/adapter.rs` | §8.4 |
| Rust types for init plan JSON | `crates/audit/src/capability.rs` or new `crates/schemas/src/` | §8.4 |
| Tool schemas in main.rs | `crates/mcp/src/main.rs` | — |

**Key design**:
- `system_plans` stores the full init output JSON per system+repo (static, not per-phase)
- `store_system_plan` takes `system_name`, `repo_root`, `plan_json` → upserts
- `get_system_plan` takes `system_name`, `repo_root` → returns stored plan or null
- Plan JSON shape follows §8.4 exactly (system, use_cases[], phases[] with depends_on, kind, script, expiry)

**Verification**: can store a sample init plan, retrieve it, parse its phases and dependencies.

---

### Phase C: Blocked-precondition logic (§8.6)

**Goal**: before running any capability, samgraha checks if its prerequisites are met.

| Deliverable | File | Spec ref |
|---|---|---|
| `check_phase_prerequisites()` function | `crates/audit/src/capability.rs` | §8.6 |
| Validity evaluation (ttl, head_commit, null) | `crates/audit/src/capability.rs` | §8.5 |
| Wire into `run_capability_for` | `crates/mcp/src/adapter.rs` | §8.6 |
| Blocked-precondition response shape | `crates/mcp/src/adapter.rs` | §8.6 |

**Key logic**:
1. Look up system's plan from `system_plans` → get phase's `depends_on` list
2. For each dependency: query `script_runs` for matching `(standard_id, repo_fingerprint, capability, phase_or_check_key)`
3. If no row → `reason: "missing_precondition"`
4. If row exists but expired per `expiry_rule_json` → `reason: "expired_precondition"`
5. If all deps satisfied → proceed with capability execution
6. Blocked response follows §8.6's JSON shape exactly

**Verification**: can detect missing deps, expired deps, and pass-through when all deps met.

---

### Phase D: End-to-end wiring + tests

**Goal**: full integration verified, compiles clean, tests pass.

| Deliverable | File |
|---|---|
| Integration test: store plan → check prereqs → run capability | `crates/audit/src/capability.rs` (unit tests) |
| MCP tool test: store_system_plan + get_system_plan round-trip | `crates/mcp/src/adapter.rs` (tests) |
| `cargo check -p mcp` clean | — |
| `cargo test -p audit` pass | — |

---

### Verification checklist (check off as each phase completes)

- [x] **A1**: `09-script_runs.sql` matches §8.5 spec column-for-column
- [x] **A2**: `10-plan_generation_inputs.sql` matches §8.3 spec
- [x] **A3**: `25-system_plans.sql` created with system+repo+plan_json columns (renumbered from `11-` — collided with pre-existing `11-rules.sql`)
- [x] **B1**: `store_system_plan` tool schema in main.rs
- [x] **B2**: `get_system_plan` tool schema in main.rs
- [x] **B3**: Init plan Rust types (Phase, UseCase, InitPlan) defined
- [x] **B4**: `handle_store_system_plan` handler in adapter.rs
- [x] **B5**: `handle_get_system_plan` handler in adapter.rs
- [x] **B6**: Tool methods registered in McpAdapter::new()
- [x] **B7**: Tool dispatch entries in handle_message match
- [x] **C1**: `check_phase_prerequisites()` implemented
- [x] **C2**: TTL validity check (`now < expires_at`)
- [x] **C3**: head_commit validity check (current HEAD == head_commit_at_run)
- [x] **C4**: null expiry = always valid
- [x] **C5**: Blocked response matches §8.6 JSON shape
- [x] **C6**: `run_capability_for` calls check before executing
- [x] **D1**: `cargo check -p mcp` compiles clean
- [x] **D2**: Unit tests for prerequisite checking pass
- [x] **D3**: Round-trip test for plan storage passes

### Phase E: Verification fixes (found by review, not self-reported)

C1-C6 above were true for the *read* side only — `check_phase_prerequisites`
could evaluate a row correctly, but nothing ever wrote one, so every gated
phase would report `missing_precondition` forever. Also two `standard_id`
stubs and one `current_head` stub were hardcoded (marked `// TODO` in the
code, not hidden). Fixed:

- [x] **E1**: `record_script_run()` added to `capability.rs` — upserts a
  `script_runs` row on a successful run, computing `expires_at`/
  `head_commit_at_run` from the phase's declared `ExpiryRule`
- [x] **E2**: `run_capability_for` calls `record_script_run` after a
  successful `execute_capability`, keyed by `phase_id` when given, else the
  capability name itself
- [x] **E3**: `resolve_standard_id()` added to `adapter.rs` (reuses
  `db_reader::from_standards_db`'s system-name/is-default lookup) — replaces
  all three `standard_id = 1i64` stubs (`run_capability_for`,
  `handle_store_system_plan`, `handle_get_system_plan`)
- [x] **E4**: Resolution is best-effort (`.ok()`), not `?`, in
  `run_capability_for` and `handle_get_system_plan` — a repo with no
  standard registered yet can still run a capability script off pre-existing
  file-based discovery (`validate`'s original behavior), it just skips
  phase-gating and run-tracking rather than hard-failing
- [x] **E5**: `common::env::current_head_sha()` added — `head_commit`-type
  expiry now actually reads git HEAD instead of a permanent `None`
- [x] **E6**: `run_system_calculate`'s and `run_system_plan_generation`'s
  tool descriptions rewritten — both previously described a fictional
  "PlanResult with determination/blocked_reasons" shape neither capability
  actually returns; the real blocking shape is the separate top-level
  `{"blocked": true, ...}` response (§8.6), returned instead of running the
  script, not a field inside the script's own result
- [x] **E7**: `system_name`/`phase_id` added to all 6 tool schemas (were
  only on the generic `run_system_script`, missing from the 5 dedicated ones)
- [x] **E8**: `report`'s `target` param — was accepted by the schema but
  never read; now merged into the script's `--in` payload
- [x] **E9**: `25-system_plans.sql` renumbered — collided with pre-existing
  `11-rules.sql`
- [x] **E10**: 4 new tests added (`capability.rs`) exercising the
  write-then-read round trip directly — missing/expired/valid/upsert cases.
  264 audit tests + 8 mcp tests pass, `cargo build --workspace` clean.

---

## 11. Relationship to `docs/crates-refactor-proposal.md`

| That doc's piece | Fate |
|---|---|
| Phase 0 (push-safety guard, §14.1) | **Keep** — real bug, independent of this pivot, already shipped |
| Phase 1 (`system.yaml`, `extends`/`drops`, drops-referential-check, tempdir cleanup) | **Keep, rescoped** — inheritance stays useful for close-sibling dedup (dev class proved it); already shipped for dev class, no rework needed |
| §3.1/§13.7 (N-level nesting is depth-agnostic) | **Keep** — still true, still useful for whatever inheritance chains genuinely earn their keep |
| Phase 2 (`base_academic`) | **Drop as designed** — §1.1 above shows it saves 2 files. Don't build an abstract base for that; if academic systems need to share the 2 fully-generic files, a plain `extends:` between the 2 concrete systems (no abstract base needed) covers it more cheaply, if it's even worth the chain at all |
| Phase 3 (DB schema: `parent_system_id`/`class_name`/`subclass_name`/`is_abstract`) | **Drop** — this is exactly the "bake a class taxonomy into samgraha's schema" move §1.2 argues against. Systems don't need samgraha to know their class; the script contract doesn't care |
| Phase 4 (class-shape validation, diamond detection) | **Diamond detection**: keep unconditionally, not gated on chain depth — cheap defensive code, and this proposal's own `dev → build_tool → rust` example (§4 item 5) means chains are expected to deepen, so the guard should exist before that happens, not after. **Class-shape validation**: drop — it's validating against the exact taxonomy Phase 3 is dropping |
| Phase 5 (cleanup + docs) | Rewrite once this proposal's shape is settled — the authoring guide needs to cover script contracts, not just override/drop syntax |
| §14.2 (name-collision `last_registered_by`) | **Keep** — orthogonal to this pivot, still a real multi-repo gap |
