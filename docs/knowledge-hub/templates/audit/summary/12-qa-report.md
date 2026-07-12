# {{ document_title }} — QA Audit Summary

> **Domain:** qa
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
| test_strategy | {{ strategy_det_rules }} | {{ strategy_det_passed }} | {{ strategy_det_failed }} | {{ strategy_det_errors }} | {{ strategy_det_warnings }} |
| unit_testing | {{ unit_det_rules }} | {{ unit_det_passed }} | {{ unit_det_failed }} | {{ unit_det_errors }} | {{ unit_det_warnings }} |
| integration_testing | {{ integration_det_rules }} | {{ integration_det_passed }} | {{ integration_det_failed }} | {{ integration_det_errors }} | {{ integration_det_warnings }} |
| security_testing | {{ security_det_rules }} | {{ security_det_passed }} | {{ security_det_failed }} | {{ security_det_errors }} | {{ security_det_warnings }} |
| purpose | {{ purpose_det_rules }} | {{ purpose_det_passed }} | {{ purpose_det_failed }} | {{ purpose_det_errors }} | {{ purpose_det_warnings }} |
| e2e_testing | {{ e2e_det_rules }} | {{ e2e_det_passed }} | {{ e2e_det_failed }} | {{ e2e_det_errors }} | {{ e2e_det_warnings }} |
| smoke_testing | {{ smoke_det_rules }} | {{ smoke_det_passed }} | {{ smoke_det_failed }} | {{ smoke_det_errors }} | {{ smoke_det_warnings }} |
| load_testing | {{ load_det_rules }} | {{ load_det_passed }} | {{ load_det_failed }} | {{ load_det_errors }} | {{ load_det_warnings }} |
| scalability_testing | {{ scalability_det_rules }} | {{ scalability_det_passed }} | {{ scalability_det_failed }} | {{ scalability_det_errors }} | {{ scalability_det_warnings }} |

---

## Semantic Document Audit

### Criteria Results

| Criterion | Weight | Score | Status | Confidence |
|---|---|---|---|---|
| C1: Strategy-Depth Alignment | mandatory (40) | {{ sem_doc_c1_score }} | {{ sem_doc_c1_status }} | {{ sem_doc_c1_confidence }} |
| C2: Architecture Consistency | mandatory (30) | {{ sem_doc_c2_score }} | {{ sem_doc_c2_status }} | {{ sem_doc_c2_confidence }} |
| C3: Design Traceability | recommended (30) | {{ sem_doc_c3_score }} | {{ sem_doc_c3_status }} | {{ sem_doc_c3_confidence }} |

### Failures

| Criterion | Severity | Evidence |
|---|---|---|
{{ sem_doc_failures }}

---

## Semantic Section Audit

### Criteria Results by Section

| Section | C1 | C2 | C3 | Score | Status |
|---|---|---|---|---|---|
| test_strategy | {{ strategy_c1 }} | {{ strategy_c2 }} | {{ strategy_c3 }} | {{ strategy_sem_score }} | {{ strategy_sem_status }} |
| unit_testing | {{ unit_c1 }} | {{ unit_c2 }} | {{ unit_c3 }} | {{ unit_sem_score }} | {{ unit_sem_status }} |
| integration_testing | {{ integration_c1 }} | {{ integration_c2 }} | {{ integration_c3 }} | {{ integration_sem_score }} | {{ integration_sem_status }} |
| security_testing | {{ security_c1 }} | {{ security_c2 }} | {{ security_c3 }} | {{ security_sem_score }} | {{ security_sem_status }} |
| purpose | {{ purpose_c1 }} | {{ purpose_c2 }} | {{ purpose_c3 }} | {{ purpose_sem_score }} | {{ purpose_sem_status }} |
| e2e_testing | {{ e2e_c1 }} | {{ e2e_c2 }} | {{ e2e_c3 }} | {{ e2e_sem_score }} | {{ e2e_sem_status }} |
| smoke_testing | {{ smoke_c1 }} | {{ smoke_c2 }} | {{ smoke_c3 }} | {{ smoke_sem_score }} | {{ smoke_sem_status }} |
| load_testing | {{ load_c1 }} | {{ load_c2 }} | {{ load_c3 }} | {{ load_sem_score }} | {{ load_sem_status }} |
| scalability_testing | {{ scalability_c1 }} | {{ scalability_c2 }} | {{ scalability_c3 }} | {{ scalability_sem_score }} | {{ scalability_sem_status }} |

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
| test_strategy | {{ strategy_det_score }} | {{ strategy_sem_score }} | {{ strategy_combined }} | {{ strategy_overall_status }} |
| unit_testing | {{ unit_det_score }} | {{ unit_sem_score }} | {{ unit_combined }} | {{ unit_overall_status }} |
| integration_testing | {{ integration_det_score }} | {{ integration_sem_score }} | {{ integration_combined }} | {{ integration_overall_status }} |
| security_testing | {{ security_det_score }} | {{ security_sem_score }} | {{ security_combined }} | {{ security_overall_status }} |
| purpose | {{ purpose_det_score }} | {{ purpose_sem_score }} | {{ purpose_combined }} | {{ purpose_overall_status }} |
| e2e_testing | {{ e2e_det_score }} | {{ e2e_sem_score }} | {{ e2e_combined }} | {{ e2e_overall_status }} |
| smoke_testing | {{ smoke_det_score }} | {{ smoke_sem_score }} | {{ smoke_combined }} | {{ smoke_overall_status }} |
| load_testing | {{ load_det_score }} | {{ load_sem_score }} | {{ load_combined }} | {{ load_overall_status }} |
| scalability_testing | {{ scalability_det_score }} | {{ scalability_sem_score }} | {{ scalability_combined }} | {{ scalability_overall_status }} |
