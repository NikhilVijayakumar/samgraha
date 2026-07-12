# {{ document_title }} — Feature Semantic Document Audit Report

> **Domain:** feature
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

**Why this matters:** Semantic document audit verifies that the collection of feature sections coheres as one specification. Section-level audits check each section individually; this audit catches cross-section contradictions — a Functional Requirement with no Acceptance Criterion, or an Acceptance Criterion that violates a Business Rule.

---

## Criterion Results

### C1 — Functional Requirements, Acceptance Criteria, and Business Rules are mutually consistent
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ c1_status }}
- **Confidence:** {{ c1_confidence }}
- **Evidence:** {{ c1_evidence }}
- **Why this matters:** If an Acceptance Criterion would pass while violating a stated Business Rule, the feature specification is internally contradictory — developers cannot satisfy both simultaneously.

### C2 — Every Functional Requirement has a corresponding Acceptance Criterion
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ c2_status }}
- **Confidence:** {{ c2_confidence }}
- **Evidence:** {{ c2_evidence }}
- **Why this matters:** A Functional Requirement with no Acceptance Criterion is unverifiable — there's no way to confirm the requirement is met, and it silently ships unchecked.

### C3 — Terminology consistent across all Feature documents
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ c3_status }}
- **Confidence:** {{ c3_confidence }}
- **Evidence:** {{ c3_evidence }}
- **Why this matters:** When the same entity is called "order" in Functional Requirements and "purchase" in Acceptance Criteria, readers cannot tell whether the difference is intentional or accidental.

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
| FR-AC-BR Consistency | 40 | {{ c1_score }} | {{ c1_status }} |
| Requirement Coverage | 30 | {{ c2_score }} | {{ c2_status }} |
| Terminology Consistency | 30 | {{ c3_score }} | {{ c3_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| purpose | {{ purpose_section_score }} | {{ purpose_section_weight }} | {{ purpose_section_status }} |
| functional_requirements | {{ freq_section_score }} | {{ freq_section_weight }} | {{ freq_section_status }} |
| acceptance_criteria | {{ ac_section_score }} | {{ ac_section_weight }} | {{ ac_section_status }} |
| business_rules | {{ br_section_score }} | {{ br_section_weight }} | {{ br_section_status }} |
| inputs | {{ inputs_section_score }} | {{ inputs_section_weight }} | {{ inputs_section_status }} |
| outputs | {{ outputs_section_score }} | {{ outputs_section_weight }} | {{ outputs_section_status }} |
| constraints | {{ constraints_section_score }} | {{ constraints_section_weight }} | {{ constraints_section_status }} |
| dependencies | {{ dependencies_section_score }} | {{ dependencies_section_weight }} | {{ dependencies_section_status }} |
| non_goals | {{ non_goals_section_score }} | {{ non_goals_section_weight }} | {{ non_goals_section_status }} |
| future_extensions | {{ future_section_score }} | {{ future_section_weight }} | {{ future_section_status }} |
| traceability | {{ trace_section_score }} | {{ trace_section_weight }} | {{ trace_section_status }} |
