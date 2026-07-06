# Architecture Audit Report — {{ created_at }}

**Overall Score:** {{ score }} / 100 — **{{ rating }}**
**Previous Score:** {% if previous_score %}{{ previous_score }} / 100{% else %}— (baseline){% endif %}
**Score Change:** {{ score_change_display }}
**Engineering Readiness:** {{ engineering_readiness }}

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

Every score in this report — overall, category, per-document, and
per-validation — is rated against the same bands, taken from
`docs/raw/audit/architecture-audit.md`'s Overall Score table:

| Range | Rating | What it means |
|---|---|---|
| 95–100 | Excellent | Modular, fully traceable, no implementation leakage — ready for Engineering with no reservations. |
| 90–94 | Very Good | Minor gaps only — safe to proceed to Engineering with light follow-up. |
| 80–89 | Good | Solid foundation — a few structural, ownership, or consistency issues to resolve before Engineering. |
| 70–79 | Acceptable | Core structure present but gaps in traceability, boundaries, or consistency — Engineering should wait for fixes. |
| Below 70 | Needs Improvement | Significant gaps in required sections, ownership, or technology independence — not ready for Engineering. |

---

## 3. Category Scores

| Category | Score | Rating | Weight |
|----------|------:|--------|------:|
| Collection Integrity | {{ collection_integrity_score }} | {{ collection_integrity_rating }} | 25% |
| Structural Integrity | {{ structural_integrity_score }} | {{ structural_integrity_rating }} | 30% |
| Consistency | {{ consistency_score }} | {{ consistency_rating }} | 30% |
| Cross-Repository Architecture | {{ cross_repo_score }} | {{ cross_repo_rating }} | 15% |

Category weights and definitions: `docs/raw/audit/architecture-audit.md#category-weights`.

---

## 4. Structural Compliance Matrix

Checks the compiled documentation collection against
`docs/raw/standards/architecture.md`'s Required Sections table — not
document-by-document scoring, but whether the *collection as a whole*
actually has each required concern somewhere in it.

| Section Type | Required | Documents With It | Status |
|---|:---:|:---:|---|
{% for s in section_compliance -%}
| {{ s.semantic_type }} | {% if s.required %}✓{% else %}—{% endif %} | {{ s.doc_count }} / {{ s.total_docs }} | {{ s.status }} |
{% endfor %}

**Missing** = no document in the collection has this section at all.
**Partial** = some but not all documents have it (acceptable for optional
types; a finding for required types — see Section 9).
**Complete** = every document in the collection has it, or it's an
optional type with no expectation of universal presence.

---

## 5. Document Scores

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

Each validation rule (A1–A13) checks one property of the architecture
collection — see `docs/raw/audit/architecture-audit.md` for the full
definition of each. Scores here are the audit's own record of how well
this collection satisfied that rule.

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

What "good" looks like for each architectural concern, drawn directly
from `docs/raw/audit-standards/architecture/*.md` — the same rubrics the
semantic audit provider checks findings against. Use this to understand
*why* a section scored the way it did without opening 11 separate files.

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
captured to justify the finding, when one was captured. Deterministic
checks (structural presence, cross-references) typically don't carry an
excerpt — that's expected, not a gap in this report. Semantic checks
(tone, clarity, technology-independence judgment calls) do, when the
provider that raised them supports it.

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
| Documentation Quality | {% if score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Architecture Quality | {% if score >= 70 %}PASS{% else %}FAIL{% endif %} |
| Engineering Readiness | {% if engineering_readiness == "YES" %}READY{% else %}NOT READY{% endif %} |
| Feature Technical Design Support | {% if score >= 80 %}READY{% else %}NOT READY{% endif %} |

---

## 12. Audit Metadata

| Field | Value |
|-------|-------|
| Audit Type | Architecture |
| Session | {{ session_id }} |
| Git Revision | {{ git_revision }} |
| Audit Date | {{ created_at }} |
| Validation Rules | A1–A13 (`docs/raw/audit/architecture-audit.md`) |
| Structure Standard | `docs/raw/standards/architecture.md` |
| Semantic Audit Rubrics | `docs/raw/audit-standards/architecture/*.md` |
| {% if previous_score %}Previous Report| Available{% else %}Previous Report| None (baseline){% endif %} |
