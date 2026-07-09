# Test Report Schema

## Purpose

Defines the JSON shape a repo's `[pipelines.test]` script must write to its declared results artifact. This is the one thing MCP requires; everything about *how* a repo produces it (test framework, coverage tool, language) is the repo's own choice.

Canonical definitions:
- Machine-readable: [`test-report.schema.json`](test-report.schema.json) (JSON Schema draft-07) — validate against this in CI if you want a hard guarantee your script's output is well-formed.
- Rust source of truth: `crates/schemas/src/test_run.rs` (`TestRunReport`, `TestSuiteResult`, `TestFailure`).

Consumers: Coverage Audit CV6 (`docs/raw/audit/coverage-audit.md`) and Implementation Audit I8 (`docs/raw/audit/implementation-audit.md`).

## Shape

```jsonc
{
  "unit": {
    "total": 42,
    "passed": 40,
    "failed": 2,
    "skipped": 0,
    "failures": [
      { "name": "test_foo", "message": "assertion failed" }
    ]
  },
  "e2e": {
    "total": 5,
    "passed": 5,
    "failed": 0,
    "skipped": 0,
    "failures": []
  },
  "coverage_percent": 78.4
}
```

- `unit` / `e2e` — same shape (`TestSuiteResult`) for both. A repo without a distinct e2e suite can omit `"e2e"` entirely; it defaults to all-zero/empty.
- `total`, `passed`, `failed` — required, non-negative integers. `skipped` defaults to `0`.
- `failures` — optional, defaults to `[]`. Each entry needs at least `name` (becomes the audit finding's `location` and the fix-plan's step target); `message` is optional free text, defaults to `""`.
- `coverage_percent` — optional, `0-100` or `null`/omitted if the script doesn't measure coverage. One overall number, not per-suite — coverage is a property of the code exercised, not of which suite exercised it.

## Where It's Read From

The path is `[pipelines.test].artifacts[0]` in `samgraha.toml` — no separate config key. See the [Custom Test Contract tutorial](../product-guide/tutorials/custom-test-contract.md) for the full wiring walkthrough.

## Compatibility

All fields beyond the required `total`/`passed`/`failed` on each suite have serde defaults on the Rust side — an older or minimal script that only emits those three fields per suite (and omits `skipped`, `failures`, `coverage_percent`, or the `e2e` object entirely) still parses correctly. Unknown top-level or nested fields are rejected (`additionalProperties: false` in the JSON Schema) rather than silently ignored, so a typo'd field name fails loudly instead of quietly not being read.

## Related

- [Coverage Audit Spec](coverage-audit.md) — CV6
- [Implementation Audit Spec](implementation-audit.md) — I8
- [Custom Test Contract Tutorial](../product-guide/tutorials/custom-test-contract.md)
