# Proposal — Knowledge Hub: Documentation Standards & Audit Model

## 0. Scope

This proposal is content design only. It describes what should live under `docs/knowledge-hub/` and how the pieces relate. It does not propose Rust code changes, does not reference existing implementation internals (crate names, function names, DB tables, MCP tools), and does not assume the current engine's behavior as a constraint — the engine is expected to be refactored later to match whatever structure is agreed here, not the other way around.

## 1. Why this move

Documentation standards, audit knowledge, report templates, and fix-plan templates are not product features — they are the rules the product's own documentation-audit engine is built against. Keeping them under `docs/raw/` (the tree that also holds this repo's *own* authored Vision/Architecture/Feature docs) blurred that distinction. `docs/knowledge-hub/` separates "the contract documentation must satisfy" from "documentation written to satisfy it."

## 2. Target structure (current, on disk)

```
docs/knowledge-hub/
├── 00-domain-relationships.md          # relationship graph between standards (Tier model)
├── audit/
│   ├── deterministic/
│   │   ├── schema/                     # raw .sql DDL — generic, standard-agnostic storage (§4a)
│   │   ├── document/                   # whole-document rules, per domain (YAML) + {domain}-relationships.yaml
│   │   └── section/                    # per-section rules, per domain/section (YAML)
│   └── semantic/
│       ├── document/                   # whole-document LLM judgment prompt, per domain
│       └── section/                    # LLM judgment prompt, per domain/section
├── documentation-standards/            # one file per document type — the authoring contract
│   ├── 01-vision-standards.md
│   ├── 02-philosophy-standards.md
│   ├── ... 03-16
├── plan/                               # placeholder — scope not yet defined, deferred
└── templates/
    └── audit/
        ├── archive/                    # superseded templates, kept for reference only
        ├── deterministic/
        │   ├── document/
        │   └── section/
        ├── semantic/
        │   ├── document/
        │   └── section/
        └── generation/
            ├── document/
            └── section/
```

This proposal covers `audit/` and `documentation-standards/` only. `templates/` and `plan/` are real, on-disk, and not yet designed — deferred to a follow-up pass (§9 tracks this). One naming note to fix whenever templates are addressed: `audit/deterministic/` is spelled correctly but `templates/audit/deterministic/` is currently misspelled `determinstic` on disk.

## 3. `documentation-standards/` — the authoring contract

