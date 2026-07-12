# {{ document_title }} — Build Semantic Section Audit Report

> **Domain:** build
> **Scope:** section
> **Kind:** semantic
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Section-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | {{ section_weight_sum }} |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | {{ section_weight_sum }} |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Semantic section audit evaluates the quality of each build section individually — whether content is substantive, internally consistent, and project-specific rather than generic. Each section contributes to the overall build policy coherence.

---

## Section: documentation_quality

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ doc_quality_c1_status }}
- **Confidence:** {{ doc_quality_c1_confidence }}
- **Evidence:** {{ doc_quality_c1_evidence }}
- **Why this matters:** A Documentation Quality section without project-specific content is a placeholder that provides no actual quality guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ doc_quality_c2_status }}
- **Confidence:** {{ doc_quality_c2_confidence }}
- **Evidence:** {{ doc_quality_c2_evidence }}
- **Why this matters:** Internally contradictory documentation quality rules create impossible compliance situations — auditors cannot satisfy both rules simultaneously.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ doc_quality_c3_status }}
- **Confidence:** {{ doc_quality_c3_confidence }}
- **Evidence:** {{ doc_quality_c3_evidence }}
- **Why this matters:** Generic quality standards ("be thorough") are unenforceable. Concrete examples give auditors a reference point for what "good" looks like.

---

## Section: security_checks

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ sec_c1_status }}
- **Confidence:** {{ sec_c1_confidence }}
- **Evidence:** {{ sec_c1_evidence }}
- **Why this matters:** Security Checks without project-specific scan definitions means builds proceed without knowing what threats are being checked.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ sec_c2_status }}
- **Confidence:** {{ sec_c2_confidence }}
- **Evidence:** {{ sec_c2_evidence }}
- **Why this matters:** Security Checks that contradict Versioning or other sections create enforcement gaps where threats slip through.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ sec_c3_status }}
- **Confidence:** {{ sec_c3_confidence }}
- **Evidence:** {{ sec_c3_evidence }}
- **Why this matters:** "Run security scans" without specifying which scans, what thresholds, or what failure means is operationally empty.

---

## Section: versioning_naming

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ version_c1_status }}
- **Confidence:** {{ version_c1_confidence }}
- **Evidence:** {{ version_c1_evidence }}
- **Why this matters:** Versioning without a defined scheme means releases have no traceable identity.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ version_c2_status }}
- **Confidence:** {{ version_c2_confidence }}
- **Evidence:** {{ version_c2_evidence }}
- **Why this matters:** Versioning that contradicts Engineering build standards creates confusion about which version scheme is authoritative.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ version_c3_status }}
- **Confidence:** {{ version_c3_confidence }}
- **Evidence:** {{ version_c3_evidence }}
- **Why this matters:** "Use semantic versioning" without showing the actual format or naming convention for this project is unimplementable.

---

## Section: purpose

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ purpose_c1_status }}
- **Confidence:** {{ purpose_c1_confidence }}
- **Evidence:** {{ purpose_c1_evidence }}
- **Why this matters:** Build documentation without a purpose section leaves readers guessing what this document is for and what it covers.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ purpose_c2_status }}
- **Confidence:** {{ purpose_c2_confidence }}
- **Evidence:** {{ purpose_c2_evidence }}
- **Why this matters:** Purpose that contradicts the actual scope of the document misleads readers about what is and isn't covered.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ purpose_c3_status }}
- **Confidence:** {{ purpose_c3_confidence }}
- **Evidence:** {{ purpose_c3_evidence }}
- **Why this matters:** A generic purpose statement ("this document describes the build") provides no value beyond what the filename already conveys.

---

## Section: size_checks

### Criteria

#### C1 — Measurable size limit defined per artifact type
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ size_c1_status }}
- **Confidence:** {{ size_c1_confidence }}
- **Evidence:** {{ size_c1_evidence }}
- **Why this matters:** "Keep the bundle small" with no numeric threshold is aspirational, not enforceable. Limits must be measurable per artifact type.

