# External Context Domain Guide

## Purpose

How to write External Context documentation — knowledge dependencies on external systems.

## Content

### Purpose of External Context Docs

External Context documents capture the integration contract with external systems: APIs, services, libraries, or platforms that the project depends on.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| Purpose | Yes | Why this external dependency exists |
| Integration Contract | Yes | How integration works |

### When to Write an External Context Doc

- The project depends on a specific external API or service.
- The integration has constraints (rate limits, auth requirements, etc.).
- Multiple features will reference the same external dependency.

### Writing Tips

- Don't duplicate the external system's documentation — reference it.
- Focus on the integration contract between your system and the external one.
- Document constraints (rate limits, latency, availability, auth).
- Link to the authoritative external documentation.

## Related

- [Feature Guide](feature.md)
- [Feature Technical Guide](feature-technical.md)
- [Standards Reference: External Context](../../standards/external-context.md)
