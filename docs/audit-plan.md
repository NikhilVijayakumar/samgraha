# Audit System — Reference Guide

Everything runs through one MCP tool, `audit()`. You call it two ways — `audit(domain: ...)` or `audit(pipeline: ...)` — and those two modes check completely different things, even when they share a name. This doc explains what exists, what each one actually checks, and gives a copy-pasteable example for every single one so you can run them all and see for yourself.

🔧 = deterministic only (rule-based, instant, no LLM). 🤖 = bundles LLM judgment (`semantic_review`) in the same response.

---

## 0. The mental model — read this before anything else

There are **three layers** every audit target (a domain name or a pipeline name) can have, and **three file trees** that back them:

| # | Layer | Backing file tree | Read live at request time? | What it's for |
|---|---|---|---|---|
| 1 | **Deterministic** | Rust heuristics in `crates/audit/src/providers.rs` (domains) / `crates/audit/src/pipelines/*.rs` (pipelines), driven by `docs/raw/standards/{domain}.md`'s section rules | Yes — always runs, instant, no LLM | Machine-checkable structure: required sections present? empty/duplicate/prohibited content? file-count heuristics? |
| 2 | **Standard** (rubric) | `docs/raw/audit-standards/{domain}/{section}.md` | Yes — opt-in via `providers: ["semantic"]`, **domain audits only** | LLM judges one section's prose quality against a Scoring Criteria table. "Is this Purpose section well-written?" |
| 3 | **Spec** (checklist) | `docs/raw/audit/{pipeline}-audit.md` (A1–A13, V1–V12, BC1–BC10, ...) | Yes — opt-in via `providers: ["semantic"]`, **pipeline audits only** | LLM judges the whole document collection against a checklist item. "Do all the architecture docs cohere as one system (A1)?" |

Layers 2 and 3 are two different things wearing similar names — a domain audit's Standard layer judges **one section of one document**; a pipeline audit's Spec layer judges **the whole collection at once**. Neither layer exists for the other audit type: a domain audit never gets a Spec-layer score, a pipeline audit never gets a Standard-layer score, even for the 11 names that are both a domain and a pipeline.

**Two gotchas that cause most of the confusion:**

- **`audit(pipeline: "doc")` isn't a real pipeline.** `"doc"` is the internal default value used when you omit `pipeline` entirely. Passing it explicitly, omitting it, or calling plain `audit()` are all the same thing: run every domain audit (1A), not a pipeline. There are **20 real pipelines**, not 21 — see the pipeline table below.
- **`dependency` pipeline is real now, not a stub.** It used to always return one placeholder finding (`D0`). It now runs 4 real checks (D1, D4, D5, D7) against your actual `Cargo.toml`/workspace manifests plus 4 honest `Suggestion`-severity deferrals (D2, D3, D6, D8) for what genuinely isn't automatable yet (no per-dependency owner field in the docs; health-check would need live crates.io queries, which conflicts with this project's own Offline-First principle). It's still excluded from `audit_fix_*` — no fix-planner logic exists for any dependency check_id yet, that's a separate gap.

**Why 11 names appear in both tables** (`architecture`, `vision`, `design`, `readme`, `prototype`, `external-context`, `engineering`, `feature`, `feature-technical`, `feature-design`, `help`): the domain audit and the pipeline audit look at the same subject from different altitudes.

- Domain audit (1A) = **per document, per section.** "Does *this* vision.md have a Purpose section, and does that Purpose section read well against the Purpose rubric?"
- Pipeline audit (1B) = **whole collection, cross-document.** "Do all the vision docs together form one coherent, non-contradictory root of the doc hierarchy?"

This is confirmed directly for `architecture` (docs/raw/audit/architecture-audit.md's A1–A13 checks are collection-level and belong to the *pipeline*, not the domain audit) and holds by construction for the other 10 shared names.

---

## 1. Prerequisites

| Step | Call | When |
|---|---|---|
| Bootstrap config | `init()` | Only if `samgraha.toml` is missing |
| Compile docs → knowledge.db | `compile()` | Required before any audit call; re-run after editing docs |

---

## 2. Domain audits (1A) — 15 total, Deterministic + Standard layers

Each domain audit does two things, always in this order:

1. **Deterministic (🔧):** required sections present? empty/duplicate/prohibited content? — same structural engine for every domain, driven by that domain's section rules.
2. **Standard layer (🤖), opt-in via `providers: ["deterministic", "semantic"]`:** an LLM judges each section's prose against the scoring rubric at `docs/raw/audit-standards/{domain}/{section}.md` (Scoring Criteria table, mandatory vs recommended criteria — see `_meta.md`'s conflict rules).

