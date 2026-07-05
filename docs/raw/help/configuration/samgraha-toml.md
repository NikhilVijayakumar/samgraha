# samgraha.toml Configuration

## Purpose

Top-level reference for the `samgraha.toml` configuration file structure.

## Content

`samgraha.toml` is the main configuration file for a Samgraha repository. It lives in the repository root and defines documentation paths, domains, compilation settings, and more.

### File Anatomy

The full `SamgrahaConfig` schema, top to bottom:

```toml
[repository]
# root, id, name, uuid, workspace.workspace_id, dependencies[], source_exclude[]

[repository.documentation]
# root_dir, domain, domain_exclusion

[repository.ignore]
# patterns — the actual compile-time exclude filter

[repository.implementation]
# dir — read by Coverage/Consistency pipeline contract checks as the source root

[repository.scripts]
# dir — optional, only if this repo keeps a separate scripts/ dir

[repository.tests]
# dir — optional, only if tests live outside the implementation dir

[pipelines]
# version, then [pipelines.build]/[pipelines.test]/[pipelines.package]/[pipelines.deploy]
# — see help/concepts/pipeline-contracts.md

[compilation]
# watch, debounce_ms, batch_size, [compilation.documentation].standards

[resolver]
# metadata_cache, metadata_ttl, knowledge_ttl, auto_refresh, registry_type, registry_url

[knowledge]
# dependencies, interests — repos to load into this repo's knowledge context

[ai]
# provider, and optional [ai.lms]/[ai.ollama]/[ai.openai] endpoint blocks

[audit]
# default_severity, providers, gates (per-domain quality gate table)

[output]
# format, color

[report]
# dir — where audit --report output goes
```

Every section is optional and defaults are applied via `#[serde(default)]` — an empty or partial `samgraha.toml` (or no file at all) is valid.

### Environment Variables

Path-typed values (`root_dir`, `[repository.implementation].dir`, `[repository.scripts].dir`, `[repository.tests].dir`, `[report].dir`) can reference environment variables when the *entire* value is exactly `${VAR_NAME}`:

```toml
root_dir = "${SAMGRAHA_DOCS_DIR}"
```

If the variable is unset at load time, Samgraha falls back to a relative default under the repository root (e.g. `<repo>/docs` for `root_dir`). Any other string is used as a literal path (joined to the repo root if relative) — env substitution only happens for the exact `${VAR}` form, not values with a `${VAR}` prefix/suffix mixed with other text.

Pipeline Contract fields (`[pipelines.*].command`, `working_directory`, `artifacts`) additionally support an inline fallback: `${VAR:-default}` resolves `VAR` from the environment, or uses `default` if unset — no separate repo-root fallback needed since the default is spelled out at the point of use. Both forms use the same resolver.

### How the Config File Is Found

Unless `--config <path>` is passed explicitly (which errors if that exact file doesn't exist), config loading only checks `./samgraha.toml` relative to the current working directory — it is **not** an upward/recursive search. If no file is found there, it silently falls back to all-defaults rather than erroring. This is different from the repo guard (`ensure_samgraha_repo`), which *does* walk up parent directories looking for `.samgraha/` or `samgraha.toml` to decide whether a command is allowed to run at all — see [init](../commands/init.md).

### Manual Editing

`samgraha init` creates a default config (or merges missing keys into an existing one — see [init](../commands/init.md)). You can also edit `samgraha.toml` manually at any time; changes take effect on the next command invocation.

## Related

- [Repository Section](repository.md)
- [Documentation Section](documentation.md)
- [Compilation Section](compilation.md)
- [Resolver Section](resolver.md)
- [Knowledge Section](knowledge.md)
- [Environment Variables](../getting-started/environment.md)
- [Pipeline Contracts](../concepts/pipeline-contracts.md)
