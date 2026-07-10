# Audit Runbook — Trigger, Test, Fix via MCP `samgraha`

Every call below is concrete and complete — copy the cell, paste, run, no placeholders to fill in. Where a value is repo-specific (a real domain, real ids), it's a real value captured from a live run against this repo, not a stand-in. 🔧 = deterministic-only. 🤖 = bundles LLM review work (`semantic_review`) in the same response.

---

## 0. Prerequisites

| Step | Call |
|---|---|
| Bootstrap config (only if `samgraha.toml` missing) | `init()` |
| Compile docs → knowledge.db (required before any audit; re-run after editing docs) | `compile()` |

---

## 1A. Documentation Audit — per domain, 🔧 + 🤖 bundled

| Domain | Call |
|---|---|
| architecture | `audit(domain: "architecture", providers: ["deterministic"])` |
| vision | `audit(domain: "vision", providers: ["deterministic"])` |
| philosophy | `audit(domain: "philosophy", providers: ["deterministic"])` |
| readme | `audit(domain: "readme", providers: ["deterministic"])` |
| feature | `audit(domain: "feature", providers: ["deterministic"])` |
| feature-design | `audit(domain: "feature-design", providers: ["deterministic"])` |
| feature-technical | `audit(domain: "feature-technical", providers: ["deterministic"])` |
| design | `audit(domain: "design", providers: ["deterministic"])` |
| engineering | `audit(domain: "engineering", providers: ["deterministic"])` |
| external-context | `audit(domain: "external-context", providers: ["deterministic"])` |
| prototype | `audit(domain: "prototype", providers: ["deterministic"])` |
| help | `audit(domain: "help", providers: ["deterministic"])` |
| build-guide | `audit(domain: "build-guide", providers: ["deterministic"])` |
| configuration | `audit(domain: "configuration", providers: ["deterministic"])` |
| tutorials | `audit(domain: "tutorials", providers: ["deterministic"])` |
| all 15 domains in one call | `audit(providers: ["deterministic"])` |
| vision, with the extra heuristic pass too (still 🔧, not LLM) | `audit(domain: "vision", providers: ["deterministic", "semantic"])` |
| vision, targeting a different local repo | `audit(domain: "vision", providers: ["deterministic"], repo_path: "E:\\Python\\other-repo")` |

**Response fields:** `score`, `findings[]` (🔧) plus `semantic_review.tasks[]` / `semantic_review.rubrics{}` / `semantic_review.instruction` (🤖 — one task per document section, its rubric inlined, and the exact next-step text).

### Doing the LLM judgment (real example, captured from a live `audit(domain: "vision")` run)

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

## 1B. Custom Pipelines — 🔧 only, no semantic_review

Returns `PipelineReport { score, categories, findings[] }` directly — no `semantic_review` field ever appears here, these are collection-level/structural checks, not per-section content judgment.

| Pipeline | Call |
|---|---|
| doc | `audit(pipeline: "doc")` |
| architecture — structural A1–A13, distinct from 1A's per-section domain audit of the same name | `audit(pipeline: "architecture")` |
| build — verify-only | `audit(pipeline: "build")` |
| build — also check the declared binary artifact exists | `audit(pipeline: "build", inspect_artifact: true)` |
| build — actually run the declared build command | `audit(pipeline: "build", execute: true)` |
| build — print the resolved build command without running it | `audit(pipeline: "build", dry_run: true)` |
| security — static docs/config check | `audit(pipeline: "security")` |
| security — also connect to the running app (auth/TLS/rate-limit) | `audit(pipeline: "security", runtime: true)` |
| consistency | `audit(pipeline: "consistency")` |
| coverage | `audit(pipeline: "coverage")` |
| dependency | `audit(pipeline: "dependency")` |
| documentation-structure — 39 checks (SI/MC/AE/CA/NP/IT/GC), corpus-as-one-system | `audit(pipeline: "documentation-structure")` |
| vision | `audit(pipeline: "vision")` |
| design | `audit(pipeline: "design")` |
| readme | `audit(pipeline: "readme")` |
| prototype | `audit(pipeline: "prototype")` |
| external-context | `audit(pipeline: "external-context")` |
| engineering | `audit(pipeline: "engineering")` |
| feature | `audit(pipeline: "feature")` |
| feature-technical | `audit(pipeline: "feature-technical")` |
| feature-design | `audit(pipeline: "feature-design")` |
| deterministic-runtime | `audit(pipeline: "deterministic-runtime")` |
| external-context-ownership | `audit(pipeline: "external-context-ownership")` |
| implementation | `audit(pipeline: "implementation")` |
| help | `audit(pipeline: "help")` |

