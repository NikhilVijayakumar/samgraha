# {{ document_title }} — Product Guide Audit Summary

> **Domain:** product-guide
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

**Why this matters:** Product Guide is a flat content domain — no derivation relationships, no upstream dependencies to validate. The combined score aggregates structural rules (sections present, populated, one-topic), cross-section coherence (Title-Body alignment, Contract-Body alignment), and per-section quality (substantive, specific, consistent).

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
| title | {{ title_det_rules }} | {{ title_det_passed }} | {{ title_det_failed }} | {{ title_det_errors }} | {{ title_det_warnings }} |
| body | {{ body_det_rules }} | {{ body_det_passed }} | {{ body_det_failed }} | {{ body_det_errors }} | {{ body_det_warnings }} |
| purpose | {{ purpose_det_rules }} | {{ purpose_det_passed }} | {{ purpose_det_failed }} | {{ purpose_det_errors }} | {{ purpose_det_warnings }} |
| product_context | {{ context_det_rules }} | {{ context_det_passed }} | {{ context_det_failed }} | {{ context_det_errors }} | {{ context_det_warnings }} |
| public_contract | {{ contract_det_rules }} | {{ contract_det_passed }} | {{ contract_det_failed }} | {{ contract_det_errors }} | {{ contract_det_warnings }} |
| related | {{ related_det_rules }} | {{ related_det_passed }} | {{ related_det_failed }} | {{ related_det_errors }} | {{ related_det_warnings }} |

---

## Semantic Document Audit

### Criteria Results

| Criterion | Weight | Score | Status | Confidence |
|---|---|---|---|---|
| C1: Title-Body Alignment | mandatory (35) | {{ sem_doc_c1_score }} | {{ sem_doc_c1_status }} | {{ sem_doc_c1_confidence }} |
| C2: Contract-Body Alignment | mandatory (35) | {{ sem_doc_c2_score }} | {{ sem_doc_c2_status }} | {{ sem_doc_c2_confidence }} |
| C3: Terminology Consistency | recommended (30) | {{ sem_doc_c3_score }} | {{ sem_doc_c3_status }} | {{ sem_doc_c3_confidence }} |

### Failures

| Criterion | Severity | Evidence |
|---|---|---|
{{ sem_doc_failures }}

---

## Semantic Section Audit

### Scores by Section

| Section | Score | C1 Status | C2 Status | C3 Status |
|---|---|---|---|---|
| title | {{ title_sem_score }} | {{ title_c1_status }} | {{ title_c2_status }} | {{ title_c3_status }} |
| body | {{ body_sem_score }} | {{ body_c1_status }} | {{ body_c2_status }} | {{ body_c3_status }} |
| purpose | {{ purpose_sem_score }} | {{ purpose_c1_status }} | {{ purpose_c2_status }} | {{ purpose_c3_status }} |
| product_context | {{ context_sem_score }} | {{ context_c1_status }} | {{ context_c2_status }} | {{ context_c3_status }} |
| public_contract | {{ contract_sem_score }} | {{ contract_c1_status }} | {{ contract_c2_status }} | {{ contract_c3_status }} |
| related | {{ related_sem_score }} | {{ related_c1_status }} | {{ related_c2_status }} | {{ related_c3_status }} |

### Failures

| Section | Criterion | Severity | Evidence |
|---|---|---|---|
{{ sem_sec_failures }}

---

## Failures Summary

### Regressions from Previous Audit

| Rule | Previous Score | Current Score | Change | Evidence |
|---|---|---|---|---|
{{ regressions_table }}

### Persistent Failures

| Rule | First Failed | Consecutive Failures | Evidence |
|---|---|---|---|
{{ persistent_failures_table }}

---

## Document-Level Breakdown

| Category | Weight | Score | Percentage | Verdict |
|---|---|---|---|---|
| Structural Completeness | 3.0 | {{ structural_score }} | {{ structural_pct }} | {{ structural_verdict }} |
| Content Richness | 1.0 | {{ content_score }} | {{ content_pct }} | {{ content_verdict }} |
| Modularity | 0.5 | {{ modularity_score }} | {{ modularity_pct }} | {{ modularity_verdict }} |

---

## Section-Level Breakdown

| Section Category | Sections | Avg Score | Worst | Verdict |
|---|---|---|---|---|
| Content Sections | title, body | {{ content_avg }} | {{ content_worst }} | {{ content_verdict }} |
| Metadata Sections | purpose, product_context, public_contract, related | {{ metadata_avg }} | {{ metadata_worst }} | {{ metadata_verdict }} |

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ combined_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})
