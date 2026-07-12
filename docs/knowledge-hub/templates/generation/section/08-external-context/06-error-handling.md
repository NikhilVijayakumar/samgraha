# Error Handling — Generation Template

> **Domain:** external-context
> **Section:** error_handling (subsection of integration_contract)
> **Source:** `documentation-standards/08-external-context-standards.md` §Integration Contract §Error Handling
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Error Handling subsection within Integration Contract for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `informs` | feature_technical / communication_paths | Error handling patterns inform how feature implementations handle external system failures |

## Template

```markdown
### Error Handling

[1 paragraph: how the external system signals errors — HTTP status codes, error response format, retry semantics]

| Error Code / Status | Meaning | Retry | Backoff Strategy |
|---------------------|---------|-------|-----------------|
| [code] | [what it means] | [yes/no] | [strategy] |

[1 paragraph: rate limiting behavior — limits, response headers, retry-after semantics]

[1 paragraph: circuit breaker or fallback behavior if applicable]
```

## Examples

**Correct:**
> ### Error Handling
> The external system returns standard HTTP status codes with a JSON error body containing `code` and `message` fields.
>
> | Error Code / Status | Meaning | Retry | Backoff Strategy |
> |---------------------|---------|-------|-----------------|
> | 400 | Malformed request | No | N/A |
> | 401 | Invalid/expired token | No | Re-authenticate |
> | 429 | Rate limit exceeded | Yes | Respect `Retry-After` header |
> | 500 | Server error | Yes | Exponential backoff, max 3 retries |
> | 503 | Service unavailable | Yes | Exponential backoff, max 5 retries |
>
> Rate limiting is enforced at 100 requests per minute. The `X-RateLimit-Remaining` header indicates remaining quota. When exceeded, the response includes a `Retry-After` header specifying wait time in seconds.

**Incorrect:**
> If the API returns an error, you should handle it gracefully.
> *Why wrong: Missing specific error codes, retry semantics, and rate limiting details.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** engineer
- **Do:** List specific error codes the repository may encounter. Specify retry behavior and backoff strategy for each. Document rate limiting details including response headers.
- **Don't:** Describe error handling implementation code. Omit retry semantics. Leave error codes vague.

**Required subsections:** none (this is a subsection of Integration Contract)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
