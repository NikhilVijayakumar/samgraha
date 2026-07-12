# {{ document_title }} — Feature Technical Semantic Document Audit Report

> **Domain:** feature-technical
> **Scope:** document
> **Kind:** semantic
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Document-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | 100 |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | 100 |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Semantic document audit verifies that a Feature Technical Design document coheres internally — Participating Components, Component Interactions, Data Ownership, and Runtime Behavior must describe one consistent realization of the feature, not four independently-written sections. Section-level audits check each section individually; this audit catches cross-section failures.

---

## Criterion Results

### C1 — Participating Components, Component Interactions, Data Ownership, and Runtime Behavior are mutually consistent
- **Weight:** mandatory (40)
- **Score if passed:** 40
- **Status:** {{ c1_status }}
- **Confidence:** {{ c1_confidence }}
- **Evidence:** {{ c1_evidence }}
- **Why this matters:** A component referenced in Component Interactions that Participating Components doesn't list, or a data owner in Data Ownership that Runtime Behavior contradicts, means the technical design is internally inconsistent — implementers cannot trust any single section.

### C2 — Terminology (component/data names) consistent across all sections and documents
- **Weight:** mandatory (30)
- **Score if passed:** 30
- **Status:** {{ c2_status }}
- **Confidence:** {{ c2_confidence }}
- **Evidence:** {{ c2_evidence }}
- **Why this matters:** When the same component or data entity is named differently across sections, implementers cannot tell whether the difference is intentional or a naming mistake.

### C3 — All Feature Technical Design documents cohere as one system
- **Weight:** recommended (30)
- **Score if passed:** 30
- **Status:** {{ c3_status }}
- **Confidence:** {{ c3_confidence }}
- **Evidence:** {{ c3_evidence }}
- **Why this matters:** Two Feature Technical Design documents that realize the same feature with contradictory component responsibilities create an impossible implementation target.

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

| Criterion | Severity | Weight | Evidence |
|---|---|---|---|
{{ failures_table }}
