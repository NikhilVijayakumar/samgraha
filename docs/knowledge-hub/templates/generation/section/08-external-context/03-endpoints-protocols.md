# Endpoints/Protocols — Generation Template

> **Domain:** external-context
> **Section:** endpoints_protocols (subsection of integration_contract)
> **Source:** `documentation-standards/08-external-context-standards.md` §Integration Contract §Endpoints/Protocols
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Endpoints/Protocols subsection within Integration Contract for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `informs` | engineering / build_standards | Endpoints inform build standards for integration tooling |
| `informs` | feature_technical / communication_paths | Endpoints define communication paths for feature implementations |

## Template

```markdown
### Endpoints / Protocols

[Description of all available endpoints, protocols, and communication patterns exposed by the external system]

| Endpoint / Protocol | Purpose | Method | Required | Authentication | Notes |
|--------------------|---------|--------|----------|---------------|-------|
| [Path or protocol name] | [what it does] | [HTTP method / protocol type] | [yes/no] | [auth type] | [rate limits, versioning, constraints] |

[1 paragraph per critical endpoint: detailed behavior, expected responses, edge cases]

[Authoritative documentation URL]
```

## Examples

**Correct:**
> | Endpoint / Protocol | Purpose | Method | Required | Authentication | Notes |
> |--------------------|---------|--------|----------|---------------|-------|
> | `/v1/transactions` | List transactions | GET | Yes | Bearer token | Paginated, max 100 per page |
> | `/v1/transactions/{id}` | Get transaction detail | GET | Yes | Bearer token | Returns 404 for deleted transactions |
> | `/v1/webhooks` | Register webhook | POST | No | Bearer token | Rate limited to 10/min |
>
> All endpoints use HTTPS. The base URL is `https://api.externalsystem.example`. Authoritative documentation: `https://docs.externalsystem.example/api`.

**Incorrect:**
> The API has various endpoints for different operations.
> *Why wrong: Too vague. Endpoints must be listed with their specific purposes, methods, and authentication requirements.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** engineer
- **Do:** List every endpoint the repository uses or may use. Specify HTTP method, authentication requirement, and any constraints. Include the authoritative documentation URL.
- **Don't:** Include client implementation code. Omit authentication requirements. Leave endpoints vague or undocumented.

**Required subsections:** none (this is a subsection of Integration Contract)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
