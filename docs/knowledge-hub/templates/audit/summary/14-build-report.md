# {{ document_title }} — Build Audit Summary

> **Domain:** build
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
| documentation_quality | {{ doc_quality_det_rules }} | {{ doc_quality_det_passed }} | {{ doc_quality_det_failed }} | {{ doc_quality_det_errors }} | {{ doc_quality_det_warnings }} |
| security_checks | {{ sec_det_rules }} | {{ sec_det_passed }} | {{ sec_det_failed }} | {{ sec_det_errors }} | {{ sec_det_warnings }} |
| versioning_naming | {{ version_det_rules }} | {{ version_det_passed }} | {{ version_det_failed }} | {{ version_det_errors }} | {{ version_det_warnings }} |
| purpose | {{ purpose_det_rules }} | {{ purpose_det_passed }} | {{ purpose_det_failed }} | {{ purpose_det_errors }} | {{ purpose_det_warnings }} |
| size_checks | {{ size_det_rules }} | {{ size_det_passed }} | {{ size_det_failed }} | {{ size_det_errors }} | {{ size_det_warnings }} |
| ml_artifact_management | {{ ml_det_rules }} | {{ ml_det_passed }} | {{ ml_det_failed }} | {{ ml_det_errors }} | {{ ml_det_warnings }} |
| cicd_validation | {{ cicd_det_rules }} | {{ cicd_det_passed }} | {{ cicd_det_failed }} | {{ cicd_det_errors }} | {{ cicd_det_warnings }} |
| obfuscation_optimization | {{ obf_det_rules }} | {{ obf_det_passed }} | {{ obf_det_failed }} | {{ obf_det_errors }} | {{ obf_det_warnings }} |

---

## Semantic Document Audit

### Criteria Results

| Criterion | Weight | Score | Status | Confidence |
|---|---|---|---|---|
| C1: Security-Versioning Consistency | mandatory (40) | {{ sem_doc_c1_score }} | {{ sem_doc_c1_status }} | {{ sem_doc_c1_confidence }} |
| C2: Gate Enforcement Alignment | mandatory (30) | {{ sem_doc_c2_score }} | {{ sem_doc_c2_status }} | {{ sem_doc_c2_confidence }} |
| C3: Terminology Consistency | recommended (30) | {{ sem_doc_c3_score }} | {{ sem_doc_c3_status }} | {{ sem_doc_c3_confidence }} |

### Failures

| Criterion | Severity | Evidence |
|---|---|---|
{{ sem_doc_failures }}

---

## Semantic Section Audit

### Criteria Results by Section

| Section | C1 (40) | C2 (30) | C3 (30) | Score | Status |
|---|---|---|---|---|---|
| documentation_quality | {{ doc_quality_c1 }} | {{ doc_quality_c2 }} | {{ doc_quality_c3 }} | {{ doc_quality_sem_score }} | {{ doc_quality_sem_status }} |
| security_checks | {{ sec_c1 }} | {{ sec_c2 }} | {{ sec_c3 }} | {{ sec_sem_score }} | {{ sec_sem_status }} |
| versioning_naming | {{ version_c1 }} | {{ version_c2 }} | {{ version_c3 }} | {{ version_sem_score }} | {{ version_sem_status }} |
| purpose | {{ purpose_c1 }} | {{ purpose_c2 }} | {{ purpose_c3 }} | {{ purpose_sem_score }} | {{ purpose_sem_status }} |
| size_checks | {{ size_c1 }} | {{ size_c2 }} | {{ size_c3 }} | {{ size_sem_score }} | {{ size_sem_status }} |
| ml_artifact_management | {{ ml_c1 }} | {{ ml_c2 }} | {{ ml_c3 }} | {{ ml_sem_score }} | {{ ml_sem_status }} |
| cicd_validation | {{ cicd_c1 }} | {{ cicd_c2 }} | {{ cicd_c3 }} | {{ cicd_sem_score }} | {{ cicd_sem_status }} |
| obfuscation_optimization | {{ obf_c1 }} | {{ obf_c2 }} | {{ obf_c3 }} | {{ obf_sem_score }} | {{ obf_sem_status }} |

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
| documentation_quality | {{ doc_quality_det_score }} | {{ doc_quality_sem_score }} | {{ doc_quality_combined }} | {{ doc_quality_overall_status }} |
| security_checks | {{ sec_det_score }} | {{ sec_sem_score }} | {{ sec_combined }} | {{ sec_overall_status }} |
| versioning_naming | {{ version_det_score }} | {{ version_sem_score }} | {{ version_combined }} | {{ version_overall_status }} |
| purpose | {{ purpose_det_score }} | {{ purpose_sem_score }} | {{ purpose_combined }} | {{ purpose_overall_status }} |
| size_checks | {{ size_det_score }} | {{ size_sem_score }} | {{ size_combined }} | {{ size_overall_status }} |
| ml_artifact_management | {{ ml_det_score }} | {{ ml_sem_score }} | {{ ml_combined }} | {{ ml_overall_status }} |
| cicd_validation | {{ cicd_det_score }} | {{ cicd_sem_score }} | {{ cicd_combined }} | {{ cicd_overall_status }} |
| obfuscation_optimization | {{ obf_det_score }} | {{ obf_sem_score }} | {{ obf_combined }} | {{ obf_overall_status }} |
