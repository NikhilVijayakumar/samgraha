# {{ document_title }} — README Audit Summary

> **Domain:** readme
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
| project_name | {{ name_det_rules }} | {{ name_det_passed }} | {{ name_det_failed }} | {{ name_det_errors }} | {{ name_det_warnings }} |
| short_description | {{ short_desc_det_rules }} | {{ short_desc_det_passed }} | {{ short_desc_det_failed }} | {{ short_desc_det_errors }} | {{ short_desc_det_warnings }} |
| overview | {{ overview_det_rules }} | {{ overview_det_passed }} | {{ overview_det_failed }} | {{ overview_det_errors }} | {{ overview_det_warnings }} |
| purpose | {{ purpose_det_rules }} | {{ purpose_det_passed }} | {{ purpose_det_failed }} | {{ purpose_det_errors }} | {{ purpose_det_warnings }} |
| key_capabilities | {{ caps_det_rules }} | {{ caps_det_passed }} | {{ caps_det_failed }} | {{ caps_det_errors }} | {{ caps_det_warnings }} |
| repository_structure | {{ repo_det_rules }} | {{ repo_det_passed }} | {{ repo_det_failed }} | {{ repo_det_errors }} | {{ repo_det_warnings }} |
| documentation_structure | {{ doc_struct_det_rules }} | {{ doc_struct_det_passed }} | {{ doc_struct_det_failed }} | {{ doc_struct_det_errors }} | {{ doc_struct_det_warnings }} |
| getting_started | {{ gs_det_rules }} | {{ gs_det_passed }} | {{ gs_det_failed }} | {{ gs_det_errors }} | {{ gs_det_warnings }} |
| installation | {{ install_det_rules }} | {{ install_det_passed }} | {{ install_det_failed }} | {{ install_det_errors }} | {{ install_det_warnings }} |
| build | {{ build_det_rules }} | {{ build_det_passed }} | {{ build_det_failed }} | {{ build_det_errors }} | {{ build_det_warnings }} |
| usage | {{ usage_det_rules }} | {{ usage_det_passed }} | {{ usage_det_failed }} | {{ usage_det_errors }} | {{ usage_det_warnings }} |
| development | {{ dev_det_rules }} | {{ dev_det_passed }} | {{ dev_det_failed }} | {{ dev_det_errors }} | {{ dev_det_warnings }} |
| contributing | {{ contrib_det_rules }} | {{ contrib_det_passed }} | {{ contrib_det_failed }} | {{ contrib_det_errors }} | {{ contrib_det_warnings }} |
| configuration | {{ config_det_rules }} | {{ config_det_passed }} | {{ config_det_failed }} | {{ config_det_errors }} | {{ config_det_warnings }} |
| license | {{ license_det_rules }} | {{ license_det_passed }} | {{ license_det_failed }} | {{ license_det_errors }} | {{ license_det_warnings }} |

---

## Semantic Document Audit

### Criteria Results

