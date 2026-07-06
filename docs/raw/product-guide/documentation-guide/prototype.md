# Prototype Domain Guide

## Purpose

How to write Prototype documentation — executable simulations of the application.

## Content

### Purpose of Prototype Docs

Prototype documentation describes executable simulations: mock APIs, data models, and the scope of what the prototype covers.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| Scope | Yes | What the prototype covers |
| Mock APIs | Yes | API contracts being simulated |
| Data Model | Yes | Data structures used |

### Writing Tips

- Clearly define what the prototype covers AND what it doesn't.
- Mock APIs should match the expected real API contract.
- Prototypes are disposable — they are simulations, not production code.
- Reference the Feature Design or Feature Technical docs being validated.

### Disposability

Prototypes are explicitly non-production code. The audit rule `proto-003` checks for this:

```markdown
## Audit Rules for Prototypes
- Must be labeled as prototype/simulation
- Should not be used as production code
```

## Related

- [Feature Design Guide](feature-design.md)
- [Feature Technical Guide](feature-technical.md)
- [Standards Reference: Prototype](../../standards/prototype.md)
