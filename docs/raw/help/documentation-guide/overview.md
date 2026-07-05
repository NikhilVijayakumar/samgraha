# Documentation Guide Overview

## Purpose

How to write good Samgraha documentation — principles and patterns that apply across all domains.

## Content

### Principles

1. **One document, one concern** — Each document describes exactly one thing. Feature documents describe exactly one feature. Architecture documents describe exactly one architectural concern.

2. **Technology independence** — Keep feature and vision docs free of implementation details. Technology choices belong in engineering docs.

3. **Traceability** — Link to upstream and downstream documents. Every document should be traceable to the Vision.

4. **Completeness** — Include all required sections for your domain. Missing sections produce audit warnings.

5. **Consistency** — Use the same heading structure across all documents in a domain. The standard defines the canonical heading names.

### File Organization

Documents live under `docs/raw/<domain>/`. Each domain has its own subdirectory:

```
docs/raw/
├── vision/
├── philosophy/
├── architecture/
├── feature/
├── feature-design/
├── feature-technical/
├── design/
├── engineering/
├── external-context/
├── prototype/
├── readme.md
└── help/          (built-in)
└── standards/     (built-in)
```

### Quick Reference by Domain

| Domain | Required Sections | Prohibited Content |
|--------|------------------|-------------------|
| readme | Title, Getting Started, Documentation | Detailed API reference |
| vision | Purpose, Vision, Problem, Solution, Target Audience | Implementation |
| feature | Purpose, Functional Requirements, Acceptance Criteria | Implementation, architecture |
| architecture | System Overview, Component Model, Communication, Data Flow, Security | Implementation |

## Related

- [Concepts: Standards](../concepts/standards.md)
- [Concepts: Domains](../concepts/domains.md)
- [Standards Reference](../../standards/overview.md)
