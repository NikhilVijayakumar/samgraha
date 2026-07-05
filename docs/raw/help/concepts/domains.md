# Domains

## Purpose

What domains are and how the domain taxonomy works.

## Content

A **domain** is a category of documentation that corresponds to a standard. Each domain has:

- A unique name (e.g., `feature`, `architecture`, `help`)
- A corresponding standard definition
- A section schema (required and optional sections)
- Audit rules specific to that domain

### Domain Taxonomy

Domains are organized into three groups:

**Product Knowledge (user-owned):**
- readme, vision, philosophy, architecture
- feature, feature-design, feature-technical
- design, engineering, external-context, prototype

**Standards Knowledge (product-shipped):**
- standards — Reference docs for all built-in standards

**Help Knowledge (product-shipped):**
- help — Product documentation for Samgraha itself

### How Domains Are Used

- **Compilation**: `--domain feature` (repeatable) compiles only the given domain(s).
- **Search**: `--domain feature` returns only feature sections.
- **Audit**: Rules are dispatched by domain.
- **Filtering**: `domain_exclusion` in `samgraha.toml` removes a domain from the effective set (`domain` minus `domain_exclusion`) without deleting it from the declared `domain` list.

### Reserved Domains

`help` and `standards` are reserved for the built-in knowledge stores (`help.db`/`standards.db`) — compilation refuses to let a repo declare `domain = ["help"]` (or `"standards"`) unless that same domain is also listed in `domain_exclusion` (so it still shows up in `samgraha info`, but isn't compiled from the repo's own documents).

## Related

- [Standards](standards.md)
- [Configuration: documentation](../configuration/documentation.md)
- [Search Filtering](../search-guide/filtering.md)
