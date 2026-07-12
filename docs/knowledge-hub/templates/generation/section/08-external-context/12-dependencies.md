# Dependencies — Generation Template

> **Domain:** external-context
> **Section:** dependencies
> **Source:** `documentation-standards/08-external-context-standards.md` §Dependencies
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Dependencies section for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| (document-owned) | — | Dependencies must be consistent with Integration Contract and Constraints sections |

## Template

```markdown
## Dependencies

[1 paragraph: overview of what the external system itself requires to function — transitive requirements of the dependency]

### Runtime Dependencies

[Transitive runtime requirements — platforms, services, companion systems the external dependency needs]

| Dependency | Purpose | Criticality | Notes |
|-----------|---------|-------------|-------|
| [Name] | [what it provides] | [critical/nice-to-have] | [behavior if unavailable] |

### Build-Time Dependencies

[Development or build-time prerequisites, if any]

| Dependency | Purpose | Criticality | Notes |
|-----------|---------|-------------|-------|
| [Name] | [what it provides] | [critical/nice-to-have] | [behavior if unavailable] |
```

## Examples

**Correct:**
> ### Runtime Dependencies
> | Dependency | Purpose | Criticality | Notes |
> |-----------|---------|-------------|-------|
> | Message broker | Webhook delivery | Critical | Without it, webhook delivery fails silently |
> | TLS certificate | Inbound connection security | Critical | All connections rejected without valid cert |
> | Companion API service | Data enrichment | Nice-to-have | Falls back to cached data if unavailable |
>
> ### Build-Time Dependencies
> | Dependency | Purpose | Criticality | Notes |
> |-----------|---------|-------------|-------|
> | Platform CLI tool | Schema validation | Critical | Build fails without it |
> | OpenAPI generator | Client SDK generation | Nice-to-have | Manual client code if unavailable |
>
> These are transitive requirements of the platform itself, not choices made by this repository.

**Incorrect:**
> This project depends on Express.js for HTTP routing, Mongoose for database access, and Jest for testing.
> *Why wrong: These are internal project dependencies (package.json entries), not transitive dependencies of the external system.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** architect
- **Do:** Distinguish runtime from build-time dependencies explicitly. Note criticality level for each. Cross-reference other External Context documents when a transitive dependency is itself documented.
- **Don't:** List this repository's package.json entries. Include development tooling or test frameworks. Omit companion systems that the integration silently depends on.

**Required subsections:** none
**Optional subsections:** Runtime Dependencies, Build-Time Dependencies
**Required diagrams:** none
**Required cross-references:** other External Context documents for transitive dependencies

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
