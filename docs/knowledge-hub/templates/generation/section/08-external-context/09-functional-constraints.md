# Functional Constraints — Generation Template

> **Domain:** external-context
> **Section:** functional_constraints (subsection of constraints)
> **Source:** `documentation-standards/08-external-context-standards.md` §Constraints §Functional Constraints
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Functional Constraints subsection within Constraints for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| (document-owned) | — | Functional constraints must be consistent with Integration Contract capabilities |

## Template

```markdown
### Functional Constraints

[1 paragraph: overview of what the external system allows and disallows at the functional level]

| Constraint | Limitation | Impact | Source |
|-----------|-----------|--------|--------|
| [Name] | [specific restriction] | [what this prevents or requires] | [external source documentation] |

[1 paragraph per critical constraint: explanation of why this limitation exists and how it affects integration design]
```

## Examples

**Correct:**
> ### Functional Constraints
> | Constraint | Limitation | Impact | Source |
> |-----------|-----------|--------|--------|
> | Max payload | 1 MB per request | Large file uploads must be chunked into 1 MB segments | Platform API documentation §3.2 |
> | No bulk delete | Single resource deletion only | Batch cleanup must iterate individual deletions | Platform API documentation §5.1 |
> | Read-only fields | `created_at`, `updated_at` immutable | Cannot backfill or correct timestamps | Platform API documentation §2.4 |
>
> The 1 MB payload limit means the repository must implement chunked upload logic for any data exceeding this threshold. This is a hard platform limit with no override mechanism.

**Incorrect:**
> The API has some limitations.
> *Why wrong: Missing specific constraints, impact analysis, and source attribution.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** architect
- **Do:** List specific functional limitations with numeric values where applicable. Explain the impact on integration design. Cite the external source.
- **Don't:** Omit specific limits. Leave impact analysis out. Use vague language for hard constraints.

**Required subsections:** none (this is a subsection of Constraints)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
