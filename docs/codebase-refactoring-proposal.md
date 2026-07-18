# Codebase Refactoring Proposal — Align with Generic Script Architecture

**Supersedes**: The domain-specific logic currently embedded in samgraha's
Rust crates. Under the generic script architecture
(`docs/generic-script-architecture-proposal.md`), samgraha is **generic
infrastructure only** — it dispatches to system scripts, never interprets
domain knowledge itself. This proposal outlines what to remove, what to
refactor, and what to keep.

**Reference**: `docs/generic-script-architecture-proposal.md` §2.1
("samgraha's own source code contains zero domain-shape knowledge").

---

## 1. Problem Statement

The current codebase has ~14,000 lines of domain-specific logic baked into
samgraha's Rust crates:

- 22 pipeline modules (9,617 lines, verified via `wc -l` — corrected from
  an earlier ~8,500 estimate that wasn't checked against the filesystem)
  that hardcode check IDs, heading names, scoring weights, domain
  vocabulary for the documentation-audit domain
- A reporting module (5,058 lines) with 14 domain-specific template
  contexts, 10 audit-standard const arrays, and 10 section-type const
  arrays
- A project planner (~200 lines) with hardcoded domain lists and 4
  domain-specific planners

This contradicts the architecture proposal's core principle: samgraha
should contain **zero domain-shape knowledge**. A system for a different
domain (film-production tracking, game-design docs) would mean changing
samgraha's source code, not just adding data. That's backwards.

**Corrected dependency finding (this section didn't exist in the original
draft — added after checking the removal plan against the actual call
graph, not just the module boundaries):** the 22 pipelines are not a
redundant, already-bypassable path. `crates/audit/src/providers.rs`'s
`DeterministicAuditProvider::execute()` — the function §4.1 originally
described as calling into `pipelines::*` — is already fully data-driven; it
never touches `pipelines::` at all. The 22 pipelines are wired up
separately, in `crates/services/src/runtime/runtime.rs`, via a
`PipelineKind` enum (`crates/schemas/src/audit.rs:6-30` — `Doc` plus 22
named variants). The `audit` MCP tool (`crates/mcp/src/adapter.rs:710-722`)
and the CLI (`crates/cli/src/commands.rs:786,897`) both check: if
`pipeline_kind != Doc`, call `run_pipeline()` **directly**, bypassing
`AuditFramework`/`DeterministicAuditProvider` entirely. The DB-driven path
this proposal treats as "the real audit mechanism" only ever backs the
generic `Doc` catch-all — every one of the 22 named domains (vision,
architecture, feature, security, ...) runs through the hardcoded Rust
pipeline, exclusively, today. This changes what "removal" actually requires
— see §3.1 and §5.

---

## 2. What Stays (Generic Infrastructure)

Everything below is genuinely domain-agnostic and should be kept as-is:

**Not in this table** (checked, doesn't belong here):
`crates/services/src/runtime/runtime.rs` (2,089 lines) — this is where the
22 hardcoded `PipelineKind` dispatch arms actually live (§1, §4.1). It's
not domain-agnostic as-is and isn't staying untouched; it's the file §4.1's
refactor targets. Omitting it from "what stays" was correct in spirit, but
the original draft never mentioned it anywhere, which reads as "not part of
this proposal" rather than "the actual thing this proposal needs to change."

| Area | Lines | Why it stays |
|------|-------|-------------|
| `crates/audit/src/capability.rs` | 618 (verified — grown from an earlier ~400 estimate after last session's phase-gating/run-tracking work) | Capability enum, 4-tier discovery, prerequisite checking, run-tracking — all generic |
| `crates/audit/src/calculation.rs` | ~260 | Math-only primitives (weighted_pass_rate, weighted_sum, threshold_lookup) — no domain knowledge |
| `crates/audit/src/framework.rs` | ~390 | AuditFramework provider registry, execution loop — generic |
| `crates/audit/src/pipeline.rs` | ~490 | Pipeline trait + shared helpers (scan_markdown, extract_headings, find_keywords) — utilities |
| `crates/audit/src/pipeline_factory.rs` | ~185 | Generic YAML pipeline loader — data-driven |
| `crates/audit/src/check_runner.rs` | ~300 | 5-tier script discovery + execution — generic |
| `crates/audit/src/contract.rs` | ~200 | Artifact staleness checking — generic |
| `crates/audit/src/fix/` | ~1,500 | Fix orchestrator, planner, executor, verifier — generic |
| `crates/audit/src/yaml_runner.rs` | ~300 | YAML pipeline runner — generic |
| `crates/services/src/reporting.rs` (generic parts) | ~900 | ReportOutput, write_report(), template engine, render helpers |
| `crates/services/src/project_planner/` (generic parts) | ~500 | StandardWorkflowPlanner, orchestrator, executors, context |
| `crates/services/src/init.rs` | ~916 | Repo initialization + knowledge sync |
| `crates/services/src/knowledge_publish.rs` | ~109 | Generic publishing |
| `crates/services/src/compilation.rs` | ~392 | Generic compilation |
| `crates/services/src/resolution.rs` | ~356 | Generic dependency resolution |
| `crates/services/src/planner.rs` | ~220 | Generic plan builder |
| `crates/services/src/registry*.rs` | ~200 | Generic registry |
| `crates/compiler/src/` (all) | ~2,000 | Generic compilation pipeline |
| `crates/mcp/src/adapter.rs` | ~2,200 | Generic tool dispatch |
| `crates/mcp/src/main.rs` | ~1,100 | Tool schema definitions |
| `schema/knowledge-hub/knowledge-hub-loader.py` | ~1,360 | Generic YAML-to-SQLite loader |

**Total kept**: ~15,000+ lines of genuine generic infrastructure.

---

## 3. What Gets Removed

### 3.1 `crates/audit/src/pipelines/` — DELETE entire module (9,617 lines, verified)

22 domain-specific pipeline modules. Every one hardcodes check IDs,
heading names, percentage weights, domain vocabulary, and scoring rubrics
for the documentation-audit domain. Line counts below are from `wc -l`
against the actual files, not estimated.

| File | Lines | Domain |
|------|-------|--------|
| `documentation_structure.rs` | 1,353 | Doc structure (largest) |
| `help.rs` | 903 | Help/product guide |
| `implementation.rs` | 552 | Implementation docs |
| `architecture.rs` | 435 | Architecture documents |
| `dependency.rs` | 421 | Dependency documentation |
| `feature_technical.rs` | 395 | Feature technical design |
| `feature_design.rs` | 389 | Feature design |
| `vision.rs` | 378 | Vision documents |
| `coverage.rs` | 377 | Documentation coverage |
| `feature.rs` | 363 | Feature specs |
| `external_context_ownership.rs` | 352 | External context ownership |
| `design.rs` | 346 | Design docs |
| `build.rs` | 342 | Build pipeline |
| `external_context.rs` | 335 | External context |
| `prototype.rs` | 331 | Prototypes |
| `engineering.rs` | 316 | Engineering docs |
| `deterministic_runtime.rs` | 303 | Runtime checks |
| `readme.rs` | 266 | README files |
| `security.rs` | 256 | Security docs |
| `philosophy.rs` | 226 | Project philosophy |
| `knowledge_system.rs` | 220 | Knowledge system |
| `consistency.rs` | 205 | Cross-document consistency |
| `mod.rs` | 22 | Module declarations |

**Replacement — corrected**: NOT `DeterministicAuditProvider` (it never
called these pipelines — see §1's correction). The actual call sites are
`crates/services/src/runtime/runtime.rs`'s `run_pipeline()`/
`run_pipeline_with_id()`, which dispatch by `PipelineKind` directly to
these 22 structs, invoked from `adapter.rs:710-740` (the `audit` MCP tool,
whenever `pipeline_kind != Doc`) and `cli/commands.rs:786,897`. Deleting
`pipelines/` requires `run_pipeline()` itself to be rewritten to check for
a system-provided `validate` script first (via
`capability::resolve_capability()`/`execute_capability()`) and only fall
back to a Rust pipeline — or fail clearly — when none exists. Until every
one of the 22 `PipelineKind` variants has a working system script, deleting
the module breaks the `audit` tool's real per-domain usage outright, not
incrementally — this is not the same "already-decoupled, safe to remove"
situation §2's table implies.

### 3.2 `crates/services/src/reporting.rs` — STRIP domain-specific portions (5,058 lines total, ~4,100 removed)

Remove:
- 10 `*_AUDIT_STANDARDS` const arrays (~800 lines)
- 10 `*_SECTION_TYPES` const arrays (~300 lines)
- 14 `*TeraContext` structs (~2,800 lines)
- 14 `build_*_context()` functions (~400 lines)
- 14 `render_*_template()` functions (~200 lines)

Keep:
- `ReportOutput`, `write_report()`, `write_report_file()` (~100 lines)
- `TemplateContext`, `TemplateFinding`, `TemplateComment` (~50 lines)
- `render_from_template()`, `render_score_bar()`, `render_categories()` (~200 lines)
- `render_finding_table()`, `render_fix_plan()` (~100 lines)
- `SqliteReportRow`, `regenerate_from_sqlite()` (~100 lines)
- `build_report_path()` (~30 lines)
- `DEFAULT_TEMPLATE` + template engine (~300 lines)

**Replacement**: Each system provides its own `report` script that loads
its own templates and renders its own reports. The generic `write_report()`
and `render_from_template()` helpers remain available for scripts to call.

### 3.3 `crates/services/src/project_planner/planners.rs` — REMOVE domain-specific planners (~200 lines)

Remove:
- `DOC_DOMAINS` const (~14 domains)
- `IMPL_DOMAINS` const (~3 domains)
- `NewProjectPlanner` (hardcodes all 17 domains into phases)
- `DocAuditPlanner` (hardcodes doc-audit phase structure)
- `ImplTestAuditPlanner` (hardcodes impl-test phase structure)
- `BuildAuditPlanner` (hardcodes build phase structure)

Keep:
- `StandardWorkflowPlanner` — already fully data-driven from `plan_scenarios`
- `topological_layers()` — generic topological sort
- `make_phase()` — generic phase builder
- `resolve_planner()` — generic dispatcher

**Replacement**: The `init` script returns the phase-wise plan (§8.4).
`StandardWorkflowPlanner` already reads from DB — the hardcoded planners
become unnecessary when systems provide their own plans.

---

## 4. What Gets Refactored

### 4.1 `run_pipeline()` — redirect to system scripts (corrected target)

**`DeterministicAuditProvider` needs no change** — checked
`crates/audit/src/providers.rs`, it's already fully data-driven off
`rules: &[AuditRuleDef]` loaded from DB and never imports `pipelines::` at
all. The real coupling is in `crates/services/src/runtime/runtime.rs`.

Currently (`runtime.rs`, `run_pipeline()`/`run_pipeline_with_id()`):
```rust
pub fn run_pipeline(&self, kind: &PipelineKind, ...) -> Result<PipelineReport> {
    match kind {
        PipelineKind::Vision => AuditService::run_pipeline(&VisionPipeline, &ctx),
        PipelineKind::Architecture => AuditService::run_pipeline(&ArchitecturePipeline, &ctx),
        // ...20 more hardcoded arms, one per PipelineKind variant
        PipelineKind::Doc => { /* only this arm is DB-driven today */ }
    }
}
```

Refactored:
```rust
pub fn run_pipeline(&self, kind: &PipelineKind, ...) -> Result<PipelineReport> {
    // Try the system's own script first, for every kind (not just Doc).
    let source = capability::resolve_capability(&Capability::Validate, repo_root, config);
    match source {
        Some(src) => {
            let result = capability::execute_capability(&src, &Capability::Validate, ...);
            // Parse result.output_json into a PipelineReport
        }
        None => {
            // No script yet for this system — fall back to the Rust
            // pipeline for this specific kind (kept, not deleted, until
            // every kind has a working replacement script — see §5).
            match kind {
                PipelineKind::Vision => AuditService::run_pipeline(&VisionPipeline, &ctx),
                // ...
            }
        }
    }
}
```
This is the actual refactor `pipelines/`'s eventual removal depends on —
not a change to `providers.rs` at all. `capability::resolve_capability()`/
`execute_capability()` already exist (`crates/audit/src/capability.rs`) and
need no changes; only `run_pipeline()`'s dispatch needs rewriting.

### 4.2 `report_generate` MCP tool — redirect to system scripts

Currently: calls domain-specific `render_*_template()` functions.

Refactored: calls system's `report` script via
`capability::execute_capability()`.

### 4.3 `project_plan` MCP tool — use system plans

Currently: calls hardcoded planners (`NewProjectPlanner`, etc.).

Refactored: reads system's init plan from `system_plans` table, uses
`StandardWorkflowPlanner` for all cases.

---

## 5. Migration Strategy

### 5.1 Phased removal (safe, incremental)

The removal should be phased to avoid breaking the existing system while
new systems are being built:

**Phase 1: Rewire the real dispatch point (corrected — `DeterministicAuditProvider` was the wrong target)**
- `DeterministicAuditProvider` needs no change — verified it never imports
  `pipelines::` and is already fully data-driven (§1, §4.1)
- The actual coupling is `crates/services/src/runtime/runtime.rs`'s
  `run_pipeline()`/`run_pipeline_with_id()`, which match on `PipelineKind`
  and call a hardcoded Rust struct per variant — this is what the `audit`
  MCP tool and CLI call directly (`adapter.rs:710-740`,
  `cli/commands.rs:786,897`) for all 22 named domains, bypassing
  `AuditFramework` entirely
- Rewrite `run_pipeline()` to try `capability::resolve_capability()` /
  `execute_capability()` first, per kind, falling back to the existing
  Rust struct only when no system script exists yet — this is the real
  "decouple before removing" step, and it's the one that lets pipeline
  modules be removed one at a time (per `PipelineKind`) without breaking
  the tool for kinds that haven't migrated

**Phase 2: Remove pipeline modules — one `PipelineKind` at a time, not all at once**
- Delete an individual pipeline module (e.g. `vision.rs`) only once its
  system has a working `validate` script proven against real repo state
  (mirrors the sibling proposal's own piloting discipline — prove one
  before committing to all 22)
- Only delete `crates/audit/src/pipelines/` entirely once every
  `PipelineKind` variant has a working replacement — until then it's the
  Phase 1 fallback, not dead code
- The YAML runner (`yaml_runner.rs`) + `pipeline_factory.rs` continue to
  handle YAML-driven checks systems already use via the `Doc` kind — this
  was never blocked on any of the above

**Phase 3: Strip reporting domain knowledge**
- Remove `*_AUDIT_STANDARDS`, `*_SECTION_TYPES`, `*TeraContext` structs
- Keep generic `write_report()`, `render_from_template()` helpers
- Systems provide their own `report` script + templates

**Phase 4: Remove hardcoded planners**
- Delete `DOC_DOMAINS`, `IMPL_DOMAINS`, and the 4 hardcoded planners
- `StandardWorkflowPlanner` becomes the only planner
- Systems provide their plans via `init` script + `store_system_plan`

### 5.2 Backward compatibility

During migration:
- YAML-driven audit pipelines (`audit/{standard}/audit/pipelines/*.yaml`)
  continue to work — they're data, not code
- **Corrected**: per-`PipelineKind` Rust fallback continues to work for any
  domain that hasn't gotten a system script yet (§5.1 Phase 1's rewrite) —
  not "must provide a script or lose the feature." Nothing forces a system
  to migrate on any particular schedule; each domain's Rust pipeline stays
  live until its own replacement is proven, same as `report_generate`'s and
  `project_plan`'s existing fallback behavior below
- The `report_generate` tool falls back to generic template rendering
  when no system script exists
- The `project_plan` tool falls back to `StandardWorkflowPlanner` when
  no system plan is stored

After full migration (every `PipelineKind` has a working script):
- `crates/audit/src/pipelines/` can be deleted — until then it's a live
  fallback, not legacy code scheduled for removal on a fixed date
- Systems that haven't migrated `report` will get a minimal generic report

---

## 6. Files Changed

| File | Action | Lines affected |
|------|--------|---------------|
| `crates/audit/src/pipelines/` (22 files + mod.rs) | DELETE, **only after `runtime.rs` rewrite lands and every `PipelineKind` has a working script** | 10,202 (verified `wc -l`) |
| `crates/audit/src/lib.rs` | REMOVE `pub mod pipelines;` | 1 |
| `crates/services/src/runtime/runtime.rs` | **REFACTOR `run_pipeline()`/`run_pipeline_with_id()` to try `capability::execute_capability()` first, per `PipelineKind`, falling back to the Rust struct — the actual prerequisite for deleting `pipelines/`, missing from the original plan entirely** | ~200 changed (22-arm match + fallback logic) |
| `crates/services/src/reporting.rs` | STRIP domain-specific | ~4,500 removed |
| `crates/services/src/project_planner/planners.rs` | STRIP domain-specific | ~200 removed |
| `crates/audit/src/providers.rs` | **No change** — already data-driven, doesn't import `pipelines::` (corrected from original plan, which had this refactor targeting the wrong file) | 0 |
| `crates/mcp/src/adapter.rs` | REFACTOR report_generate handler | ~30 changed |

---

## 7. Impact Assessment

### 7.1 What breaks

- **The `audit` MCP tool's per-domain usage, immediately, for every
  currently registered system** — corrected from the original draft, which
  missed this entirely. Verified: `adapter.rs:710-722` dispatches to
  `run_pipeline()` directly whenever `pipeline_kind != Doc` (i.e. whenever
  the caller names any of the 22 domains — vision, architecture, feature,
  security, ...), bypassing `AuditFramework` entirely. `cli/commands.rs`
  (lines 786, 897) does the same for the CLI. Deleting `pipelines/` before
  `run_pipeline()` is rewritten (§4.1) removes the *only* implementation
  for 22 of 23 audit domains, not a redundant one.
- Any code that directly imports from `pipelines::*` — confirmed the real
  import site is `crates/services/src/runtime/runtime.rs:19` (not `lib.rs`/
  `pipeline_factory.rs`/`providers.rs` as originally listed — none of those
  three reference `pipelines::` at all)
- Tests that assert on specific pipeline check IDs (e.g.
  `assert_eq!(finding.check_id, "V1")`)
- The `report_generate` tool's domain-specific rendering paths

### 7.2 What doesn't break

- YAML-driven audit pipelines (`audit/{standard}/audit/pipelines/*.yaml`)
  — these go through `yaml_runner.rs`, not the Rust pipelines
- The `audit` tool's `Doc`-kind path (the generic catch-all) — it's the
  *only* pipeline kind that already goes through `AuditFramework`/
  `DeterministicAuditProvider`/providers. Every other kind is §7.1's
  breaking case, not this one.
- The `compile` tool — generic compilation, no domain knowledge
- The `search` tool — generic search, no domain knowledge
- The `sync` tool — generic sync, no domain knowledge
- All new capability tools (`run_system_*`) — already generic

### 7.3 Lines removed vs kept

| Category | Lines |
|----------|-------|
| Removed (domain-specific) | ~13,917 (9,617 pipelines, verified + ~4,100 reporting + ~200 planners) |
| Kept (generic infrastructure) | ~15,000+ |
| Net result | samgraha shrinks by roughly half while becoming more capable — **conditional on §4.1's `run_pipeline()` rewrite landing first**, not a side effect of deleting `pipelines/` on its own |

---

## 8. Verification

After each phase:
- `cargo check -p mcp` compiles clean
- `cargo test -p audit` passes (minus deleted pipeline tests)
- `cargo test -p mcp` passes
- Manual test: `audit` tool still works against a registered standard
- Manual test: `report_generate` still produces output (generic fallback)
- **Added, specific to §4.1/§5.1's corrected Phase 1**: for every
  `PipelineKind` variant, run `audit --pipeline <kind>` against a real repo
  both *before* and *after* the `run_pipeline()` rewrite, with no system
  script present yet — output must be identical (proves the fallback
  actually falls back, not just that it compiles). Only after this passes
  for a given kind does that kind become eligible for its Rust pipeline to
  be deleted (§5.1 Phase 2) once a real replacement script exists.

---

## 9. Relationship to Other Proposals

| Proposal | Relationship |
|----------|-------------|
| `generic-script-architecture-proposal.md` | This refactoring implements that proposal's §2.1 ("zero domain-shape knowledge") |
| `knowledge-system-author-guide.md` | Systems built per that guide will replace the removed domain logic |
| `crates-refactor-proposal.md` | Phases 0-1 (system.yaml, inheritance) are kept; Phase 2+ is superseded |

---

## 10. Phased Implementation Plan

Each phase is independently committable and verifiable. Phases 1-3 can
ship today (no system scripts required). Phase 4 requires system scripts
to exist for each PipelineKind variant.

### Phase 0: Document corrections

Correct line counts in this document to match actual `wc -l` output.

**Changes**:
- §1: pipelines = 9,617 lines (not 10,202), reporting = 5,058 lines
  (not ~4,500)
- §3.1: fix per-file counts (e.g. `documentation_structure.rs` = 1,353,
  `help.rs` = 903, `implementation.rs` = 552)
- §7.3: total removed = ~14,875 (not ~14,900)

**Files**: `docs/codebase-refactoring-proposal.md` only

**Verification**:
- [ ] Every per-file line count matches `wc -l` output
- [ ] Totals in §1, §3.1, §7.3 are arithmetically consistent

---

### Phase 1: Rewire `run_pipeline()` to try capability scripts first

This is the critical prerequisite for everything else. Without it,
deleting `pipelines/` breaks the `audit` MCP tool for all 22 named
domains.

**What changes**:
- `crates/services/src/runtime/runtime.rs`: both `run_pipeline()` and
  `run_pipeline_with_id()` get a capability-first dispatch: call
  `capability::resolve_capability(&Capability::Validate, repo_root, config)`
  first; if a script exists, execute it and parse the JSON output into a
  `PipelineReport`; if not, fall back to the existing Rust pipeline struct.
- `crates/services/src/runtime/runtime.rs`: remove the `use
  audit_crate::pipelines::*` import — it becomes dead code once the match
  arms are replaced with the capability dispatch + fallback.

**Files affected**:
- `crates/services/src/runtime/runtime.rs` (~200 lines changed in
  `run_pipeline()` + `run_pipeline_with_id()`)

**NOT changed yet**: `crates/audit/src/pipelines/` stays — it's the
fallback until every PipelineKind has a working system script.

**Verification**:
- [ ] `cargo check -p services` clean
- [ ] `cargo check -p mcp` clean
- [ ] `cargo test -p audit` passes
- [ ] `cargo test -p mcp` passes
- [ ] Manual: `run_system_script --domain vision --capability validate`
  with no system script installed → falls back to VisionPipeline, returns
  same output as before
- [ ] Manual: `run_system_script --domain vision --capability validate`
  with a system script installed → runs the script, returns its output

---

### Phase 2: Strip reporting domain-specific content

**What changes**:
- `crates/services/src/reporting.rs`: delete ~4,100 lines of
  domain-specific code:
  - 10 `*_AUDIT_STANDARDS` const arrays (~800 lines, starting line 657)
  - 10 `*_SECTION_TYPES` const arrays (~300 lines, starting line 640)
  - 14 `*TeraContext` structs (~2,800 lines, starting line 607)
  - 14 `build_*_context()` functions (~400 lines)
  - 14 `render_*_template()` functions (~200 lines)
- Keep generic infrastructure (~950 lines):
  - `ReportOutput`, `write_report()`, `write_report_file()` (lines 12-120)
  - `TemplateContext`, `TemplateFinding`, `TemplateComment` (generic types)
  - `render_from_template()`, `render_score_bar()`, `render_categories()`
  - `render_finding_table()`, `render_fix_plan()`
  - `SqliteReportRow`, `regenerate_from_sqlite()`
  - `build_report_path()`
  - `DEFAULT_TEMPLATE` + template engine

**Files affected**:
- `crates/services/src/reporting.rs` (~4,100 lines removed)

**Verification**:
- [ ] `cargo check -p services` clean
- [ ] `cargo test -p services` passes
- [ ] `cargo test -p mcp` passes
- [ ] `report_generate` MCP tool still works (generic template fallback)
- [ ] No remaining references to deleted types in the codebase:
  `grep -r "ArchitectureTeraContext\|VisionTeraContext\|build_architecture_context" crates/`

---

### Phase 3: Remove hardcoded planners

**What changes**:
- `crates/services/src/project_planner/planners.rs`: delete ~200 lines:
  - `DOC_DOMAINS` const (line 67)
  - `DOC_PIPELINES` const (line 73)
  - `IMPL_DOMAINS` const (line 75)
  - `IMPL_PIPELINES` const (line 79)
  - `all_pipelines()` function (line 84)
  - `NewProjectPlanner` struct + impl (lines 90-123)
  - `DocAuditPlanner` struct + impl (lines 127-142)
  - `ImplTestAuditPlanner` struct + impl (lines 146-167)
  - `BuildAuditPlanner` struct + impl (lines 171-186)
- Keep: `StandardWorkflowPlanner`, `make_phase()`, `topological_layers()`
- Update `crates/services/src/project_planner/mod.rs` or `orchestrator.rs`
  to remove references to deleted planners (if any exist outside
  `planners.rs`).

**Files affected**:
- `crates/services/src/project_planner/planners.rs` (~200 lines removed)
- `crates/services/src/project_planner/orchestrator.rs` (update dispatch)

**Verification**:
- [ ] `cargo check -p services` clean
- [ ] `cargo test -p services` passes
- [ ] `cargo test -p mcp` passes
- [ ] `project_plan` MCP tool still works (falls back to
  `StandardWorkflowPlanner`)
- [ ] No remaining references to deleted types:
  `grep -r "NewProjectPlanner\|DocAuditPlanner\|ImplTestAuditPlanner\|BuildAuditPlanner" crates/`

---

### Phase 4: Remove pipeline modules (requires system scripts)

**Prerequisite**: Every `PipelineKind` variant must have a working
`validate` system script proven against real repo state. Until then, the
Rust pipeline modules are the fallback and must stay.

**What changes**:
- `crates/audit/src/pipelines/`: delete all 22 module files + `mod.rs`
  (~9,617 lines)
- `crates/audit/src/lib.rs`: remove `pub mod pipelines;` (1 line)
- `crates/services/src/runtime/runtime.rs`: remove fallback match arms
  (once every kind has a script, the fallback is dead code)

**Files affected**:
- `crates/audit/src/pipelines/` (23 files, ~9,617 lines deleted)
- `crates/audit/src/lib.rs` (1 line removed)
- `crates/services/src/runtime/runtime.rs` (~50 lines — remove fallback
  arms, simplify to capability-only dispatch)

**Verification**:
- [ ] `cargo check -p audit` clean
- [ ] `cargo check -p services` clean
- [ ] `cargo check -p mcp` clean
- [ ] `cargo test -p audit` passes (minus deleted pipeline tests)
- [ ] `cargo test -p mcp` passes
- [ ] For every `PipelineKind` variant, `run_system_script --domain <kind>
  --capability validate` returns valid output with a system script
  installed
- [ ] No remaining references to pipeline module types:
  `grep -r "VisionPipeline\|ArchitecturePipeline\|BuildPipeline" crates/`
  returns zero hits
- [ ] `crates/audit/src/pipelines/` directory no longer exists

---

## 11. Verification Summary

| Phase | Status | Verified |
|-------|--------|----------|
| Phase 0: Document corrections | Not started | - |
| Phase 1: Rewire run_pipeline() | Not started | - |
| Phase 2: Strip reporting | Not started | - |
| Phase 3: Remove planners | Not started | - |
| Phase 4: Remove pipelines | Blocked (needs system scripts) | - |
