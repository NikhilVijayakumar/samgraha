# Feature Audit Report — {{ created_at }}

**Overall Score:** {{ score }} / 100 — **{{ rating }}**
**Previous Score:** {% if previous_score %}{{ previous_score }} / 100{% else %}— (baseline){% endif %}
**Score Change:** {{ score_change_display }}
**Feature Design Readiness:** {{ engineering_readiness }}

{{ rating_description }}

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

Every score in this report is rated against the same bands, taken from
`docs/raw/audit/feature-audit.md`'s Scoring Model:

| Range | Rating | What it means |
|---|---|---|
| 95–100 | Excellent | Every feature is atomic, business-focused, implementation-independent, and traceable to the Vision. |
| 90–94 | Very Good | Minor gaps only — safe to hand off to Feature Design with light follow-up. |
| 80–89 | Good | Solid coverage — a few completeness or traceability issues to resolve. |
| 70–79 | Acceptable | Core product behavior documented but gaps remain — Feature Design should verify before relying on this. |
| Below 70 | Needs Improvement | Significant gaps — product behavior isn't reliably captured. |

---

## 3. Category Scores

| Category | Score | Rating | Weight |
|----------|------:|--------|------:|
| Feature Definition | {{ feature_definition_score }} | {{ feature_definition_rating }} | 30% |
| Product Definition | {{ product_definition_score }} | {{ product_definition_rating }} | 35% |
| Documentation Quality | {{ documentation_quality_score }} | {{ documentation_quality_rating }} | 20% |
| Product Readiness | {{ product_readiness_score }} | {{ product_readiness_rating }} | 15% |

Category weights and definitions: `docs/raw/audit/feature-audit.md#category-weights`.

---

## 4. Structural Compliance Matrix

Checks the compiled documentation collection against
`docs/raw/standards/feature.md`'s Required Sections table.

| Section Type | Required | Documents With It | Status |
|---|:---:|:---:|---|
{% for s in section_compliance -%}
| {{ s.semantic_type }} | {% if s.required %}✓{% else %}—{% endif %} | {{ s.doc_count }} / {{ s.total_docs }} | {{ s.status }} |
{% endfor %}

**Missing** = no document in the collection has this section at all.
**Partial** = some but not all documents have it.
**Complete** = every document has it, or it's optional with no expectation of universal presence.

---

## 5. Document Scores (per feature)

{% if doc_scores | length > 0 %}
| Document | Score | Rating |
|----------|------:|--------|
{% for doc in doc_scores -%}
| {{ doc.name }} | {{ doc.score }} | {{ doc.rating }} |
{% endfor %}
{% else %}
_No document scores recorded._
{% endif %}

---

## 6. Validation Scores

Each validation rule (F1–F14) checks one property of the Feature
Documentation collection — see `docs/raw/audit/feature-audit.md` for the
full definition of each.

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

## 7. Audit Standard Rubrics

What "good" looks like for each feature-specification concern, drawn
directly from `docs/raw/audit-standards/feature/*.md` — the same rubrics
the semantic audit provider checks findings against.

{% for a in audit_standards %}
### {{ a.semantic_type }}

{{ a.engineering_intent }}

Checked for:
{% for obj in a.top_objectives %}
- {{ obj }}
{% endfor %}

{% endfor %}

---

## 8. Trend Analysis

{% if previous_score %}
**Previous Score:** {{ previous_score }} / 100
**Current Score:** {{ score }} / 100
**Change:** {{ score_change_display }}

{{ trend_text }}
{% else %}
Baseline audit established. No previous report for comparison.
{% endif %}

---

## 9. Findings

Every finding includes an Evidence column: the excerpt the audit provider
captured to justify the finding, when one was captured.

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

## 10. Recommendations

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

## 11. Readiness Assessment

| Area | Status |
|------|--------|
| Documentation Quality | {% if documentation_quality_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Product Specification | {% if product_definition_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Feature Design Readiness | {% if engineering_readiness == "READY" %}READY{% else %}NOT READY{% endif %} |
| Engineering Assumption Risk | {% if product_readiness_score >= 90 %}LOW{% elif product_readiness_score >= 70 %}MEDIUM{% else %}HIGH{% endif %} |

---

## 12. Audit Metadata

| Field | Value |
|-------|-------|
| Audit Type | Feature |
| Session | {{ session_id }} |
| Git Revision | {{ git_revision }} |
| Audit Date | {{ created_at }} |
| Validation Rules | F1–F14 (`docs/raw/audit/feature-audit.md`) |
| Structure Standard | `docs/raw/standards/feature.md` |
| Semantic Audit Rubrics | `docs/raw/audit-standards/feature/*.md` |
| {% if previous_score %}Previous Report| Available{% else %}Previous Report| None (baseline){% endif %} |