Already backfilled (see recent commits) so that every `Required Sections` table row has a full per-section contract: `semantic_type`, `scope`, `out_of_scope`, `contributes`, `relationships`, `responsibilities`, `generation_rules`, `enhancement_rules`, `validation_rules`, `audit_rules`, a `Template`, `Examples` (correct/incorrect), and `Writing Guidance` (tone/voice/structure/audience/do/don't).

This layer answers **"what should a human or AI author write, and why."** It is prose-first — meant to be read, not parsed. It should stay authoritative for tone, examples, and rationale, but should not be the runtime source of truth for structural validation once `audit/deterministic/` exists (see §7).

## 4. `audit/` — deterministic vs semantic, whole vs section

Four independent knowledge buckets, one per combination of two axes:

|                    | **Whole document**                                                              | **Section**                                                                 |
|--------------------|----------------------------------------------------------------------------------|------------------------------------------------------------------------------|
| **Deterministic**  | Schema for the document as a collection: which sections must exist, required cross-references between documents, tier/relationship edges. Structural, no judgment — code-checkable. | Schema per section: required subsections, semantic_type, minimum content shape, allowed relationship targets. Structural, code-checkable. |
| **Semantic**       | LLM judgment prompt for the document as a whole: cross-section consistency, terminology drift, contradictions between sections, collection-level coherence. | LLM judgment prompt for the section — **not a scoring rubric alone**. Judges this section's own content *and* this section's cross-section relationships *and* any other relationship validation the section participates in (e.g. against another document). |

The deterministic/semantic boundary is: **existence and shape of a relationship or section is deterministic; correctness and consistency of what's actually said is semantic.**

The whole/section boundary is **not** "section = local only, whole = everything cross-cutting." Both levels can judge relationships — the split is about which relationships each owns:

- **Section-level semantic** judges relationships *this section declares or participates in* — its own cross-references, its consistency with the specific sections it's supposed to relate to, any relationship validation anchored at this section.
- **Whole-level semantic** judges what emerges only from reading the full document together — contradictions, gaps, or drift that isn't anchored to any one section's declared relationships, plus overall collection coherence.

This split must be explicit per domain so the same relationship isn't checked — and scored — twice in both places.

## 4a. `audit/deterministic/schema/` — generic storage, not this-standard-only storage

The goal of a schema layer is bigger than "define tables for documentation-standards." It's to remove the engine's dependency on any *specific* standard entirely, so a wholly different rule set — a hackathon judging standard, a research-paper submission standard, anything with its own sections/rules/scoring — can be added later as pure data, no code or migration change.

This only works if nothing in the schema names a domain, a section type, a standard, or a use-case. Everything specific becomes a **row**, never a **table or column** — and it needs one more level above `standards` than originally drafted: a **system**.

A "system" is the outer use-case this whole apparatus is being pointed at — "Samagrha documentation audit" is one system; a hackathon judging tool, a research-paper submission reviewer, or an academic-writing checker would each be a different system. A system owns however many standards it needs (Samagrha's system owns one — `documentation-standards` — but a hackathon system might own several: `frontend`, `backend`, `qa`, `creative-work`, one per judged category). Without this layer, "different standard" and "different use-case entirely" collapse into the same concept, which they aren't:

```sql
-- systems: the outer use-case/tenant. "samagrha-documentation" is one row;
-- "hackathon-eval", "research-paper-publishing", "academic-writing" are siblings, not new schema.
CREATE TABLE systems (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,   -- e.g. "samagrha-documentation", "hackathon-eval"
    description TEXT
);

-- standards: the registry of pluggable rule sets, scoped to a system. "documentation-standards-v1"
-- belongs to the samagrha-documentation system; a hackathon system might own several standards
-- side by side ("frontend-v1", "backend-v1", "qa-v1", "creative-work-v1").
CREATE TABLE standards (
    id          INTEGER PRIMARY KEY,
    system_id   INTEGER NOT NULL REFERENCES systems(id),
    name        TEXT NOT NULL,   -- e.g. "documentation-standards"
    version     TEXT NOT NULL,
    description TEXT
);

-- documents: one row per audited document, tagged to the standard and domain it belongs to.
CREATE TABLE documents (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    domain       TEXT NOT NULL,   -- "architecture", "vision", ... or a hackathon/paper's own domain names
    title        TEXT,
    path         TEXT
);

-- sections: one row per section of a document.
CREATE TABLE sections (
    id           INTEGER PRIMARY KEY,
    document_id  INTEGER NOT NULL REFERENCES documents(id),
    semantic_type TEXT NOT NULL,
    name         TEXT NOT NULL
);

-- rules: one row per check. rule_ref points at the YAML/prompt file (in document/ or section/)
-- that defines what the check actually evaluates — the schema doesn't need to know that content.
CREATE TABLE rules (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    domain       TEXT NOT NULL,
    section_type TEXT,            -- NULL for whole-document rules
    kind         TEXT NOT NULL CHECK (kind IN ('deterministic','semantic')),
    scope        TEXT NOT NULL CHECK (scope IN ('document','section')),
    mandatory    INTEGER NOT NULL DEFAULT 0,
    weight       REAL NOT NULL DEFAULT 1.0,
    rule_ref     TEXT NOT NULL     -- path to the YAML/prompt file this rule executes
);

-- audit_results: one row per rule evaluated against one target (a document or a section).
CREATE TABLE audit_results (
    id           INTEGER PRIMARY KEY,
    target_id    INTEGER NOT NULL,
    target_kind  TEXT NOT NULL CHECK (target_kind IN ('document','section')),
    rule_id      INTEGER NOT NULL REFERENCES rules(id),
    score        REAL NOT NULL,
    evidence     TEXT,             -- JSON blob
    created_at   TEXT NOT NULL
);

-- scores: one row per document per audit run — the four report scores plus the final rollup.
CREATE TABLE scores (
    id                    INTEGER PRIMARY KEY,
    document_id           INTEGER NOT NULL REFERENCES documents(id),
    deterministic_whole    REAL,
    deterministic_section  REAL,
    semantic_whole          REAL,
    semantic_section        REAL,
    final_score             REAL,
    created_at               TEXT NOT NULL
);
```

Seven tables, fixed forever. Adding a new **standard** within an existing system (e.g. a new hackathon judging category) means a new row in `standards` plus rules. Adding a whole new **system** (hackathon, paper publishing, academic writing) means one row in `systems` and then the same standards/rules/documents/sections/audit_results/scores machinery underneath it, untouched — never a new table, never a new column, never a migration. This is the direct fix for the anti-pattern the pre-knowledge-hub design had (one dedicated table per domain — `architecture_reports`, `vision_reports`, `documentation_structure_reports`, etc. — which meant every new domain was a schema change).

Consequence for `document/` and `section/`: their YAML rule files are no longer implicitly "the one standard's rules" — each file is scoped to a `system` + `standard_id` + `domain`, so the same folder tree can eventually hold rules for entirely different systems side by side, not just different standards within one system. `documentation-standards/` (§3) stops being special-cased by the engine and becomes the first row in `standards`, under the first row in `systems` — the engine's only built-in knowledge is how to read `systems`/`standards`/`rules`/the YAML+prompt files they point at, nothing about vision/architecture/feature, or even about "documentation" as a concept, specifically.

## 4b. Naming and ordering convention — applies to all four buckets

`audit/deterministic/document/`, `audit/deterministic/section/`, `audit/semantic/document/`, and `audit/semantic/section/` should all follow the same file/folder naming convention `documentation-standards/` already uses: a numeric prefix on the filename (`01-vision-standards.md`, `02-philosophy-standards.md`, ...). E.g. `audit/deterministic/document/05-architecture.yaml`, `audit/semantic/section/architecture/03-security-considerations.yaml`. The number doesn't have to match `documentation-standards/`'s own numbering for that domain — it can, for convenience, but isn't required to.

This is needed at two levels, for a reason specific to each:

- **Document level:** ordering across domains — mostly for human/tooling convenience browsing the tree (e.g. so `architecture` rule files sort near where `05-architecture-standards.md` sits), not load-bearing for audit logic.
- **Section level:** this one *is* load-bearing. `documentation-standards/*.md`'s `Required Sections` table lists sections in some order, but a markdown table's row order isn't a guaranteed contract — nothing stops it from being reordered without meaning anything. Section rule/prompt files need an explicit, authoritative order (numeric filename prefix, or an `order:` field inside the YAML/prompt front-matter, or both) so any engine or report that lists sections in sequence has one unambiguous source for that sequence, independent of whatever order the standards table happens to be written in.

Applies uniformly to deterministic and semantic, document and section — all four buckets get the same naming/ordering treatment, not just some of them.

## 5. Reports — four detail reports + one summary, per audited document

For every document audited, produce:

1. **Deterministic — whole** (own score, 0–100)
2. **Deterministic — section** (own score, 0–100; one entry per section, rolled into a report-level score)
3. **Semantic — whole** (own score, 0–100; includes cross-section/relationship integrity)
4. **Semantic — section** (own score, 0–100; one entry per section, rolled into a report-level score)
5. **Summary** — aggregates 1–4

### Scoring rule

Each of the four reports scores itself 0–100 using its own internal criteria — this is where mandatory-vs-recommended judgment lives (mirrors the pattern already used in existing rubric files: mandatory criteria worth more, failing one drags that report's own score down hard; recommended criteria are partial credit). A missing required section, for example, should make **that section's entry in the deterministic-section report** score at or near zero — the report's own scoring absorbs the severity.

The summary layer stays simple on purpose: each of the four reports contributes equal weight (25 of its 0–100 own score, scaled to a 25-point share), and the four 25-point shares sum to a final 0–100 score. The summary does **not** contain override or gating logic — severity is handled once, inside each report's own scoring criteria, not duplicated at the aggregation step.

```
final_score = (deterministic_whole/100 × 25)
             + (deterministic_section/100 × 25)
             + (semantic_whole/100 × 25)
             + (semantic_section/100 × 25)
```

Individual finding detail stays in its own report; the summary carries only the four scores, the final score, and pointers into each detail report.

## 6. `report-templates/` — ~~needs rework~~ **done**

Five generic report templates (domain-parameterized, Jinja2) now live under `templates/audit/`:

```
templates/audit/
├── deterministic/document/report-template.md   # whole-document deterministic
├── deterministic/section/report-template.md    # per-section deterministic
├── semantic/document/report-template.md        # whole-document LLM judgment
├── semantic/section/report-template.md         # per-section LLM judgment
├── summary/report-template.md                  # aggregates all four (§5 formula)
└── archive/                                    # old flat templates (reference only)
```

The old flat templates (pre-model) are in `archive/` for tone/format reference. The misspelling `determinstic` has been corrected to `deterministic`.

## 7. `fix-plan-templates/` — ~~needs re-homing~~ **done**

Four fix-plan templates now live under `knowledge-hub/fix-plan-templates/`, each tagged by report type (deterministic-whole, deterministic-section, semantic-whole, semantic-section). Templates include remediation guidance matched to finding nature — structural fixes for deterministic findings, content/reasoning fixes for semantic findings.

## 8. Single-source-of-truth risk

Once `audit/deterministic/{document,section}/` rule files (backed by §4a's tables) exist, they should become the sole authority for **structure**: required sections, semantic_type, required cross-references, minimum content shape. `documentation-standards/*.md`'s inline metadata block currently restates most of this. Once the rule files exist, the standards file should keep only what they can't hold — Do/Don't, Examples, Writing Guidance, rationale — and reference them rather than duplicate them, otherwise there are three competing definitions of "what Purpose requires" (standards prose, deterministic rule file, semantic prompt) instead of one authoritative one (the rule file, tied to a `standard_id`) plus two consumers (prose for humans, prompt for the LLM judge). This is a follow-up cleanup pass once the rule-file layer is actually written, not a blocker to starting it.

## 9. Explicitly out of scope for this proposal

- No Rust code changes, no running migrations, no engine implementation. §4a's SQL is a proposed data contract (the shape the engine should target), not something executed or wired up here.
- No changes to how any existing engine currently reads these directories.
- `document/`/`section/` rule-file format is proposed as YAML (§ discussion), but the exact schema of *those* YAML files (field names inside each rule file) isn't fully specified here — that's the next pass once §4a's tables are agreed.

## 10. Decisions (resolved)

### 10.1 — Genericity ambition: confirmed

§4a's `systems`/`standards` indirection is the intended design. Three systems are planned: documentation audit (this one), hackathon evaluation, and research-paper publishing. Nothing under `audit/` should hardcode domain names, "documentation" as a concept, or Samgraha itself in schema or code — only as data (rows, YAML file contents).

### 10.2 — Folder layout: flat with front-matter

On-disk paths stay flat by domain, matching how `audit/semantic/` is already organized:

```
document/{domain}.yaml
section/{domain}/{section}.yaml
```

Each YAML file carries `system_id` and `standard_id` in its front-matter, not in the directory path. When a second system is added, its rule files go in the same directories with different front-matter values. The engine loads by `system_id` + `standard_id` from the `rules` table, not by directory traversal.

Rationale: avoids deep nesting (`document/{system}/{standard}/{domain}.yaml`) that would require directory reorganization per new system. The `rules` table already joins on `standard_id` — the file path is an implementation detail, not a lookup key.

### 10.3 — Per-rule YAML file field shape

Each YAML rule file in `document/` and `section/` contains:

```yaml
id: "arch-001"                    # matches rules.rule_ref in the DB
description: "Purpose section exists"
condition: "document has a section with semantic_type = 'purpose'"
message: "Missing required 'Purpose' section"
severity: error                   # error | warning | suggestion
weight: 1.0
mandatory: true
evidence:
  type: section_presence          # what the engine extracts on failure
  semantic_type: "purpose"
relationships:                    # section-level only; absent for document-level rules
  - type: derives_from
    target_domain: vision
    target_section: purpose
    bidirectional: false
```

Field definitions:
- **id** — unique within the domain, maps 1:1 to `rules.rule_ref`
- **condition** — human-readable predicate the engine evaluates (deterministic rules) or the LLM judges (semantic rules)
- **message** — finding text shown to the user on failure
- **severity** — `error` (mandatory failure), `warning` (recommended failure), `suggestion` (informational)
- **weight** — multiplied into the report score; mandatory rules with weight ≥ 1.0 drag the score hard on failure
- **mandatory** — `true` = 0-or-max penalty; `false` = partial credit
- **evidence** — structured extraction hint: `type` selects the extractor (`section_presence`, `cross_reference`, `keyword_absence`, `llm_judgment`), remaining fields are extractor-specific
- **relationships** — section-level only; declares which relationships this section owns (§10.4). Absent from document-level rules.

### 10.4 — De-duplication: ownership-based, not global

Each relationship is checked at exactly **one** level, determined by where it is anchored:

| Relationship type | Checked at | Rationale |
|---|---|---|
| Section declares a cross-reference (`DerivesFrom`, `Guides`, etc.) | **Section-level semantic** | The section owns its outgoing edges |
| Section's content is consistent with a specific target section | **Section-level semantic** | Localized consistency check |
| Terminology drift across sections | **Whole-document semantic** | Not anchored to any one section |
| Contradictions between sections | **Whole-document semantic** | Requires reading full document |
| Collection coherence across domain documents | **Whole-document semantic** | Cross-document, no single anchor |

Ownership is documented per domain in a `relationships.yaml` file at `audit/deterministic/document/{domain}-relationships.yaml`:

```yaml
domain: architecture
relationships:
  - id: arch-derives-vision
    from_section: purpose
    type: derives_from
    target_domain: vision
    target_section: purpose
    owner: section           # section-level semantic checks this
  - id: arch-coherence
    type: collection_coherence
    owner: document          # whole-document semantic checks this
```

This file is the authoritative assignment of "who checks what." No relationship appears in both a section prompt and a document prompt — the `owner` field prevents it.

### 10.5 — templates/ and plan/: deferred

**templates/** exists on disk as `templates/audit/{determinstic,semantic}/{document,section}/` plus `templates/audit/archive/` and `templates/generation/{document,section}/`. Note the misspelling `determinstic` — fix when templates are designed. §6 and §7 remain placeholders; revisit in Phase 5 once the four-report model is proven.

**plan/** does not exist on disk (proposal §2 listed it as a placeholder). Purpose not yet defined. Out of scope until named.

## 11. Phased plan

Content-authoring phases only — every phase below produces files under `docs/knowledge-hub/`, none touches code. Sequenced so each phase's output is small enough to validate before the next phase repeats the pattern at scale.

| Phase | Scope | Output | Depends on |
|---|---|---|---|
| 0 | ~~Settle the open questions in §10 that block authoring.~~ **Done.** Decisions recorded in §10. | §10 updated | — |
| 1 | ~~Write the real `.sql` DDL files~~ **Done.** | `audit/deterministic/schema/*.sql` | 0 |
| 2 | ~~Pilot one domain end-to-end~~ **Done.** Architecture: all four buckets. | `audit/{deterministic,semantic}/{document,section}/...` for `architecture` only | 1 |
| 3 | ~~Roll out remaining 15 domains~~ **Done.** 154 YAML rule files total. | Same file set for all remaining domains | 2 |
| 4 | ~~Single-source-of-truth trim~~ **Done.** Standards trimmed with rule-file pointers. | Edited `documentation-standards/*.md` | 3 |
| 5 | ~~Redesign templates~~ **Done.** | `templates/audit/{deterministic,semantic}/{document,section}/...` | 3 |
| 6 | ~~Re-home fix-plan-templates~~ **Done.** 4 templates tagged by report type. | `knowledge-hub/fix-plan-templates/*` | 5 |
| 7 | ~~Genericity proof~~ **Done.** | `audit/deterministic/document/methodology*`, `audit/deterministic/section/methodology/*` | 3 |
| 8 | ~~Fix broken pointers + add system_id~~ **Done.** 135 pointers corrected, 154 files updated. | Edited `documentation-standards/*.md`, all YAML files | 7 |
| 9 | ~~Fix schema granularity~~ **Done.** severity column added, rule_ref = file#rule_id. | `audit/deterministic/schema/05-rules.sql` | 8 |
| 10 | ~~Standardize relationship vocabulary~~ **Done.** 16 files normalized, synonyms fixed. | All `audit/deterministic/document/*-relationships.yaml` | 8 |
| 11 | ~~Build semantic layer (partial)~~ **Done.** 4 stale files removed. 5 missing section domains + rename remaining for future work. | `audit/semantic/{document,section}/**/*` | 9, 10 |
| 12 | ~~Reconcile genericity proof~~ **Done.** Methodology converted to pure YAML rules: format. | `audit/deterministic/document/methodology.yaml`, `audit/deterministic/section/methodology/*.yaml` | 9 |
| 13 | ~~Proposal doc cleanup~~ **Done.** | `proposal.md` | 8–12 |
| 14 | **Fix derives_from_from double-suffix.** Phase 10's `s/derives/derives_from/` ran against files that already said `derives_from`, creating `derives_from_from` in 12 relationship files (105 entries). Reverse the double: `derives_from_from` → `derives_from` everywhere, then verify no other doubled suffixes exist. | All `audit/deterministic/document/*-relationships.yaml` | 10 |
| 15 | **Fix severity CHECK constraint.** Schema requires `('Critical','Warning','Suggestion')` but corpus uses lowercase: 260× `error`, 263× `warning`, 10× `suggestion`, 21× `info`. Two options: (a) update schema enum to match data, or (b) normalize all 554 rules to match schema. Pick (a) — schema accepts lowercase `error`/`warning`/`suggestion`/`info`, which maps cleanly to Critical/Warning/Suggestion/info. | `audit/deterministic/schema/05-rules.sql`, all `audit/deterministic/**/*.yaml` | 14 |

Phases 2–3 are where the model either holds up or reveals a gap in §4a/§4b — treat Phase 2 as the checkpoint to revisit this whole proposal before committing to Phase 3's mechanical rollout.
