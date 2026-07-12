# Semantic Section Report — Architecture

**Document:** {{ document_path }}
**Standard:** `documentation-standards/05-architecture-standards.md`
**Audit Date:** {{ created_at }}

---

## 1. Purpose — `section/05-architecture/01-purpose.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['purpose.C1'].passed_display }} | Architectural purpose is clearly stated with scope boundaries |
| C2 | mandatory (0 or 30) | {{ results['purpose.C2'].passed_display }} | Primary architectural goals and their priorities are defined |
| C3 | recommended (0 or 30) | {{ results['purpose.C3'].passed_display }} | Purpose is consistent with requirements and downstream sections |

## 2. System Overview — `02-system_overview.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['system_overview.C1'].passed_display }} | System purpose and architectural style are clearly described |
| C2 | mandatory (0 or 30) | {{ results['system_overview.C2'].passed_display }} | Deployment context and external dependencies are identified |
| C3 | recommended (0 or 30) | {{ results['system_overview.C3'].passed_display }} | Overview is consistent with detailed architecture sections |

## 3. Component Model — `03-component_model.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 30) | {{ results['component_model.C1'].passed_display }} | All system components identified |
| C2 | mandatory (0 or 30) | {{ results['component_model.C2'].passed_display }} | Each component has a clear responsibility |
| C3 | recommended (0 or 20) | {{ results['component_model.C3'].passed_display }} | Interfaces between components documented |
| C4 | recommended (0 or 20) | {{ results['component_model.C4'].passed_display }} | No overlapping component responsibilities |

## 4. Communication Paths — `04-communication_paths.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['communication_paths.C1'].passed_display }} | All inter-component communication paths identified with protocol and direction |
| C2 | mandatory (0 or 30) | {{ results['communication_paths.C2'].passed_display }} | Synchronization model and quality-of-service defined per path |
| C3 | recommended (0 or 30) | {{ results['communication_paths.C3'].passed_display }} | Error handling and retry strategies are documented |

## 5. Data Flow — `05-data_flow.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['data_flow.C1'].passed_display }} | All major data flows identified with sources and sinks |
| C2 | mandatory (0 or 30) | {{ results['data_flow.C2'].passed_display }} | Processing semantics (sync/async, batch/stream) are defined |
| C3 | recommended (0 or 30) | {{ results['data_flow.C3'].passed_display }} | Data transformations and storage boundaries are documented |

## 6. Security Considerations — `06-security_considerations.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 30) | {{ results['security_considerations.C1'].passed_display }} | Trust boundaries and attack surface are enumerated |
| C2 | mandatory (0 or 30) | {{ results['security_considerations.C2'].passed_display }} | Threat model uses structured methodology (STRIDE/PASTA/OWASP) with threats mapped to mitigations |
| C3 | mandatory (0 or 20) | {{ results['security_considerations.C3'].passed_display }} | Authentication and authorization mechanisms are described |
| C4 | recommended (0 or 20) | {{ results['security_considerations.C4'].passed_display }} | Data protection (at rest and in transit) is addressed per classification |

## 7. Rationale — `07-rationale.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['rationale.C1'].passed_display }} | Each entry has Context, Decision, Alternatives Considered, and Rejection Reason |
| C2 | mandatory (0 or 30) | {{ results['rationale.C2'].passed_display }} | Rejection Reason is architectural, not implementation/technology-level |
| C3 | recommended (0 or 30) | {{ results['rationale.C3'].passed_display }} | Every decision ties to a named architectural goal or pillar |

## 8. Constraints — `08-constraints.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['constraints.C1'].passed_display }} | Constraints are architectural, not implementation-specific |
| C2 | mandatory (0 or 30) | {{ results['constraints.C2'].passed_display }} | Each constraint has a documented justification |
| C3 | recommended (0 or 30) | {{ results['constraints.C3'].passed_display }} | No contradictory constraints |

## 9. Traceability — `09-traceability.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['traceability.C1'].passed_display }} | Architecture elements are traceable to source requirements or decisions |
| C2 | mandatory (0 or 30) | {{ results['traceability.C2'].passed_display }} | Cross-references between architecture sections are present and resolvable |
| C3 | recommended (0 or 30) | {{ results['traceability.C3'].passed_display }} | Decision records (ADR) are referenced with stable numeric or slug IDs |

## 10. Operational Readiness — `10-operational_readiness.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 30) | {{ results['operational_readiness.C1'].passed_display }} | Deployment automation and production promotion gating documented |
| C2 | mandatory (0 or 30) | {{ results['operational_readiness.C2'].passed_display }} | Rollback procedure defined with time target |
| C3 | recommended (0 or 20) | {{ results['operational_readiness.C3'].passed_display }} | Runbooks linked per failure mode with on-call routing |
| C4 | recommended (0 or 20) | {{ results['operational_readiness.C4'].passed_display }} | RTO/RPO targets and DR plan documented |

## 11. Observability — `11-observability.md`

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['observability.C1'].passed_display }} | Telemetry backend identified and correlation ID strategy documented |
| C2 | mandatory (0 or 30) | {{ results['observability.C2'].passed_display }} | Log aggregation pipeline with retention policy described |
| C3 | recommended (0 or 30) | {{ results['observability.C3'].passed_display }} | SLO monitoring architecture and on-call routing documented |

## Generic — `generic.md` (sections with no matching semantic_type)

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['generic.C1'].passed_display }} | Content is architecture-relevant, not implementation-specific |
| C2 | mandatory (0 or 30) | {{ results['generic.C2'].passed_display }} | Claims and assertions are justified by evidence or reasoning |
| C3 | recommended (0 or 30) | {{ results['generic.C3'].passed_display }} | No duplication of content from other architecture section types |

---

## Findings

{% if findings | length > 0 %}
| Section | Criterion | Severity | Evidence | Message |
|---|---|---|---|---|
{% for f in findings -%}
| {{ f.section_type }} | {{ f.criterion_id }} | {{ f.severity }} | {{ f.evidence.excerpt | default('—') }} | {{ f.message }} |
{% endfor %}
{% else %}
No findings.
{% endif %}

---

## Score

**Semantic Section Score:** {{ score }} / 100

Rolled up across all present sections' criteria above. Sections absent from the document (per its own choice among the optional ones) don't contribute a criterion failure — only present-but-deficient sections do.

---

## Metadata

| Field | Value |
|---|---|
| Domain | architecture |
| Standard | documentation-standards |
| Section Rubric Files | `audit/semantic/section/05-architecture/*.md` |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
