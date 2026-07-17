# Proposal: Standard-Driven Workflow Engine (Generation + Audit + Script)

**Created:** 2026-07-17
**Status:** Proposal — pending review
**Supersedes:** the JSON-report-output section previously in this file (folded in below as Phase 5 — still valid, now scoped as one piece of the larger design)

---

## Correction: The Ingestion Pipeline Already Exists

Before writing Phase 1/2 below, checked whether `register_standard` already does what those phases were about to propose building. It does. `crates/mcp/src/adapter.rs::handle_register_standard` doesn't parse YAML in Rust at all — it shells out to `schema/knowledge-hub/knowledge-hub-loader.py` (resolved via `services::knowledge_publish::resolve_knowledge_hub_loader`), a ~1400-line, already-generic (`--layout` override), already-working 8-pass loader that reads a standard's authoring files and writes `standards.db`:

| Pass | Reads | Writes |
|---|---|---|
| 1 | `00-domain-relationships.md` (embedded YAML: tiers + relationships) + `plan/core/loop.yaml`'s `within_tier_ordering` | `domains`, `relationship_types`, `domain_relationships` |
| 5 | `audit/deterministic/**/*.yaml`, `audit/semantic/**/*.{yaml,md}` | `rules`, `rule_evidence_params` |
| 7 | `calculation/**/*.yaml` | `calculation_rules`, `calculation_inputs`, `score_bands` |
| 8 | `plan/core/loop.yaml`, `plan/usecase/**` | `plan_settings`, `plan_scenarios` |
| 2, 3, 4, 6 | templates, script schemas, documentation-standards | `section_catalog`, `script_checks`, `standard_docs`, `templates` |

Every layout key the loader looks for (`audit_deterministic`, `audit_semantic`, `calculation_root`, `plan_loop`, `plan_usecase`, `domain_relationships`) matches the real directory names in both `python_hackathon` and `base_dev` exactly — this wasn't built against a guess. `db_reader.rs` already reads all of it back out into `StandardRegistry` (`AuditRuleDef`, `ScoringConfig{ calculation_rules, calculation_inputs, score_bands }`, `PlanSetting`, `PlanScenario`, `ScriptCheck` — all real Rust structs, all populated by real SQL SELECTs, not stubs).

So: **"no hardcode rules, everything in YAML, schema-defined, saved in DB" is already true for audit rules, calculation formulas, domain/tier/relationship graphs, and generate/audit/fix tier content.** Nothing in this proposal needs to rebuild that. What's actually missing, now correctly scoped:

1. **Never verified end-to-end against these two standards.** Has anyone actually run `register_standard` against `python_hackathon`/`base_dev` and confirmed the row counts land right? Needs doing before anything downstream is trusted.
2. **Nothing consumes what's already loaded.** `StandardRegistry.audit_rules`/`.scoring`/`.plan_scenarios` are populated (once #1 is confirmed) but no audit pipeline reads `audit_rules` to actually run checks, no calculation code reads `.scoring.calculation_rules` to actually compute a score, and no execution engine reads `.plan_scenarios`/`.plan_settings` to actually run a generate→audit→fix loop. This is genuinely new Rust work — but it's *consumption* of already-modeled data, not a new parser.
3. **Genuinely new, not covered by any of the 8 passes:** `python_hackathon/plan/core/loop.yaml`'s `stages:` sequence itself (Phase 3a — small, one file), and the script runtime matrix gap (Phase 4a — `.py` isn't invokable today). Team/session/leaderboard are explicitly not samgraha's concern (see Phase 3a) — not a gap, a decision.

Phase 1 and Phase 2 below are rewritten accordingly — from "build a loader" to "wire the audit engine to `StandardRegistry`, already populated by a pipeline that exists."

## Problem

Verified against two real standards — `python_hackathon` (Kriti hackathon judging) and `base_dev` (general repo documentation) — neither runs on samgraha today. As of the correction above, the gap is narrower than originally scoped: it's not ingestion, it's execution.

Both standards use the same **distributed structure**:

```
standard/
├── audit/
│   ├── deterministic/{document,section}/*.yaml
│   └── semantic/{document,section}/*.{yaml,md}
├── calculation/
│   ├── deterministic/{document,section}/*.yaml
│   ├── semantic/{document,section,ensemble}/*.yaml
│   ├── aggregation/domain/*.yaml
│   ├── summary/{final_score,score_bands,trend}.yaml
│   ├── validation/scoring_validation.yaml
│   └── weights.yaml
├── plan/
│   ├── core/{loop.yaml,tiers.yaml}
│   └── usecase/**
├── templates/{audit,generation,reports}/**
└── script/*.py
```

`docs/raw/proposal.md`'s "Model A" (one monolithic `audit/pipelines/*.yaml` file) is real, works, and is now implemented (`crates/audit/src/yaml_runner.rs`, `pipeline_factory.rs::load_yaml_pipelines_for_standard`) — but zero files under either standard use it. It's a viable authoring shortcut for a standard with one or two simple domains; it is not what `python_hackathon` or `base_dev` need. Everything from here on is about the distributed structure ("Model B"), because that's what's actually sitting on disk.

