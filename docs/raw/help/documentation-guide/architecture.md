# Architecture Domain Guide

## Purpose

How to write Architecture documentation — system structural organization.

## Content

### Purpose of Architecture Docs

Architecture documentation describes how the system is organized: components, responsibilities, boundaries, communication, data flow, and security.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| System Overview | Yes | High-level system description |
| Component Model | Yes | Components and their responsibilities |
| Communication | Yes | How components communicate |
| Data Flow | Yes | How data moves through the system |
| Security Considerations | Yes | Security architecture |

### Architecture as a Collection

Architecture is typically split into multiple focused documents under `docs/raw/architecture/`:

```
architecture/
├── system-overview.md
├── component-model.md
├── communication.md
├── data-flow.md
└── security.md
```

### Writing Tips

- Each architecture document should have one primary responsibility.
- Describe responsibilities before technologies.
- Define boundaries explicitly.
- Architecture is not implementation — avoid code, frameworks, libraries.
- Reference external architectures rather than duplicating them.

## Related

- [Documentation Guide Overview](overview.md)
- [Feature Technical Guide](feature-technical.md)
- [Standards Reference: Architecture](../../standards/architecture.md)