---

## 2. Verify without re-running

| Source | Call |
|---|---|
| 1A deterministic findings, vision example | `audit(domain: "vision", providers: ["deterministic"])` — re-call, read `findings[]` |
| 1A semantic findings, section stage | `get_audit_report(domain: "vision", stage: "section")` |
| 1A semantic findings, document stage | `get_audit_report(domain: "vision", stage: "document")` |
| 1A semantic findings, cross-domain stage | `get_audit_report(domain: "vision", stage: "cross_domain")` |
| 1B pipeline findings, architecture example | `audit(pipeline: "architecture")` — re-call, read `findings[]` (no persisted-report fetch tool yet, see Known Gaps) |

Readiness bands (both `AuditReport` and `PipelineReport`):

| Score | Readiness |
|---|---|
| ≥90%, no errors | Production |
| ≥80% | Implementation |
| ≥70% | Engineering |
| ≥60% | Design |
| ≥50% | Architecture |
| <50% | Product |

`error` counts against score; `warning`/`suggestion` don't. Orphan findings (code without docs) are always `warning`. Every `check_id` traces to a spec file in `docs/raw/audit/*.md` (index: `docs/raw/audit/README.md`).

---

## 3. Fix — once a report exists

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

**Marking a finding's status — two tools, not interchangeable:**

| Tool | Call | Works on |
|---|---|---|
| Mark a semantic-stage finding fixed | `update_finding_status(report_id: 1, criterion_id: "C1", status: "fixed")` | Findings from `store_section_report`/`store_document_report`/`store_cross_domain_report` — `report_id` must be the numeric id of that stored `SemanticReport` |
| Same, shorthand | `audit_fix_accept(report_id: 1, criterion_id: "C1")` | Same as above, forces `status: "fixed"` |
| Same, shorthand for accepted-not-fixed | `audit_fix_reject(report_id: 1, criterion_id: "C1")` | Same as above, forces `status: "accepted"` |
| Mark a pipeline-report finding fixed | `update_report_finding_status(finding_id: 1, status: "fixed")` | Pipeline-report findings (architecture/documentation-structure/build/security/consistency/coverage/help) — `finding_id` must be a row id in `report_findings`, **no MCP tool returns this yet** (see Known Gaps) |

`status` values: `open`, `fixed`, `accepted`, `ignored`, `false_positive`. Plain 1A deterministic findings and 1B pipeline findings otherwise have no round-tripping status tool — track fix completion via the `FixSession`'s own status (`audit_fix_status`) instead.

---

## 4. Quick reference — run everything

| Step | Call |
|---|---|
| 1 | `init()` then `compile()` |
| 2 | Run every row in §1A's domain table |
| 3 | For each `semantic_review.tasks[]` in those responses: judge against `rubrics`, `store_section_report(...)` per §1A's worked example |
| 4 | `check_gate(stage: "section")` → `store_document_report(...)` → `check_gate(stage: "document")` → `store_cross_domain_report(...)` |
| 5 | Run every row in §1B's pipeline table |
| 6 | For findings worth fixing: `audit_fix_plan(...)` → review → `audit_fix_apply(...)` → `audit_fix_status(session_id: ...)` |

CLI equivalents for local/non-agent use: `samgraha audit`, `samgraha audit --pipeline documentation-structure --report`, `samgraha compile`, `samgraha report --pipeline architecture` (Tera-rendered version with rubric summaries baked into the template).

---

## Known gaps (code, not docs)

| Gap | Detail |
|---|---|
| No query tool returns `report_findings.id` | `update_report_finding_status` exists but nothing hands back the row id it needs for pipeline-report findings — each pipeline has its own stored-report getter (`get_architecture_report_with_findings`, etc.) but none are exposed over MCP. Track via `FixSession` status instead, or query the registry directly. |
| `docs/raw/audit/*.md` never parsed at runtime | The Rust code in `crates/audit/src/pipelines/*.rs` is the real implementation, written to match what the `.md` describes. Only the fix planner (`crates/audit/src/fix/planning_context.rs`) reads these files live, to pull a check's requirement text when building a fix plan. |
