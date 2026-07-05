# samgraha package

## Purpose

Package compiled knowledge into a portable Knowledge Package for distribution.

## Content

### Synopsis

```bash
samgraha package [output] [--profile minimal|development|documentation|engineering|ai-assistant|full] [--json]
```

### Description

`package` builds a Knowledge Package from the repository's compiled knowledge (auto-compiling first if `knowledge.db` is empty). `output` is a positional path, optional. There is no `--domain` or `--document` flag — profiles, not domain/document filters, control what's included.

By default the package is written as a directory (`Physical` layout — knowledge databases and documentation copied into the output directory) at `knowledge-package/` under the repository root. With `--json`, a single legacy JSON file is written instead, at `knowledge-package.json` by default.

### Options

| Flag | Description |
|------|-------------|
| `output` | Positional, optional. Output path (default: `knowledge-package` directory, or `knowledge-package.json` with `--json`) |
| `--profile <name>` | Package profile: `minimal`, `development`, `documentation`, `engineering`, `ai-assistant`, `full` (default: `full`) |
| `--json` | Write a single legacy JSON file instead of a directory |

### Profiles

Profiles are fixed, built-in package presets — `minimal`, `development`, `documentation`, `engineering`, `ai-assistant`, `full` — not per-standard or per-domain profiles.

### Examples

```bash
# Package everything (default profile: full) to ./knowledge-package
samgraha package

# Package with a specific profile and output path
samgraha package dist/pkg --profile documentation

# Legacy single-file JSON package
samgraha package --json
```

## Related

- [Concepts: Knowledge Package](../concepts/knowledge-package.md)
- [Concepts: Profiles](../concepts/profiles.md)
- [Build Guide: Packaging](../build-guide/packaging.md)
