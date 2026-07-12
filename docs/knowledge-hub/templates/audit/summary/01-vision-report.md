# {{ document_title }} — Vision Audit Summary

> **Domain:** vision
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Overall Score

| Metric | Value |
|---|---|
| **Deterministic Document** | {{ det_doc_score }} |
| **Deterministic Section** | {{ det_sec_score }} |
| **Semantic Document** | {{ sem_doc_score }} |
| **Semantic Section** | {{ sem_sec_score }} |
| **Combined Score** | {{ combined_score }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** The combined score aggregates deterministic rules (mechanical checks), section-level mechanical checks, semantic document coherence, and semantic section quality. No single audit level tells the full story — a document can pass mechanical rules while failing semantic coherence.

---

## Deterministic Document Audit

### Rule Pass Rate

| Rules Checked | Passed | Failed | Errors | Warnings |
|---|---|---|---|---|
| {{ det_doc_rules_checked }} | {{ det_doc_passed }} | {{ det_doc_failed }} | {{ det_doc_errors }} | {{ det_doc_warnings }} |

### Failures

| Rule | Severity | Weight | Evidence |
|---|---|---|---|
{{ det_doc_failures }}

---

## Deterministic Section Audit

### Rule Pass Rate by Section

| Section | Rules | Passed | Failed | Errors | Warnings |
|---|---|---|---|---|---|
| purpose | {{ purpose_det_rules }} | {{ purpose_det_passed }} | {{ purpose_det_failed }} | {{ purpose_det_errors }} | {{ purpose_det_warnings }} |
| vision_statement | {{ vs_det_rules }} | {{ vs_det_passed }} | {{ vs_det_failed }} | {{ vs_det_errors }} | {{ vs_det_warnings }} |
| problem | {{ problem_det_rules }} | {{ problem_det_passed }} | {{ problem_det_failed }} | {{ problem_det_errors }} | {{ problem_det_warnings }} |
| solution | {{ solution_det_rules }} | {{ solution_det_passed }} | {{ solution_det_failed }} | {{ solution_det_errors }} | {{ solution_det_warnings }} |
| target_audience | {{ ta_det_rules }} | {{ ta_det_passed }} | {{ ta_det_failed }} | {{ ta_det_errors }} | {{ ta_det_warnings }} |
| pillars | {{ pillars_det_rules }} | {{ pillars_det_passed }} | {{ pillars_det_failed }} | {{ pillars_det_errors }} | {{ pillars_det_warnings }} |
| philosophy | {{ philosophy_det_rules }} | {{ philosophy_det_passed }} | {{ philosophy_det_failed }} | {{ philosophy_det_errors }} | {{ philosophy_det_warnings }} |
| guiding_principles | {{ gp_det_rules }} | {{ gp_det_passed }} | {{ gp_det_failed }} | {{ gp_det_errors }} | {{ gp_det_warnings }} |
| success_criteria | {{ sc_det_rules }} | {{ sc_det_passed }} | {{ sc_det_failed }} | {{ sc_det_errors }} | {{ sc_det_warnings }} |
| traceability | {{ trace_det_rules }} | {{ trace_det_passed }} | {{ trace_det_failed }} | {{ trace_det_errors }} | {{ trace_det_warnings }} |

---

## Semantic Document Audit

### Criteria Results

| Criterion | Weight | Score | Status | Confidence |
|---|---|---|---|---|
| C1: Problem-Solution-VS Alignment | mandatory (35) | {{ sem_doc_c1_score }} | {{ sem_doc_c1_status }} | {{ sem_doc_c1_confidence }} |
| C2: Technology Independence | mandatory (35) | {{ sem_doc_c2_score }} | {{ sem_doc_c2_status }} | {{ sem_doc_c2_confidence }} |
| C3: Terminology Consistency | recommended (30) | {{ sem_doc_c3_score }} | {{ sem_doc_c3_status }} | {{ sem_doc_c3_confidence }} |

### Failures

| Criterion | Severity | Evidence |
|---|---|---|
{{ sem_doc_failures }}

---

## Semantic Section Audit

### Criteria Results by Section

| Section | C1 | C2 | C3 | Score | Status |
|---|---|---|---|---|---|
| purpose | {{ purpose_c1 }} | {{ purpose_c2 }} | {{ purpose_c3 }} | {{ purpose_sem_score }} | {{ purpose_sem_status }} |
| vision_statement | {{ vs_c1 }} | {{ vs_c2 }} | {{ vs_c3 }} | {{ vs_sem_score }} | {{ vs_sem_status }} |
| problem | {{ problem_c1 }} | {{ problem_c2 }} | {{ problem_c3 }} | {{ problem_sem_score }} | {{ problem_sem_status }} |
| solution | {{ solution_c1 }} | {{ solution_c2 }} | {{ solution_c3 }} | {{ solution_sem_score }} | {{ solution_sem_status }} |
| target_audience | {{ ta_c1 }} | {{ ta_c2 }} | {{ ta_c3 }} | {{ ta_sem_score }} | {{ ta_sem_status }} |
| pillars | {{ pillars_c1 }} | {{ pillars_c2 }} | {{ pillars_c3 }} | {{ pillars_sem_score }} | {{ pillars_sem_status }} |
| philosophy | {{ philosophy_c1 }} | {{ philosophy_c2 }} | {{ philosophy_c3 }} | {{ philosophy_sem_score }} | {{ philosophy_sem_status }} |
| guiding_principles | {{ gp_c1 }} | {{ gp_c2 }} | {{ gp_c3 }} | {{ gp_sem_score }} | {{ gp_sem_status }} |
| success_criteria | {{ sc_c1 }} | {{ sc_c2 }} | {{ sc_c3 }} | {{ sc_sem_score }} | {{ sc_sem_status }} |
| traceability | {{ trace_c1 }} | {{ trace_c2 }} | {{ trace_c3 }} | {{ trace_sem_score }} | {{ trace_sem_status }} |

---

## Score History

| Date | Auditor | Combined Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ combined_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})

