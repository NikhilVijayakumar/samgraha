# Audit Summary — Architecture

**Document:** {{ document_path }}
**Standard:** `documentation-standards/05-architecture-standards.md`
**Audit Date:** {{ created_at }}

---

## Final Score

**{{ final_score }} / 100** — **{{ rating }}**

```
final_score = (deterministic_whole/100 × 25)
            + (deterministic_section/100 × 25)
            + (semantic_whole/100 × 25)
            + (semantic_section/100 × 25)
```

| Report | Score | Weight | Contribution |
|---|---:|---|---|
| Deterministic — Whole | {{ deterministic_whole }} | 25% | {{ (deterministic_whole / 100 * 25) | round(1) }} |
| Deterministic — Section | {{ deterministic_section }} | 25% | {{ (deterministic_section / 100 * 25) | round(1) }} |
| Semantic — Whole | {{ semantic_whole }} | 25% | {{ (semantic_whole / 100 * 25) | round(1) }} |
| Semantic — Section | {{ semantic_section }} | 25% | {{ (semantic_section / 100 * 25) | round(1) }} |

Mandatory-criterion severity is absorbed inside each of the four reports' own scoring (see each report's Scoring Criteria table) — this rollup is a plain weighted sum, no additional gating here.

---

## Score Bands

| Range | Rating | Meaning |
|---|---|---|
| 95–100 | Excellent | Fully compliant, no reservations |
| 90–94 | Very Good | Minor gaps, safe to proceed |
| 80–89 | Good | Solid foundation, a few issues to resolve |
| 70–79 | Acceptable | Core present but gaps exist |
| Below 70 | Needs Improvement | Significant gaps, not ready |

---

## Report Links

| Report | File |
|---|---|
| Deterministic — Whole | `{{ det_whole_report_path }}` (`audit/deterministic/document/05-architecture.yaml`) |
| Deterministic — Section | `{{ det_section_report_path }}` (`audit/deterministic/section/05-architecture/*.yaml`) |
| Semantic — Whole | `{{ sem_whole_report_path }}` (`audit/semantic/document/05-architecture.md`) |
| Semantic — Section | `{{ sem_section_report_path }}` (`audit/semantic/section/05-architecture/*.md`) |

---

## Top Findings

{% if top_findings | length > 0 %}
| Severity | Source | Rule/Criterion | Message |
|---|---|---|---|
{% for f in top_findings -%}
| {{ f.severity }} | {{ f.report_type }} | {{ f.rule_id }} | {{ f.message }} |
{% endfor %}
{% else %}
No findings.
{% endif %}

---

## Trend

{% if previous_score %}
**Previous:** {{ previous_score }} / 100
**Current:** {{ final_score }} / 100
**Change:** {{ score_change_display }}
{% else %}
Baseline audit established. No previous report for comparison.
{% endif %}

---

## Metadata

| Field | Value |
|---|---|
| Domain | architecture |
| Standard | documentation-standards |
| Document | {{ document_path }} |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
| Reports | 4 detail + 1 summary |
