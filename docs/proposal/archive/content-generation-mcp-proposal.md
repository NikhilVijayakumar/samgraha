# Content Generation via MCP — Proposal

## Status / Relationship to Existing Docs

This concretizes work already **designed and resolved at the decision
level** in `generic-script-architecture-proposal.md`, specifically:

- §7.2 — semantic determination output saved to DB, not a repo file
- §7.4 — the 4-step document generation pipeline (scaffold → section
  headings → semantic section content, saved to DB → assemble from DB)
- §4 item #2 ("Semantic-fill ownership") — **RESOLVED**: samgraha never
  runs an LLM itself. It exposes a task list + a place to write the
  result back; whatever client is on the other end of the MCP session
  (Claude Code, opencode, anything) does the actual semantic work.

That resolution already covers generation in principle. **Confirmed by
source audit, not assumed: it was never implemented.** Grepped
`crates/audit`, `crates/mcp`, `crates/services` for
`semantic_review`/`SemanticReviewBundle` — the machinery exists **only**
in the `audit` path (`build_semantic_review`,
`build_pipeline_semantic_review`, wired into `audit`'s dispatch and
`reporting.rs`). `run_system_scaffold` dispatches straight to
`handle_run_system_script_with_capability(Scaffold)` — a plain script
call, zero semantic-task construction. This document specifies the
concrete MCP tools, schema reuse, and the two mechanics §7.4 doesn't
cover (per-standard granularity choice, section-level dependency
ordering, code-vs-documentation branching) needed to close that gap.

## 1. What Already Exists, Reusable As-Is

- **`documents`/`document_sections`** (per-repo `knowledge.db`,
  `crates/registry/src/migration.rs` V1/V4/V5) — `documents.body` for
  whole-document content, `document_sections.content` per section,
  with `section_order`/`parent_id` already modeling structure. Today
  these are populated **only** by `compile`, reading a file that
  already exists on disk. This proposal's core move: let generation
  write into these same tables **before** a file exists, not only
  after one does — no new content-storage table needed.
- **`audit`'s `semantic_review.tasks` → judge → `store_section_report`**
  round-trip — the pattern to mirror for generation, not reinvent.
  Confirmed working (this is the real, implemented half of the
  semantic-fill design).
- **`templates` table** (`kind = 'generation'`, `schema/knowledge-hub/16-templates.sql`)
  — already stores full generation-template bodies
  (`templates/generation/document|section`), confirmed populated by
  `knowledge-hub-loader.py` Pass 6.
