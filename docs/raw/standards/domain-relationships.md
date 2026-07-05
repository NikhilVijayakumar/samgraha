# Domain Relationships

## Purpose

Cross-domain dependency map — every `relationship` declared on a `StandardDefinition` in `crates/standards/src/builtin.rs`, in one table. Use it to see what a domain feeds into, or what feeds into it, without opening every standard document.

## Sections

### All Declared Relationships

| From | Relationship | To |
|------|--------------|-----|
| readme | references | vision |
| vision | derives | feature |
| vision | inspires | philosophy |
| philosophy | guides | architecture |
| philosophy | guides | design |
| architecture | constrains | feature-technical |
| architecture | guides | engineering |
| feature | designs | feature-design |
| feature | implements | feature-technical |
| feature-design | implements | feature |
| design | applies-to | feature-design |
| feature-technical | realizes | feature |
| engineering | implements | architecture |
| engineering | guides | feature-technical |
| external-context | informs | feature-technical |
| external-context | informs | engineering |
| prototype | validates | feature-design |
| prototype | validates | feature-technical |

`help` and `standards` declare no relationships — both are flat, built-in content with no structured cross-domain graph. Cross-references between help topics or standards documents are handled by the freeform `Related` section in each document rather than a machine-readable relationship, since neither participates in the traceability chain the other 11 domains form (Vision → Feature → Feature Design/Technical → Architecture → Engineering → Implementation).

### Traceability Chain (the 11 non-built-in domains)

```text
Vision
  │
  ├── Philosophy ── guides ──> Architecture, Design
  │
  └── Feature ──┬── designs ──────> Feature Design ──┐
                └── implements ───> Feature Technical ┤
                                                       ↓
                                                 Architecture ── constrains ──> Feature Technical
                                                       │
                                                       ↓
                                                 Engineering ── implements ──> Architecture
                                                       ↓
                                                 Implementation

External Context and Prototype inform/validate the chain from the side rather than sitting on the main spine.
README references Vision but sits outside the chain entirely (it's a navigation document, not an engineering artifact).
```

## Usage

Check this before adding a new `relationship(...)` entry to a `StandardDefinition` — if the relationship you're about to add isn't in this table, add it here too, otherwise this document drifts out of sync with the code exactly the way `overview.md` warns about.

## Related

- [index.md](index.md) — table of contents for the standards collection
- [overview.md](overview.md) — what a `relationship` is and how it's declared
- [best-practices.md](best-practices.md) — cross-standard usage patterns