4 of the 15 domains (`help`, `build-guide`, `configuration`, `tutorials`) have no rubric folder yet, so their Standard layer has nothing to score against — stick to `providers: ["deterministic"]` for those.

| Domain | What it checks (deterministic + rubric topics) | Example call |
|---|---|---|
| vision | Purpose, vision statement, problem, solution, pillars, philosophy, guiding principles, target audience, traceability | `audit(domain: "vision", providers: ["deterministic"])` |
| philosophy | Purpose, values, tradeoffs | `audit(domain: "philosophy", providers: ["deterministic"])` |
| architecture | Purpose, data flow, system overview, communication paths, observability, operational readiness, traceability, constraints | `audit(domain: "architecture", providers: ["deterministic"])` |
| feature | Purpose, inputs/outputs, business rules, acceptance criteria, non-goals, future extensions, dependencies, stakeholders, traceability, observability | `audit(domain: "feature", providers: ["deterministic"])` |
| feature-design | Purpose, user experience, workflow, states, non-goals, constraints, traceability | `audit(domain: "feature-design", providers: ["deterministic"])` |
| feature-technical | Purpose, component responsibilities/interactions, integration points, failure handling, runtime/architectural constraints, data ownership/governance, versioning, traceability | `audit(domain: "feature-technical", providers: ["deterministic"])` |
| design | Purpose, design principles, constraints, traceability | `audit(domain: "design", providers: ["deterministic"])` |
| engineering | Purpose, guiding principles, rationale, code standards, security standards, constraints, traceability | `audit(domain: "engineering", providers: ["deterministic"])` |
| external-context | Purpose, integration contract, constraints, traceability | `audit(domain: "external-context", providers: ["deterministic"])` |
| prototype | Scope, mock APIs, data model, constraints, traceability | `audit(domain: "prototype", providers: ["deterministic"])` |
| readme | Getting started, documentation completeness | `audit(domain: "readme", providers: ["deterministic"])` |
| help | Structural only (no rubric yet) | `audit(domain: "help", providers: ["deterministic"])` |
| build-guide | Structural only (no rubric yet) | `audit(domain: "build-guide", providers: ["deterministic"])` |
| configuration | Structural only (no rubric yet) | `audit(domain: "configuration", providers: ["deterministic"])` |
| tutorials | Structural only (no rubric yet) | `audit(domain: "tutorials", providers: ["deterministic"])` |

Other ways to call it:

| Variant | Call |
|---|---|
| All 15 domains in one call (same as `audit()` or `audit(pipeline: "doc")`) | `audit(providers: ["deterministic"])` |
| One domain, with the Standard-layer LLM pass included | `audit(domain: "vision", providers: ["deterministic", "semantic"])` |
| One domain, against a different local repo | `audit(domain: "vision", providers: ["deterministic"], repo_path: "E:\\Python\\other-repo")` |

**Response fields:** `score`, `findings[]` (🔧) plus, when `semantic` is requested, `semantic_review.tasks[]` / `semantic_review.rubrics{}` / `semantic_review.instruction` — one task per document section, its rubric inlined, and the exact next-step text.

### Doing the Standard-layer judgment (real example, captured from a live `audit(domain: "vision")` run)

One task from that response looked like this — `document_id: 224`, `section_id: 8877`, `domain: "vision"`, `semantic_type: "purpose"`. The rubric to judge it against was already inlined at `rubrics["vision/purpose"]` in the same response — no extra fetch needed. Judge the content against that rubric's Scoring Criteria table, then:

