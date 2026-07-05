# samgraha version

## Purpose

Display the Samgraha version.

## Content

### Synopsis

```bash
samgraha version
samgraha --version
samgraha -V
```

`version` is a real subcommand (no arguments) and does not require the repo guard — it works outside a samgraha repo. `--version`/`-V` are clap's auto-generated global flags and print just the raw version string; the `version` subcommand instead prints a small JSON object (via the same JSON formatter used elsewhere, regardless of `--json`).

### Description

Prints the current Samgraha package name, version, and description.

### Example Output

```bash
$ samgraha version
{
  "name": "samgraha",
  "version": "0.1.0",
  "description": "Knowledge Engineering Platform"
}

$ samgraha --version
samgraha 0.1.0
```

Version numbers come from the crate's `Cargo.toml` (`CARGO_PKG_VERSION`).

## Related

- [Build Guide: Release](../build-guide/release.md)
- [Getting Started: Installation](../getting-started/installation.md)
