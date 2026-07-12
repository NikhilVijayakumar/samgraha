# {{ document_title }} — Vision Semantic Document Audit Report

> **Domain:** vision
> **Scope:** document
> **Kind:** semantic
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Document-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | 5.0 |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | 5.0 |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Semantic document audit verifies that the collection of vision sections coheres as one aspirational statement. Section-level audits check each section individually; this audit catches cross-section contradictions — a Solution that doesn't address the Problem, or a Vision Statement disconnected from the Solution.

---

## Criterion Results

### C1 — Problem, Solution, and Vision Statement align without contradiction
- **Weight:** mandatory
- **Score if passed:** 35
- **Status:** {{ c1_status }}
- **Confidence:** {{ c1_confidence }}
- **Evidence:** {{ c1_evidence }}
- **Why this matters:** If the Solution addresses a different problem than the one Problem describes, or the Vision Statement is an unrelated ambition, the document is internally contradictory.

### C2 — No technology/implementation references anywhere in the document
- **Weight:** mandatory
- **Score if passed:** 35
- **Status:** {{ c2_status }}
- **Confidence:** {{ c2_confidence }}
- **Evidence:** {{ c2_evidence }}
- **Why this matters:** A technology reference split across two sentences in different sections is still a violation. Vision must remain technology-independent as a whole, not just section by section.

### C3 — Terminology consistent and all Vision documents cohere as one system
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ c3_status }}
- **Confidence:** {{ c3_confidence }}
- **Evidence:** {{ c3_evidence }}
- **Why this matters:** When the same value or goal is named differently across sections, readers cannot tell whether the difference is intentional or accidental.

---

## Red Flags Detected

{{ red_flags_table }}

---

## Edge Cases Evaluated

{{ edge_cases_table }}

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ weighted_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})

---

## Failures

| Criterion | Severity | Evidence | Regression? |
|---|---|---|---|
{{ failures_table }}

---

## Summary

{{ summary_text }}

### Document-Level Breakdown

| Category | Weight | Score | Status |
|---|---|---|---|
| Problem-Solution-VS Alignment | 35 | {{ c1_score }} | {{ c1_status }} |
| Technology Independence | 35 | {{ c2_score }} | {{ c2_status }} |
| Terminology Consistency | 30 | {{ c3_score }} | {{ c3_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| purpose | {{ purpose_section_score }} | {{ purpose_section_weight }} | {{ purpose_section_status }} |
| vision_statement | {{ vs_section_score }} | {{ vs_section_weight }} | {{ vs_section_status }} |
| problem | {{ problem_section_score }} | {{ problem_section_weight }} | {{ problem_section_status }} |
| solution | {{ solution_section_score }} | {{ solution_section_weight }} | {{ solution_section_status }} |
| target_audience | {{ ta_section_score }} | {{ ta_section_weight }} | {{ ta_section_status }} |
| pillars | {{ pillars_section_score }} | {{ pillars_section_weight }} | {{ pillars_section_status }} |
| philosophy | {{ philosophy_section_score }} | {{ philosophy_section_weight }} | {{ philosophy_section_status }} |
| guiding_principles | {{ gp_section_score }} | {{ gp_section_weight }} | {{ gp_section_status }} |
| success_criteria | {{ sc_section_score }} | {{ sc_section_weight }} | {{ sc_section_status }} |
| traceability | {{ trace_section_score }} | {{ trace_section_weight }} | {{ trace_section_status }} |
