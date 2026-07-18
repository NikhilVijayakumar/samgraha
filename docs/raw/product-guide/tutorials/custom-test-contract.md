# Tutorial: Custom Test Contract

## Purpose

Step-by-step: declare a `[pipelines.test]` contract so Coverage Audit (CV6) and Implementation Audit (I8) report real unit/e2e pass-fail counts and coverage %, instead of a static advisory message or a file-presence guess.

## Content

### What's Configurable Today

`[pipelines.test]` in `samgraha.toml` has the same shape as `[pipelines.build]`: a `command` the repository owns, plus `artifacts` — paths MCP checks afterward. For the test contract specifically, the first declared artifact is where your script must write a JSON results file. The repository supplies the script that produces it, in whatever tooling fits its own stack (`cargo test` + `tarpaulin`/`llvm-cov`, `pytest` + `coverage.py`, `jest` + `nyc`, ...).

This repo's own contract (`scripts/test-coverage.ps1` / `.sh`) is a worked example — a single `cargo llvm-cov --workspace --json` run covers both `--lib` unit tests and the `tests` crate's `e2e_*.rs` integration tests, split by parsing cargo's own "Running unittests" vs "Running tests\<file>.rs" log headers.

### Step 1: Declare the Contract

```toml
[pipelines.test]
command = ["pwsh", "-File", "scripts/test-coverage.ps1"]
working_directory = "${PROJECT_ROOT}"
artifacts = ["docs/report/test-results.json"]
description = "Run unit+e2e tests with coverage, emit structured results"
```

Use a `.sh` script (or a `cmd`/`pwsh` variant guarded by platform) if the repo needs to run on both Windows and Unix CI.

### Step 2: Write the Results JSON

Your script's last step, regardless of whether tests passed or failed, is writing this shape to the declared artifact path:

```jsonc
{
  "unit": {
    "total": 42, "passed": 40, "failed": 2, "skipped": 0,
    "failures": [{ "name": "test_foo", "message": "assertion failed" }]
  },
  "e2e": {
    "total": 5, "passed": 5, "failed": 0, "skipped": 0,
    "failures": []
  },
  "coverage_percent": 78.4
}
```

Fields mirror `TestRunReport` in `crates/schemas/src/test_run.rs`. All fields default sanely if omitted (e.g. a repo with no e2e suite can omit `"e2e"` entirely) — `unit`/`e2e` both default to zeroed/empty, `coverage_percent` defaults to `null`.

A nonzero exit code from your test command is expected and fine when tests fail — write the JSON first, then let the command exit however your test runner naturally exits.

### Step 3: Run It

```bash
samgraha audit --pipeline coverage --execute
samgraha audit --pipeline implementation --execute
```

`--execute` runs the declared command and reads the fresh results. Without `--execute`, both pipelines read a previously-produced results file if one already exists at the declared path, or fall back to their pre-existing advisory/heuristic behavior if nothing's been produced yet.

### What You Learned

- `[pipelines.test]` reuses the same `ContractSpec` shape as `[pipelines.build]` — no new config surface
- The contract's first `artifacts` entry is where your script must write results, in the documented `TestRunReport` JSON shape
- CV6 (Coverage) and I8 (Implementation) both consume it; CV6 stays advisory (never blocks a score), I8 uses it as a stronger signal than its file-presence fallback
- `--execute` is required to actually run the script; a plain audit reads whatever's already on disk

## Related

- [Custom Standard Tutorial](custom-standard.md)
