# Legal/Compliance Constraints — Generation Template

> **Domain:** external-context
> **Section:** legal_compliance_constraints (subsection of constraints)
> **Source:** `documentation-standards/08-external-context-standards.md` §Constraints §Legal/Compliance Constraints
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Legal/Compliance Constraints subsection within Constraints for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| (document-owned) | — | Legal constraints must be consistent with Integration Contract data handling |

## Template

```markdown
### Legal / Compliance Constraints

[1 paragraph: overview of licensing restrictions, regulatory obligations, and data handling rules imposed by the external system]

| Constraint | Requirement | Impact | Source |
|-----------|-------------|--------|--------|
| [Name] | [specific legal or compliance requirement] | [what this requires of the repository] | [regulatory body, license, policy] |

[1 paragraph per critical constraint: explanation of the legal/compliance obligation and what the repository must do to comply]
```

## Examples

**Correct:**
> ### Legal / Compliance Constraints
> | Constraint | Requirement | Impact | Source |
> |-----------|-------------|--------|--------|
> | GDPR data retention | No personal data stored beyond 30 days | Repository must implement automatic data purging | EU GDPR Article 17 |
> | Data residency | User data must remain in EU region | Integration must use EU-hosted endpoints only | Platform compliance policy |
> | Audit logging | All data access must be logged for 2 years | Integration must emit audit events for every API call | Platform compliance policy §4.1 |
>
> The 30-day data retention limit means the repository cannot cache user data beyond this window. Automatic purge jobs must run at least daily. The EU residency requirement means the integration configuration must use `eu.api.externalsystem.example` as the base URL.

**Incorrect:**
> We need to follow GDPR and keep our data secure.
> *Why wrong: Too vague. Legal/compliance constraints must specify exact requirements, impacts, and regulatory sources.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** architect
- **Do:** Cite specific regulations, licenses, or policies. State exact requirements with numeric values where applicable. Explain what the repository must do to comply.
- **Don't:** Use vague compliance language. Omit regulatory source. Leave compliance obligations unanalyzed.

**Required subsections:** none (this is a subsection of Constraints)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
