# Implementation Audit Report — {{ created_at }}

**Overall Score:** {{ score }} / 100 — **{{ rating }}**
**Previous Score:** {% if previous_score %}{{ previous_score }} / 100{% else %}— (baseline){% endif %}
**Score Change:** {{ score_change_display }}
**Production Readiness:** {{ engineering_readiness }}

{{ rating_description }}

This audit reads actual source code under the declared implementation
folder (`repository.implementation.dir`, resolved the same way Coverage
Audit resolves it) rather than a single `docs/raw/*.md` collection, so it
has no Structural Compliance Matrix or Audit Standard Rubrics section.
Implementation is never the source of truth — documentation defines the
contract and source code must conform to it.

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
`docs/raw/audit/implementation-audit.md`'s Scoring Model:

| Range | Rating | What it means |
|---|---|---|
| 95–100 | Excellent | Implementation faithfully realizes Architecture, Feature Technical Design, and Engineering standards with no undocumented drift. |
| 90–94 | Very Good | Minor gaps only — safe to rely on with light follow-up. |
| 80–89 | Good | Solid conformance — a few completeness or naming issues to resolve. |
| 70–79 | Acceptable | Core conformance holds but gaps remain — verify before relying on this. |
| Below 70 | Needs Improvement | Significant gaps — implementation and documentation have drifted apart. |

---

## 3. Category Scores

| Category | Score | Rating | Weight |
|----------|------:|--------|------:|
| Architectural Conformance | {{ architectural_conformance_score }} | {{ architectural_conformance_rating }} | 30% |
| Feature Conformance | {{ feature_conformance_score }} | {{ feature_conformance_rating }} | 25% |
| Engineering Conformance | {{ engineering_conformance_score }} | {{ engineering_conformance_rating }} | 20% |
| Documentation Integrity | {{ documentation_integrity_score }} | {{ documentation_integrity_rating }} | 15% |
| Implementation Quality | {{ implementation_quality_score }} | {{ implementation_quality_rating }} | 10% |

Category weights and definitions: `docs/raw/audit/implementation-audit.md#category-weights`.

---

## 4. Validation Scores

Each validation rule (I1–I15) checks one property of implementation
conformance — see `docs/raw/audit/implementation-audit.md` for the full
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
| Architecture Conformance | {% if architectural_conformance_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Feature Conformance | {% if feature_conformance_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Engineering Conformance | {% if engineering_conformance_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Documentation Conformance | {% if documentation_integrity_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Production Readiness | {% if engineering_readiness == "READY" %}READY{% else %}NOT READY{% endif %} |
| Documentation Drift Risk | {% if documentation_integrity_score >= 90 %}LOW{% elif documentation_integrity_score >= 70 %}MEDIUM{% else %}HIGH{% endif %} |

---

## 9. Audit Metadata

| Field | Value |
|-------|-------|
| Audit Type | Implementation |
| Session | {{ session_id }} |
| Git Revision | {{ git_revision }} |
| Audit Date | {{ created_at }} |
| Validation Rules | I1–I15 (`docs/raw/audit/implementation-audit.md`) |
| Scope | Declared implementation folder + Architecture, Feature Technical Design, Engineering, External Context docs |
| {% if previous_score %}Previous Report| Available{% else %}Previous Report| None (baseline){% endif %} |
