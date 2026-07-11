# Deterministic Runtime Audit Report — {{ created_at }}

**Overall Score:** {{ score }} / 100 — **{{ rating }}**
**Previous Score:** {% if previous_score %}{{ previous_score }} / 100{% else %}— (baseline){% endif %}
**Score Change:** {{ score_change_display }}
**Engineering Readiness:** {{ engineering_readiness }}

{{ rating_description }}

This audit is cross-cutting: it scans Architecture and Engineering
documentation together rather than a single documentation collection, so
it has no Structural Compliance Matrix or per-section Audit Standard
Rubrics — those concepts assume one domain with a Required Sections table.

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
`docs/raw/audit/deterministic-runtime-audit.md`'s Scoring Model:

| Range | Rating | What it means |
|---|---|---|
| 95–100 | Excellent | Stage contracts are explicit, execution is documented as deterministic and stateless, and no hidden runtime state exists. |
| 90–94 | Very Good | Minor gaps only — safe to rely on with light follow-up. |
| 80–89 | Good | Solid coverage — a few completeness or consistency issues to resolve. |
| 70–79 | Acceptable | Core runtime model documented but gaps remain — verify determinism claims before relying on this. |
| Below 70 | Needs Improvement | Significant gaps — the runtime model isn't reliably captured or reproducible. |

---

## 3. Category Scores

| Category | Score | Rating | Weight |
|----------|------:|--------|------:|
| Runtime Model | {{ runtime_model_score }} | {{ runtime_model_rating }} | 40% |
| Engineering Principles | {{ engineering_principles_score }} | {{ engineering_principles_rating }} | 30% |
| Runtime Integrity | {{ runtime_integrity_score }} | {{ runtime_integrity_rating }} | 30% |

Category weights and definitions: `docs/raw/audit/deterministic-runtime-audit.md#category-weights`.

---

## 4. Validation Scores

Each validation rule (S1–S12) checks one property of the runtime model —
see `docs/raw/audit/deterministic-runtime-audit.md` for the full definition
of each.

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
| Runtime Documentation | {% if runtime_model_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Deterministic Execution | {% if engineering_principles_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Stateless Design | {% if runtime_integrity_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Reproducibility | {% if runtime_model_score >= 70 and runtime_integrity_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Engineering Readiness | {% if engineering_readiness == "READY" %}READY{% else %}NOT READY{% endif %} |
| Operational Risk | {% if runtime_integrity_score >= 90 %}LOW{% elif runtime_integrity_score >= 70 %}MEDIUM{% else %}HIGH{% endif %} |

---

## 9. Audit Metadata

| Field | Value |
|-------|-------|
| Audit Type | Deterministic Runtime |
| Session | {{ session_id }} |
| Git Revision | {{ git_revision }} |
| Audit Date | {{ created_at }} |
| Validation Rules | S1–S12 (`docs/raw/audit/deterministic-runtime-audit.md`) |
| Scope | `docs/raw/architecture/`, `docs/raw/engineering/` |
| {% if previous_score %}Previous Report| Available{% else %}Previous Report| None (baseline){% endif %} |
