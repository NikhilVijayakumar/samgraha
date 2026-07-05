# samgraha info

## Purpose

Display repository and environment information.

## Content

### Synopsis

```bash
samgraha info [path]
```

`path` is a positional argument, optional, defaulting to the current directory.

### Description

`info` prints a summary of the current Samgraha environment. In text mode it prints only the lines that have content:

- Repository name
- Registry (`knowledge.db`) path
- Document count
- Standards — effective domains (declared `domain` list minus `domain_exclusion`), comma-joined
- Built-in — status of the `help`/`standards` built-in stores shipped next to the binary

With `--json`, the output additionally includes `services` (registered services, e.g. registry, planner, resolver, search) and `policy` (runtime policy), which aren't printed in plain-text mode.

### Example Output

```
Repository:  samgraha
Registry:    E:\Python\samgraha\.samgraha\knowledge.db
Documents:   42
Standards:   readme, vision, philosophy, architecture, feature, feature-design, feature-technical, design, engineering, external-context, prototype
Built-in:    standards (loaded), help (loaded)
```

If a built-in store's `.db` file is missing next to the binary, it's reported as `help (missing)` or `standards (missing)` instead of `(loaded)`.

## Related

- [Command: env](env.md)
- [Getting Started: Environment](../getting-started/environment.md)
- [Concepts: Repository](../concepts/repository.md)
