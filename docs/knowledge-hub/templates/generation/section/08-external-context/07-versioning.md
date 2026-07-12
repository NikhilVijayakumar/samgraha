# Versioning — Generation Template

> **Domain:** external-context
> **Section:** versioning (subsection of integration_contract)
> **Source:** `documentation-standards/08-external-context-standards.md` §Integration Contract §Versioning
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Versioning subsection within Integration Contract for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `informs` | feature_technical / communication_paths | Versioning strategy informs how feature implementations handle API version transitions |

## Template

```markdown
### Versioning

[1 paragraph: how the external system handles API versioning — URL path, header, or query parameter]

| Aspect | Detail |
|--------|--------|
| Versioning scheme | [URL path, header, query parameter] |
| Current version | [version identifier] |
| Deprecation policy | [how deprecation is communicated] |
| Migration guidance | [how to upgrade between versions] |

[1 paragraph: compatibility guarantees — breaking vs non-breaking changes, advance notice policy]
```

## Examples

**Correct:**
> ### Versioning
> The external system uses URL path versioning (`/v1/`, `/v2/`).
>
> | Aspect | Detail |
> |--------|--------|
> | Versioning scheme | URL path prefix |
> | Current version | v1 |
> | Deprecation policy | 12-month notice via email and changelog |
> | Migration guidance | Migration guide published at `/docs/migration/v1-to-v2` |
>
> Breaking changes only occur in new major versions. Minor versions may add fields but never remove or rename them. The current version (v1) is stable with no planned deprecation within 18 months.

**Incorrect:**
> The API has versions.
> *Why wrong: Missing versioning scheme, current version, deprecation policy, and migration guidance.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** engineer
- **Do:** Specify the versioning scheme, current version, and deprecation policy. Document what constitutes a breaking change. Include migration guidance if available.
- **Don't:** Omit the current version. Leave deprecation policy ambiguous. Forget to mention compatibility guarantees.

**Required subsections:** none (this is a subsection of Integration Contract)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
