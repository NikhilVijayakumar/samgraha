# Repository

## Purpose

What a Samgraha repository is and how it maps to a Git repository.

## Content

A Samgraha **repository** is any directory that has been initialized with `samgraha init`. This writes `samgraha.toml` at the repository root and creates a `.samgraha/` directory, which subsequent commands populate with the compiled knowledge database, manifest, and (if used) registry state.

Key properties:

- **Self-contained** — All Samgraha state lives in `samgraha.toml` (root) and `.samgraha/` (generated state). No external services required.
- **Documentation owned by the repo** — Documentation files live under `[repository.documentation].root_dir` (default `${SAMGRAHA_DOCS_DIR}`, falling back to `<repo>/docs`), organized in a subdirectory per domain.
- **Standards are product-owned** — Built-in standards (Feature, Architecture, etc.) are defined in the Samgraha binary itself (`crates/standards`), not in each repository.

A repository declares its identity (id, name, uuid) and dependencies in `samgraha.toml`; `samgraha compile` writes the corresponding `.samgraha/manifest.json`, which is what enables multi-repo resolution (the Planner and Resolver read it, not `samgraha.toml` directly).

### Repository Detection vs. Config Loading — a gotcha

These two lookups behave differently and are easy to conflate:

- **Repository detection** (the "repo guard" every command but `init`/`version` runs, and `samgraha info`'s fallback root) walks *upward* from the current directory looking for `.samgraha/` or `samgraha.toml` — the same way `git` finds `.git/`.
- **`samgraha.toml` loading** does **not** search upward. It only ever reads `./samgraha.toml` relative to the current working directory (or the path given via the global `--config <path>` flag, which errors if missing). Run a command from a subdirectory of a valid repo and the repo guard passes, but config silently falls back to all-defaults because no `samgraha.toml` is sitting in that subdirectory.

## Related

- [Initialization](../getting-started/initialization.md)
- [Workspace](workspace.md)
- [Manifest](registry.md)
- [Knowledge Database](knowledge-db.md)
