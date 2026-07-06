# Feature Technical Domain Guide

## Purpose

How to write Feature Technical documentation — architectural realization of a single feature.

## Content

### Purpose of Feature Technical Docs

Feature Technical documentation describes how a feature is realized within the architecture. It connects feature specs to implementation.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| Purpose | Yes | Why this technical design exists |
| Participating Components | Yes | Which components are involved |
| Component Interactions | Yes | How components interact |
| Data Ownership | Yes | Who owns what data |

### Relationship to Other Docs

```
Feature → Feature Technical → Implementation
          ↑
Architecture (constrains)
```

Feature Technical is where architecture meets feature specification. It describes the "how" at the architectural level, not the code level.

### Writing Tips

- Reference the Feature spec but don't duplicate it.
- Reference the Architecture decisions that constrain this design.
- Be specific about component responsibilities and interaction patterns.
- Address security, performance, and failure handling.

## Related

- [Feature Guide](feature.md)
- [Architecture Guide](architecture.md)
- [Engineering Guide](engineering.md)
