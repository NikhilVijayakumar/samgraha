# Documentation Staleness, Round 2 — Align `docs/raw/` With What Actually Shipped

**Trigger**: `documentation-cleanup-proposal.md` fixed the *first* wave of
staleness (references to the 22 deleted Rust pipelines, `DOC_DOMAINS`,
`DeterministicAuditProvider` by name, broken `knowledge-hub/` links) and
added 8 new capability-dispatch docs. Since then, in the same session, the
implementation moved again — the pipelines were *actually* deleted (not
just planned), the fallback was removed entirely by explicit decision, the
phase/script dependency graph was normalized out of a JSON blob into three
real tables, and a real bug in `ProjectContext` got fixed. The 8 new docs
(and several older ones) were never updated to match, and on inspection
some of them describe a `system.yaml` shape that **never matched the real
implementation at all** — not staleness from drift, fabrication from the
start.

**Status**: PROPOSED — not implemented. Every finding below is grep-
confirmed against the current files; representative files were read in
full, the rest confirmed via targeted search (noted per category).

---

## 1. Audit Summary

| Category | Files affected | Severity |
|---|---|---|
| A. "Rust pipeline fallback" language — describes deleted functionality as still-present | 7 | High — directly contradicts the shipped "no fallback, ever" decision |
| B. `system_plans` table references — schema was normalized this session | 3 | High — wrong table names, would mislead anyone reading for the real schema |
| C. Fabricated `system.yaml` shape (`capabilities:`/`domains:`/inline `init:` blocks) | 6 | High — never matched reality, not just outdated |
| D. `script_runs` table schema shown with entirely wrong columns | 1 | Medium — one file, but actively wrong reference material |
| E. MCP tool param names wrong (`system_id`/`domain`/`options` vs real `system_name`/`repo_root`/`input_json`) | 3+ | Medium |
| F. Discovery-tier names/count wrong | 4 | Medium |
| G. Capability name serialization (`plan_generation` vs real `plan-generation`) | 2+ | Low |
| H. Unverified `project_plans`/`project_phases` "V29" schema section | 1 | Needs verification, not confirmed wrong |

Overlap is real — several files hit multiple categories (e.g.
`feature-technical/capability-dispatch.md` hits A, B, D, E, F).

---

## 2. Category A: "Rust Pipeline Fallback" Language

The 22 hardcoded Rust pipeline modules are deleted, and — per this
session's explicit decision — there is **no fallback of any kind**. A
missing script now fails clearly (`"No validate script found for
pipeline kind '{}' ... the hardcoded Rust pipeline was removed"`), it
does not silently run built-in logic. Every one of these files says the
opposite:

| File | Stale line | Fix |
|---|---|---|
| `feature/capability-dispatch.md` | FR2: "Built-in fallback — Rust-compiled logic (existing pipeline modules)... If no script is found, the built-in fallback executes." Non-Goals: "does not replace the built-in Rust pipeline modules (they serve as fallback)." Acceptance Criteria: "script failures gracefully fall back to built-in logic." | Remove all three. Discovery has 4 real tiers, none of them a Rust fallback (§ below). A missing/failing script is a clear error, full stop. |
| `feature-technical/capability-dispatch.md` | Discovery diagram's 4th tier: "Built-in: Rust pipeline module fallback" | Remove; see Category F for the real 4 tiers |
| `feature/project-planner.md` | FR2: "Audit → system `validate` scripts (via capability dispatch) or Rust pipeline fallback" | Remove "or Rust pipeline fallback" |
| `feature-technical/project-planner.md` | Architecture diagram: "AuditPhaseExecutor — ... with Rust pipeline fallback"; Integration table: "with Rust pipeline fallback for domains without a working script"; V1 Scope: "Auto-execute audit phase (capability dispatch + fallback)" | Remove all three |
| `product-guide/concepts/scripts.md` | Discovery tier 4: "Built-in fallback — Rust-compiled logic" | Remove; see Category F |
| `product-guide/concepts/systems.md` | Priority list #4: "Built-in Rust logic (fallback, always available)" | Remove — not always available, doesn't exist |
| `product-guide/audit-guide/deterministic.md` | Not read in full this pass — confirmed via grep only | Verify and fix during execution |

---

## 3. Category B: Stale `system_plans` Table

