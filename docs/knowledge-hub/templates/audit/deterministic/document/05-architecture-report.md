# Deterministic Whole-Document Report — Architecture

**Document:** {{ document_path }}
**Standard:** `documentation-standards/05-architecture-standards.md`
**Rule File:** `audit/deterministic/document/05-architecture.yaml`
**Audit Date:** {{ created_at }}

---

## 1. Collection Completeness

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-doc-001 | Required sections present (System Overview, Component Model, Communication Paths, Data Flow, Security Considerations) | {{ results['arch-doc-001'].status }} | error |
| arch-doc-002 | No empty required sections | {{ results['arch-doc-002'].status }} | error |

## 2. Modularity

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-doc-003 | Document covers one architectural concern, not several unrelated ones | {{ results['arch-doc-003'].status }} | warning |

## 3. Technology Independence

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-doc-004 | No implementation technology references (languages, frameworks, libraries, schemas, protocols) | {{ results['arch-doc-004'].status }} | error |

## 4. Cross-References

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-doc-005 | References Vision (derives_from) and Philosophy (guided_by) where applicable | {{ results['arch-doc-005'].status }} | warning |

## 5. Duplicate Content

| Rule | Check | Result | Severity |
|---|---|---|---|
| arch-doc-006 | No section repeats information already stated in another section | {{ results['arch-doc-006'].status }} | warning |

---

## Failures

{% if failed_rules | length > 0 %}
| Rule | Message | Evidence |
|---|---|---|
{% for r in failed_rules -%}
| {{ r.id }} | {{ r.message }} | {{ r.evidence | default('—') }} |
{% endfor %}
{% else %}
No failures.
{% endif %}

---

## Score

**Deterministic Whole Score:** {{ score }} / 100

Mandatory rules (arch-doc-001, 002, 004) failing drags this score to 0 for that rule's weight share; arch-doc-003 and arch-doc-005/006 are recommended and cost partial credit only. See `audit/deterministic/document/05-architecture.yaml` for the authoritative weight of each rule.

---

## Metadata

| Field | Value |
|---|---|
| Domain | architecture |
| Standard | documentation-standards |
| Rule File | `audit/deterministic/document/05-architecture.yaml` |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
