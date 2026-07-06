# README Audit Report — {{ created_at }}

**Overall Score:** {{ score }} / 100 — **{{ rating }}**
**Previous Score:** {% if previous_score %}{{ previous_score }} / 100{% else %}— (baseline){% endif %}
**Score Change:** {{ score_change_display }}
**Engineering Readiness:** {{ engineering_readiness }}

{{ rating_description }}

Unlike every other domain report, this one audits a single repo-root file
(`README.md`), not a `docs/raw/<domain>/` collection — there is no
Structural Compliance Matrix here for that reason.

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
`docs/raw/audit/readme-audit.md`'s Scoring Model:

| Range | Rating | What it means |
|---|---|---|
| 95–100 | Excellent | The README orients a new reader in minutes, navigates cleanly to detailed docs, and stays in sync with the repository. |
| 90–94 | Very Good | Minor gaps only — safe as the repository's front door with light follow-up. |
| 80–89 | Good | Solid entry point — a few navigation or scope issues to resolve. |
| 70–79 | Acceptable | Core purpose present but gaps in navigation or documentation quality — onboarding is harder than it should be. |
| Below 70 | Needs Improvement | Significant gaps — the README fails as a repository entry point. |

---

## 3. Category Scores

| Category | Score | Rating | Weight |
|----------|------:|--------|------:|
| Repository Introduction | {{ repo_introduction_score }} | {{ repo_introduction_rating }} | 30% |
| Documentation Navigation | {{ doc_navigation_score }} | {{ doc_navigation_rating }} | 30% |
| Documentation Quality | {{ doc_quality_score }} | {{ doc_quality_rating }} | 25% |
| Maintainability | {{ maintainability_score }} | {{ maintainability_rating }} | 15% |

Category weights and definitions: `docs/raw/audit/readme-audit.md#category-weights`.

---

## 4. Document Scores

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

## 5. Validation Scores

Each validation rule (R1–R12) checks one property of the README — see
`docs/raw/audit/readme-audit.md` for the full definition of each.

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

## 6. Audit Standard Rubrics

What "good" looks like for each README concern, drawn directly from
`docs/raw/audit-standards/readme/*.md` — the same rubrics the semantic
audit provider checks findings against.

{% for a in audit_standards %}
### {{ a.semantic_type }}

{{ a.engineering_intent }}

Checked for:
{% for obj in a.top_objectives %}
- {{ obj }}
{% endfor %}

{% endfor %}

---

## 7. Trend Analysis

{% if previous_score %}
**Previous Score:** {{ previous_score }} / 100
**Current Score:** {{ score }} / 100
**Change:** {{ score_change_display }}

{{ trend_text }}
{% else %}
Baseline audit established. No previous report for comparison.
{% endif %}

---

## 8. Findings

Every finding includes an Evidence column: the excerpt the audit provider
captured to justify the finding, when one was captured. Deterministic
checks (heading presence, word counts) typically don't carry an excerpt —
that's expected, not a gap in this report.

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

## 9. Recommendations

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

## 10. Readiness Assessment

| Area | Status |
|------|--------|
| Repository Introduction | {% if repo_introduction_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Documentation Navigation | {% if doc_navigation_score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Onboarding Experience | {% if score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Repository Discoverability | {% if engineering_readiness == "READY" %}READY{% else %}NOT READY{% endif %} |
| Documentation Synchronization | {% if maintainability_score >= 70 %}PASS{% else %}FAIL{% endif %} |

---

## 11. Audit Metadata

| Field | Value |
|-------|-------|
| Audit Type | README |
| Session | {{ session_id }} |
| Git Revision | {{ git_revision }} |
| Audit Date | {{ created_at }} |
| Validation Rules | R1–R12 (`docs/raw/audit/readme-audit.md`) |
| Structure Standard | `docs/raw/standards/readme.md` |
| Semantic Audit Rubrics | `docs/raw/audit-standards/readme/*.md` |
| {% if previous_score %}Previous Report| Available{% else %}Previous Report| None (baseline){% endif %} |