- **`domain_relationships`** — the existing cross-domain dependency
  graph, already drives generation ordering at domain granularity
  (§7.1/§7.4's "Vision generates before Philosophy" pattern).

## 2. What's Actually Missing

- No MCP tool returns a content-generation task list the way `audit`
  returns `semantic_review.tasks`.
- No MCP tool accepts generated content back the way
  `store_section_report` does for audit findings.
- No `assemble` capability that reads `document_sections` rows back out
  and writes the final file — `scaffold` only creates the initial
  skeleton; nothing today reads stored-but-not-yet-filed content back
  into a document.
- No declared per-standard generation granularity. §7.4 describes the
  section-by-section pipeline as if it's the only mode. It needs to be
  a per-standard choice — whole-document, section-by-section, or
  hybrid (generate sections individually, then a whole-document
  coherence pass to align them) — not a fixed assumption baked into
  the mechanism.
- No section-level dependency graph. `domain_relationships` operates at
  domain granularity only; nothing today captures "within `vision`,
  the `solution` section needs `problem`'s content as context first."
- No code-vs-documentation branch. §7.4's pipeline assumes every
  generation target is a document with sections. A standard generating
  code/scripts (a `python_hackathon`-style target, or any future
  code-scaffolding standard) needs the DB-persistence step skipped
  entirely — code correctness comes from running it / from
  `script_result`-kind checks, not from re-reading stored prose, and
  persisting code text to `document_sections` would add a
  file-vs-DB synchronization burden with no matching benefit.

## 3. Proposed MCP Tools

### `generate` — mirrors `audit`

Params: `standard`, `domain`, `repo_path`. `mode` is **not**
caller-supplied — resolved from the standard's own declared
granularity (§5), same principle as §4 item #7 of
`generic-script-architecture-proposal.md` ("domain cardinality is a
per-system workflow choice, not something samgraha tracks or a caller
decides at call time").

- `mode: document` → one task, full upstream context (already-generated
  upstream domains, per `domain_relationships`), template content.
- `mode: section` → one task per `section_catalog` entry not yet
  generated, respecting `section_dependencies` (§6) — a section whose
  dependency hasn't been generated yet is deferred to a later call, not
  returned early with incomplete context.
- `mode: hybrid` → section tasks first; once all are stored (§3's
  `store_generated_content`), a second call returns one whole-document
  coherence task, pre-loaded with every section's stored content as
  context, for a reconciliation pass.

Returns `generation_review.tasks[]`, each `{target: {domain,
section|null}, template_content, upstream_context, instruction}` — same
shape discipline `semantic_review.tasks` already established.

### `store_generated_content` — mirrors `store_section_report`

Params: `{document_id|new, domain, section|null, content,
git_revision}`. Writes into `document_sections.content` (section mode)
or `documents.body` (document mode) — the same tables `compile` already
populates, just earlier in the document's lifecycle. Validates the
target section/domain against `section_catalog` before accepting, same
discipline `store_section_report` already applies against `rules`.

### New capability: `assemble`

Script-kind, standard-provided — joins the existing 6
(`init`/`validate`/`calculate`/`report`/`scaffold`/`plan-generation`).
Reads `document_sections` for a document (ordered by `section_order`) +
the document-level template's structural shell, writes the final file.
This is what keeps "the final document is always produced by script"
true — the calling agent's LLM never writes the file directly, only
ever writes into DB via `store_generated_content`; script does the
last-mile assembly.

Per §4 item #9 of `generic-script-architecture-proposal.md` (already
resolved: new capability types don't need a new dedicated MCP tool,
they go through the generic `run_system_script(system, capability,
params)` dispatch) — `assemble` likely doesn't need its own dedicated
tool, just a new valid `capability` value. Flagged in §9, not decided
here.

## 4. Code vs. Documentation — Explicit Branch

A domain declares its generation output kind — `documentation` (full
generate→store→assemble pipeline, content persisted for reuse in later
audit/fix passes) or `code` (the calling agent generates and writes the
target file directly; `scaffold` still creates the skeleton/folder
structure, but `store_generated_content` is never called, `assemble`
never runs, nothing persists to DB). See §7 for where this gets
declared.

## 5. Generation Granularity — Per-Standard Declaration

New column, `standards.generation_granularity` — `document` | `section`
| `hybrid`. `generate`'s `mode` resolves from it. Systems own their own
workflow shape; samgraha reads, never decides — same principle already
applied to domain cardinality (§4 item #7 of the base proposal).

### 5.1 Error/Retry Semantics for `store_generated_content`

- **Stale-dependency reject:** `store_generated_content` must reject
  content for a section whose `section_dependencies` entries have not
  all been stored yet. This is a hard reject, not a warning — storing
  content with incomplete upstream context produces content that looks
  valid but was generated against missing information. The caller must
  generate and store dependencies first, then return for this section.
- **Concurrent-write protection:** If two MCP sessions attempt to store
  content for the same `(document_id, section_catalog_id)` pair, the
  second write is rejected with a conflict error. The caller must
  re-generate against the now-updated upstream context. This matches
  the single-writer discipline `store_section_report` already follows.
- **Dependency violation error shape:** Returns `{error:
  "dependency_unmet", missing_sections: [...]}` — the caller can
  dispatch generation for the missing sections and retry.

## 6. Section-Level Dependency Graph — New

Mirrors `domain_relationships`'s existing shape, one level deeper:

```sql
CREATE TABLE section_dependencies (
    id                     INTEGER PRIMARY KEY,
    standard_id            INTEGER NOT NULL REFERENCES standards(id) ON DELETE CASCADE,
    domain_id              INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    section_catalog_id     INTEGER NOT NULL REFERENCES section_catalog(id) ON DELETE CASCADE,
    depends_on_section_id  INTEGER NOT NULL REFERENCES section_catalog(id) ON DELETE CASCADE,
    UNIQUE(standard_id, section_catalog_id, depends_on_section_id)
);
```

**Decision: explicit-required (option b).** No file in any standard
today declares this explicitly. The numbered filename convention
(`templates/generation/section/{domain}/01-purpose.md`,
`02-vision_statement.md`, ...) implies a linear order, but that's a
sort convention (`section_catalog.sort_order`), not a declared
dependency edge — and it's very likely wrong as a blanket default: many
sections are independent (e.g. a QA domain's `property_testing` and
`benchmark_testing` sections don't obviously depend on `purpose`).

Linear-default is silently wrong for most non-trivial standards —
`purpose` ≠ dependency, it's just sort position. Worse, a standard
author who doesn't declare dependencies gets no generation errors, just
silently wrong context ordering.

**Validation check:** If `section_dependencies` is empty and
`generation_granularity` is `section` or `hybrid`, `validate` must emit
a warning or fail. Every section with at least one dependency edge must
have that edge declared — no implicit linear fallback.

## 7. Schema Changes Summary

- `standards.generation_granularity` (new column, or a 1:1 config
  table if the "everything specific is a row" rule in
  `schema/knowledge-hub/README.md` argues against widening `standards`
  directly) — `document`|`section`|`hybrid`
- A `content_kind` on domains (or a join table) — `documentation`|`code`,
  driving §4's branch
- New `section_dependencies` table (§6)
- **No changes needed to `documents`/`document_sections`** — reused
  exactly as they are today

## 8. Relationship to Existing Local (Consumer-Side) Work

A consuming repo's local, hand-built workaround for this exact gap
(scaffold → extract raw `[...]`/`<!-- TODO -->` placeholder gaps client-side
→ LLM fills them → apply back to the file, no DB round-trip) becomes
unnecessary once this lands — `generate`/`store_generated_content`/
`assemble` replace it with a mechanism every standard gets for free,
instead of every system re-inventing its own gap-extraction script. Not
urgent to rip out an already-working local version once this ships —
just worth new standards not needing to build their own copy of it.

**Separately flagged, different subject, found during the same audit
pass:** `crates/registry/src/migration.rs` already defines
`fix_plans`/`fix_plan_steps`/`fix_sessions` with real MCP tools
(`audit_fix_plan`, `audit_fix_apply`, `audit_fix_accept`,
`audit_fix_reject`, `audit_fix_status`, `audit_fix_list`,
`audit_fix_plan_list/get/render`) — propose-a-fix-plan,
persist-it-with-status, accept-or-reject, already fully built. A
consumer's locally-built "analyze → save fix plan to JSON → human
confirms → fix executes" flow appears to duplicate this existing, more
complete mechanism (it already has accept/reject semantics, attempt
tracking, and rollback instructions per step — more than the local
version has). **Tracked in §9.1 — do not lose during implementation.**

## 9. Open Questions — Resolved

- **§6 dependency-graph population:** Resolved as explicit-required
  (§6). Linear-default is silently wrong; validation check enforces
  declaration.
- **§3 `assemble` dispatch:** Resolved — dispatches through generic
  `run_system_script(system, capability='assemble', params)` per
  §4 item #9 of `generic-script-architecture-proposal.md`. No
  dedicated MCP tool. Already decided in the base proposal;
  re-listed here was a self-contradiction.
- **§3 hybrid coherence pass:** Default to polish-layer — coherence
  output stored as a separate `document_sections` row (flagged as
  `coherence_pass` or distinct `section_kind`), not a wholesale
  replacement of per-section content. Preserves original section
  content as audit trail, enables rollback of just the polish pass
  without regenerating sections.
- **§4/§7 `content_kind` placement:** Resolved to `domains`. A single
  dev-class standard can generate both documentation and scaffolded
  code/config in the same run — the pipeline branch is per-domain,
  not per-standard. Column on `domains`, nullable (default =
  `documentation`).

## 9.1 Separate Tracking Item — Fix-Plans Duplication

`crates/registry/src/migration.rs` defines
`fix_plans`/`fix_plan_steps`/`fix_sessions` with full MCP tooling
(`audit_fix_plan`, `audit_fix_apply`, `audit_fix_accept`,
`audit_fix_reject`, `audit_fix_status`, `audit_fix_list`,
`audit_fix_plan_list/get/render`). A consumer's locally-built
"analyze → save fix plan to JSON → human confirms → fix executes" flow
duplicates this existing, more complete mechanism. **Out of scope for
this document** — flagged for separate proposal or backlog item. Do not
lose track of this during generation-focused implementation.

## 10. Explicitly Out of Scope

Implementation. Migrating any consumer's local scripts to use these
tools once built. Reconciling the `fix_plans` duplication noted in §8
— flagged, not resolved. Any change to the `audit` path's existing,
working `semantic_review`/`store_section_report` mechanism.
