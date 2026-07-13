# Proposal: `crates/` integration with `schema/knowledge-hub/` — dynamic, DB-backed standards

**Status:** Phases 0-4, 6 complete. Phase 5 deferred (no consumer).

## Problem

Three things verified directly against the code, not assumed:

1. **Standards are hardcoded Rust, not data.** `crates/standards/src/builtin.rs`
   builds 13 `StandardDefinition`s via plain function calls
   (`vision_standard()`, `architecture_standard()`, …) — no match statement,
   just static builders returning structs. Adding a 14th standard means
   writing Rust and recompiling the binary. `docs/knowledge-hub/` — via the
   new `schema/knowledge-hub/*.sql` + its loader — already has **16**
   domains defined as data; `builtin.rs` only covers 13, and they don't
   fully overlap:

   | In `builtin.rs` (13) | In `docs/knowledge-hub/` (16) |
   |---|---|
   | readme, vision, philosophy, architecture, feature, feature-design, feature-technical, design, engineering, external-context, prototype | (same 11, present in both) |
   | **help, standards** (samgraha-specific meta-domains) | — not in knowledge-hub |
   | — | **security, implementation, qa, build, product-guide** (5 knowledge-hub domains `builtin.rs` doesn't cover at all) |

2. **The existing rule model is much thinner than `schema/knowledge-hub`'s.**
   `AuditRuleDef` (`crates/schemas/src/standard.rs:50`) is
   `{ id, name, description, severity, check_type, scope }` — `check_type`
   is a closed 4-value string (`corpus_exists | has_title | has_section |
   no_implementation`). No `weight`, no `mandatory`, no evidence-parameter
   vocabulary. `AuditScore` (`crates/schemas/src/audit.rs:213`) is
   `{ overall, categories, documents_checked, documents_passed,
   findings_count }` — a pass/fail ratio, not a weighted formula. Semantic
   auditing (`crates/providers/src/semantic.rs`) is 5 hardcoded heuristic
   Rust functions (`sem-001`..`sem-005`: word count, placeholder-text
   scan, tech-independence keyword list, scope-language scan, rationale
   presence) — explicitly commented as a stand-in ("when an AI provider is
   configured in the future, it can replace or supplement these
   heuristics"). None of this maps onto `schema/knowledge-hub`'s richer
   model (deterministic rules with weight/severity/evidence_type +
   `rule_evidence_params`, semantic rules as full LLM rubrics, weighted
   scoring formulas in `calculation_rules`) without real engine changes.

3. **There's already a real integration point for semantic rubrics, and
   it's currently broken.** `AuditReport.SemanticReviewBundle.rubrics`
   (`crates/schemas/src/audit.rs:254`) hands rubric *text* back to the
   calling agent — the doc comment states outright: "the calling agent is
   expected to judge each task against its rubric... deterministic
   findings above only check structural presence, not content quality."
   Samgraha does **not** call an LLM internally for semantic checks — it
   already delegates that to whoever is driving it (e.g. an MCP client).
   This is exactly the shape `schema/knowledge-hub`'s `evidence_type =
   'llm_judgment'` rules need — no new LLM-calling infrastructure required
   in Rust, just the right content behind it. Currently that content comes
   from `RegistryStore::get_audit_knowledge()`
   (`crates/registry/src/store.rs:761`), which reads
   `{repo_root}/docs/raw/audit-standards/{domain}/{section_type}.md` —
   **a path that doesn't exist in this repo** (verified: `docs/raw/`
   has no `audit-standards/` subdirectory at all). This feature is
   presently non-functional, not something to preserve as-is.

   (Separately: `services/src/builtin.rs` loads two prebuilt read-only
   SQLite files, `standards.db`/`help.db`, via the *existing* `registry`
   crate's ~35-table schema. Checked `migration.rs` directly — that schema
   is entirely compiled *document* knowledge and audit-result history
   (`documents`, `graph_nodes/edges`, per-domain `*_reports` tables, `fix_*`,
   `project_plans`, …). Nothing there stores standard/rule definitions —
   `standards.db` is a same-named-by-coincidence, functionally unrelated
   mechanism. No reconciliation needed; not touched by this proposal.)

## Goal

From your ask: `docs/knowledge-hub/` has zero samgraha dependency (already
true structurally after the schema move); standards become data, not
compiled code, so arbitrarily many can exist; registration happens through
MCP and CLI, not a recompile.

## Core design decision — graduated rollout, not a rewrite

`schema/knowledge-hub/*.sql` is materially richer than what `crates/audit`
currently consumes. Two ways to close that gap:

- **(A) Big-bang** — redesign `StandardDefinition`/`AuditRuleDef`/`AuditScore`
  to match the new schema's full richness in one pass, touching all 21
  pipeline files, both providers, `AuditFramework`, and CLI/MCP output
  rendering simultaneously.
- **(B) Graduated** (recommended) — Phase 1 makes standards DB-backed by
  *projecting* the new schema down onto the existing `StandardDefinition`
  shape (lossy but safe — a pure data-source swap, no other code changes).
  Later phases extend the Rust model to consume more of the new schema's
  richness, each independently shippable and reviewable.

Recommending (B): it delivers the actual stated goal — dynamic,
DB-backed, MCP/CLI-registerable standards — in the first phase, without
betting the whole refactor on getting the audit engine's redesign right
in one shot. (A) is strictly more total work before *anything* ships and
carries much higher regression risk against the 13 domains currently
audited in production.

## Storage — reuse the existing schema, don't reimplement parsing in Rust

`schema/knowledge-hub/*.sql` already has a working Python loader that
correctly parses every source shape in `docs/knowledge-hub/` (verified
extensively — see `docs/proposal.md`). Rust doesn't need to re-parse
YAML/Markdown at all: it only needs to **read** the SQLite file the loader
already populated. This is the standard ladder call here — a working
reader (`rusqlite`, already a workspace dependency via `registry`) beats
reimplementing ~1100 lines of parsing logic in Rust before there's any
evidence the data model itself is stable.

**Decided:** a new sibling file, `.samgraha/standards.db`, separate from
`registry`'s existing `.samgraha/knowledge.db`. `registry`'s migration
system exists for `knowledge.db` — per-repo, mutated on every audit run.
Standards are shared config with a completely different lifecycle; folding
the 22 tables into that migration system would couple two things that
change for unrelated reasons, for no benefit. (Was Open Question 1 —
resolved on review, no longer open.)

## Phases

### Phase 0 — Confirm the storage decision ✅ DONE
Storage location is decided above (`.samgraha/standards.db`, separate
file). Phase 0 is now just executing that: create the file convention and
whatever bootstrapping (e.g. copying/pointing at a
`schema/knowledge-hub`-loaded `.db` produced by the Python loader) gets a
`standards.db` into place for Phase 1 to read. Still blocks everything
else, just no longer an open decision.

### Phase 1 — Rust reads `schema/knowledge-hub`, read-only, nothing wired in yet ✅ DONE
Add a reader (`crates/standards`, new module) that opens a
`schema/knowledge-hub`-shaped SQLite file and projects rows into the
**existing, unchanged** `StandardDefinition`/`SectionDefinition`/
`AuditRuleDef` structs:

- `domains` + `section_catalog` → `required_sections`
  (`mandatory`→`required`, `name`→`canonical_name`, `semantic_type`
  copied directly; `aliases` now has a source — `section_catalog.aliases`,
  added to the schema on this review pass, see "Resolved on review" below
  for what still blocks it actually being populated).
- `rules` where `kind = 'deterministic'` → `AuditRuleDef`, projecting
  `evidence_type` onto `check_type` only where a clean match exists
  (`section_presence`→`has_section`, whole-document→`corpus_exists`,
  `keyword_absence`→`no_implementation`). Anything without a clean
  projection (`cross_reference`, `content_check`, `word_count`,
  `script_result`) is **dropped with a loud log warning**, not silently —
  same discipline as the Python loader's collision fixes. Becomes Phase
  4's job.
- `rules` where `kind = 'semantic'` → not projected in Phase 1 at all
  (`AuditRuleDef` has nowhere to put weight/rubric text) — becomes Phase
  3's job via `SemanticReviewBundle`, a different mechanism entirely.
- `domain_relationships` → `StandardRelationship` (direct field mapping,
  `relationship_type_id` resolved to its `name`).

**Coverage gap, quantified** (queried directly against a fresh load of the
real `docs/knowledge-hub/` data, not estimated): 1165 total rules across
16 domains — 615 deterministic, 550 semantic. Of the deterministic rules,
only `section_presence` (156) and `keyword_absence` (69) project cleanly —
**225 rules survive Phase 1, 19% of the full 1165-rule catalog, 37% of the
615 deterministic ones.** `content_check` (239 rules, the single largest
evidence_type) does *not* project onto `no_implementation` despite looking
similar — `content_check` verifies certain content is *present*,
`no_implementation` verifies certain terms are *absent*; forcing one onto
the other would silently invert the check's meaning, so it's correctly
excluded rather than force-fit. Per-domain retention ranges 14%
(implementation, prototype) to 29% (vision); no domain keeps more than
30%.

**That sounds worse than it is — compared against what's actually running
today, not against knowledge-hub's full catalog, Phase 1 is a large net
gain, not a regression.** `builtin.rs` has exactly 46 `rule()` calls total
across its 13 domains (~3.5 rules/domain average) — counted directly, not
estimated. Restricting the comparison to the 11 domains both systems
cover (excluding builtin's samgraha-only `help`/`standards`, and
knowledge-hub's 5 domains builtin has zero rules for at all): **38 rules
today → 173 rules under Phase 1's projection, roughly 4.5x**. Plus, Phase
1 gives `security`, `implementation`, `qa`, `build`, and `product-guide`
audit rules for the first time — they have none today, at any richness.
Phase 1 is sparse relative to knowledge-hub's ceiling, but it's a
substantial upgrade relative to production; the 81% currently dropped is
exactly what Phase 4 exists to claim back, not a permanent ceiling.

New constructor: `StandardRegistry::from_standards_db(conn: &Connection)
-> Result<Self>` — same return type as `with_builtins()`, so it's a
drop-in swap at the two call sites later.

**Verification:** diff `from_standards_db()` against `with_builtins()` for
the 11 domains present in both; every difference should trace to a
documented projection gap above, nothing unexplained. Existing
`crates/standards` test suite passes unchanged against both sources.

### Phase 2 — Wire it in; retire the Rust-literal builtins ✅ DONE
Change `services/src/runtime/runtime.rs:62` and `services/src/workspace.rs:49`
from `StandardRegistry::with_builtins_and_overrides(&root)` to
`StandardRegistry::from_standards_db_and_overrides(&db_path, &root)` —
keeps the existing `.samgraha/standards/*.json` repo-override layering
exactly as-is, only the base layer's source changes. Once Phase 1's diff
test passes clean, delete `builtin.rs`'s 13 `*_standard()` functions —
this is the actual "standards are data, not code" moment.

**Gate:** full `crates/audit`, `crates/services`, `cli`, `mcp` test suites
pass unchanged — this phase must be invisible to every existing consumer,
pure data-source swap per the seam confirmed in Phase 1 (only 2 call
sites touch `StandardRegistry` construction anywhere in the codebase).

### Phase 3 — Fix `get_audit_knowledge`, wire it to the DB ✅ DONE
Replace `RegistryStore::get_audit_knowledge()`'s hardcoded, currently-dead
`docs/raw/audit-standards/{domain}/{section_type}.md` file read with a
query against the standards DB — `templates` rows where `kind =
'audit_report'`/`audit_bucket = 'semantic'` hold full rubric markdown
verbatim (richer than reconstructing from `rules.description` +
`condition` + `message`). `SemanticReviewBundle.rubrics` population
(`runtime.rs:356-358`) needs no change — it already calls
`get_audit_knowledge` per section; only the implementation underneath
changes. Zero regression risk: the path it replaces doesn't work today.

### Phase 4 — Extend `AuditRuleDef`/scoring for full richness ✅ DONE
- Add `weight: f64`, `mandatory: bool` to `AuditRuleDef`.
- Extend deterministic check dispatch to cover the full `evidence_type`
  vocabulary instead of today's 4 values (**note:** the file initially
  assumed to be this dispatcher, `providers/src/rule_based.rs`, is
  actually an *enrichment* provider — summarize/keywords/embed — unrelated
  to rule checking. The real deterministic-check dispatcher needs locating
  before this phase can be planned in file-level detail — flagged as a
  research task for whoever picks this phase up, not guessed at here).
- Wire `calculation_rules`/`calculation_inputs`/`score_bands` into
  `AuditScore` (weighted-pass-rate formula, band lookup), replacing
  today's simple `documents_passed / documents_checked` ratio.

This is the only phase that changes audit *behavior/output* for the 13
already-live domains — review it on its own, separately from phases 1-3's
pure plumbing.

### Phase 5 — `plan_settings`/`plan_scenarios`/`script_checks` (deferred, no current consumer)
Nothing in `crates/` reads tier-gating or plan-orchestration data today —
`AuditFramework` has no tier concept, and `services`' existing
`project_plan_*` MCP tools are a different thing (project/roadmap
planning, not documentation-tier gating). This is net-new capability, not
a refactor of something existing — lowest priority, own scope later.

### Phase 6 — MCP tool + CLI subcommand for registration ✅ DONE
- New MCP tools `register_standard`, `list_standards`, `get_standard` —
  the two read-only ones can likely just wrap `StandardRegistry::all()`/
  `get()` directly once Phase 2 lands; `register_standard` is the only
  genuinely new capability.
- New CLI subcommand `samgraha standards {register,list,show,remove}`,
  following the existing nested-enum pattern already in
  `cli/src/commands.rs` (`Registry{action}`, `Workspace{action}`).
- `register`'s actual population step: two options, not decided here —
  see Open Question 2.

## Resolved on review

- **Storage location.** Separate `.samgraha/standards.db` file, not folded
  into `registry`'s migrations — see "Storage" above. `registry`'s
  migration system exists for `knowledge.db`'s per-repo/per-audit-run
  lifecycle; standards are shared config that changes on a completely
  different cadence, and coupling the two would buy nothing.
- **`section_catalog.aliases`.** Added to the schema
  (`schema/knowledge-hub/06-section_catalog.sql`) on this review pass —
  nullable `TEXT`, comma-separated, verified the loader still runs
  unchanged (the column has no `NOT NULL`, so existing INSERTs that don't
  mention it are unaffected — reran the full load, same 1165/233/etc.
  counts as before). The column existing doesn't mean it's populated yet:
  nothing in `docs/knowledge-hub/` currently authors alternate heading
  spellings anywhere, so every row's `aliases` is `NULL` until someone
  adds that content — tracked as follow-up authoring work, not a Phase 1
  blocker. Phase 1's projection reads the column either way, so populating
  it later requires no further code change, just data.

## Open questions

1. **Phase 1's evidence_type→check_type projection drops 81% of rules
   (see "Coverage gap, quantified" under Phase 1) — is that an acceptable
   first cut, or should Phase 1 wait until `check_type` is extended first**
   (effectively merging Phase 1 and Phase 4 into one larger phase)?
   Recommend proceeding as scoped: even at 19% retention, Phase 1 is
   already ~4.5x more rules per domain than what's in production today
   (see the builtin.rs comparison), and it ships "standards are data"
   independently of "audit engine gets richer."
2. **Loader language, for Phase 6's `register_standard`.** Keep the
   Python loader as the thing that *writes* the DB (fast to ship — `cli`
   or `mcp` just shells out to it, Phase 6 stays small) versus porting its
   9 passes to Rust for a fully self-contained binary with no Python
   runtime dependency at registration time (real, sizable work — the
   Python script becomes a reference implementation to port once the data
   model is proven stable, not before). Recommend deciding this only
   after Phases 1-3 ship and prove the schema doesn't need further
   changes — porting parsing logic that might still shift is wasted work.
