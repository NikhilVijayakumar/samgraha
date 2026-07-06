# Profiles

## Purpose

What standard profiles are and how they control section selection for packages.

## Content

A **profile** is a named subset of sections within a standard. Profiles enable selective packaging of documentation for specific consumers.

### Example Profiles

Feature standard defines three profiles:

- **implementation** — Includes functional requirements, business rules, constraints, dependencies.
- **review** — Includes purpose, acceptance criteria, traceability.
- **architecture** — Includes constraints, dependencies.

### Creating Profiles

Profiles are defined in the standard definition (Rust code for built-in standards, via `StandardDefinition.profiles: Vec<ProfileDef>`). There is currently no config surface for custom standards to define their own (see [Standards: Custom Standards](standards.md)).

### Using Profiles

Per-standard section profiles (`sections_for_profile()`) are a query-time helper on `StandardDefinition` — not currently exposed as a CLI flag. The `samgraha package` command's own `--profile` flag is a *different*, coarser mechanism: it selects entire domains/documents to include (see [Knowledge Package](knowledge-package.md) for its six values — `minimal`, `development`, `documentation`, `engineering`, `ai-assistant`, `full`). Don't confuse the two: a standard's named section profile (e.g. Feature's `implementation`) and `package --profile` are unrelated today.

### Built-in Help Profiles

The Help standard defines:

- **quickref** — Title + body (for quick reference context)
- **full** — All sections (for comprehensive context)

## Related

- [Knowledge Package](knowledge-package.md)
- [Standards](standards.md)
- [Command: package](../commands/package.md)
