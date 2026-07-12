# {{ document_title }} — QA Semantic Document Audit Report

> **Domain:** qa
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

**Why this matters:** Semantic document audit verifies that the collection of QA sections coheres as one test strategy. Section-level audits check each test type individually; this audit catches cross-section contradictions — a test strategy that claims security is highest priority but has a thin security testing section.

---

## Criterion Results

### C1 — Test Strategy's priorities are reflected in the depth of each test type section
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ c1_status }}
- **Confidence:** {{ c1_confidence }}
- **Evidence:** {{ c1_evidence }}
- **Why this matters:** If Test Strategy marks Security Testing as highest priority but the Security Testing section is one sentence, the strategy is aspirational, not operational.

### C2 — Integration Testing boundaries consistent with Architecture(05)
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ c2_status }}
- **Confidence:** {{ c2_confidence }}
- **Evidence:** {{ c2_evidence }}
- **Why this matters:** Integration Testing that tests boundaries Architecture doesn't define is testing phantom interfaces. Integration Testing that ignores Architecture-defined boundaries leaves real interfaces untested.

### C3 — E2E journeys traceable to Design(06) workflows
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ c3_status }}
- **Confidence:** {{ c3_confidence }}
- **Evidence:** {{ c3_evidence }}
- **Why this matters:** E2E journeys invented independently of Design workflows have no upstream source of truth — they test assumptions, not documented user expectations.

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
| Strategy-Depth Alignment | 40 | {{ c1_score }} | {{ c1_status }} |
| Architecture Consistency | 30 | {{ c2_score }} | {{ c2_status }} |
| Design Traceability | 30 | {{ c3_score }} | {{ c3_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| test_strategy | {{ strategy_section_score }} | {{ strategy_section_weight }} | {{ strategy_section_status }} |
| unit_testing | {{ unit_section_score }} | {{ unit_section_weight }} | {{ unit_section_status }} |
| integration_testing | {{ integration_section_score }} | {{ integration_section_weight }} | {{ integration_section_status }} |
| security_testing | {{ security_section_score }} | {{ security_section_weight }} | {{ security_section_status }} |
| purpose | {{ purpose_section_score }} | {{ purpose_section_weight }} | {{ purpose_section_status }} |
| e2e_testing | {{ e2e_section_score }} | {{ e2e_section_weight }} | {{ e2e_section_status }} |
| smoke_testing | {{ smoke_section_score }} | {{ smoke_section_weight }} | {{ smoke_section_status }} |
| load_testing | {{ load_section_score }} | {{ load_section_weight }} | {{ load_section_status }} |
| scalability_testing | {{ scalability_section_score }} | {{ scalability_section_weight }} | {{ scalability_section_status }} |
