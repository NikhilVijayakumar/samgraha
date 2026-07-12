# Semantic Whole-Document Report — Architecture

**Document:** {{ document_path }}
**Standard:** `documentation-standards/05-architecture-standards.md`
**Rubric:** `audit/semantic/document/05-architecture.md`
**Audit Date:** {{ created_at }}

---

## Judgment

Verifies Architecture Documentation coheres as one structural model — Component Model, Data Flow, and Communication Paths must describe the same system without contradicting each other, and the collection as a whole must read as one architecture, not several.

## Scoring Criteria

| ID | Weight | Result | Description |
|---|---|---|---|
| C1 | mandatory (0 or 40) | {{ results['C1'].passed_display }} | Component Model, Data Flow, and Communication Paths are mutually consistent |
| C2 | mandatory (0 or 30) | {{ results['C2'].passed_display }} | Terminology (component names) consistent across all sections and documents |
| C3 | recommended (0 or 30) | {{ results['C3'].passed_display }} | All Architecture documents cohere as one system |

---

## Findings

{% if findings | length > 0 %}
| Criterion | Severity | Evidence | Message |
|---|---|---|---|
{% for f in findings -%}
| {{ f.criterion_id }} | {{ f.severity }} | {{ f.evidence.excerpt | default('—') }} | {{ f.message }} |
{% endfor %}
{% else %}
No findings — document reads as one coherent architecture.
{% endif %}

---

## Score

**Semantic Whole Score:** {{ score }} / 100

C1 and C2 are mandatory — a mandatory failure drags this score down hard (0 for that criterion's weight share). C3 is recommended, partial credit only.

---

## Metadata

| Field | Value |
|---|---|
| Domain | architecture |
| Standard | documentation-standards |
| Rubric File | `audit/semantic/document/05-architecture.md` |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