#### C2 — Measurement method specified
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ size_c2_status }}
- **Confidence:** {{ size_c2_confidence }}
- **Evidence:** {{ size_c2_evidence }}
- **Why this matters:** Size limits without a measurement method (gzip vs uncompressed, per-chunk vs total) are ambiguous — different measurement approaches yield different pass/fail outcomes.

#### C3 — Enforcement action stated for a breach
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ size_c3_status }}
- **Confidence:** {{ size_c3_confidence }}
- **Evidence:** {{ size_c3_evidence }}
- **Why this matters:** A size breach with no defined consequence (warn, block, require approval) is a policy with teeth removed.

---

## Section: ml_artifact_management

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ ml_c1_status }}
- **Confidence:** {{ ml_c1_confidence }}
- **Evidence:** {{ ml_c1_evidence }}
- **Why this matters:** ML artifact management without project-specific content means model packaging and distribution are undocumented.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ ml_c2_status }}
- **Confidence:** {{ ml_c2_confidence }}
- **Evidence:** {{ ml_c2_evidence }}
- **Why this matters:** ML artifact rules that contradict Size Checks or Versioning create unresolvable conflicts during build.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ ml_c3_status }}
- **Confidence:** {{ ml_c3_confidence }}
- **Evidence:** {{ ml_c3_evidence }}
- **Why this matters:** Generic ML artifact guidance without project-specific formats, paths, or naming conventions is operationally empty.

---

## Section: cicd_validation

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ cicd_c1_status }}
- **Confidence:** {{ cicd_c1_confidence }}
- **Evidence:** {{ cicd_c1_evidence }}
- **Why this matters:** CI/CD Validation without specific pipeline definitions means builds proceed without knowing what validation gates exist.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ cicd_c2_status }}
- **Confidence:** {{ cicd_c2_confidence }}
- **Evidence:** {{ cicd_c2_evidence }}
- **Why this matters:** CI/CD gates that contradict Security Checks or Documentation Quality create enforcement gaps.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ cicd_c3_status }}
- **Confidence:** {{ cicd_c3_confidence }}
- **Evidence:** {{ cicd_c3_evidence }}
- **Why this matters:** "Run CI/CD validation" without specifying which pipelines, what triggers, or what failure means is unimplementable.

---

## Section: obfuscation_optimization

### Criteria

#### C1 — Transformations specified per build type
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ obf_c1_status }}
- **Confidence:** {{ obf_c1_confidence }}
- **Evidence:** {{ obf_c1_evidence }}
- **Why this matters:** Obfuscation without per-build-type distinction means the same transformations run in dev and prod, breaking debuggability without justification.

#### C2 — Configuration/tooling referenced
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ obf_c2_status }}
- **Confidence:** {{ obf_c2_confidence }}
- **Evidence:** {{ obf_c2_evidence }}
- **Why this matters:** "We optimize the build" without naming the tool or configuration is unverifiable — there's nothing to check against.

#### C3 — Debuggability impact addressed
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ obf_c3_status }}
- **Confidence:** {{ obf_c3_confidence }}
- **Evidence:** {{ obf_c3_evidence }}
- **Why this matters:** Production obfuscation without addressing how to debug production issues (source maps, symbol retention) creates an operational blind spot.

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

| Criterion | Severity | Section | Evidence | Regression? |
|---|---|---|---|---|
{{ failures_table }}

---

## Summary

{{ summary_text }}

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| documentation_quality | {{ doc_quality_weight }} | {{ doc_quality_score }} | {{ doc_quality_status }} |
| security_checks | {{ sec_weight }} | {{ sec_score }} | {{ sec_status }} |
| versioning_naming | {{ version_weight }} | {{ version_score }} | {{ version_status }} |
| purpose | {{ purpose_weight }} | {{ purpose_score }} | {{ purpose_status }} |
| size_checks | {{ size_weight }} | {{ size_score }} | {{ size_status }} |
| ml_artifact_management | {{ ml_weight }} | {{ ml_score }} | {{ ml_status }} |
| cicd_validation | {{ cicd_weight }} | {{ cicd_score }} | {{ cicd_status }} |
| obfuscation_optimization | {{ obf_weight }} | {{ obf_score }} | {{ obf_status }} |
