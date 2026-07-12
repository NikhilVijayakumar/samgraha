# Runtime Dependencies — Generation Template

> **Domain:** external-context
> **Section:** runtime_dependencies (subsection of dependencies)
> **Source:** `documentation-standards/08-external-context-standards.md` §Dependencies §Runtime Dependencies
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Runtime Dependencies subsection within Dependencies for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| (document-owned) | — | Runtime dependencies must be consistent with Integration Contract availability requirements |

## Template

```markdown
### Runtime Dependencies

[1 paragraph: overview of transitive runtime requirements the external system needs to function]

| Dependency | Purpose | Criticality | Behavior if Unavailable | Source |
|-----------|---------|-------------|------------------------|--------|
| [Name] | [what it provides to the external system] | [critical/nice-to-have] | [what fails] | [where documented] |

[1 paragraph per critical dependency: explanation of failure modes and impact on the integration]
```

## Examples

**Correct:**
> ### Runtime Dependencies
> | Dependency | Purpose | Criticality | Behavior if Unavailable | Source |
> |-----------|---------|-------------|------------------------|--------|
> | Message broker | Webhook delivery | Critical | Webhooks fail silently — no error returned | Platform documentation §6.2 |
> | TLS certificate | Inbound connection security | Critical | All connections rejected with TLS handshake failure | Platform documentation §2.1 |
> | Redis cache | Session deduplication | Nice-to-have | Duplicate events may be processed — idempotency key required | Platform documentation §6.4 |
>
> The message broker is the most critical transitive dependency. Without it, the external system cannot deliver webhooks, and the repository receives no notification of upstream data changes. The repository must implement polling as a fallback.

**Incorrect:**
> The platform needs a message broker and Redis.
> *Why wrong: Missing purpose, criticality, failure behavior, and source attribution.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** architect
- **Do:** Specify what each dependency provides, its criticality, and what fails if it's unavailable. Cross-reference other External Context documents for documented transitive dependencies.
- **Don't:** List internal project dependencies. Omit failure behavior. Leave criticality ambiguous.

**Required subsections:** none (this is a subsection of Dependencies)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** other External Context documents

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
