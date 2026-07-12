# Build-Time Dependencies — Generation Template

> **Domain:** external-context
> **Section:** build_time_dependencies (subsection of dependencies)
> **Source:** `documentation-standards/08-external-context-standards.md` §Dependencies §Build-Time Dependencies
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Build-Time Dependencies subsection within Dependencies for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| (document-owned) | — | Build-time dependencies must be consistent with Integration Contract versioning |

## Template

```markdown
### Build-Time Dependencies

[1 paragraph: overview of development or build-time prerequisites the external system requires]

| Dependency | Purpose | Criticality | Behavior if Unavailable | Source |
|-----------|---------|-------------|------------------------|--------|
| [Name] | [what it provides during build] | [critical/nice-to-have] | [what fails] | [where documented] |

[1 paragraph per critical dependency: installation instructions and version requirements]
```

## Examples

**Correct:**
> ### Build-Time Dependencies
> | Dependency | Purpose | Criticality | Behavior if Unavailable | Source |
> |-----------|---------|-------------|------------------------|--------|
> | Platform CLI tool | Schema validation during build | Critical | Build fails — schemas not validated | Platform documentation §8.1 |
> | OpenAPI generator | Client SDK generation | Nice-to-have | Manual client code required | Platform documentation §8.3 |
>
> The Platform CLI tool (v2.4+) must be installed in the build environment. It validates API schemas against the platform's published schema definitions. Install via `npm install -g @platform/cli@2.4+`. The build step `validate-schemas` will fail without it.

**Incorrect:**
> We need to install some tools for building.
> *Why wrong: Missing specific tool names, purposes, criticality, and installation guidance.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** engineer
- **Do:** Specify exact tool names and version requirements. Document installation commands where non-trivial. Explain what fails if the dependency is missing.
- **Don't:** List internal project build tools. Omit version requirements. Leave installation ambiguous.

**Required subsections:** none (this is a subsection of Dependencies)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** other External Context documents

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