---

## Failures Summary

| Category | Rule/Criterion | Severity | Section | Evidence | Regression? |
|---|---|---|---|---|---|
{{ all_failures }}

---

## Summary

{{ summary_text }}

### Document-Level Breakdown

| Category | Weight | Score | Status |
|---|---|---|---|
| Deterministic Document | {{ det_doc_weight }} | {{ det_doc_score }} | {{ det_doc_status }} |
| Deterministic Section | {{ det_sec_weight }} | {{ det_sec_score }} | {{ det_sec_status }} |
| Semantic Document | {{ sem_doc_weight }} | {{ sem_doc_score }} | {{ sem_doc_status }} |
| Semantic Section | {{ sem_sec_weight }} | {{ sem_sec_score }} | {{ sem_sec_status }} |

### Section-Level Breakdown

| Section | Deterministic | Semantic | Combined | Status |
|---|---|---|---|---|
| purpose | {{ purpose_det_score }} | {{ purpose_sem_score }} | {{ purpose_combined }} | {{ purpose_overall_status }} |
| vision_statement | {{ vs_det_score }} | {{ vs_sem_score }} | {{ vs_combined }} | {{ vs_overall_status }} |
| problem | {{ problem_det_score }} | {{ problem_sem_score }} | {{ problem_combined }} | {{ problem_overall_status }} |
| solution | {{ solution_det_score }} | {{ solution_sem_score }} | {{ solution_combined }} | {{ solution_overall_status }} |
| target_audience | {{ ta_det_score }} | {{ ta_sem_score }} | {{ ta_combined }} | {{ ta_overall_status }} |
| pillars | {{ pillars_det_score }} | {{ pillars_sem_score }} | {{ pillars_combined }} | {{ pillars_overall_status }} |
| philosophy | {{ philosophy_det_score }} | {{ philosophy_sem_score }} | {{ philosophy_combined }} | {{ philosophy_overall_status }} |
| guiding_principles | {{ gp_det_score }} | {{ gp_sem_score }} | {{ gp_combined }} | {{ gp_overall_status }} |
| success_criteria | {{ sc_det_score }} | {{ sc_sem_score }} | {{ sc_combined }} | {{ sc_overall_status }} |
| traceability | {{ trace_det_score }} | {{ trace_sem_score }} | {{ trace_combined }} | {{ trace_overall_status }} |
