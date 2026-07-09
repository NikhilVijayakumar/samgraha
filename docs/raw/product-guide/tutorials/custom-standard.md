# Tutorial: Custom Standard

## Purpose

Step-by-step: select which built-in standards a repository uses, and how to override or extend them with your own `StandardDefinition`.

## Content

### What's Configurable Today

Samgraha ships a fixed catalog of built-in standards (`readme`, `vision`, `philosophy`, `architecture`, `feature`, `feature-design`, `feature-technical`, `design`, `engineering`, `external-context`, `prototype`, `help`, `standards`, ...). A repository selects a subset of the built-in catalog via `samgraha.toml`, and can additionally supply its own `StandardDefinition` JSON/TOML files to override a built-in's rules or add net-new ones.

### Step 0 (Optional): Override or Add a Standard

Drop one `StandardDefinition` JSON or TOML file per standard under `.samgraha/standards/` at the repository root. Every compile and audit run picks these up automatically (`StandardRegistry::with_builtins_and_overrides`) — a file whose `id`+`version` matches a built-in **replaces** it; anything else is **added** alongside the built-in catalog.

```json
// .samgraha/standards/architecture.json
{
  "id": "architecture",
  "name": "Architecture Standard",
  "version": "1.0.0",
  "domain": "architecture",
  "description": "Structural organization of the system.",
  "required_sections": [
    { "canonical_name": "System Overview", "semantic_type": "system_overview", "aliases": ["Overview"], "required": true, "description": "High-level system description" }
  ],
  "prohibited_content": ["TODO", "FIXME"],
  "relationships": [],
  "audit_rules": [
    { "id": "A1", "name": "Corpus Exists", "description": "At least one architecture document must exist", "severity": "error", "check_type": "corpus_exists", "scope": "" }
  ],
  "profiles": []
}
```

Fields mirror `StandardDefinition` in `crates/schemas/src/standard.rs`. Nothing else needs configuring — a missing `.samgraha/standards/` directory is fine and behaves exactly like the plain built-in catalog.

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

- Samgraha ships a fixed built-in catalog, and a repository can override or extend it via `.samgraha/standards/*.json|.toml`
- How to opt a repository out of a built-in standard via `domain_exclusion`
- Repo-supplied standards win over a built-in of the same `id`+`version`; anything else is added alongside

## Related

- [Concepts: Standards](../concepts/standards.md)
- [Standards Reference: Overview](../../standards/overview.md)
- [Concepts: Domains](../concepts/domains.md)
