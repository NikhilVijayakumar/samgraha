# System Inheritance

## Purpose

System inheritance lets a knowledge system extend another system's files
and override only what differs — deduplicating shared content across
closely related systems (e.g. `rust_dev`/`fastapi_dev` both extending a
common `base_dev`).

## Inheritance Model

Inheritance is `system.yaml`-declared and operates on the **file tree**,
not on any `capabilities:`/`domains:` block (`system.yaml` has no such
fields — see [Knowledge Systems](systems.md) for its real, five-field
shape):

```
base_dev/
  system.yaml          # abstract: true
  scripts/validate.py
  documentation-standards/*.md

rust_dev/
  system.yaml          # extends: base_dev, drops: [...]
  scripts/validate.py  # overrides base_dev's file at the same path
  documentation-standards/*.md  # only files that differ from base_dev
```

## How Inheritance Works

1. A system declares `extends: <base-system-name>` in `system.yaml`
2. samgraha loads every file from the base system's tree
3. The child system's own files are overlaid on top — same relative path
   overrides the base's file entirely, whole-file, not merged field-by-field
4. If the child declares `drops: [...]`, those paths are removed from the
   merged tree entirely — the child doesn't have them, not "reverted to
   base"
5. The result is one complete tree, handed to registration exactly as if
   it had been authored standalone

## Inheritance Rules

- **Whole-file override** — a child's file at the same relative path
  replaces the base's file entirely (this applies to scripts, templates,
  and `documentation-standards/*.md` alike — anything under the tree)
- **Single parent only** — no diamond inheritance (a system extends at
  most one base)
- **Arbitrary depth** — `A extends B extends C extends D` works; a chain
  can be as short as one hop or several, mixed depths across different
  branches of a system family are fine
- **Circular detection** — samgraha rejects a chain that loops back on itself
- **Drops are cascading** — a dropped path is removed from the merged
  tree, not reset to some other value

## Example

```
samgraha/system/
  base_dev/
    system.yaml              # abstract: true
    scripts/
      validate.py             # shared validation logic
      calculate.py             # shared scoring formula
    documentation-standards/
      01-vision-standards.md
      06-design-standards.md
      ...

  rust_dev/
    system.yaml              # extends: base_dev
                              # drops: ["06-design-standards.md"]
    scripts/
      validate.py             # override: Rust-specific validation
      # calculate.py inherited from base_dev unchanged
    documentation-standards/
      # 06-design-standards.md dropped — Rust doesn't need frontend design
```

## Related

- [Knowledge Systems](systems.md)
- [Capability Scripts](scripts.md)
- [Knowledge System Author Guide](../../../proposal/archive/knowledge-system-author-guide.md)
