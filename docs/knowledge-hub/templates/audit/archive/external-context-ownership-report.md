# External Context Ownership Audit Report — {{ created_at }}

**Overall Score:** {{ score }} / 100 — **{{ rating }}**
**Previous Score:** {% if previous_score %}{{ previous_score }} / 100{% else %}— (baseline){% endif %}
**Score Change:** {{ score_change_display }}
**Engineering Readiness:** {{ engineering_readiness }}

{{ rating_description }}

This audit is cross-cutting: it checks External Context usage across the
whole documentation ecosystem (Vision, Architecture, Design, Feature,
Feature Design, Feature Technical, Engineering, Prototype) rather than
auditing the External Context collection in isolation — see the
`external-context` pipeline for that. It therefore has no Structural
Compliance Matrix or per-section Audit Standard Rubrics.

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
`docs/raw/audit/external-context-ownership-audit.md`'s Scoring Model.

| Range | Rating | What it means |
|---|---|---|
| 95–100 | Excellent | Every external dependency is documented once, referenced consistently, and correctly applied throughout the documentation ecosystem. |
| 90–94 | Very Good | Minor gaps only — safe to rely on with light follow-up. |
| 80–89 | Good | Solid coverage — a few completeness or consistency issues to resolve. |
| 70–79 | Acceptable | Core dependency ownership documented but gaps remain — verify before relying on this. |
| Below 70 | Needs Improvement | Significant gaps — dependency knowledge isn't reliably owned or consistently applied. |

---

## 3. Category Scores

| Category | Score | Rating | Weight |
|----------|------:|--------|------:|
| Dependency Coverage | {{ dependency_coverage_score }} | {{ dependency_coverage_rating }} | 35% |
| Documentation Integration | {{ documentation_integration_score }} | {{ documentation_integration_rating }} | 35% |
| Consistency | {{ consistency_score }} | {{ consistency_rating }} | 30% |

Category weights and definitions: `docs/raw/audit/external-context-ownership-audit.md#category-weights`.

---

## 4. Validation Scores

Each validation rule (EC1–EC12 in this audit's own numbering, distinct
from the `external-context` pipeline's EC1–EC12) checks one property of
External Context ownership across the ecosystem — see
`docs/raw/audit/external-context-ownership-audit.md` for the full
definition of each.

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
| Documentation Quality | {% if documentation_integration_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Dependency Documentation | {% if dependency_coverage_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Architecture Support | {% if consistency_score >= 70 %}READY{% else %}NOT READY{% endif %} |
| Engineering Support | {% if engineering_readiness == "READY" %}READY{% else %}NOT READY{% endif %} |
| Feature Technical Design Support | {% if dependency_coverage_score >= 70 and documentation_integration_score >= 70 %}READY{% else %}NOT READY{% endif %} |

---

## 9. Audit Metadata

| Field | Value |
|-------|-------|
| Audit Type | External Context Ownership |
| Session | {{ session_id }} |
| Git Revision | {{ git_revision }} |
| Audit Date | {{ created_at }} |
| Validation Rules | EC1–EC12 (`docs/raw/audit/external-context-ownership-audit.md`) |
| Scope | `docs/raw/external-context/` and every referencing domain |
| {% if previous_score %}Previous Report| Available{% else %}Previous Report| None (baseline){% endif %} |
