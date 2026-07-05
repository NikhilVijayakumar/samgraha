# Knowledge Package Packaging

## Purpose

How to package a repository's compiled knowledge for distribution or sharing.

## Content

### What Gets Packaged

`samgraha package` filters the current repository's already-compiled documents by profile (whole documents, matched by their `standard`) and writes them out as either a directory or a single JSON file. There is no `.kpkg` file format — that name doesn't exist in the codebase.

### Packaging Command

```bash
samgraha package [output] [--profile <name>] [--json]
```

- `output` — positional, output path. Defaults to `./knowledge-package` (directory mode) or `./knowledge-package.json` (with `--json`).
- `--profile <name>` — one of `minimal`, `development`, `documentation`, `engineering`, `ai-assistant`, `full` (default `full`).
- `--json` — write a single legacy JSON file instead of a directory.

There is no `--domain` or `--document` flag; the package always draws from whatever is currently compiled in the repository's `.samgraha/knowledge.db`, filtered by profile.

### Package Contents

Directory mode (the default) writes a `samgraha-package.json` manifest plus the filtered documents/sections; JSON mode writes everything into the single output file. The manifest includes the profile used, included domains, document count, and an integrity hash of the packaged artifacts.

### Profiles

Each profile filters by document `standard`:

- `minimal` — `vision`, `readme`
- `documentation` — `vision`, `readme`, `design`, `feature`
- `engineering` — `vision`, `architecture`, `engineering`, `feature`, `feature-technical`
- `ai-assistant` — everything except `prototype`
- `development` / `full` — everything

### Loading Packages

There is currently no built-in mechanism to load a `.kpkg`-style package back in as an extra knowledge store (no `extra_stores` config key exists). Packages are for export/distribution only. The closest related feature is `samgraha registry resolve runtime`, which assembles a virtual, read-only package of a repository plus its resolved dependencies under `.samgraha/resolved`.

### Use Cases

- **Cross-team sharing** — Share a filtered subset of documentation without exposing everything.
- **CI/CD artifacts** — Ship only what the next stage needs.

## Related

- [Build Overview](overview.md)
- [Distribution](distribution.md)
- [Command: package](../commands/package.md)
