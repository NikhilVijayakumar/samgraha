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

The current codebase has ~13,000 lines of domain-specific logic baked into
samgraha's Rust crates:

- 22 pipeline modules (~8,500 lines) that hardcode check IDs, heading
  names, scoring weights, and domain vocabulary for the documentation-audit
  domain
- A reporting module (~4,500 lines) with 14 domain-specific template
  contexts, 10 audit-standard const arrays, and 10 section-type const
  arrays
- A project planner (~200 lines) with hardcoded domain lists and 4
  domain-specific planners

This contradicts the architecture proposal's core principle: samgraha
should contain **zero domain-shape knowledge**. A system for a different
domain (film-production tracking, game-design docs) would mean changing
samgraha's source code, not just adding data. That's backwards.

---

## 2. What Stays (Generic Infrastructure)

Everything below is genuinely domain-agnostic and should be kept as-is:

| Area | Lines | Why it stays |
|------|-------|-------------|
| `crates/audit/src/capability.rs` | ~400 | Capability enum, 5-tier discovery, prerequisite checking — all generic |
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

### 3.1 `crates/audit/src/pipelines/` — DELETE entire module (~8,521 lines)

22 domain-specific pipeline modules. Every one hardcodes check IDs,
heading names, percentage weights, domain vocabulary, and scoring rubrics
for the documentation-audit domain.

| File | Lines | Domain |
|------|-------|--------|
| `vision.rs` | 378 | Vision documents |
| `architecture.rs` | 435 | Architecture documents |
| `engineering.rs` | 316 | Engineering docs |
| `feature.rs` | 363 | Feature specs |
| `feature_design.rs` | 389 | Feature design |
| `feature_technical.rs` | 395 | Feature technical design |
| `philosophy.rs` | 226 | Project philosophy |
| `prototype.rs` | 331 | Prototypes |
| `readme.rs` | 266 | README files |
| `security.rs` | 256 | Security docs |
| `help.rs` | 903 | Help/product guide |
| `consistency.rs` | 205 | Cross-document consistency |
| `coverage.rs` | 377 | Documentation coverage |
| `dependency.rs` | 421 | Dependency documentation |
| `design.rs` | 346 | Design docs |
| `documentation_structure.rs` | 1,353 | Doc structure (largest) |
| `external_context.rs` | 335 | External context |
| `external_context_ownership.rs` | 352 | External context ownership |
| `implementation.rs` | 552 | Implementation docs |
| `knowledge_system.rs` | 220 | Knowledge system |
| `build.rs` | 342 | Build pipeline |
| `deterministic_runtime.rs` | 303 | Runtime checks |

**Replacement**: Each system provides its own `validate` script that
implements these checks externally. The `DeterministicAuditProvider`
(currently calls these pipelines) becomes a thin dispatcher that runs the
system's `validate` script via `capability::execute_capability()`.

### 3.2 `crates/services/src/reporting.rs` — STRIP domain-specific portions (~4,500 lines)

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

### 4.1 `DeterministicAuditProvider` — redirect to system scripts

Currently (`crates/audit/src/providers.rs`):
```rust
pub fn execute(docs, rules, standard, ...) -> Vec<AuditFinding> {
    // Calls into pipeline modules directly
    let findings = pipeline::run(docs, rules);
    findings
}
```

Refactored:
```rust
pub fn execute(docs, rules, standard, ...) -> Vec<AuditFinding> {
    // Dispatches to system's validate script
    let source = capability::resolve_capability(&Capability::Validate, repo_root, config);
    match source {
        Some(src) => {
            let result = capability::execute_capability(&src, &Capability::Validate, ...);
            // Parse result.output_json into AuditFinding
        }
        None => {
            // Fallback: run YAML-driven pipelines if system has no validate script
            let findings = yaml_runner::run(docs, rules);
            findings
        }
    }
}
```

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

**Phase 1: Decouple pipelines from audit framework**
- Make `DeterministicAuditProvider` fall through to YAML runner when no
  Rust pipeline exists for a domain
- This lets us remove pipeline modules one at a time without breaking
  the framework

**Phase 2: Remove pipeline modules**
- Delete `crates/audit/src/pipelines/` entirely
- The YAML runner (`yaml_runner.rs`) + `pipeline_factory.rs` handle
  YAML-driven checks that systems still need
- Any system that was using Rust-native pipelines now uses the YAML runner
  or its own `validate` script

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
- The `report_generate` tool falls back to generic template rendering
  when no system script exists
- The `project_plan` tool falls back to `StandardWorkflowPlanner` when
  no system plan is stored

After migration:
- Systems that haven't migrated to scripts will need to provide at least
  a `validate` script (can delegate to YAML pipelines internally)
- Systems that haven't migrated `report` will get a minimal generic report

---

## 6. Files Changed

| File | Action | Lines affected |
|------|--------|---------------|
| `crates/audit/src/pipelines/` (22 files) | DELETE | ~8,521 |
| `crates/audit/src/pipelines/mod.rs` | DELETE | 22 |
| `crates/audit/src/lib.rs` | REMOVE `pub mod pipelines;` | 1 |
| `crates/services/src/reporting.rs` | STRIP domain-specific | ~4,500 removed |
| `crates/services/src/project_planner/planners.rs` | STRIP domain-specific | ~200 removed |
| `crates/audit/src/providers.rs` | REFACTOR to dispatch to scripts | ~50 changed |
| `crates/mcp/src/adapter.rs` | REFACTOR report_generate handler | ~30 changed |

---

## 7. Impact Assessment

### 7.1 What breaks

- Any code that directly imports from `pipelines::*` (check pipeline
  module imports in `lib.rs`, `pipeline_factory.rs`, `providers.rs`)
- Tests that assert on specific pipeline check IDs (e.g.
  `assert_eq!(finding.check_id, "V1")`)
- The `report_generate` tool's domain-specific rendering paths

### 7.2 What doesn't break

- YAML-driven audit pipelines (`audit/{standard}/audit/pipelines/*.yaml`)
  — these go through `yaml_runner.rs`, not the Rust pipelines
- The `audit` tool itself — it calls `AuditFramework::run()` which calls
  providers, which will be refactored to dispatch to scripts
- The `compile` tool — generic compilation, no domain knowledge
- The `search` tool — generic search, no domain knowledge
- The `sync` tool — generic sync, no domain knowledge
- All new capability tools (`run_system_*`) — already generic

### 7.3 Lines removed vs kept

| Category | Lines |
|----------|-------|
| Removed (domain-specific) | ~13,221 |
| Kept (generic infrastructure) | ~15,000+ |
| Net result | samgraha shrinks by ~47% while becoming more capable |

---

## 8. Verification

After each phase:
- `cargo check -p mcp` compiles clean
- `cargo test -p audit` passes (minus deleted pipeline tests)
- `cargo test -p mcp` passes
- Manual test: `audit` tool still works against a registered standard
- Manual test: `report_generate` still produces output (generic fallback)

---

## 9. Relationship to Other Proposals

| Proposal | Relationship |
|----------|-------------|
| `generic-script-architecture-proposal.md` | This refactoring implements that proposal's §2.1 ("zero domain-shape knowledge") |
| `knowledge-system-author-guide.md` | Systems built per that guide will replace the removed domain logic |
| `crates-refactor-proposal.md` | Phases 0-1 (system.yaml, inheritance) are kept; Phase 2+ is superseded |
