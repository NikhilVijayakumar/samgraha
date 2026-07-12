# Constraints — Generation Template

> **Domain:** external-context
> **Section:** constraints
> **Source:** `documentation-standards/08-external-context-standards.md` §Constraints
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Constraints section for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| (document-owned) | — | Constraints must be consistent with Integration Contract and not contradict Dependencies |

## Template

```markdown
## Constraints

[1 paragraph: overview of constraints the external system imposes on the repository — what limitations exist and why they matter]

### Functional Constraints

[Limitations on what the integration can or cannot do]

| Constraint | Limitation | Impact | Source |
|-----------|-----------|--------|--------|
| [Name] | [specific restriction] | [what this prevents] | [external source] |

### Performance Constraints

[Rate limits, latency requirements, throughput boundaries]

| Constraint | Limitation | Impact | Source |
|-----------|-----------|--------|--------|
| [Name] | [specific numeric limit] | [what this prevents] | [external source] |

### Legal / Compliance Constraints

[Licensing restrictions, regulatory obligations, data handling rules]

| Constraint | Requirement | Impact | Source |
|-----------|-------------|--------|--------|
| [Name] | [specific requirement] | [what this prevents] | [external source] |
```

## Examples

**Correct:**
> ### Functional Constraints
> | Constraint | Limitation | Impact | Source |
> |-----------|-----------|--------|--------|
> | Max payload | 1 MB per request | Large file uploads must be chunked | Platform service limits |
> | Max concurrent connections | 50 per client | Connection pooling required for high-throughput scenarios | Platform service limits |
>
> ### Performance Constraints
> | Constraint | Limitation | Impact | Source |
> |-----------|-----------|--------|--------|
> | Rate limit | 10 requests/sec, burst 20 | Request throttling needed for batch operations | Platform service limits |

**Incorrect:**
> We decided to use connection pooling because our application needs high throughput. Our team prefers TypeScript over JavaScript for type safety.
> *Why wrong: These are internal design decisions, not constraints imposed by the external system.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Categorize every constraint by type. Cite the external source for each. State limits as specific numeric values where possible.
- **Don't:** List internal project decisions as constraints. Use vague qualifiers for hard limits. Omit constraints that affect data handling or compliance.

**Required subsections:** none
**Optional subsections:** Functional Constraints, Performance Constraints, Legal / Compliance Constraints
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
