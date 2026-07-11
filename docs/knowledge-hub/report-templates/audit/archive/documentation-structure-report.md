# Documentation Structure Audit Report — {{ created_at }}

**Overall Score:** {{ score }} / 100 — **{{ rating }}**
**Previous Score:** {% if previous_score %}{{ previous_score }} / 100{% else %}— (baseline){% endif %}
**Score Change:** {{ score_change_display }}
**Engineering Readiness:** {{ engineering_readiness }}

{{ rating_description }}

This audit validates the documentation corpus as an integrated system: structural
integrity, one-to-one mapping between related domains, feature atomicity,
cross-document alignment, name preservation across compilation layers,
implementation traceability, and generation compliance. Some checks delegate
to another pipeline's finding (SI5, MC5, AE4, CA2, CA3) rather than
reimplementing logic that already exists elsewhere — see
`docs/proposal.md`.

---

## 1. Executive Summary

**Rating:** {{ rating }} — {{ rating_description }}

**Validation Checks Executed:** {{ total_checks }}

{% if critical_findings | length > 0 %}
**Critical Findings:** {{ critical_findings | length }}
{% endif %}
{% if major_findings | length > 0 %}
**Major Findings:** {{ major_findings | length }}
{% endif %}
{% if minor_findings | length > 0 %}
**Minor Findings:** {{ minor_findings | length }}
{% endif %}
{% if observations | length > 0 %}
**Observations:** {{ observations | length }}
{% endif %}

---

## 2. Score Rubric

| Range | Rating | What it means |
|---|---|---|
| 95–100 | Excellent | The documentation corpus is structurally sound, mapped 1:1 where required, atomic, aligned, and traceable to code. |
| 90–94 | Very Good | Minor gaps only — safe to rely on with light follow-up. |
| 80–89 | Good | Solid structure — a few mapping or atomicity issues to resolve. |
| 70–79 | Acceptable | Core structure holds but gaps remain — verify before relying on this. |
| Below 70 | Needs Improvement | Significant structural gaps — the corpus is not reliably coherent as a system. |

---

## 3. Category Scores

| Category | Score | Rating |
|----------|------:|--------|
| Structural Integrity | {{ structural_integrity_score }} | {{ structural_integrity_rating }} |
| Mapping Consistency | {{ mapping_consistency_score }} | {{ mapping_consistency_rating }} |
| Atomicity Enforcement | {{ atomicity_enforcement_score }} | {{ atomicity_enforcement_rating }} |
| Cross-Document Alignment | {{ cross_document_alignment_score }} | {{ cross_document_alignment_rating }} |
| Name Preservation | {{ name_preservation_score }} | {{ name_preservation_rating }} |
| Implementation Traceability | {{ implementation_traceability_score }} | {{ implementation_traceability_rating }} |
| Generation Compliance | {{ generation_compliance_score }} | {{ generation_compliance_rating }} |

Equal-weighted average across all seven categories. Definitions:
`docs/raw/audit/documentation-structure-audit.md`.

---

## 4. Validation Scores

{% if validation_scores | length > 0 %}
| Rule | Score | Rating |
|------|------:|--------|
{% for v in validation_scores -%}
| {{ v.id }} | {{ v.score }} | {{ v.rating }} |
{% endfor %}
{% else %}
_No validation scores recorded._
{% endif %}

---

## 5. Trend Analysis

{% if previous_score %}
**Previous Score:** {{ previous_score }} / 100
**Current Score:** {{ score }} / 100
**Change:** {{ score_change_display }}

{{ trend_text }}
{% else %}
Baseline audit established. No previous report for comparison.
{% endif %}

---

## 6. Findings

Every finding includes an Evidence column: the excerpt the audit provider
captured to justify the finding, when one was captured. Findings whose
message begins with `(delegated to ...)` were produced by another
pipeline's check and forwarded here rather than reimplemented — see the
named pipeline/check-id for the original.

{% if critical_findings | length > 0 %}
### Critical

| Check | Message | Location | Evidence |
|---|---|---|---|
{% for f in critical_findings -%}
| {{ f.check_id }} | {{ f.message }} | {% if f.location %}{{ f.location }}{% else %}—{% endif %} | {% if f.evidence_excerpt %}"{{ f.evidence_excerpt }}" ({{ f.evidence_source }}){% else %}—{% endif %} |
{% endfor %}
{% endif %}

{% if major_findings | length > 0 %}
### Major

| Check | Message | Location | Evidence |
|---|---|---|---|
{% for f in major_findings -%}
| {{ f.check_id }} | {{ f.message }} | {% if f.location %}{{ f.location }}{% else %}—{% endif %} | {% if f.evidence_excerpt %}"{{ f.evidence_excerpt }}" ({{ f.evidence_source }}){% else %}—{% endif %} |
{% endfor %}
{% endif %}

{% if minor_findings | length > 0 %}
### Minor

| Check | Message | Location | Evidence |
|---|---|---|---|
{% for f in minor_findings -%}
| {{ f.check_id }} | {{ f.message }} | {% if f.location %}{{ f.location }}{% else %}—{% endif %} | {% if f.evidence_excerpt %}"{{ f.evidence_excerpt }}" ({{ f.evidence_source }}){% else %}—{% endif %} |
{% endfor %}
{% endif %}

{% if observations | length > 0 %}
### Observations

| Check | Message | Location | Evidence |
|---|---|---|---|
{% for f in observations -%}
| {{ f.check_id }} | {{ f.message }} | {% if f.location %}{{ f.location }}{% else %}—{% endif %} | {% if f.evidence_excerpt %}"{{ f.evidence_excerpt }}" ({{ f.evidence_source }}){% else %}—{% endif %} |
{% endfor %}
{% endif %}

{% if critical_findings | length == 0 and major_findings | length == 0 and minor_findings | length == 0 and observations | length == 0 %}
_No findings recorded._
{% endif %}

---

## 7. Recommendations

{% if recommendations | length > 0 %}
{% for r in recommendations %}
### {{ r.category }} — Priority {{ r.priority }}

**{{ r.description }}**
{% if r.file_path %}**File:** {{ r.file_path }}{% endif %}

{% endfor %}
{% else %}
_No recommendations._
{% endif %}

---

## 8. Readiness Assessment

| Area | Status |
|------|--------|
| Structural Integrity | {% if structural_integrity_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Mapping Consistency | {% if mapping_consistency_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Atomicity | {% if atomicity_enforcement_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Cross-Document Alignment | {% if cross_document_alignment_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Implementation Traceability | {% if implementation_traceability_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Overall | {% if engineering_readiness == "READY" %}READY{% else %}NOT READY{% endif %} |

---

## 9. Audit Metadata

| Field | Value |
|-------|-------|
| Audit Type | Documentation Structure |
| Session | {{ session_id }} |
| Git Revision | {{ git_revision }} |
| Audit Date | {{ created_at }} |
| Validation Rules | SI1–SI7, MC1–MC8, AE1–AE6, CA1–CA8, NP1–NP6, IT1–IT5, GC1–GC5 (`docs/raw/audit/documentation-structure-audit.md`) |
| Scope | `docs/raw/`, `README.md`, and the declared implementation directory |
| {% if previous_score %}Previous Report| Available{% else %}Previous Report| None (baseline){% endif %} |
