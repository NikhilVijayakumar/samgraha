# Deterministic Audit Schema

Generic, standard-agnostic storage for the audit engine. Eight tables, fixed forever.

## Design rules

- Nothing names a domain, a section type, a standard, or a use-case — everything specific is a **row**, never a table or column.
- Adding a new standard within an existing system = new row in `standards` + rules.
- Adding a whole new system = one row in `systems` + the same tables underneath.
- Never a migration for a new domain.

## Tables

| # | Table | Purpose |
|---|-------|---------|
| 01 | `systems` | Outer use-case / tenant (e.g. "samgraha-documentation", "hackathon-eval") |
| 02 | `standards` | Pluggable rule sets scoped to a system |
| 03 | `documents` | One row per audited document |
| 04 | `sections` | One row per section of a document |
| 05 | `rules` | One row per rule (not per file). Each YAML file's `rules:` list produces multiple rows. `rule_ref` = `path#rule_id` |
| 06 | `audit_results` | One row per rule evaluation (score + evidence JSON) |
| 07 | `scores` | Aggregated scores per document per audit run |
| 08 | `script_cache` | Last execution result per script check per repo fingerprint (§2e cache) |

## Loading order

Run `00-reset.sql` first for a clean slate, then `01` through `08` in order.

## See also

- `proposal.md` §4a — schema rationale
- `proposal.md` §5 — scoring formula
- `proposal.md` §10.3 — per-rule YAML file field shape
- `proposal.md` §2e — script cache strategy
