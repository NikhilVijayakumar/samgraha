# {{ document_title }} — Product Guide Semantic Document Audit Report

> **Domain:** product-guide
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

**Why this matters:** Semantic document audit verifies that a Product Guide topic coheres internally — Title matches Body, Purpose aligns with Product Context, Public Contract matches what the Body actually describes — and that the guide collection as a whole doesn't contradict itself. Section-level audits check each section individually; this audit catches cross-section failures.

---

## Criterion Results

### C1 — Title accurately reflects Body content
- **Weight:** mandatory (35)
- **Score if passed:** 35
- **Status:** {{ c1_status }}
- **Confidence:** {{ c1_confidence }}
- **Evidence:** {{ c1_evidence }}
- **Why this matters:** If Title promises something the Body doesn't deliver, readers arrive with expectations the guide cannot satisfy. A predictive title is the first contract the guide makes with its reader.

### C2 — Public Contract matches what Body instructions actually use
- **Weight:** mandatory (35)
- **Score if passed:** 35
- **Status:** {{ c2_status }}
- **Confidence:** {{ c2_confidence }}
- **Evidence:** {{ c2_evidence }}
- **Why this matters:** If the Public Contract lists a flag never used in Body instructions, or Body uses a flag missing from the Contract, the ground-truth reference is wrong. Users checking the Contract before running commands will be misled.

### C3 — Terminology consistent across all Product Guide topics
- **Weight:** recommended (30)
- **Score if passed:** 30
- **Status:** {{ c3_status }}
- **Confidence:** {{ c3_confidence }}
- **Evidence:** {{ c3_evidence }}
- **Why this matters:** When two Product Guide topics use different names for the same concept, cross-references break and readers cannot tell whether the difference is intentional or a naming mistake.

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
