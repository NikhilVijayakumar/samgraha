# {{ document_title }} — README Semantic Document Audit Report

> **Domain:** readme
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

**Why this matters:** Semantic document audit verifies that the collection of README sections coheres as one accurate entry point. Section-level audits check each section individually; this audit catches cross-section contradictions — Usage examples that don't match Installation commands, or Development that contradicts Contributing.

---

## Criterion Results

### C1 — Usage examples consistent with Build/Installation instructions
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ c1_status }}
- **Confidence:** {{ c1_confidence }}
- **Evidence:** {{ c1_evidence }}
- **Why this matters:** If Usage shows a command that doesn't match the binary installed per Installation, the README is internally contradictory — readers follow the steps and hit errors.

### C2 — Development and Contributing describe a consistent workflow
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ c2_status }}
- **Confidence:** {{ c2_confidence }}
- **Evidence:** {{ c2_evidence }}
- **Why this matters:** If Development describes one test command and Contributing implies a different one is what CI actually runs, contributors don't know which to follow.

### C3 — Terminology (commands, flags, names) consistent throughout
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ c3_status }}
- **Confidence:** {{ c3_confidence }}
- **Evidence:** {{ c3_evidence }}
- **Why this matters:** When the same command is named differently across sections, readers cannot tell whether the difference is intentional or a typo.

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
| Usage-Build Consistency | 40 | {{ c1_score }} | {{ c1_status }} |
| Dev-Contrib Consistency | 30 | {{ c2_score }} | {{ c2_status }} |
| Terminology Consistency | 30 | {{ c3_score }} | {{ c3_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| project_name | {{ name_section_score }} | {{ name_section_weight }} | {{ name_section_status }} |
| short_description | {{ short_desc_section_score }} | {{ short_desc_section_weight }} | {{ short_desc_section_status }} |
| overview | {{ overview_section_score }} | {{ overview_section_weight }} | {{ overview_section_status }} |
| purpose | {{ purpose_section_score }} | {{ purpose_section_weight }} | {{ purpose_section_status }} |
| key_capabilities | {{ caps_section_score }} | {{ caps_section_weight }} | {{ caps_section_status }} |
| repository_structure | {{ repo_section_score }} | {{ repo_section_weight }} | {{ repo_section_status }} |
| documentation_structure | {{ doc_struct_section_score }} | {{ doc_struct_section_weight }} | {{ doc_struct_section_status }} |
| getting_started | {{ gs_section_score }} | {{ gs_section_weight }} | {{ gs_section_status }} |
| installation | {{ install_section_score }} | {{ install_section_weight }} | {{ install_section_status }} |
| build | {{ build_section_score }} | {{ build_section_weight }} | {{ build_section_status }} |
| usage | {{ usage_section_score }} | {{ usage_section_weight }} | {{ usage_section_status }} |
| development | {{ dev_section_score }} | {{ dev_section_weight }} | {{ dev_section_status }} |
| contributing | {{ contrib_section_score }} | {{ contrib_section_weight }} | {{ contrib_section_status }} |
| configuration | {{ config_section_score }} | {{ config_section_weight }} | {{ config_section_status }} |
| license | {{ license_section_score }} | {{ license_section_weight }} | {{ license_section_status }} |
