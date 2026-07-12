# Deterministic Section Report — Architecture

**Document:** {{ document_path }}
**Standard:** `documentation-standards/05-architecture-standards.md`
**Audit Date:** {{ created_at }}

---

## 1. Purpose — `section/05-architecture/01-purpose.yaml`

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-purpose-001 | Purpose section exists | {{ results['arch-sec-purpose-001'].status }} | error |
| arch-sec-purpose-002 | States architectural intent | {{ results['arch-sec-purpose-002'].status }} | error |
| arch-sec-purpose-003 | Technology-independent | {{ results['arch-sec-purpose-003'].status }} | error |
| arch-sec-purpose-004 | Scope boundaries defined | {{ results['arch-sec-purpose-004'].status }} | warning |

## 2. System Overview — `02-system_overview.yaml`

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-so-001 | Section exists | {{ results['arch-sec-so-001'].status }} | error |
| arch-sec-so-002 | Has Overview and Diagram subsections | {{ results['arch-sec-so-002'].status }} | error |
| arch-sec-so-003 | Describes system purpose | {{ results['arch-sec-so-003'].status }} | error |
| arch-sec-so-004 | Technology-independent | {{ results['arch-sec-so-004'].status }} | error |
| arch-sec-so-005 | Includes a diagram | {{ results['arch-sec-so-005'].status }} | warning |

## 3. Component Model — `03-component_model.yaml`

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-cm-001 | Section exists | {{ results['arch-sec-cm-001'].status }} | error |
| arch-sec-cm-002 | Has Components and Component Diagram subsections | {{ results['arch-sec-cm-002'].status }} | error |
| arch-sec-cm-003 | Every component has a Responsibility field | {{ results['arch-sec-cm-003'].status }} | error |
| arch-sec-cm-004 | Every component has an Ownership field | {{ results['arch-sec-cm-004'].status }} | error |
| arch-sec-cm-005 | No overlapping responsibilities | {{ results['arch-sec-cm-005'].status }} | error |
| arch-sec-cm-006 | Technology-independent | {{ results['arch-sec-cm-006'].status }} | error |
| arch-sec-cm-007 | Component relationship diagram present | {{ results['arch-sec-cm-007'].status }} | warning |

## 4. Communication Paths — `04-communication_paths.yaml`

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-comm-001 | Section exists | {{ results['arch-sec-comm-001'].status }} | error |
| arch-sec-comm-002 | Has Communication Paths and Communication Diagram subsections | {{ results['arch-sec-comm-002'].status }} | error |
| arch-sec-comm-003 | All Component Model interactions have a documented path | {{ results['arch-sec-comm-003'].status }} | error |
| arch-sec-comm-004 | Every path has a classified pattern (sync/async/event-driven/queue-based) | {{ results['arch-sec-comm-004'].status }} | error |
| arch-sec-comm-005 | Technology-independent | {{ results['arch-sec-comm-005'].status }} | error |
| arch-sec-comm-006 | Sequence diagram present | {{ results['arch-sec-comm-006'].status }} | warning |

## 5. Data Flow — `05-data_flow.yaml`

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-df-001 | Section exists | {{ results['arch-sec-df-001'].status }} | error |
| arch-sec-df-002 | Has Data Paths and Data Flow Diagram subsections | {{ results['arch-sec-df-002'].status }} | error |
| arch-sec-df-003 | Data ownership boundaries defined per path | {{ results['arch-sec-df-003'].status }} | error |
| arch-sec-df-004 | Technology-independent | {{ results['arch-sec-df-004'].status }} | error |
| arch-sec-df-005 | Data flow diagram present | {{ results['arch-sec-df-005'].status }} | warning |

## 6. Security Considerations — `06-security_considerations.yaml`

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-sec-001 | Section exists | {{ results['arch-sec-sec-001'].status }} | error |
| arch-sec-sec-002 | Has Trust Boundaries and Threat Model subsections | {{ results['arch-sec-sec-002'].status }} | error |
| arch-sec-sec-003 | Trust boundaries defined for all external interfaces | {{ results['arch-sec-sec-003'].status }} | error |
| arch-sec-sec-004 | Every threat has a mitigation | {{ results['arch-sec-sec-004'].status }} | error |
| arch-sec-sec-005 | Technology-independent | {{ results['arch-sec-sec-005'].status }} | error |
| arch-sec-sec-006 | Trust boundary diagram present | {{ results['arch-sec-sec-006'].status }} | warning |

## 7. Rationale — `07-rationale.yaml` (optional)

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-rat-001 | Section present | {{ results['arch-sec-rat-001'].status }} | warning |
| arch-sec-rat-002 | Each decision has alternatives documented | {{ results['arch-sec-rat-002'].status }} | warning |
| arch-sec-rat-003 | Each decision has a rejection reason | {{ results['arch-sec-rat-003'].status }} | warning |
| arch-sec-rat-004 | References architectural goals | {{ results['arch-sec-rat-004'].status }} | warning |

## 8. Constraints — `08-constraints.yaml` (optional)

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-con-001 | Section present | {{ results['arch-sec-con-001'].status }} | warning |
| arch-sec-con-002 | Hard constraints have source attribution | {{ results['arch-sec-con-002'].status }} | error |
| arch-sec-con-003 | Hard and soft constraints distinguished | {{ results['arch-sec-con-003'].status }} | warning |
| arch-sec-con-004 | Technology-independent | {{ results['arch-sec-con-004'].status }} | error |

## 9. Traceability — `09-traceability.yaml` (optional)

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-trace-001 | Section present | {{ results['arch-sec-trace-001'].status }} | warning |
| arch-sec-trace-002 | Derivation chain documented | {{ results['arch-sec-trace-002'].status }} | warning |
| arch-sec-trace-003 | Non-contradiction rule stated | {{ results['arch-sec-trace-003'].status }} | warning |
| arch-sec-trace-004 | Downstream standards listed | {{ results['arch-sec-trace-004'].status }} | warning |

## 10. Operational Readiness — `10-operational_readiness.yaml` (optional, not in Required Sections table)

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-or-001 | Section present | {{ results['arch-sec-or-001'].status }} | suggestion |
| arch-sec-or-002 | Covers monitoring, alerting, deployment, rollback | {{ results['arch-sec-or-002'].status }} | warning |

## 11. Observability — `11-observability.yaml` (optional, not in Required Sections table)

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-sec-obs-001 | Section present | {{ results['arch-sec-obs-001'].status }} | suggestion |
| arch-sec-obs-002 | Covers logging, metrics, tracing | {{ results['arch-sec-obs-002'].status }} | warning |

---

## Failures

{% if failed_rules | length > 0 %}
| Section | Rule | Message | Evidence |
|---|---|---|---|
{% for r in failed_rules -%}
| {{ r.section_type }} | {{ r.id }} | {{ r.message }} | {{ r.evidence | default('—') }} |
{% endfor %}
{% else %}
No failures.
{% endif %}

---

## Score

**Deterministic Section Score:** {{ score }} / 100

Rolled up from all 11 sections above. System Overview, Component Model, Communication Paths, Data Flow, and Security Considerations are required (missing them fails mandatory checks hard); Purpose, Rationale, Constraints, Traceability, Operational Readiness, and Observability are optional per `documentation-standards/05-architecture-standards.md`'s Required Sections table (the last two aren't in that table at all — recommended-only).

---

## Metadata

| Field | Value |
|---|---|
| Domain | architecture |
| Standard | documentation-standards |
| Section Rule Files | `audit/deterministic/section/05-architecture/*.yaml` |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
