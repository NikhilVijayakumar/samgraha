# Deterministic Audit Schema

Generic, standard-agnostic storage for the audit engine. Seven tables, fixed forever.

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
| 05 | `rules` | One row per check, pointing at a YAML rule file via `rule_ref` |
| 06 | `audit_results` | One row per rule evaluation (score + evidence JSON) |
| 07 | `scores` | Aggregated scores per document per audit run |

## Loading order

Run `00-reset.sql` first for a clean slate, then `01` through `07` in order.

## See also

- `proposal.md` §4a — schema rationale
- `proposal.md` §5 — scoring formula
- `proposal.md` §10.3 — per-rule YAML file field shape
