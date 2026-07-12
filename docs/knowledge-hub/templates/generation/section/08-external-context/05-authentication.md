# Authentication — Generation Template

> **Domain:** external-context
> **Section:** authentication (subsection of integration_contract)
> **Source:** `documentation-standards/08-external-context-standards.md` §Integration Contract §Authentication
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Authentication subsection within Integration Contract for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `informs` | feature_technical / communication_paths | Authentication mechanisms define how feature implementations establish secure communication |

## Template

```markdown
### Authentication

[1 paragraph: authentication mechanism name and flow overview]

| Aspect | Detail |
|--------|--------|
| Mechanism | [OAuth 2.0, API key, mutual TLS, etc.] |
| Flow | [client credentials, authorization code, etc.] |
| Token type | [Bearer, MAC, etc.] |
| Token lifetime | [expiration duration] |
| Refresh | [supported / not supported] |
| Credential storage | [how credentials should be stored] |

[1 paragraph: credential lifecycle — how tokens are obtained, refreshed, and revoked]

[1 paragraph: security considerations — what must be protected and how]
```

## Examples

**Correct:**
> ### Authentication
> The external system uses OAuth 2.0 client credentials flow for API authentication.
>
> | Aspect | Detail |
> |--------|--------|
> | Mechanism | OAuth 2.0 |
> | Flow | Client credentials |
> | Token type | Bearer |
> | Token lifetime | 3600 seconds |
> | Refresh | Not supported — re-authenticate required |
> | Credential storage | Environment variable `EXTERNAL_API_CLIENT_SECRET` |
>
> Tokens are obtained by POSTing client credentials to the `/oauth/token` endpoint. The token must be included as a `Bearer` token in the `Authorization` header. Authoritative documentation: `https://docs.externalsystem.example/auth`.

**Incorrect:**
> You need to authenticate to use the API. Check the docs.
> *Why wrong: Missing specific mechanism, flow type, token details, and credential lifecycle information.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** engineer
- **Do:** Specify the exact authentication mechanism, flow type, token type, and lifetime. Document credential storage requirements. Include the authoritative documentation URL.
- **Don't:** Describe implementation code. Omit token lifetime or refresh behavior. Leave authentication ambiguous.

**Required subsections:** none (this is a subsection of Integration Contract)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
