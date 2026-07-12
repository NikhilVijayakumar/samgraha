# {{ document_title }} — Feature Audit Summary

> **Domain:** feature
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
| functional_requirements | {{ freq_det_rules }} | {{ freq_det_passed }} | {{ freq_det_failed }} | {{ freq_det_errors }} | {{ freq_det_warnings }} |
| acceptance_criteria | {{ ac_det_rules }} | {{ ac_det_passed }} | {{ ac_det_failed }} | {{ ac_det_errors }} | {{ ac_det_warnings }} |
| business_rules | {{ br_det_rules }} | {{ br_det_passed }} | {{ br_det_failed }} | {{ br_det_errors }} | {{ br_det_warnings }} |
| inputs | {{ inputs_det_rules }} | {{ inputs_det_passed }} | {{ inputs_det_failed }} | {{ inputs_det_errors }} | {{ inputs_det_warnings }} |
| outputs | {{ outputs_det_rules }} | {{ outputs_det_passed }} | {{ outputs_det_failed }} | {{ outputs_det_errors }} | {{ outputs_det_warnings }} |
| constraints | {{ constraints_det_rules }} | {{ constraints_det_passed }} | {{ constraints_det_failed }} | {{ constraints_det_errors }} | {{ constraints_det_warnings }} |
| dependencies | {{ dependencies_det_rules }} | {{ dependencies_det_passed }} | {{ dependencies_det_failed }} | {{ dependencies_det_errors }} | {{ dependencies_det_warnings }} |
| non_goals | {{ non_goals_det_rules }} | {{ non_goals_det_passed }} | {{ non_goals_det_failed }} | {{ non_goals_det_errors }} | {{ non_goals_det_warnings }} |
| future_extensions | {{ future_det_rules }} | {{ future_det_passed }} | {{ future_det_failed }} | {{ future_det_errors }} | {{ future_det_warnings }} |
| traceability | {{ trace_det_rules }} | {{ trace_det_passed }} | {{ trace_det_failed }} | {{ trace_det_errors }} | {{ trace_det_warnings }} |

---

## Semantic Document Audit

### Criteria Results

| Criterion | Weight | Score | Status | Confidence |
|---|---|---|---|---|
| C1: FR-AC-BR Consistency | mandatory (40) | {{ sem_doc_c1_score }} | {{ sem_doc_c1_status }} | {{ sem_doc_c1_confidence }} |
| C2: Requirement Coverage | mandatory (30) | {{ sem_doc_c2_score }} | {{ sem_doc_c2_status }} | {{ sem_doc_c2_confidence }} |
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
| functional_requirements | {{ freq_c1 }} | {{ freq_c2 }} | {{ freq_c3 }} | {{ freq_sem_score }} | {{ freq_sem_status }} |
| acceptance_criteria | {{ ac_c1 }} | {{ ac_c2 }} | {{ ac_c3 }} | {{ ac_sem_score }} | {{ ac_sem_status }} |
| business_rules | {{ br_c1 }} | {{ br_c2 }} | {{ br_c3 }} | {{ br_sem_score }} | {{ br_sem_status }} |
| inputs | {{ inputs_c1 }} | {{ inputs_c2 }} | {{ inputs_c3 }} | {{ inputs_sem_score }} | {{ inputs_sem_status }} |
| outputs | {{ outputs_c1 }} | {{ outputs_c2 }} | {{ outputs_c3 }} | {{ outputs_sem_score }} | {{ outputs_sem_status }} |
| constraints | {{ constraints_c1 }} | {{ constraints_c2 }} | {{ constraints_c3 }} | {{ constraints_sem_score }} | {{ constraints_sem_status }} |
| dependencies | {{ dependencies_c1 }} | {{ dependencies_c2 }} | {{ dependencies_c3 }} | {{ dependencies_sem_score }} | {{ dependencies_sem_status }} |
| non_goals | {{ non_goals_c1 }} | {{ non_goals_c2 }} | {{ non_goals_c3 }} | {{ non_goals_sem_score }} | {{ non_goals_sem_status }} |
| future_extensions | {{ future_c1 }} | {{ future_c2 }} | {{ future_c3 }} | {{ future_sem_score }} | {{ future_sem_status }} |
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
| functional_requirements | {{ freq_det_score }} | {{ freq_sem_score }} | {{ freq_combined }} | {{ freq_overall_status }} |
| acceptance_criteria | {{ ac_det_score }} | {{ ac_sem_score }} | {{ ac_combined }} | {{ ac_overall_status }} |
| business_rules | {{ br_det_score }} | {{ br_sem_score }} | {{ br_combined }} | {{ br_overall_status }} |
| inputs | {{ inputs_det_score }} | {{ inputs_sem_score }} | {{ inputs_combined }} | {{ inputs_overall_status }} |
| outputs | {{ outputs_det_score }} | {{ outputs_sem_score }} | {{ outputs_combined }} | {{ outputs_overall_status }} |
| constraints | {{ constraints_det_score }} | {{ constraints_sem_score }} | {{ constraints_combined }} | {{ constraints_overall_status }} |
| dependencies | {{ dependencies_det_score }} | {{ dependencies_sem_score }} | {{ dependencies_combined }} | {{ dependencies_overall_status }} |
| non_goals | {{ non_goals_det_score }} | {{ non_goals_sem_score }} | {{ non_goals_combined }} | {{ non_goals_overall_status }} |
| future_extensions | {{ future_det_score }} | {{ future_sem_score }} | {{ future_combined }} | {{ future_overall_status }} |
| traceability | {{ trace_det_score }} | {{ trace_sem_score }} | {{ trace_combined }} | {{ trace_overall_status }} |
