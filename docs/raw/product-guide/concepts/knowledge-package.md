# Knowledge Package

## Purpose

The concept of packaging a subset of knowledge for distribution or partial loading.

## Content

A **Knowledge Package** is a portable bundle of compiled documentation. Packages enable:

- **Selective inclusion** — Include only the domains relevant to a consumer instead of the full knowledge database.
- **Distribution** — Share knowledge with other teams or repositories.
- **Profiles** — Use one of six built-in package profiles to control which domains are included.

### Package Profiles

`samgraha package`'s `--profile` filters which *domains/standards* are included (whole documents, not a section subset):

| Profile | Domains included |
|---------|------------------|
| `minimal` | vision, readme |
| `documentation` | vision, readme, design, feature |
| `engineering` | vision, architecture, engineering, feature, feature-technical |
| `ai-assistant` | everything except prototype |
| `development`, `full` | everything |

(Separately, each built-in standard also defines its own named section-level profiles, e.g. the Feature standard's `implementation`/`review`/`architecture` profiles — see [Profiles](profiles.md). That mechanism exists in the standard definitions but the `package` command's `--profile` flag above does not currently use it; the two "profile" concepts are distinct.)

### Packaging Command

```bash
samgraha package [output] --profile <minimal|development|documentation|engineering|ai-assistant|full> [--json]
```

`output` defaults to `./knowledge-package` (a directory containing `knowledge.db`, a `docs/` tree, and `samgraha-package.json`) or `./knowledge-package.json` if `--json` is passed (a single legacy JSON file). There is no `--domain` flag on `package` — domain selection is entirely driven by `--profile`.

## Related

- [Command: package](../commands/package.md)
- [Standards and Profiles](standards.md)
- [Build Guide: Packaging](../build-guide/packaging.md)
