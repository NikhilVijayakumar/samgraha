# Tutorial: Custom Standard

## Purpose

Step-by-step: select which built-in standards a repository uses, and what "custom" standards currently means in Samgraha.

## Content

### What's Actually Configurable Today

Samgraha ships a fixed catalog of built-in standards (`readme`, `vision`, `philosophy`, `architecture`, `feature`, `feature-design`, `feature-technical`, `design`, `engineering`, `external-context`, `prototype`, `help`, `standards`, ...). A repository doesn't declare wholly new standards — it selects a subset of the built-in catalog via `samgraha.toml`.

There is a `StandardLoader` in the `standards` crate (`discover_from_path` / `load_from_declarations`) capable of reading `StandardDefinition` JSON/TOML files, but nothing in the compile pipeline currently calls it — it isn't wired up yet. Don't expect hand-written standard definitions to be picked up automatically.

### Step 1: Check `samgraha init`'s Default

`samgraha init` writes every built-in standard's domain into `[repository.documentation] domain` by default:

```toml
[repository.documentation]
domain = ["readme", "vision", "architecture", "feature", "..."]
domain_exclusion = []
```

### Step 2: Exclude Standards You Don't Use

Rather than deleting entries from `domain`, list unused ones in `domain_exclusion` — this keeps the full catalog visible in the toml while excluding it from compilation and `samgraha info` output:

```toml
[repository.documentation]
domain_exclusion = ["prototype", "external-context"]
```

### Step 3: Write Documents

Create documents under `docs/raw/<domain>/` for any domain still enabled, following that standard's required sections (see [Standards Reference](../../standards/overview.md)).

### Step 4: Compile

```bash
samgraha compile --domain feature
```

### What You Learned

- Samgraha's standards are a fixed built-in catalog, not user-declared standards
- How to opt a repository out of a built-in standard via `domain_exclusion`
- That runtime-loaded custom standard definitions exist as scaffolding (`StandardLoader`) but aren't connected to compilation yet

## Related

- [Concepts: Standards](../concepts/standards.md)
- [Standards Reference: Overview](../../standards/overview.md)
- [Concepts: Domains](../concepts/domains.md)
