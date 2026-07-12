# Code Standards — Generation Template

> **Domain:** engineering
> **Section:** code_standards
> **Source:** `documentation-standards/07-engineering-standards.md` §Code Standards
> **Relationships:** `audit/deterministic/document/07-engineering-relationships.yaml`

Generate the Code Standards section for an Engineering document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `constrains` | feature-technical / component_responsibilities | Code Standards constrain how components are structured in feature technical designs |

## Template

```markdown
## Code Standards

> [metadata block]

### Style Guide

[1 paragraph: language-specific style guide reference, key rules — connected to engineering principles]

### Linting Configuration

[1 paragraph: linter tool name, configuration file location, key rules enforced]

| Rule Category | Description | Enforcement |
|--------------|-------------|-------------|
| [Category] | [what it checks] | [error/warning] |

### Naming Conventions

[Optional: naming patterns for files, modules, functions, variables]
```

## Examples

**Correct:**
> ### Style Guide
> All modules follow the TypeScript Official Style Guide. Functions are named with verb-noun convention (`getUserProfile`, `validateInput`). Files are named after the module they contain. This convention is enforced through linting and code review.
>
> ### Linting Configuration
> ESLint is configured via `.eslintrc.cjs` at the repository root. All CI builds must pass the linter before merge. Key rules: no unused imports, consistent return types, explicit access modifiers.
>
> | Rule Category | Description | Enforcement |
> |--------------|-------------|-------------|
> | No unused imports | Prevents dead code accumulation | error |
> | Consistent returns | Ensures predictable function behavior | warning |
> | Explicit access modifiers | Improves code clarity | warning |

**Incorrect:**
> We use tabs for indentation and camelCase for variables. Our linter catches some errors.
> *Why wrong: Missing rationale, missing configuration location, no connection to engineering principles, and lacks completeness on scope.*

## Writing Guidance

- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** Reference the specific style guide by name and language. Document linting configuration file location and key rules. Explain non-obvious conventions that deviate from defaults.
- **Don't:** Assume universal knowledge of conventions. Omit configuration file locations. Describe feature-specific patterns or implementation details.

**Required subsections:** Style Guide, Linting Configuration
**Optional subsections:** Naming Conventions
**Required diagrams:** none
**Required cross-references:** Engineering Principles

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