Three things are missing, confirmed by reading the standards directly rather than by guessing at their shape:

1. **No loader for the distributed audit-rule files.** `audit/deterministic/document/01-infrastructure.yaml` and its semantic counterpart are never read by anything in `crates/`.
2. **No calculation interpreter.** `calculation/**/*.yaml` files declare a `calculation:` method name (`weighted_pass_rate`, `weighted_merge`, `reliability_aware_ensemble`, `weighted_sum`, `threshold_lookup`, `trend_comparison`, ...) and a `formula:` — nothing executes them. `docs/raw/proposal.md` §5 already catalogs all the methods that appear in both standards; this proposal treats that catalog as ground truth (verified against the real `calculation/` directories while writing this).
3. **No workflow engine for `plan/core/loop.yaml`.** Both standards ship one, and they are *not* the same workflow: `python_hackathon/plan/core/loop.yaml` is a flat competition loop (audit → ensemble → aggregate → relative-to-competition scoring → cap at 20 → report); `base_dev/plan/core/loop.yaml` is a completely different generate-or-audit-then-fix-loop keyed off `tiers.yaml`'s domain tiers and relationship graph. Nothing in samgraha reads either file. This is also where **generation** lives — `base_dev`'s loop has an explicit `path_selection: generate | audit` branch per domain, so generation and audit are two outcomes of the *same* workflow engine, not two separate systems.

## Core Principle (unchanged from `docs/raw/proposal.md`, restated because it's the test every design decision below has to pass)

**samgraha has zero opinion about what a standard's workflow, domains, scoring, or generation look like.** Every fact in the problem statement above — the tier list, the domain relationships, the scoring formula, the workflow shape, the leaderboard math — is data the standard owns. samgraha's job is to be a generic executor: read the standard's YAML, run the stages it declares, hand off to the standard's own scripts where it says to.

Concretely, this rules out:
- A Rust `enum` or hardcoded `match` for domain names, tier structure, or workflow shape (this is the mistake `PipelineKind` already made for the 22 built-in pipelines — not repeating it for standards).
- A Rust implementation of any standard-specific scoring/ranking formula. `crates/mcp` shipped a `standard_audit_runs` archive table with a SQL leaderboard query earlier this session — the leaderboard was removed after checking `python_hackathon/script/leaderboard.py` and finding it does z-score + semantic-agreement-bonus math no fixed SQL query can replicate. The archive (raw facts: standard/domain/model/score/report) stays; the ranking logic stays in the standard's own script, invoked through the *existing* `run_check`/`list_script_checks` mechanism (`ScriptCheck` is already data-driven — this is proof the principle already works elsewhere in the codebase).

## Architecture

```
plan/core/loop.yaml (repository scope) + plan_scenarios (tier scope, already in DB, Phase 3a)
  │
  ▼
StandardWorkflowPlanner (new ProjectPlanner impl, Phase 3)
  — generates ProjectPhase[] from tiers.yaml + a workflow file instead of
    resolve_planner()'s hardcoded DOC_DOMAINS/IMPL_DOMAINS const arrays
  │
  ▼
PlanOrchestrator (EXISTING — services::project_planner, unchanged)
  — same create_plan/execute_phase/get_plan machinery DocAuditPlanner etc.
    already use; project_plan_execute/status/abort MCP tools already wired
  │
  ├─ PhaseType::Generate → existing compile_repository (already data-driven
  │                         over StandardRegistry.domain), + standard-scoped
  │                         template lookup (new — see Phase 3)
  ├─ PhaseType::Audit    → NEW distributed loader (Phase 1) → existing
  │                         deterministic/semantic evidence executors
  │                         (yaml_runner.rs already has these, built for
  │                         Model A — Phase 1 reuses them against Model B
  │                         rule files instead of writing new evidence code)
  ├─ PhaseType::Fix      → existing fix-plan machinery (audit_fix_plan/apply/
  │                         accept/reject — already generic, already wired)
  ├─ "calculate"/"aggregate"/"validate" stages → NEW calculation interpreter
  │                         (Phase 2), invoked between phases, not itself a
  │                         PhaseType — a workflow-file concept, not a plan one
  ├─ "report" stages     → existing render_report_from_pipeline (already
  │                         generic over PipelineReport) + standard_audit_runs
  │                         archive (already built)
  └─ "script" stages     → existing run_check / ScriptCheck for check-shaped
                            scripts, new workflow-script contract for
                            pipeline-shaped scripts like leaderboard.py
                            (Phase 4/4a) — two contracts, one dispatch layer
```

The real reframe from the previous version of this proposal: this is not a new engine to build, it's `resolve_planner()`'s closed 4-case match getting a 5th, data-fed arm. Every other box above already exists — the work is wiring, not invention. That's deliberate: the fastest way to violate the core principle is to write a bespoke Rust handler per stage instead of routing through the one generic mechanism that already handles that concern.