| Step | Call |
|---|---|
| (optional) skip if unchanged since last judged | `get_section_changed(section_id: 8877)` |
| store the judgment for that section | `store_section_report(report_json: {"report_id":"vision-224-purpose-1","stage":"section","domain":"vision","document_id":224,"section_id":8877,"score":70,"findings":[{"check_id":"C1","severity":"error","message":"Purpose doesn't distinguish document intent from product mission","location":"docs/raw/vision/vision.md","document_id":224,"provider":"semantic","stage":"section","section_id":8877,"confidence":0.9,"evidence":null,"status":"open","strategy":null}],"created_at":"2026-07-10T00:00:00Z"})` |
| all sections in the domain done → gate | `check_gate(stage: "section")` |
| store the doc-level synthesis | `store_document_report(report_json: {"report_id":"vision-224-doc-1","stage":"document","domain":"vision","document_id":224,"section_id":null,"score":75,"findings":[],"created_at":"2026-07-10T00:00:00Z"})` |
| all docs in the domain done → gate | `check_gate(stage: "document")` |
| store the cross-domain synthesis | `store_cross_domain_report(report_json: {"report_id":"vision-crossdomain-1","stage":"cross_domain","domain":"vision","document_id":null,"section_id":null,"score":80,"findings":[],"created_at":"2026-07-10T00:00:00Z"})` |
| verify section-stage report persisted | `get_audit_report(domain: "vision", stage: "section")` |
| verify document-stage report persisted | `get_audit_report(domain: "vision", stage: "document")` |
| verify cross-domain report persisted | `get_audit_report(domain: "vision", stage: "cross_domain")` |

`docs/raw/audit-standards/_meta.md` governs mandatory-vs-conditional per doc type and conflict order (security > mandatory constraint > specific standard > documented exception).

---

## 3. Pipeline audits (1B) — 20 total, Deterministic + Spec layers

Structural/collection-level checks. Like domain audits, each pipeline now does two things:

1. **Deterministic (🔧):** always runs. Returns `PipelineReport { score, categories, findings[], semantic_review }` — `semantic_review` is the empty default unless you ask for it (see below), so nothing about existing callers changes.
2. **Spec layer (🤖), opt-in via `providers: ["deterministic", "semantic"]`:** an LLM judges the *whole document collection* against the pipeline's checklist in `docs/raw/audit/{pipeline}-audit.md` (A1–A13, V1–V12, BC1–BC10, ... — one task per checklist item). Evidence is every document in the pipeline's matching domain, when one exists (11 of the 20 pipelines share a name with a domain — see §0). The other 9 pipelines (`build`, `security`, `consistency`, `coverage`, `dependency`, `documentation-structure`, `deterministic-runtime`, `external-context-ownership`, `implementation`) have no 1:1 domain, so their Spec-layer tasks come back with empty `evidence` — the checklist still parses and returns tasks, there's just no ready-made document set to hand the judging model yet.

