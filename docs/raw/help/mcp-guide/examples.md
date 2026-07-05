# MCP Workflow Examples

## Purpose

Common MCP usage patterns and workflows with AI assistants.

## Content

### Example 1: Implementing a Feature

```
Developer: "Implement the password reset feature according to our spec."

AI Assistant (via MCP):
1. Searches Samgraha for "password reset" in feature domain
2. Gets the feature specification with requirements
3. Searches feature-technical for implementation guidance
4. Searches engineering for coding standards
5. Generates code following all documented constraints
```

### Example 2: Architecture Review

```
Developer: "Review my proposed architecture for the new payment service."

AI Assistant:
1. Searches Samgraha architecture docs for system constraints
2. Gets current architecture component model
3. Searches external-context for payment gateway integration
4. Finds relevant design principles from philosophy docs
5. Provides review feedback aligned with documented principles
```

### Example 3: Onboarding

```
Developer: "How do I set up the development environment?"

AI Assistant:
1. Searches Samgraha's built-in help docs (`domain: "help"` — "getting-started" is a folder under help, not a domain name)
2. Finds installation and initialization guides
3. Searches engineering for build and test standards
4. Provides step-by-step setup instructions
```

## Related

- [Tools Reference](tools.md)
- [MCP Overview](overview.md)
- [Search Guide](../search-guide/overview.md)
