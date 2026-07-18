# Knowledge Systems

## Purpose

A knowledge system is an external package that provides domain-specific validation, calculation, reporting, scaffolding, and plan generation logic to samgraha. Instead of hardcoding every audit check in Rust, samgraha delegates to system-provided scripts via capability dispatch.

## What Is a Knowledge System?

A knowledge system is a directory containing:

- **`system.yaml`** — declares the system's identity and, optionally, inheritance (`extends`/`drops`/`abstract`) — nothing else
- **`scripts/`** — executable files (Python, Node, shell, etc.), one per capability, discovered by filename (`validate.py`, `calculate.py`, `report.py`, `scaffold.py`, `plan_generation.py`, `init.py`)
- **`documentation-standards/`** — optional, defines audit rules for the system's domains
- **`templates/`** — optional, templates the system's own scripts load and render — samgraha never reads them directly

## How Systems Work

1. A knowledge system is registered with samgraha via `register_standard`
2. Samgraha discovers the system's scripts by **filesystem probing** — a
   `scripts/` directory with files named after each capability — not
   anything declared inside `system.yaml`
3. When an audit or fix operation targets a domain the system owns,
   samgraha invokes the system's `validate` script. If no script exists
   for that capability, the call fails with a clear error — there is no
   fallback
4. The script receives its input via a `--in` JSON file and writes its
   result to `--out`; samgraha stores whatever comes back without
   interpreting it

## Script Discovery Order

Multiple sources may provide a script for the same capability. Discovery
checks four tiers, in order, first match wins:

1. **Override** — `check_overrides` in `samgraha.toml`
2. **RepoScript** — `{repo}/scripts/<capability>.<ext>`
3. **LocalScript** — `.samgraha/scripts/<capability>.<ext>` (synced copy)
4. **GlobalScript** — scripts shipped next to the MCP binary

There is no fifth, built-in-fallback tier — if none of the four tiers has
a script, the capability fails clearly rather than running built-in Rust
logic.

## `system.yaml` Structure

The entire, real structure — five possible fields, no more:

```yaml
name: rust_dev
description: "Documentation standards for Rust projects"

# Optional — inherit files from another system
extends: base_dev

# Optional — files to drop from the inherited tree (requires extends)
drops:
  - "06-design-standards.md"
  - "09-feature-design-standards.md"

# Optional — marks this system as a base only, never registrable standalone
abstract: false
```
There is no `capabilities:` block (scripts are discovered by filename, not
declared here), no `domains:` list, and no inline `init:` block. A
system's supported use cases and phases come from **running** its `init`
script, not from anything written in `system.yaml` — see
[Workflows](workflows.md).

## Related

- [Capability Scripts](scripts.md)
- [System Inheritance](inheritance.md)
- [Workflows](workflows.md)
- [Knowledge System Author Guide](../../../knowledge-system-author-guide.md)