| Criterion | Weight | Score | Status | Confidence |
|---|---|---|---|---|
| C1: Usage-Build Consistency | mandatory (40) | {{ sem_doc_c1_score }} | {{ sem_doc_c1_status }} | {{ sem_doc_c1_confidence }} |
| C2: Dev-Contrib Consistency | mandatory (30) | {{ sem_doc_c2_score }} | {{ sem_doc_c2_status }} | {{ sem_doc_c2_confidence }} |
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
| project_name | {{ name_c1 }} | {{ name_c2 }} | {{ name_c3 }} | {{ name_sem_score }} | {{ name_sem_status }} |
| short_description | {{ short_desc_c1 }} | {{ short_desc_c2 }} | {{ short_desc_c3 }} | {{ short_desc_sem_score }} | {{ short_desc_sem_status }} |
| overview | {{ overview_c1 }} | {{ overview_c2 }} | {{ overview_c3 }} | {{ overview_sem_score }} | {{ overview_sem_status }} |
| purpose | {{ purpose_c1 }} | {{ purpose_c2 }} | {{ purpose_c3 }} | {{ purpose_sem_score }} | {{ purpose_sem_status }} |
| key_capabilities | {{ caps_c1 }} | {{ caps_c2 }} | {{ caps_c3 }} | {{ caps_sem_score }} | {{ caps_sem_status }} |
| repository_structure | {{ repo_c1 }} | {{ repo_c2 }} | {{ repo_c3 }} | {{ repo_sem_score }} | {{ repo_sem_status }} |
| documentation_structure | {{ doc_struct_c1 }} | {{ doc_struct_c2 }} | {{ doc_struct_c3 }} | {{ doc_struct_sem_score }} | {{ doc_struct_sem_status }} |
| getting_started | {{ gs_c1 }} | {{ gs_c2 }} | {{ gs_c3 }} | {{ gs_sem_score }} | {{ gs_sem_status }} |
| installation | {{ install_c1 }} | {{ install_c2 }} | {{ install_c3 }} | {{ install_sem_score }} | {{ install_sem_status }} |
| build | {{ build_c1 }} | {{ build_c2 }} | {{ build_c3 }} | {{ build_sem_score }} | {{ build_sem_status }} |
| usage | {{ usage_c1 }} | {{ usage_c2 }} | {{ usage_c3 }} | {{ usage_sem_score }} | {{ usage_sem_status }} |
| development | {{ dev_c1 }} | {{ dev_c2 }} | {{ dev_c3 }} | {{ dev_sem_score }} | {{ dev_sem_status }} |
| contributing | {{ contrib_c1 }} | {{ contrib_c2 }} | {{ contrib_c3 }} | {{ contrib_sem_score }} | {{ contrib_sem_status }} |
| configuration | {{ config_c1 }} | {{ config_c2 }} | {{ config_c3 }} | {{ config_sem_score }} | {{ config_sem_status }} |
| license | {{ license_c1 }} | {{ license_c2 }} | {{ license_c3 }} | {{ license_sem_score }} | {{ license_sem_status }} |

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
| project_name | {{ name_det_score }} | {{ name_sem_score }} | {{ name_combined }} | {{ name_overall_status }} |
| short_description | {{ short_desc_det_score }} | {{ short_desc_sem_score }} | {{ short_desc_combined }} | {{ short_desc_overall_status }} |
| overview | {{ overview_det_score }} | {{ overview_sem_score }} | {{ overview_combined }} | {{ overview_overall_status }} |
| purpose | {{ purpose_det_score }} | {{ purpose_sem_score }} | {{ purpose_combined }} | {{ purpose_overall_status }} |
| key_capabilities | {{ caps_det_score }} | {{ caps_sem_score }} | {{ caps_combined }} | {{ caps_overall_status }} |
| repository_structure | {{ repo_det_score }} | {{ repo_sem_score }} | {{ repo_combined }} | {{ repo_overall_status }} |
| documentation_structure | {{ doc_struct_det_score }} | {{ doc_struct_sem_score }} | {{ doc_struct_combined }} | {{ doc_struct_overall_status }} |
| getting_started | {{ gs_det_score }} | {{ gs_sem_score }} | {{ gs_combined }} | {{ gs_overall_status }} |
| installation | {{ install_det_score }} | {{ install_sem_score }} | {{ install_combined }} | {{ install_overall_status }} |
| build | {{ build_det_score }} | {{ build_sem_score }} | {{ build_combined }} | {{ build_overall_status }} |
| usage | {{ usage_det_score }} | {{ usage_sem_score }} | {{ usage_combined }} | {{ usage_overall_status }} |
| development | {{ dev_det_score }} | {{ dev_sem_score }} | {{ dev_combined }} | {{ dev_overall_status }} |
| contributing | {{ contrib_det_score }} | {{ contrib_sem_score }} | {{ contrib_combined }} | {{ contrib_overall_status }} |
| configuration | {{ config_det_score }} | {{ config_sem_score }} | {{ config_combined }} | {{ config_overall_status }} |
| license | {{ license_det_score }} | {{ license_sem_score }} | {{ license_combined }} | {{ license_overall_status }} |
