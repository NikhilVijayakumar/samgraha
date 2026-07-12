# Performance Constraints — Generation Template

> **Domain:** external-context
> **Section:** performance_constraints (subsection of constraints)
> **Source:** `documentation-standards/08-external-context-standards.md` §Constraints §Performance Constraints
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Performance Constraints subsection within Constraints for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| (document-owned) | — | Performance constraints must be consistent with Integration Contract rate limiting |

## Template

```markdown
### Performance Constraints

[1 paragraph: overview of rate limits, latency requirements, and throughput boundaries imposed by the external system]

| Constraint | Limitation | Impact | Source |
|-----------|-----------|--------|--------|
| [Name] | [specific numeric limit] | [what this requires] | [external source documentation] |

[1 paragraph per critical constraint: explanation of how this affects integration design and what workarounds exist]
```

## Examples

**Correct:**
> ### Performance Constraints
> | Constraint | Limitation | Impact | Source |
> |-----------|-----------|--------|--------|
> | Rate limit | 100 requests/minute | Request throttling and queuing required for batch operations | Platform API documentation §7.1 |
> | Max response size | 10 MB | Pagination required for large result sets | Platform API documentation §3.4 |
> | Timeout | 30 seconds | Long-running operations must use async/polling pattern | Platform API documentation §4.2 |
>
> The 100 requests/minute rate limit means batch operations processing more than 100 items must implement client-side rate limiting with exponential backoff. The `X-RateLimit-Remaining` header should be monitored to avoid hitting the limit.

**Incorrect:**
> The API is rate limited.
> *Why wrong: Missing specific numeric limits, impact analysis, and mitigation strategies.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** architect
- **Do:** State specific numeric limits for rate limits, timeouts, and response sizes. Explain impact on integration design. Document available headers or mechanisms for monitoring limits.
- **Don't:** Omit numeric limits. Leave performance impact unanalyzed. Forget rate limit monitoring guidance.

**Required subsections:** none (this is a subsection of Constraints)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
