# Pipeline Contracts

## Purpose

Explains how a repository declares its own build/test/package/deploy
commands, so Saṃgraha never needs to guess or hardcode which build system,
language, or ecosystem a repository uses.

## Content

**"The repository knows itself."** Instead of Saṃgraha detecting Cargo,
npm, Maven, or any other build system, a repository declares its own
operational contracts in `samgraha.toml [pipelines.*]` — plain commands,
paths, and expected outputs. Saṃgraha reads the contract and verifies the
repository keeps its own promises. Zero new code is needed to support a new
language or build system; the repository just declares its contract.

```toml
[pipelines]
version = "1.0"

[pipelines.build]
command = ["${BUILD_SCRIPT}"]
working_directory = "${PROJECT_ROOT}"
artifacts = ["${PROJECT_ROOT}/target/release/samgraha.exe"]
success_exit_code = 0
timeout = "30m"

[pipelines.test]
command = ["${TEST_SCRIPT}"]
working_directory = "${PROJECT_ROOT}"
artifacts = ["${PROJECT_ROOT}/coverage.xml", "${PROJECT_ROOT}/junit.xml"]

[pipelines.package]  # optional
command = ["${PACKAGE_SCRIPT}"]
working_directory = "${PROJECT_ROOT}"
artifacts = ["${PROJECT_ROOT}/release/"]

[pipelines.deploy]  # optional
command = ["${DEPLOY_SCRIPT}"]
working_directory = "${PROJECT_ROOT}"
artifacts = []
```

All 4 contract types share the same shape (`command`, `working_directory`,
`artifacts`, plus optional `success_exit_code`/`timeout`/`description`/
`produces`/`consumes`) — one verification algorithm runs over all of them,
no per-type special-casing.

### Env Variable Resolution

Values use `${VAR}` (error if unset, falls back to a repo-root-relative
default for path fields) or `${VAR:-default}` (falls back to the literal
`default` instead). `${PROJECT_ROOT}` is always auto-set to the repository
root. This is the same resolver used for `[repository.implementation].dir`
and every other env-resolvable config path — see
[samgraha.toml Configuration](../configuration/samgraha-toml.md).

### Execution Modes

| Mode | Trigger | Behavior |
|------|---------|----------|
| Verify-only (default) | No flag | Checks declared `artifacts` exist and are fresh relative to source. Never executes anything. |
| Execute | `--execute` | Runs the declared command, captures stdout/stderr/exit code, collects fresh evidence. |
| Dry-run | `--dry-run` | Prints what would execute to stderr, does nothing. |

The command is always printed to stderr before it runs. First `--execute`
of a new contract prompts for confirmation (suppressible with `--yes`).

### Evidence Freshness

Each declared artifact is classified before verification:

- **Fresh** — artifact newer than source → verify directly
- **Stale** — artifact older than source, or missing → suggests `--execute`
- **Unknown** — no timestamp available, or the contract declares no
  artifacts at all (e.g. `deploy`) → verify with a warning, never blocks

### Trust Boundary

`--execute` (and every other mode) only ever runs contracts declared by the
repository the command was invoked in. Saṃgraha's registry can load
documentation and standards from other, registered repositories
(dependencies, interests) — it never executes a pipeline contract belonging
to any repository other than the current one. A dependency's `[pipelines.*]`
entries are inert data: readable, never executable from outside their own
repository.

### Not This

- Not a build-system replacement — Saṃgraha never builds anything itself,
  it only verifies the repository's own declared contract.
- Not `[audit]` — audit checks (Build/Security/Consistency/Coverage) read a
  Pipeline Contract as *evidence*; the contract itself is a separate,
  simpler concept: "how does this repository run its own commands."
- Not the audit-type `Pipeline` (`crates/audit/src/pipeline.rs`) — that name
  is reserved for the 5 audit types (Build, Security, Consistency, Coverage,
  Dependency). Pipeline Contracts are implemented as `PipelineContractConfig`
  + `ContractRunner`, deliberately not sharing the `Pipeline` name.

## Related

- [Audit Concept](audit.md)
- [Build Audit](build-audit.md)
- [samgraha.toml Configuration](../configuration/samgraha-toml.md)
