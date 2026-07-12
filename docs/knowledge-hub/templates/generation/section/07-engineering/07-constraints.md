# Constraints — Generation Template

> **Domain:** engineering
> **Section:** constraints
> **Source:** `documentation-standards/07-engineering-standards.md` §Constraints
> **Relationships:** `audit/deterministic/document/07-engineering-relationships.yaml`

Generate the Constraints section for an Engineering document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `derives_from` | architecture / constraints | Engineering Constraints must derive from architectural constraints |
| `constrains` | feature-technical / runtime_constraints | Engineering Constraints limit how features can be implemented at runtime |

## Template

```markdown
## Constraints

> [metadata block]

[1–2 paragraphs: overview of non-functional requirements and engineering limitations that bound all implementation decisions, categorized by type]

### Performance Constraints

[Optional: latency, throughput, memory bounds — with specific thresholds and source attribution]

| Constraint | Threshold | Source | Verifiability |
|-----------|-----------|--------|---------------|
| [Constraint name] | [specific value] | [where it comes from] | [how to verify] |

### Security Constraints

[Optional: authentication, encryption, access control requirements — with source attribution]

| Constraint | Requirement | Source | Verifiability |
|-----------|-------------|--------|---------------|
| [Constraint name] | [specific requirement] | [where it comes from] | [how to verify] |

### Compliance Constraints

[Optional: regulatory, organizational policy requirements — with source attribution]
```

## Examples

**Correct:**
> ### Performance Constraints
> | Constraint | Threshold | Source | Verifiability |
> |-----------|-----------|--------|---------------|
> | API response time | ≤200ms at p95 | Architecture Section 4.3 | Load testing |
> | Memory per request | ≤512MB | Architecture Section 4.3 | Profiling |
>
> ### Security Constraints
> | Constraint | Requirement | Source | Verifiability |
> |-----------|-------------|--------|---------------|
> | Transit encryption | TLS 1.2+ | External Context compliance | Certificate inspection |
> | Authentication | OAuth 2.0 client credentials | External Context contract | Token inspection |

**Incorrect:**
> The application should be fast and secure. We follow industry best practices.
> *Why wrong: Not verifiable, no specific thresholds, no source attribution, and no categorization by type — making it impossible to audit.*

## Writing Guidance

- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** Categorize every constraint by type (performance, security, compliance). Cite the source of each constraint. Make every constraint verifiable with specific thresholds.
- **Don't:** State vague requirements like "the application should be fast." Omit source attribution. List constraints without categorization or verification criteria.

**Required subsections:** none
**Optional subsections:** Performance Constraints, Security Constraints, Compliance Constraints
**Required diagrams:** none
**Required cross-references:** Architecture(05), External Context

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
