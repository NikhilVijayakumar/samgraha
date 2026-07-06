# README Domain Guide

## Purpose

How to write a README — primary repository entry point and documentation navigation.

## Content

### Purpose of a README

The README is the first thing people see when they visit a repository. It introduces the project, explains its purpose, and guides readers to detailed documentation.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| Title | Yes | Repository name and brief description |
| Getting Started | Yes | Installation and basic usage |
| Documentation | Yes | Links to key documentation |

### Writing Tips

- Keep it concise. The README introduces; detailed docs explain.
- Provide clear navigation to all major documentation.
- Include installation and basic usage to get started quickly.
- Describe the repository's structure and role in the ecosystem.
- Don't duplicate other documentation — link to it.

### README as Navigation

The README acts as a documentation index:

```markdown
## Documentation
- [Vision](docs/raw/vision/README.md)
- [Features](docs/raw/feature/)
- [Architecture](docs/raw/architecture/)
- [Engineering](docs/raw/engineering/)
```

## Related

- [Getting Started Guide](../getting-started/installation.md)
- [Concepts: Repository](../concepts/repository.md)
- [Standards Reference: README](../../standards/readme.md)