`help` is the one pipeline with **no spec file at all** (`docs/raw/audit/help-audit.md` doesn't exist) — its Spec layer always returns 0 tasks, not an error.

| Pipeline | What it verifies | Example call | Relevant when |
|---|---|---|---|
| architecture | All architecture docs cohere as one implementation-independent system (no overlapping responsibilities, consistent terminology) — A1–A13, distinct from 1A's per-section domain audit of the same name | `audit(pipeline: "architecture")` | You've added/changed an architecture doc and want to know if it still fits the whole picture |
| build | Build docs + config + produced artifacts describe one complete, consistent, reproducible build strategy | `audit(pipeline: "build")` (verify-only) | Before a release, or after changing build config |
| build (deep) | Also check the declared binary artifact exists | `audit(pipeline: "build", inspect_artifact: true)` | You want proof the last build actually produced its output |
| build (execute) | Actually runs the declared build command, then audits the result | `audit(pipeline: "build", execute: true)` | CI-style "does it still build" check |
| build (dry run) | Prints the resolved build command without running it | `audit(pipeline: "build", dry_run: true)` | Sanity-check what command the pipeline would run |
| security | Security docs + config describe a complete, implementation-independent security strategy (static check) | `audit(pipeline: "security")` | After touching auth/config/trust-boundary docs |
| security (runtime) | Also connects to the running app to check auth/TLS/rate-limit behavior | `audit(pipeline: "security", runtime: true)` | You have the app running locally and want live verification, not just doc review |
| consistency | Adjacent layers (Vision→Architecture→Feature→Feature-Technical→Engineering→Implementation, plus Build→Implementation and Security→Implementation) don't contradict each other; owns cross-doc terminology consistency | `audit(pipeline: "consistency")` | After any cross-layer change, to catch drift between docs |
| coverage | Bidirectional: every documented capability is implemented, every implemented capability is documented. Owns **all** orphan detection | `audit(pipeline: "coverage")` | You want to know what's undocumented code or unimplemented spec |
| dependency | Real checks now: D1 (every dependency mentioned in Engineering docs), D4 (no unconstrained `"*"` versions), D5 (git-sourced deps declared), D7 (no prohibited async runtimes; `notify` actually feature-gated as claimed). D2/D3/D6/D8 stay honest `Suggestion` deferrals — see §0 | `audit(pipeline: "dependency")` | After adding/changing a dependency, or changing `docs/raw/engineering/dependency-standards.md` |
| documentation-structure | 45 checks (SI/MC/AE/CA/NP/IT/GC) treating the whole doc corpus as one system: structural integrity, 1:1 domain mapping, feature atomicity, name preservation across compile layers | `audit(pipeline: "documentation-structure")` | Big doc reorganizations, or a periodic corpus health check |
| vision | Vision docs collectively form one coherent, stable root of the doc hierarchy | `audit(pipeline: "vision")` | Same trigger as domain vision, but for the whole collection |
| design | Design docs form one complete, reusable, technology-independent design system | `audit(pipeline: "design")` | After adding/changing shared design principles |
| readme | The README is a concise, accurate, maintainable entry point that doesn't duplicate detailed docs | `audit(pipeline: "readme")` | Before publishing, after restructuring docs |
| prototype | The prototype runtime + docs actually validate the documented product before production build begins | `audit(pipeline: "prototype")` | You're validating a prototype phase, not production code |
| external-context | Every external dependency/integration is documented once, atomically, without leaking internal architecture | `audit(pipeline: "external-context")` | After adding a new external dependency |
| engineering | Engineering docs form a complete, rationale-driven, implementation-independent foundation feeding build/security/deterministic-runtime/prototype audits | `audit(pipeline: "engineering")` | After changing repo structure, principles, or tech choices |
| feature | Feature docs form a complete, atomic, business-focused product spec traceable to Vision | `audit(pipeline: "feature")` | After adding/changing a feature spec |
| feature-technical | Feature Technical docs fully translate Feature Specs into architecture-consistent technical design | `audit(pipeline: "feature-technical")` | After writing/updating a feature's technical design |
| feature-design | Feature Design docs translate Feature Specs into UX consistent with the shared Design system | `audit(pipeline: "feature-design")` | After writing/updating a feature's UX design |
| deterministic-runtime | Architecture + engineering docs define a deterministic, stateless, reproducible execution model | `audit(pipeline: "deterministic-runtime")` | Checking the system is designed to behave predictably/reproducibly |
| external-context-ownership | External Context docs form one complete, consistent dependency knowledge base (no duplicate/conflicting ownership) | `audit(pipeline: "external-context-ownership")` | Auditing the dependency documentation as a whole, not per-doc |
| implementation | Source code faithfully implements the documented system (architecture, feature-technical, engineering, external-context realized correctly) — orphan detection is coverage's job, not this one | `audit(pipeline: "implementation")` | Checking code actually matches what was designed |
| help | The CLI/product-guide help docs have complete, one-file-per-command/config/topic coverage. No Spec layer (no spec file exists) | `audit(pipeline: "help")` | After adding a command or config option, to check its help doc exists |

Not a real pipeline, don't use it as one: `audit(pipeline: "doc")` — silently redirects to "run all 15 domain audits" (§2), because `"doc"` is the internal default sentinel for "no pipeline given."

### Doing the Spec-layer judgment (real example, captured from a live `audit(pipeline: "architecture", providers: ["semantic"])` run)

`semantic_review.tasks` had 13 entries (A1–A13, parsed from `docs/raw/audit/architecture-audit.md`). One looked like this: `check_id: "A1"`, `title: "Modular Architecture"`, `audit_rule: "Architecture is modular."`. `semantic_review.evidence` was a map of every real document path under `docs/raw/architecture/` to its raw content — judge each task against that evidence, then:

| Step | Call |
|---|---|
| store the judgment for one check | `store_pipeline_check_report(report_json: {"report_id":"architecture-a1-1","pipeline":"architecture","check_id":"A1","score":90,"findings":[],"git_revision":null,"created_at":"2026-07-11T00:00:00Z"})` |
| all checks judged → gate | `check_pipeline_gate(pipeline: "architecture")` |
| verify a specific check's stored judgment | `get_pipeline_check_report(pipeline: "architecture", check_id: "A1")` |
| roll everything up into one score | `get_summary_report(target_type: "pipeline", target_name: "architecture")` |

`check_pipeline_gate` and `get_audit_report`'s section/document/cross_domain gates share the same shape but not the same table — a pipeline's Spec-layer judgments live in `pipeline_semantic_reports`, keyed by `pipeline` + `check_id`, not `AuditStage`. Like the Standard layer's tables, storing is append-only: re-judging a check adds a new row rather than replacing the old one, so `check_pipeline_gate`/`get_summary_report`'s spec-score both look at the **latest row per check_id**, not a flat average — a stale low score from an earlier judgment can't skew a later, better one.

---

## 4. Summary reports — roll up whichever layers ran

`get_summary_report(target_type: "domain" | "pipeline", target_name: "...")` computes one score + readiness verdict from whichever of the layers have data:

| target_type | deterministic_score | standard_score | spec_score |
|---|---|---|---|
| `"domain"` | always present (live re-audit) | present if a `cross_domain`-stage report has been stored | **always `null`** — Spec belongs to pipelines only |
| `"pipeline"` | always present (live re-run) | **always `null`** — Standard belongs to domains only | present if any check has been judged via `store_pipeline_check_report` |

Example: `get_summary_report(target_type: "pipeline", target_name: "architecture")` → `{"target_type":"pipeline","target_name":"architecture","deterministic_score":92.3,"standard_score":null,"spec_score":90.0,"overall_score":91.15,"readiness":"Production","created_at":"..."}`.

It always recomputes the deterministic (and, for pipelines, spec) layer live rather than reading a cache — there's no single accessor across the 20 pipelines' differently-shaped stored-report tables yet (see Known Gaps), and re-running deterministic checks is cheap, so recomputing is simpler and never stale.

---

## 5. Verify without re-running

| Source | Call |
|---|---|
| 1A deterministic findings, vision example | `audit(domain: "vision", providers: ["deterministic"])` — re-call, read `findings[]` |
| 1A Standard-layer findings, section stage | `get_audit_report(domain: "vision", stage: "section")` |
| 1A Standard-layer findings, document stage | `get_audit_report(domain: "vision", stage: "document")` |
| 1A Standard-layer findings, cross-domain stage | `get_audit_report(domain: "vision", stage: "cross_domain")` |
| 1B deterministic findings, architecture example | `audit(pipeline: "architecture")` — re-call, read `findings[]` (no persisted-report fetch tool yet, see Known gaps) |
| 1B Spec-layer findings, one check | `get_pipeline_check_report(pipeline: "architecture", check_id: "A1")` |
| Either layer, rolled into one score | `get_summary_report(target_type: "pipeline", target_name: "architecture")` (or `target_type: "domain"`) |

Readiness bands (`AuditReport`, `PipelineReport`, and `SummaryReport` all use these):

| Score | Readiness |
|---|---|
| ≥90%, no errors | Production |
| ≥80% | Implementation |
| ≥70% | Engineering |
| ≥60% | Design |
| ≥50% | Architecture |
| <50% | Product |

`error` counts against score; `warning`/`suggestion` don't. Orphan findings (code without docs) are always `warning`. Every deterministic `check_id` traces to a spec file in `docs/raw/audit/*.md` (index: `docs/raw/audit/README.md`) — and now that file is also the live source for the Spec-layer `semantic_review.tasks` on the matching pipeline.

---

## 6. Fix — once a report exists

Concrete example: this real finding came back from `audit(domain: "vision")` — `{"check_id":"vision-002","severity":"Warning","message":"Vision must define target audience: 'docs\\raw\\product-guide\\documentation-guide\\vision.md'","location":"docs\\raw\\product-guide\\documentation-guide\\vision.md","document_id":163,"provider":"deterministic","stage":null,"section_id":null,"confidence":null,"evidence":null,"status":null,"strategy":null}`.

| Action | Call | Modifies files? |
|---|---|---|
| Preview a fix plan for that finding | `audit_fix_plan(finding: {"check_id":"vision-002","severity":"Warning","message":"Vision must define target audience","location":"docs/raw/product-guide/documentation-guide/vision.md","document_id":163,"provider":"deterministic","stage":null,"section_id":null,"confidence":null,"evidence":null,"status":null,"strategy":null}, domain: "vision", report_id: 0, report_type: "vision", target_path: "docs/raw/product-guide/documentation-guide/vision.md")` | No |
| Plan + execute + verify + retry that finding | `audit_fix_apply(finding: {"check_id":"vision-002","severity":"Warning","message":"Vision must define target audience","location":"docs/raw/product-guide/documentation-guide/vision.md","document_id":163,"provider":"deterministic","stage":null,"section_id":null,"confidence":null,"evidence":null,"status":null,"strategy":null}, domain: "vision", report_id: 0, report_type: "vision", target_path: "docs/raw/product-guide/documentation-guide/vision.md")` | Yes |
| Check the fix session `audit_fix_apply` returned (example id 1 — use the real `session.id` from your response) | `audit_fix_status(session_id: 1)` | No |
| List all fix sessions | `audit_fix_list(limit: 20, offset: 0)` | No |
| List plans generated in session 1 | `audit_fix_plan_list(session_id: 1)` | No |
| Get plan 1 and its steps | `audit_fix_plan_get(plan_id: 1)` | No |
| Render plan 1 as markdown | `audit_fix_plan_render(plan_id: 1, template: "documentation")` | No |
| List available fix-plan templates | `audit_fix_templates()` | No |

`report_id`/`report_type` are **bookkeeping tags on the fix session, not a foreign key** — never validated against a stored report, so any values that identify the source are fine (`report_type: "vision"`, `report_id: 0` for a plain domain audit is a valid literal choice, not a placeholder).

Reminder: `dependency` is still excluded from `audit_fix_*` entirely — even though its deterministic layer produces real findings now (D1/D4/D5/D7), no fix-planner logic exists yet for any dependency check_id. That's a separate, still-open gap, not the "it's a stub" reason from before.

**Marking a finding's status — three tools, not interchangeable:**

| Tool | Call | Works on |
|---|---|---|
| Mark a Standard-layer (domain) finding fixed | `update_finding_status(report_id: 1, criterion_id: "C1", status: "fixed")` | Findings from `store_section_report`/`store_document_report`/`store_cross_domain_report` — `report_id` must be the numeric id of that stored `SemanticReport` |
| Same, shorthand | `audit_fix_accept(report_id: 1, criterion_id: "C1")` | Same as above, forces `status: "fixed"` |
| Same, shorthand for accepted-not-fixed | `audit_fix_reject(report_id: 1, criterion_id: "C1")` | Same as above, forces `status: "accepted"` |
| Mark a pipeline deterministic-report finding fixed | `update_report_finding_status(finding_id: 1, status: "fixed")` | Pipeline-report findings (architecture/documentation-structure/build/security/consistency/coverage/help/...) — `finding_id` must be a row id in `report_findings`, **no MCP tool returns this yet** (see Known gaps) |

`status` values: `open`, `fixed`, `accepted`, `ignored`, `false_positive`. Plain 1A deterministic findings and 1B pipeline deterministic findings otherwise have no round-tripping status tool — track fix completion via the `FixSession`'s own status (`audit_fix_status`) instead. Spec-layer (pipeline) findings have no status tool at all yet — only the score/findings themselves round-trip via `store_pipeline_check_report`/`get_pipeline_check_report`.

---

## 7. Quick reference — run everything to test it all

| Step | Call |
|---|---|
| 1 | `init()` then `compile()` |
| 2 | Run every row in §2's domain table (15 calls) |
| 3 | For each `semantic_review.tasks[]` in those responses: judge against `rubrics`, `store_section_report(...)` per §2's worked example |
| 4 | `check_gate(stage: "section")` → `store_document_report(...)` → `check_gate(stage: "document")` → `store_cross_domain_report(...)` |
| 5 | Run every row in §3's pipeline table (20 calls) — `dependency` now returns real findings, no need to skip it |
| 6 | For any pipeline, also try `audit(pipeline: "architecture", providers: ["deterministic", "semantic"])`: judge each `semantic_review.tasks[]` entry against `evidence`, `store_pipeline_check_report(...)` per §3's worked example |
| 7 | `check_pipeline_gate(pipeline: "architecture")`, then `get_summary_report(target_type: "pipeline", target_name: "architecture")` |
| 8 | For findings worth fixing: `audit_fix_plan(...)` → review → `audit_fix_apply(...)` → `audit_fix_status(session_id: ...)` |

CLI equivalents for local/non-agent use: `samgraha audit`, `samgraha audit --pipeline documentation-structure --report`, `samgraha audit --pipeline dependency` (see real output below), `samgraha compile`, `samgraha report --pipeline architecture` (Tera-rendered version with rubric summaries baked into the template).

Real `dependency` pipeline output from this repo, captured live — a believable, non-noisy score, not 0% or 100%:

```
Pipeline: dependency
Score: 74.2%
Categories:
  Dependency Health: 50.0%
  Version Policy: 100.0%
  Cross-References: 100.0%
  Dependency Justification: 66.7%

  [WARN ] D1  — Dependencies with no rationale found in docs/raw/engineering/: chrono, regex, tera, toml_edit, tracing-subscriber, uuid
  [SUGG ] D2  — docs/raw/engineering/dependency-standards.md documents dependencies by category, not with per-dependency owner/purpose/version-policy fields — structured per-dependency metadata is needed before this can be automated
  [SUGG ] D3  — No per-dependency owner is tracked in Engineering docs today — not yet automated
  [SUGG ] D6  — Deprecated/unmaintained/yanked status requires live crates.io queries — deferred rather than violating this project's Offline-First engineering principle for the default deterministic run; would need an explicit runtime-mode opt-in
  [WARN ] D7  — cli: notify is an unconditional [dependencies] entry (optional=false, has [features] gate=false), but dependency-standards.md claims 'the watcher is a compile-time optional feature'
  [SUGG ] D8  — Orphan dependency detection (documented dependencies missing from the manifest, or vice versa) is Coverage Audit's responsibility (CV12), not Dependency Governance's
```

---

## Known gaps (code, not docs)

| Gap | Detail |
|---|---|
| No query tool returns `report_findings.id` | `update_report_finding_status` exists but nothing hands back the row id it needs for pipeline-report findings — each pipeline has its own stored-report getter (`get_architecture_report_with_findings`, etc.) but none are exposed over MCP. Track via `FixSession` status instead, or query the registry directly. |
| `help` pipeline has no Spec-layer source | `docs/raw/audit/help-audit.md` doesn't exist — `audit(pipeline: "help", providers: ["semantic"])` correctly returns 0 tasks rather than erroring, but there's no checklist to judge until someone writes that file. |
| `get_summary_report` recomputes rather than reads a cache | It doesn't read the 20 pipelines' own dedicated report tables (`architecture_reports`, `vision_reports`, ...) — no generic accessor exists across their differently-shaped schemas. It re-runs the deterministic (and pipeline Spec) layer live instead, which is cheap and never stale, but means it's not free to call in a tight loop. |
| Gate/spec-score staleness | `check_gate` (domain) and `check_pipeline_gate` (pipeline) both count every stored row with `score < 100`, not just the latest judgment per section/check — a re-judged item that improved still leaves its earlier low score counted, so the gate can stay blocked even after a fix. `get_pipeline_spec_score` (used by `get_summary_report`) *does* correctly average only the latest judgment per check_id — only the gates have this quirk, not the score rollup. |
| No fix-planner logic for `dependency` findings | `audit_fix_*` still refuses the `dependency` domain outright — its deterministic layer produces real findings now (D1/D4/D5/D7), but nothing in `crates/audit/src/fix/` knows how to plan a fix for any of its check_ids yet. |
