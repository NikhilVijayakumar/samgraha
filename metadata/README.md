# `metadata/` — samgraha's own JSON Schema definitions

Samgraha's own repo, versioned with its source — same tier as
`schema/knowledge/`, `schema/registration/`, `schema/standards/`, just
describing document shapes instead of table shapes. See
`docs/proposal/standard-metadata-contract-and-completeness-validation-proposal.md`
for the full design.

## Files

- **`standard.metadata.schema.json`** — validates a standard's own
  `standard.metadata.json` (lives alongside `standard.yaml` and its
  seeder script, copied the same way). Checked at
  `register_standard_globally` time, right after the `smoke_test`
  verify-gate, before the standard is upserted into `standard_registry`.
  Optional — a standard with no custom tables, no templates, and no
  proposal generation doesn't need this file at all.
- **`proposal.schema.json`** — validates `result.proposal` envelopes from
  a script's JSON output, but *only* for standards that declare a
  `proposal_template` in their `standard.metadata.json`. Checked inside
  `run_script_step`, before the `proposal` row is inserted. Reference
  correctness (do the named domains/usecases/steps actually exist and
  belong to each other, does the `git` block match what
  `capture_git_state` actually observed for this run) is cross-checked in
  Rust (`services::step_execution`) — a JSON Schema can validate shape,
  not "does this id exist in this SQLite database."

## What's *not* here

The structural completeness audit (every `domain` has ≥1 `usecase`, every
`usecase` has ≥1 `step`, every deterministic/semantic step has its
script/prompt mapped, every catalogued custom table exists and vice
versa) needs no schema file — it's pure SQL against rows that already
exist or don't, implemented in
`crates/services/src/layer_a_audit.rs` and run as the mandatory final
step of `activate_standard`.
