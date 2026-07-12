# Documentation System Schema

Generic, standard-agnostic storage for registering document systems and
running the audit engine against them. Fixed forever at 22 tables — a new
domain, standard, or whole system is rows, never a migration.

Scope is `docs/knowledge-hub` only — deciding which system a given repo
uses is samgraha's own concern, not this schema's (no `repo_registrations`
table here for that reason).

## Design rules

- Nothing names a domain, a section type, a standard, or a use-case —
  everything specific is a **row**, never a table or column.
- Adding a new domain or section within a standard = new row in `domains` /
  `section_catalog`. Adding a new standard within a system = new row in
  `standards` + `rules` + the content tables underneath. Adding a whole new
  system = one row in `systems` + the same tables underneath.
- No table or column stores a filesystem path into this repo's own
  directory tree for definitional data. A second system registers by
  inserting rows, never by pointing back at this repo's file layout — the
  old `rules.rule_ref = "path/to/file.yaml#rule_id"` pointer was retired for
  exactly this reason. `templates.source_file` / `standard_docs.source_file`
  don't violate this: they're nullable debugging breadcrumbs nothing looks
  a row up by, not a dependency any query or FK relies on.
- Exactly one `systems` row may have `is_default = 1` (DB-enforced by a
  partial unique index). Which system a given repo uses is decided outside
  this schema (samgraha's concern) — this flag only marks the fallback.
- Runtime execution output (`audit_results.evidence`, `script_cache.result_json`)
  stays JSON — its shape is inherently per-rule/per-check variant, unlike
  system *definition* data, which is fully relational.

## Tables

### Registry — what systems and standards exist

| # | Table | Purpose |
|---|-------|---------|
| 01 | `systems` | Outer use-case / tenant (e.g. "samgraha-documentation", "research-paper-publishing"), with the single `is_default` flag |
| 02 | `standards` | Pluggable rule sets scoped to a system |
| 03 | `domains` | Canonical domain catalog per standard, with tier assignment |
| 04 | `relationship_types` | Closed, per-standard vocabulary of domain-graph edge types |
| 05 | `domain_relationships` | The derivation graph edges between domains (e.g. vision derives feature) |
| 06 | `section_catalog` | Canonical required/optional sections per domain — the documentation content section rules |

### Content — the actual definitions a standard is made of

| # | Table | Purpose |
|---|-------|---------|
| 07 | `script_checks` | Registry of script-backed audit checks (manifest + result schema) |
| 08 | `script_check_dependencies` | A check's `depends_on` list as real edges |
| 16 | `templates` | Full body of generation and audit-report templates. `source_file` is an optional debugging breadcrumb, not a lookup key |
| 17 | `standard_docs` | Full body of each domain's documentation-standards spec. Same optional `source_file` breadcrumb |
| 18 | `calculation_rules` | Per-standard scoring bucket configs (deterministic/semantic x document/section, final_score, trend) |
| 19 | `calculation_inputs` | `final_score`'s weighted-sum inputs |
| 20 | `score_bands` | Per-standard rating thresholds (Excellent, Good, ...) |
| 21 | `plan_settings` | Per-standard tier-loop orchestration config |
| 22 | `plan_scenarios` | The repo-state x doc-state x tier x step generation/audit/fix matrix |

### Runtime — one audit run's data

| # | Table | Purpose |
|---|-------|---------|
| 09 | `documents` | One row per audited document |
| 10 | `sections` | One row per section of a document |
| 11 | `rules` | One row per rule (not per file) — self-contained content, not a file pointer. `is_fallback` flags a domain's section-scope fallback rule. `UNIQUE(standard_id, domain_id, section_catalog_id, scope, kind, rule_key)` — short criterion ids (`C1`/`C2`) are reused by design across sections, so the key needs all five columns, not just `rule_key` |
| 12 | `rule_evidence_params` | A rule's evidence-extractor parameters, as rows instead of JSON |
| 13 | `audit_results` | One row per rule evaluation (score + evidence JSON) |
| 14 | `scores` | Aggregated scores per document per audit run |
| 15 | `script_cache` | Last execution result per script check per repo fingerprint |

## Loading order

Run `00-reset.sql` first for a clean slate, then `01` through `22` in
order — foreign keys only ever point at a lower-numbered table.

## Population

This directory defines schema only. Populating a system's rows (domains,
relationships, section catalog, rules, templates, standard docs,
calculation config, plan settings/scenarios) is that system's own loader
script to write — e.g. a `samgraha-documentation` loader reads
`documentation-standards/`, `audit/`, `calculation/`, `script/`,
`templates/`, and `plan/` from this repo and inserts the matching rows.
Nothing in this schema depends on those directories existing.

See `docs/proposal.md` for the full pass-by-pass mapping of
`samgraha-documentation`'s files to these tables.

## See also

- `00-domain-relationships.md` — the human-readable source `domains` +
  `relationship_types` + `domain_relationships` transcribe for the
  samgraha-documentation standard
- `plan/core/tiers.yaml`, `plan/core/loop.yaml` — human-readable source for
  `plan_settings` / the tier-gating fields on `domain_relationships`
- `calculation/**/*.yaml` — human-readable source for `calculation_rules`,
  `calculation_inputs`, and `score_bands`