Replaced this session by `workflow_use_cases` + `workflow_phases` +
`workflow_phase_dependencies` (`schema-redesign-proposal.md` §2.1) — real
edges, not an embedded `depends_on` JSON array.

| File | Stale reference | Fix |
|---|---|---|
| `feature-technical/capability-dispatch.md` | "Load phase from system_plans table" in the prerequisite-gating flow diagram | "Query `workflow_phases`/`workflow_phase_dependencies` (joins, not a JSON parse)" |
| `feature-technical/project-planner.md` | "ProjectPlan (persisted in system_plans table)" | Needs disambiguation — see §8 below, this may be conflating two different mechanisms |
| `product-guide/concepts/workflows.md` | "stored in the `system_plans` table" | Same fix as above |

---

## 4. Category C: Fabricated `system.yaml` Shape

The real `system.yaml` (verified against `knowledge-system-author-guide.md`
§3, which matches the actually-shipped format) has exactly five possible
fields: `name`, `description`, `extends`, `drops`, `abstract`. That's it.
Capabilities are discovered by **filesystem probing** (a `scripts/`
directory, `resolve_capability`'s tiers) — never declared inside
`system.yaml`. `init` is a **script** (`scripts/init.py`) that gets
**run** and returns JSON — it is never embedded as static YAML inside
`system.yaml`.

Every file below shows a `system.yaml` with a `capabilities:` block
(mapping capability names to script paths + an `async` flag that doesn't
exist anywhere in the real contract), a `domains:` list (not a real
field), and/or an inline `init:` block with a `step:` key (not a real
`PlanPhase` field). This isn't drift — grepping the git history isn't
necessary to know this was never real, since the actual `system.yaml`
loader/parser (`capability.rs`, `db_reader.rs`, `knowledge-hub-loader.py`)
never had code to read a `capabilities:` or `domains:` key at any point.

| File | What's fabricated |
|---|---|
| `feature/capability-dispatch.md` | Not directly shown as YAML, but implies system.yaml-declared capabilities |
| `feature-technical/mcp-adapter.md` | `capabilities:` block reference — confirmed via grep, needs full read during execution |
| `product-guide/concepts/inheritance.md` | `capabilities:`/`domains:` block reference — confirmed via grep, needs full read during execution |
| `product-guide/concepts/scripts.md` | Discovery tier 1 description: "from `system.yaml`'s `capabilities.*.script` paths" |
| `product-guide/concepts/systems.md` | Full fabricated example (`id`, `capabilities: {validate: {script, async}}`, `domains: [...]`, inline `init: {use_cases: [...]}`) |
| `product-guide/tutorials/first-system.md` | Confirmed via grep, needs full read during execution |

**Fix**: replace every `system.yaml` example with the real shape from
`knowledge-system-author-guide.md` §3. Move capability/domain/init
explanation to prose describing filesystem discovery and the `init`
*script*, not YAML config.

---

## 5. Category D: Wrong `script_runs` Schema

`feature-technical/capability-dispatch.md`'s "script_runs Table (§8.5)"
section shows:
```sql
CREATE TABLE script_runs (
    id INTEGER PRIMARY KEY,
    system_id TEXT NOT NULL,
    capability TEXT NOT NULL,
    domain TEXT,
    input_hash TEXT,
    output_hash TEXT,
    exit_code INTEGER,
    duration_ms INTEGER,
    head_commit TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
```
None of these columns exist. The real table (`09-script_runs.sql`,
verified):
```sql
CREATE TABLE script_runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id         INTEGER NOT NULL REFERENCES standards(id),
    repo_fingerprint    TEXT    NOT NULL,
    capability          TEXT    NOT NULL,
    phase_or_check_key  TEXT    NOT NULL,
    ran_at              TEXT    NOT NULL DEFAULT (datetime('now')),
    expiry_rule_json    TEXT,
    expires_at          TEXT,
    head_commit_at_run  TEXT,
    UNIQUE(standard_id, repo_fingerprint, capability, phase_or_check_key)
);
```
**Fix**: replace verbatim with the real DDL.

---

## 6. Category E: Wrong MCP Tool Param Names

Multiple docs show tool params as `{system_id, domain, target?, options?,
phase_id?}`. The real params (verified against `main.rs`'s
`tool_definitions()`): `system_name`, `repo_root`, `input_json`,
`phase_id`, `timeout_secs`, `repo_path`, and `target` (only on
`run_system_report`/`run_system_script`). There is no `system_id`, no bare
`domain` param, no `options` param — `input_json` is a path to a file the
caller assembles, not an inline object.

Affected (at least): `feature-technical/capability-dispatch.md`,
`feature/capability-dispatch.md`, `product-guide/concepts/scripts.md`.

**Fix**: replace every params table with the real schema from `main.rs`.

---

## 7. Category F: Wrong Discovery-Tier Names

Every doc describes 4 tiers as "System → Standard → Override → Built-in,"
with "System" meaning a `system.yaml`-declared path and "Standard" meaning
something bundled with "the documentation standard." The real tiers
(`capability.rs::resolve_capability`, verified): **Override**
(`samgraha.toml`'s `check_overrides`) → **RepoScript** (`{repo}/scripts/`)
→ **LocalScript** (`.samgraha/scripts/`) → **GlobalScript**
(`mcp_dir()/scripts/`). Different names, different order, no "Standard"
tier at all, and (per Category A) no built-in-fallback tier.

Affected: `feature-technical/capability-dispatch.md`,
`feature/capability-dispatch.md`, `product-guide/concepts/scripts.md`,
`product-guide/concepts/systems.md`.

**Fix**: replace with the real 4-tier list and names.

---

## 8. Needs Verification, Not Yet Confirmed Wrong

`feature-technical/project-planner.md`'s "SQLite Schema — V29" section
describes `project_plans`/`project_phases` tables in `knowledge.db`, with
`ProjectPlan`/`ProjectPhase`/`PlanStatus`/`PhaseStatus` Rust types. This is
a **different mechanism** from `system_plans`/`workflow_phases` (which are
about a system's static *init plan*, not a specific repo's *running
project plan*) — `e2e_planner.rs`'s tests (`orchestrator.create_plan`,
`execute_phase`, `registry.try_start_phase`) confirm these types are real
and live. What's unverified: whether the *exact* column list/table names
shown here still match `crates/registry`'s actual schema, since that
wasn't part of this session's schema-redesign or pipeline-deletion work.
**Action**: read the real registry schema during execution before editing
this section — don't assume it's wrong, and don't assume it's right either.

---

## 9. Execution Order

1. **Category D first** (one file, one clear wrong-vs-right DDL swap) —
   fastest, zero ambiguity.
2. **Category C** (`system.yaml` shape) — highest-leverage fix, since it's
   referenced by/duplicated across 6 files; fix the canonical description
   once (point at `knowledge-system-author-guide.md` §3 as the source of
   truth) and update each file's example to match.
3. **Categories A, F, E, G together** per file — they cluster in the same
   files (`capability-dispatch.md` ×2, `project-planner.md` ×2,
   `systems.md`, `scripts.md`), fix in one pass per file rather than one
   pass per category.
4. **Category B** — straightforward table-name swap in 3 files.
5. **§8's verification** — read the real registry schema, then fix or
   confirm `feature-technical/project-planner.md`'s V29 section.
6. **Verify**: grep for every stale pattern this proposal names (same
   commands as §1's audit) returns zero hits, except in this proposal
   itself and `documentation-cleanup-proposal.md`/
   `schema-redesign-proposal.md` (which correctly discuss the history).

---

## 10. Verification Checklist

- [ ] Zero matches for `Rust pipeline fallback|built-in fallback|Built-in fallback|fallback executes|fallback, always available` in `docs/raw/`
- [ ] Zero matches for `system_plans` in `docs/raw/` (outside historical proposal docs)
- [ ] Zero `system.yaml` examples in `docs/raw/` show a `capabilities:` or `domains:` key
- [ ] `script_runs` schema shown anywhere in `docs/raw/` matches `09-script_runs.sql` exactly
- [ ] Every MCP tool params table in `docs/raw/` matches `main.rs`'s real schema
- [ ] Every discovery-tier list in `docs/raw/` names Override/RepoScript/LocalScript/GlobalScript in that order
- [ ] §8's V29 schema section confirmed accurate or fixed
- [ ] `cargo build --workspace` still clean (docs-only change, should be a no-op check)
