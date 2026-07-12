# Integration Contract — Generation Template

> **Domain:** external-context
> **Section:** integration_contract
> **Source:** `documentation-standards/08-external-context-standards.md` §Integration Contract
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Integration Contract section for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `informs` | engineering / build_standards | Integration Contract informs how engineering build standards handle this dependency |
| `informs` | feature_technical / communication_paths | Integration Contract defines communication paths for feature technical designs |

## Template

```markdown
## Integration Contract

[Integration surface overview — what the external system exposes and how the repository interacts with it]

### Endpoints / Protocols

[Description of available endpoints, protocols, and communication patterns]

| Endpoint / Protocol | Purpose | Method | Authentication |
|--------------------|---------|--------|---------------|
| [Name] | [what it does] | [method/type] | [auth requirement] |

### Data Formats

[Request and response formats, encoding, content types]

| Direction | Format | Content Type | Notes |
|-----------|--------|-------------|-------|
| Request | [format] | [content type] | [constraints] |
| Response | [format] | [content type] | [constraints] |

### Authentication

[Authentication mechanisms, credentials handling, token lifecycle]

### Error Handling

[Error codes, retry policies, rate limiting behavior]

### Versioning

[Version compatibility, deprecation policy, migration guidance]
```

## Examples

**Correct:**
> The external system exposes a REST API over HTTPS. Authentication uses OAuth 2.0 client credentials flow.
>
> ### Endpoints / Protocols
> | Endpoint / Protocol | Purpose | Method | Authentication |
> |--------------------|---------|--------|---------------|
> | `/v1/data` | Submit data | POST | Bearer token |
> | `/v1/data/{id}` | Retrieve data | GET | Bearer token |
>
> ### Authentication
> OAuth 2.0 client credentials flow. Tokens expire after 3600 seconds. Refresh tokens are not supported — clients must re-authenticate. Authoritative documentation: `https://docs.externalsystem.example/api`.

**Incorrect:**
> Here is the code we use to call the API:
> ```python
> import requests
> resp = requests.post("https://api.externalsystem.example/v1/data", ...)
> ```
> *Why wrong: Includes implementation code rather than describing the contract. The Integration Contract defines what the external system exposes, not how the repository calls it.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** mixed
- **Audience:** engineer
- **Do:** Describe the external system's contract surface in implementation-neutral terms. Always include the authoritative documentation URL. Distinguish required endpoints from optional ones.
- **Don't:** Paste code snippets or client implementations. Document internal request/response transformation logic. Omit authentication mechanism details.

**Required subsections:** Endpoints / Protocols, Authentication
**Optional subsections:** Data Formats, Error Handling, Versioning
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