`tiers.yaml` is read alongside a workflow file, not instead of it — it supplies the domain list, tier ordering, and relationship graph that `path_selection`/`tier_gate` stages (or `StandardWorkflowPlanner`'s dependency-graph construction) reference by name. Both files are per-standard data; a hackathon standard's tiers (`infrastructure, security, engineering, ...`) and a doc-standard's tiers (`vision, philosophy, architecture, ...`) are equally valid, equally unknown to Rust.

## Phase 1 — Wire Audit Execution to `StandardRegistry.audit_rules` (Done)

**Not a new loader.** `AuditRuleDef` (`schemas/src/standard.rs:51`) already has `id`, `description`, `severity`, `evidence_type`, `scope`, `weight`, `mandatory` — one row per rule from `rules`/`rule_evidence_params`, already populated by loader Pass 5 from the exact real files (`audit/deterministic/document/01-infrastructure.yaml`'s `evidence: {type, target}` / `check: {file_globs}` forms, semantic document YAML, semantic section markdown rubrics). `StandardRegistry.standards[name].audit_rules: Vec<AuditRuleDef>` is where it already lands.

What's new: a `crates/audit` execution path that, given a standard name + domain, pulls `audit_rules` for that (standard, domain) out of `StandardRegistry` and runs each one through the evidence executors `yaml_runner.rs` already has (`file_presence`, `content_check`, `llm_judgment`, ...) — dispatching on `AuditRuleDef.evidence_type` (a plain string, e.g. `"file_presence"`, `"script_result"`, `"llm_judgment"` — the DB's `evidence_type` column and Model A's `EvidenceDef` enum tag overlap almost 1:1, so this is a string-to-enum-variant match, not new evidence logic). `rule_evidence_params` (already loaded as key/value pairs per rule) supplies each evidence type's variable-shape config (e.g. `content_check`'s `keywords`, `keyword_absence`'s `categories`).

**Non-goal:** re-parsing `audit/**/*.yaml` files at request time. The loader already turned them into rows once at `register_standard` time; re-parsing them per-audit-call would be the DB doing nothing.

## Phase 2 — Wire Scoring to `StandardRegistry.scoring` (Done, Narrower Than First Scoped)

**Shipped:** `crates/audit/src/calculation.rs` — `weighted_pass_rate`, `weighted_sum`, `threshold_lookup`, `find_bucket` (bucket-name lookup into `ScoringConfig.calculation_rules`, no hardcoded bucket list). Wired into `framework.rs`'s existing scoring loop, replacing inline math that used to run unconditionally with a real lookup: before computing a bucket's score, `find_bucket(&scoring.calculation_rules, "deterministic_document")` checks what method the standard actually declared, and logs a warning (doesn't silently misapply) if it's ever anything other than `weighted_pass_rate`. Same pattern for `summary_final_score` → `weighted_sum`. Both real standards declare exactly these methods for these buckets today, so behavior is unchanged for `python_hackathon`/`base_dev` — the difference is that it's now checked, not assumed.

**Deliberately not built — no real data path exists yet, would be guessing:**
- `sum_capped_at_100`, `reliability_aware_ensemble` — need semantic-rule execution. Checked while implementing this: `StandardDefinition.audit_rules` only loads `kind = 'deterministic'` rows (`db_reader.rs`'s rules query has `WHERE r.kind = 'deterministic'` — confirmed, not assumed) — semantic rules from `standards.db` aren't loaded into anything today. The `"semantic"` provider registered in `runtime.rs` exists but has no semantic rules to receive. This is a real, separate, bigger gap (loading `kind = 'semantic'` rows, executing them via LLM judgment) than "add a calculation method" — flagged, not attempted here.
- `weighted_merge` (domain aggregation of deterministic+semantic) — same blocker, needs both halves to exist first.
- `trend_comparison` — needs historical score tracking. `CalculationRule` (the Rust struct) doesn't even carry the DB's `tolerance_*`/`min_samples`/`fallback_*` columns yet (`db_reader.rs`'s SELECT only pulls `bucket, calculation_method, formula` — confirmed while checking this). `standard_audit_runs` (built earlier) is the natural history source once this is built, but wiring that up is its own unit of work.

None of these are stubbed with fake logic — a function with no real caller and no real test data would just be a guess wearing a type signature. They stay as open items, same as the proposal already flagged.

`calculation/validation/scoring_validation.yaml` (weight-sum == 100, score bounds, domain count) isn't in the loader's 8 passes yet — a real gap, not a correction. Small: same shape as `calculation_rules`, `rule:` instead of `formula:`, boolean result instead of a score. Worth a new loader pass + a `validation_rules` table mirroring `calculation_rules`, rather than special-casing it in Rust. (Numbered as Pass 9 below alongside Phase 3a's workflow pass — the two are independent additions, not sequential.)

## Phase 3 — Workflow Engine: Generalize `ProjectPlan` (Done)

Correction that held up: samgraha already had a phase-execution engine of exactly this shape (`PhaseType::{Generate,Audit,Fix,Verify}`, `ProjectPhase`, `PlanOrchestrator`, already-wired `project_plan_execute`/`status`/`abort` MCP tools) — the gap was only what fed it. Shipped:

- **`ProjectCase::Standard`** (`schemas/src/planning.rs`) — a 5th variant alongside the 4 fixed built-in cases, `Display`/`from_str` round-trip `"standard"`. `resolve_planner()`'s match is exhaustive, so this was the one enum the compiler forced every call site to handle — all found and updated (just the one match arm; `ProjectCase` round-trips through `registry/store.rs` as a string already, no change needed there).
- **`StandardWorkflowPlanner`** (`services/src/project_planner/planners.rs`) — builds phases from `ctx.standard.plan_scenarios`, grouped by tier (ascending), steps within a tier in `generation → audit → fix` order, cross-tier dependency chain (tier N's first phase depends on tier N-1's last). `PhaseType` comes directly from `plan_scenarios.step`'s own values — no new vocabulary.
- **`StandardWorkflowContext`** (`services/src/project_planner/context.rs`) — carries `plan_scenarios`/`plan_settings`/`domains_by_tier`, threaded through a new `ProjectContext::detect_with_registry()` (old `detect()` kept, delegates with `None`). `repo_state` ("existing" vs "new") is a new detection: presence of `.samgraha/manifest.json`.
- **Fixed a real gap found while building this**: `domains.tier` (the DB column) was queried but never selected by `db_reader.rs` (`SELECT id, key, name, description FROM domains` — no `tier`), so `StandardDefinition` had nowhere to carry it. Added `tier: Option<i32>` to `StandardDefinition`, extended the query, updated all 4 struct-literal construction sites. Without this, `StandardWorkflowPlanner`-generated phases would have empty `domains` — technically a plan, but not an executable one.
- **`AuditPhaseExecutor`** (`executors.rs`) — the phase-execution side had the same problem Phase 1 found in `handle_audit`: keyed by `PhaseType` (generic-looking) but hardcoded to `PipelineKind::from_str` underneath (not generic). Added a fallback: when `phase.pipeline_ids` is empty, iterate `phase.domains` through `runtime.audit(domain, ...)` instead — gated on `pipeline_ids.is_empty()` specifically so the other 4 planners (which populate both fields with the same built-in names) never double-run.

**Verified against real data** (Python loader → `standards.db` → `StandardRegistry` → `ProjectContext` → `StandardWorkflowPlanner`, not synthetic fixtures): `python_hackathon`'s 10 domains across 4 tiers produced exactly 12 phases (4 tiers x 3 steps), correct domain grouping per tier, correct dependency chain across tier boundaries.

**Explicitly not built — flagged, not guessed at:** `FixPhaseExecutor` has no domain-audit fallback. `apply_finding_fix(finding, domain, report_id: i64, ...)` expects a numeric id from the `pipeline_reports`-style storage `run_pipeline_with_id` returns; `runtime.audit()`'s `AuditReport` has a `String` id from a different storage path with no equivalent number to pass through. Guessing one (e.g. `0`) would silently misattribute a fix session rather than fail loudly, so a `StandardWorkflowPlanner`-generated Fix phase currently just no-ops (empty `pipeline_ids`, the existing loop never iterates) instead of doing something wrong. Real follow-up work, not this session's to guess at.

`domain_relationships`' `enforce_order`/`mutual` and `relationship_types.tier_gating` (populated by Pass 1) are **not yet wired into phase generation** — the shipped version approximates tier-gating as one linear chain (tier N's first phase waits on tier N-1's last), not the finer per-domain dependency graph those columns encode. Noted as a known simplification, not silently pretended away.

### Phase 3a — `repository` and `tier` Scope Only; Team/Session/Leaderboard Explicitly Out of Scope

**Decision:** samgraha has no concept of "team," "session," or "leaderboard," and doesn't get one. Its job stops at: run an audit, store the result (`standard_audit_runs`, already built), expose it as structured JSON (`audit_runs`, Phase 5). Any standard that wants team rollups or a competition leaderboard writes its own script against that JSON — `python_hackathon/script/score_aggregator.py` and `leaderboard.py` already do exactly this, already outside samgraha, and stay that way. Reached through the workflow-script contract (Phase 4), not through a samgraha-native `team`/`session` workflow scope. This removes the earlier version of this section's `workflows`/`workflow_stages` tables and the `team`/`session` rows from the scope table below entirely — not deferred, not phased later, just not samgraha's concern.

What's left, both already have a home:

| `scope` | Runs against | Home |
|---|---|---|
| `tier` | one domain/tier within one repo | already `plan_scenarios` (Pass 8) — no new file, no new table, Phase 3's `StandardWorkflowPlanner` reads it as-is |
| `repository` | one whole repo | `python_hackathon`'s `plan/core/loop.yaml` `stages:` list — **the one real gap left here.** Pass 1 only reads the file's `within_tier_ordering`, Pass 8 only reads its `threshold`/`max_iterations`/`fallback` header; the actual `stages:` sequence (`repository → audit → calculate → aggregate → ... → report`) lands nowhere in the DB today. Small, contained fix: one more loader pass + one small table (`workflow_stages`: standard_id, sort_order, stage_type, stage_params) for this one file shape — not the multi-workflow system the earlier version proposed, just closing the gap on the file that already exists. |

## Phase 4 — Script Execution: Two Contracts, Not One (Done)

`run_check`/`ScriptCheck`/the 5-tier resolution chain reused as-is, not rebuilt — confirmed still true after building this. Both contracts:

1. **Check-script contract** (existing, `check_runner.rs`, unchanged) — fixed protocol: `--repo-root/--repo-fingerprint/--out` in, `{status, evidence, metrics}` JSON out.
2. **Workflow-script contract** (new, shipped) — `common::env::run_workflow_script(script_path, cwd, args, env, timeout_secs) -> WorkflowScriptOutput{ exit_code, stdout, stderr }`. Caller-supplied args/env, no fixed flags, reuses `script_command`'s interpreter dispatch (Phase 4a) via a `run_with_optional_timeout` helper extracted out of `run_check_script` so both contracts share one spawn/poll/kill implementation instead of two. This is the "engine's job is just process invocation" primitive the proposal called for — it does not itself parse a workflow YAML `script` stage or decide what output file to read back; that's the not-yet-built workflow-stage executor's job (Phase 3a's `repository`-scope `workflow_stages` table, still open).

**Found and fixed a real bug while testing this**, not something introduced by the new code: `run_with_optional_timeout`'s `spawn()` path never set piped stdio, so `wait_with_output()` always returned empty stdout/stderr on the timeout-tracking path — meaning `run_check_script`'s own "script did not write an output file; stderr: ... stdout: ..." error message has always printed nothing there, silently, since before this session. Fixed by setting `Stdio::piped()` before `spawn()`. Only caught because a new test asserted on actual captured output instead of just exit status — a real illustration of why the verification habit in this document matters.

**When scripts run:** wherever a `script`-typed stage sits in a workflow's (linear) stage list — no separate hook/trigger mechanism, confirmed against both real `loop.yaml`s having no conditional branching or lifecycle callbacks. Not building hook infrastructure until a standard actually needs `on_failure: notify.py`.

### Phase 4a — Script Runtime Matrix (Done)

`.py`/`.js` support added to both `common::env::script_command` (interpreter dispatch: `python3`/`python` via the already-existing `python_command()`, `node` via new `node_command()`) and `check_runner.rs::probe_script` (extension probing). `.rs`/compiled languages still explicitly not supported — same reasoning as before, neither real standard ships one.

**Found and fixed a second real bug while wiring this up**: `probe_script(dir, name)` always treated `name` as a bare check name and appended `.sh`/`.ps1` — but `python_hackathon`'s real `rule_evidence_params` rows store the `script` param as `"script/audit_testing.py"` (a relative path, already carrying its own extension), not a bare name. Appending `.sh` to that built a nonsense path (`script/audit_testing.py.sh`) that could never exist. Fixed: `probe_script` now checks whether `name` already has a recognized extension and, if so, resolves it as a direct `dir.join(name)` path instead of running it through the bare-name+extension-append convention at all.

**Flagged, not fixed — genuinely separate question**: even with both fixes, `python_hackathon`'s own `script/*.py` files still aren't *discoverable* by `run_check`/`resolve_check`'s 4 real tiers (`check_overrides`, repo `scripts/`, `.samgraha/scripts/`, `mcp_dir()/scripts/`) — none of them point at a *registered standard's own* script directory. `sync` (`services::init`) copies `mcp_dir/scripts/` → `.samgraha/scripts/`, but nothing copies a standard's `script/` directory into `mcp_dir/scripts/` (or anywhere else `resolve_check` looks) during `register_standard`. This is a script-*distribution* gap, distinct from the extension-*dispatch* gap Phase 4a was scoped to close — real, but a different piece of work, not guessed at here.

8 new tests across `common::env` and `audit::check_runner` (dispatch, path-vs-bare-name resolution, a real subprocess run with captured stdout/exit-code/env, timeout kill). Full workspace build and test suite green.

## Workflow & Rule File Schemas

Formalizing the shapes this proposal has been describing by example, as the actual contract implementation targets:

**Workflow file** (`plan/core/loop.yaml` — `repository` scope only; `tier` scope is `plan_scenarios`, already in the DB, no file-level schema needed):
```yaml
id: string                  # unique within the standard
version: string
scope: repository            # Phase 3a — tier scope doesn't use this file shape at all
stages:
  - type: repository | generate | audit | calculate | aggregate | validate | report | script
    # generate/audit: domain + template/rule source implied by stage type + tiers.yaml
    # calculate/aggregate/validate: source: path to a calculation/*.yaml file (Phase 2)
    # script: name, args: [...], env: {...}, expects: [output file paths] (Phase 4)
```

**Audit rule file** (`audit/deterministic/document/{domain}.yaml`, verified against real files, Phase 1):
```yaml
system_id: string
domain: string
scope: document | section
kind: deterministic
rules:
  - id: string
    description: string
    severity: error | warning
    weight: number
    mandatory: bool
    evidence: { type: string, target: string }   # OR:
    check: { file_globs: [string] }               # both forms are real, both accepted
```

**Semantic rule file** (`audit/semantic/document/{domain}.yaml`):
```yaml
system_id: string
domain: string
scope: document
kind: semantic
metadata_fields: [string]
ensemble: { required_models: [string] }   # optional
prompt_template: string
evidence_requirements: [{ type: string, schema: object }]
```

**Calculation file** (`calculation/**/*.yaml`, Phase 2): `id`, `calculation` (method name from the Phase 2 table), `scope`, `inputs`, `formula` (prose description of the method — the method name is what's executed, `formula` is documentation for a human reading the file, same relationship `condition` has to `evidence` in rule files).

## Phase 5 — JSON Alongside Markdown Reports (Already Done — Predates This Proposal)

Checked before building anything, per this document's own established habit: **every piece of this phase already exists in the codebase**, not from this session — `ReportOutput { markdown, json }` (`reporting.rs:12`), `render_report()` and `render_report_from_pipeline()` both already return it (`reporting.rs:3488`, `:3699`), the CLI already writes `report.json` next to `report.md` for both `latest/` and `archive/`, gated by `config.report.json` (`cli/src/commands.rs:832`), and MCP's `report_generate` already returns `{ markdown, json }` (`adapter.rs:1276`). This phase's own earlier text ("Status: Proposal — pending review") was stale — the design it describes was already shipped; the doc just never got marked done. Nothing to build here. Feeds a `script` stage (Phase 4) an audit's structured output without markdown-parsing, and is what `standard_audit_runs` already archives as `report` (`serde_json::to_string(report)`).

## What's Already Done

- Model A monolithic loader (`yaml_runner.rs`, `pipeline_factory.rs`) — works, unused by either real standard, kept as a lightweight option for simple standards.
- `standard`/`domain` dispatch fix in `handle_audit` — was matching pipeline name against standard name (silently broken for any standard with >1 domain); now resolves the standard's directory first, then the domain within it.
- `standard_audit_runs` archive table + `audit_runs` MCP tool — raw facts only (standard/domain/model/score/report), self-reported `model` param, no aggregation opinion.
- **Found, not yet wired:** `ProjectPlan`/`ProjectPhase`/`PlanOrchestrator` (`services::project_planner`) already implements generate→audit→fix phase execution with dependency tracking, status tracking (`Pending/InProgress/Completed/Failed/Blocked`), and MCP tools (`project_plan_execute`/`status`/`abort`) — built for samgraha's own 4 built-in cases (`NewProject/DocAudit/ImplTestAudit/BuildAudit`), matches `base_dev/plan_scenarios`' `{generation,audit,fix}` shape closely enough that Phase 3 is now "add a 5th data-fed planner," not "build a workflow engine."
- **Found, not yet wired (the big one):** `schema/knowledge-hub/knowledge-hub-loader.py`, already wired to `register_standard`, already ingests `audit/**`, `calculation/**`, `00-domain-relationships.md`, and `plan/core/loop.yaml` + `plan/usecase/**` into `standards.db` (`rules`, `rule_evidence_params`, `calculation_rules`, `calculation_inputs`, `score_bands`, `domains`, `domain_relationships`, `plan_settings`, `plan_scenarios`), and `db_reader.rs` already reads every one of those tables back into typed `StandardRegistry` fields. Phases 1 and 2 above were rewritten from "build a loader" to "consume what's already loaded" after finding this — the biggest correction in this document's history.

## Post-Phase Gap Fixes

Closed after all 5 phases, working through the backlog those phases had flagged:

- **`handle_audit`'s `standard` param now reaches the DB-backed path** (`handle_db_backed_standard_audit`, new) — was previously only routed to Model A (`audit/pipelines/*.yaml`), so `standard: "python_hackathon"` had no way to reach Phase 1/2's actual work at all. Now: Model A if the standard shipped one, else builds a fresh `StandardRegistry`/`AuditFramework` scoped to that standard name specifically (not `self.runtime`'s, which is scoped to whatever `system_name` `samgraha.toml` configured).
- **3 real correctness bugs found and fixed while verifying the above end-to-end** (real `python_hackathon` data, not fixtures — same discipline as every phase before this):
  1. `AuditFramework::execute`'s scoring loop defaulted every domain to a 100.0 score whenever it had zero `Document` objects — true for `python_hackathon`'s entire rule set (`file_presence`/`glob_match` act on the filesystem, no `Document` ever gets created), so a domain audit with real findings still silently reported "no findings." Fixed: domain-wide `weighted_pass_rate` over all findings when there's no document to average over, preserving the existing per-document-average path unchanged for `base_dev`-shaped standards.
  2. `report.score.overall` (feeds `rating`/`readiness`) has *always* defaulted to 100 for every standard, not just the new path — `calculation_inputs` names bare bucket keys (`"deterministic_whole"`) but `bucket_scores` only ever held domain-prefixed keys (`"infrastructure_deterministic_whole"`), so the lookup never matched anything. Fixed: strip the audited domain's prefix before the `weighted_sum` lookup when one domain is being audited; whole-standard (`domain: None`) audits keep the prior behavior (no single prefix to strip — a real cross-domain final score is Phase 3a's already-declined team/session territory, not rebuilt here).
  3. Introduced by this session's own earlier Pass 7 fix (deriving bucket names from file paths instead of a hardcoded list) and caught by the same end-to-end check: `db_reader.rs`'s `calculation_inputs` query still had `WHERE cr.bucket = 'final_score'` — a literal string that stopped matching the moment bucket naming stopped being a fixed hardcoded set (the bucket is now named `summary_final_score`). Fixed: filter by `calculation_method = 'weighted_sum'` instead — the semantic property that actually matters, not a name a standard happens to spell one particular way.
- **Script distribution** (gap #3) — `register_standard` now copies a standard's `script/` directory into `.samgraha/scripts/` (immediate, this repo) and `mcp_dir()/scripts/` (global, so a future `sync` elsewhere picks it up too) — the same dual-write pattern already used for `standards.db` itself. Closes the "scripts are referenced by rules but never physically copied anywhere `run_check` looks" gap Phase 4a flagged.

Verified end-to-end against real `python_hackathon` data throughout (not synthetic fixtures): a scratch repo with only `uv.lock` present scored `infrastructure` at 40.0 (matches hand-computed `weighted_pass_rate` against the real rule weights: 1.0 passed / 2.5 total) and an overall `70.0` (40×0.25 det_whole + 40×0.25 det_section + 100×0.25 + 100×0.25 semantic-not-yet-built default). 8 new tests across `audit::framework` and `mcp::adapter`. Full workspace build and test suite green.

- **`calculation/validation/scoring_validation.yaml` loader pass** (gap #7) — new `validation_rules` table + loader Pass 9, mirroring `calculation_rules`'s shape exactly (one row per check, `rule` is prose same as `condition` is on audit rules — not an expression this or any Rust code evaluates, loaded and exposed via `ScoringConfig.validation_rules`, not enforced). Verified against real data: `python_hackathon` → 12 rows (`val-001`..`val-012`), `base_dev` → 0 rows, no error (it has no `calculation/validation/` directory at all — same "optional, not missing" convention Pass 7 already uses).
- **`domain_relationships.enforce_order` wired into `StandardWorkflowPlanner`** (gap #6) — a tier's domains now split into topological layers (Kahn's algorithm, cycle-safe) only when an `enforce_order` edge exists between two domains *within that tier*; a tier with no such edge stays exactly the single flat group it was before (true for every tier in both real standards but one). `StandardRelationship` gained `enforce_order`/`tier_gating_strict` fields — `db_reader.rs` was querying `domain_relationships`/`relationship_types` already but never selected those two columns. Verified against `base_dev`'s one real documented exception (External Context before Engineering, both tier 2): the generated plan splits tier 2 into `[architecture, design, external-context, feature, security]` then `[engineering]`, with the second layer's phases correctly depending on the first's.
- **`FixPhaseExecutor` domain-audit gap (gap #1) — investigated deeper, correctly left unbuilt, not force-closed.** The original blocker (a numeric `report_id`) turned out not to be real: `fix_sessions.report_id` has no FK constraint (`registry/src/migration.rs` V28) — pure bookkeeping. The actual blocker is upstream of that: `resolve_finding_path` requires `finding.document_id` or `finding.location`; `file_presence`/`glob_match` findings (python_hackathon's real shape — "Dockerfile missing") have neither, because they're about a file's *absence*, not content inside some document. Even with a path, every `FixPlanner` variant generates edits to *existing* content — none can scaffold a missing file from nothing. Building that is new judgment-requiring capability (what should a generated Dockerfile contain?), not a plumbing gap to close here. Left explicitly documented in `executors.rs`, not silently no-op'd.
- **Semantic-rule execution wired end-to-end (gap #4) — bigger than first scoped.** `StandardDefinition` gained a `semantic_rules` field (`kind = 'semantic'` rows, previously not loaded into anything — `db_reader.rs`'s rules query filtered them out entirely); `AuditFramework::execute` now feeds `semantic_rules` to the `"semantic"` provider specifically instead of the deterministic `audit_rules` both providers previously received; `SemanticAuditProvider` (previously ignored its `rules` param completely — hardcoded heuristics only, e.g. word-count/vague-language checks, unrelated to any standard's own rules) now additionally turns each semantic rule into a review-task-eligible finding per document (or one repo-wide finding when there's no document at all — python_hackathon's shape), letting the existing `semantic_review` bundling pick them up unchanged. **Checked deeper and found the loader itself had a gap**: Pass 5b only globs `audit/semantic/**/*.md` (base_dev's per-criterion markdown-table rubric shape) — `python_hackathon`'s entire semantic layer is `audit/semantic/document/*.yaml` (one `prompt_template`/`ensemble.required_models` prompt per domain, a structurally different shape), which Pass 5b's glob never even saw. Added Pass 5b2 for that shape. Verified end-to-end against real data: `python_hackathon`'s `rules` count went from 32 (deterministic only) to 42 (10 real semantic rules added, one per domain), and a full `AuditFramework` run against `infrastructure` returned the real `prompt_template` text as a `provider: "semantic"` finding.
- **`plan/core/loop.yaml`'s `stages:` list ingested (gap #5)** — new `workflow_stages` table (one row per stage, `params_json` for that stage's own flat param dict — no per-param child table, unlike `rule_evidence_params`, since no real stage has a multi-value param) + loader Pass 10, exposed via `StandardRegistry.workflow_stages()`. `python_hackathon` → 10 stages, in order, matching the real file exactly (`repository → audit(deterministic) → audit(semantic) → calculate(deterministic) → calculate(semantic/ensemble) → aggregate(domain) → calculate(relative/competition) → normalize(final,cap:20) → validate(scoring) → report(generation)`); `base_dev` → 0 (no flat `stages:` list in its own, structurally different loop.yaml — correctly empty, not an error). Data availability only, per Phase 3a's original scoping ("closing the gap on the file that already exists," not a new execution engine) — nothing yet reads `workflow_stages()` to actually drive a `repository`-scope run; that's the next real piece once something needs it.

## Explicit Non-Goals

- No hardcoded leaderboard/ensemble-ranking formula in Rust, ever — that's `script/leaderboard.py`'s job, reached via Phase 4's workflow-script contract.
- No second phase-execution/DAG engine — `ProjectPlan`/`ProjectPhase`/`PlanOrchestrator` already exists and already does dependency-tracked, status-tracked phase execution; Phase 3 feeds it standard data instead of replacing it. Model A's DAG-conditions code (from the earlier monolithic-pipeline work) also already exists and stays unused until a standard's workflow file actually needs conditional branching — building a third mechanism for the same concern would be the same mistake three times.
- No new template-authoring format — `templates/generation/**` and `templates/audit/**` are Tera markdown, same as `docs/raw/report-templates/` already uses.
- No hook/callback system (`on_success`/`on_failure` per stage) — a `script` stage runs where the workflow file puts it in sequence; that's the whole answer to "when do scripts run," confirmed against both real `loop.yaml` files having no conditional stages at all.
- No second YAML→DB ingestion pipeline in Rust — `knowledge-hub-loader.py` already owns that job for rules/calculation/domains/plan-scenarios; new schema (Phase 3a, Phase 2's validation pass) extends it with new passes in the same file, not a parallel Rust parser.
- No calculation-method free-text expression language (parsing `formula:`'s prose into an executable AST). `calculation_method` is already a plain DB string dispatched by a small fixed match (Phase 2's table) — extending the *vocabulary* by adding a row + a match arm satisfies "no hardcode" without needing a general-purpose formula parser nothing in either real standard actually requires.

## Open Questions

1. Semantic section rubrics are markdown prose (Phase 1) — is a rubric-per-section-file the right long-term shape, or should `base_dev` eventually move these into the same structured YAML as document-scope semantic rules? Not blocking Phase 1 (loader treats them as opaque text either way), but worth deciding before a third standard copies whichever pattern exists first.
2. `reliability_aware_ensemble` (Phase 2) needs real scores from `ensemble.required_models` to average — today nothing in samgraha calls out to multiple LLM providers concurrently. Until that exists, the interpreter can compute the formula against whatever scores are supplied (e.g., one score per `audit` call, tagged by `model`, pulled from `standard_audit_runs`), leaving true concurrent multi-provider orchestration as a separate, later decision.
3. Phase 3's template fallback tier — repo override beats standard default, but should a *user's* `.samgraha/standards/{name}/templates/` override (if they've locally customized a registered standard) sit between those two tiers, same as the script chain's `.samgraha/scripts/` tier does?
4. `ProjectCase`'s exact extension shape (Phase 3) — a new variant carrying `{standard, workflow}` vs. some other mechanism for selecting `StandardWorkflowPlanner` alongside the 4 fixed built-in cases. Needs a look at every existing `match case` site before picking, since `ProjectCase` is matched in more places than just `resolve_planner()`.
5. ~~Phase 3a's workflow composition~~ — moot, team/session dropped from scope entirely.
6. ~~Has `register_standard` ever actually been run against `python_hackathon`/`base_dev`?~~ **Resolved.** Ran `knowledge-hub-loader.py --dry-run` directly against both while writing this update. Both load cleanly, zero errors: `python_hackathon` — 10 domains, 32 rules, 6 calculation_rules, 24 plan_scenarios (`script_checks`/`templates` correctly 0 — its scripts aren't check-shaped, see Phase 4). `base_dev` — 16 domains, 1165 rules, 6 calculation_rules, 96 plan_scenarios, 18 script_checks, 233 templates. Nothing blocking Phase 1/2 consumption work.
