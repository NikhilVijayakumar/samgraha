# {{ document_title }} — Build Semantic Document Audit Report

> **Domain:** build
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

**Why this matters:** Semantic document audit verifies that the collection of build sections coheres as one policy. Section-level audits check each section individually; this audit catches cross-section contradictions and drift that only surface when sections are read together.

---

## Criterion Results

### C1 — Security Checks and Versioning & Naming are mutually consistent
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ c1_status }}
- **Confidence:** {{ c1_confidence }}
- **Evidence:** {{ c1_evidence }}
- **Why this matters:** A severity threshold that blocks a release must not be contradicted by a versioning rule that allows shipping anyway. If these two sections disagree, builds ship with unvetted vulnerabilities.

### C2 — Documentation Quality's claimed gates match what's actually enforced
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ c2_status }}
- **Confidence:** {{ c2_confidence }}
- **Evidence:** {{ c2_evidence }}
- **Why this matters:** If Documentation Quality claims a gate exists that CI/CD Validation doesn't implement, the gate is theater — it provides false assurance without actual enforcement.

### C3 — Terminology (artifact/gate names) consistent across all sections and documents
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ c3_status }}
- **Confidence:** {{ c3_confidence }}
- **Evidence:** {{ c3_evidence }}
- **Why this matters:** When the same artifact is called "bundle" in one section and "package" in another, readers cannot tell whether the difference is intentional or accidental. Ambiguous naming creates audit blind spots.

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
| Security-Versioning Consistency | 40 | {{ c1_score }} | {{ c1_status }} |
| Gate Enforcement Alignment | 30 | {{ c2_score }} | {{ c2_status }} |
| Terminology Consistency | 30 | {{ c3_score }} | {{ c3_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| documentation_quality | {{ doc_quality_section_score }} | {{ doc_quality_section_weight }} | {{ doc_quality_section_status }} |
| security_checks | {{ sec_section_score }} | {{ sec_section_weight }} | {{ sec_section_status }} |
| versioning_naming | {{ version_section_score }} | {{ version_section_weight }} | {{ version_section_status }} |
| purpose | {{ purpose_section_score }} | {{ purpose_section_weight }} | {{ purpose_section_status }} |
| size_checks | {{ size_section_score }} | {{ size_section_weight }} | {{ size_section_status }} |
| ml_artifact_management | {{ ml_section_score }} | {{ ml_section_weight }} | {{ ml_section_status }} |
| cicd_validation | {{ cicd_section_score }} | {{ cicd_section_weight }} | {{ cicd_section_status }} |
| obfuscation_optimization | {{ obf_section_score }} | {{ obf_section_weight }} | {{ obf_section_status }} |
