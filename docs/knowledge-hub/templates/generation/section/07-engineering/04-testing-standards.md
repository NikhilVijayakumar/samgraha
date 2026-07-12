# Testing Standards — Generation Template

> **Domain:** engineering
> **Section:** testing_standards
> **Source:** `documentation-standards/07-engineering-standards.md` §Testing Standards
> **Relationships:** `audit/deterministic/document/07-engineering-relationships.yaml`

Generate the Testing Standards section for an Engineering document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `derives_from` | architecture / observability | Testing Standards must derive from architecture's observability requirements |
| `derives_from` | philosophy / guiding_principles | Testing Standards must align with guiding philosophy |

## Template

```markdown
## Testing Standards

> [metadata block]

### Test Types

[1 paragraph per test type: unit, integration, e2e — purpose, scope, tooling]

| Test Type | Purpose | Scope | Tooling | Execution |
|-----------|---------|-------|---------|-----------|
| [Type name] | [what it validates] | [what it covers] | [test runner, assertions] | [when it runs] |

### Coverage Expectations

[1 paragraph: coverage targets, what is measured, thresholds]

| Metric | Target | Scope | Rationale |
|--------|--------|-------|-----------|
| [Metric name] | [threshold] | [what it covers] | [why this threshold] |

### Test Tooling

[Optional: test runner, assertion libraries, mocking frameworks — with configuration rationale]
```

## Examples

**Correct:**
> ### Test Types
> | Test Type | Purpose | Scope | Tooling | Execution |
> |-----------|---------|-------|---------|-----------|
> | Unit | Validate individual modules in isolation | Single module | Vitest, ts-mockito | Every commit |
> | Integration | Verify module interactions against contracts | Cross-module interfaces | Vitest, supertest | Every commit |
> | E2E | Exercise critical user journeys | Full system | Playwright | Nightly + pre-release |
>
> ### Coverage Expectations
> | Metric | Target | Scope | Rationale |
> |--------|--------|-------|-----------|
> | Line coverage (unit) | 80% | Core modules | Balances thoroughness with maintenance cost |
> | Interface coverage (integration) | 100% | Cross-module | Every interface must be verified |

**Incorrect:**
> We have unit tests and some integration tests. Coverage is pretty good.
> *Why wrong: Vague, no specific test type definitions, no coverage targets, and no rationale for the testing strategy.*

## Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** mixed
- **Audience:** engineer
- **Do:** Define each test type with its purpose, scope, and tooling. Set measurable coverage targets with specific thresholds. Document test tooling configuration and rationale.
- **Don't:** Use vague claims like "coverage is pretty good." Omit specific coverage thresholds. Describe feature-specific test cases rather than repository-wide test strategy.

**Required subsections:** Test Types, Coverage Expectations
**Optional subsections:** Test Tooling
**Required diagrams:** Test strategy flowchart
**Required cross-references:** Architecture(05), Build Standards

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
