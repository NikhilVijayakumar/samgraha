# {{ document_title }} — Build Section Audit Report

> **Domain:** build
> **Scope:** section
> **Standard:** documentation-standards
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

**Why this matters:** Build sections define specific packaging, validation, and distribution rules. Section-level audits verify each concern is internally consistent and substantiated — the building blocks of a coherent build policy.

---

## Section: documentation_quality

### Rules

#### build-sec-doc-quality-001 — Documentation quality section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'documentation_quality'
- **Status:** {{ doc_quality_001_status }}
- **Evidence:** {{ doc_quality_001_evidence }}

#### build-sec-doc-quality-002 — Documentation quality defines standards
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section specifies documentation quality standards or requirements
- **Status:** {{ doc_quality_002_status }}
- **Evidence:** {{ doc_quality_002_evidence }}

#### build-sec-doc-quality-003 — Documentation quality defines checks
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section specifies quality checks or validation steps
- **Status:** {{ doc_quality_003_status }}
- **Evidence:** {{ doc_quality_003_evidence }}

#### build-sec-doc-quality-004 — Documentation quality references implementation
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Implementation Documentation generation plan
- **Status:** {{ doc_quality_004_status }}
- **Evidence:** {{ doc_quality_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| build-doc-quality-derives-implementation | derives_from | implementation:generation_plan | incoming | {{ rel_doc_quality_impl }} |

---

## Section: security_checks

### Rules

#### build-sec-security-001 — Security checks section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'security_checks'
- **Status:** {{ sec_001_status }}
- **Evidence:** {{ sec_001_evidence }}

#### build-sec-security-002 — Security checks define scan steps
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section specifies security scan or audit steps to run during build
- **Status:** {{ sec_002_status }}
- **Evidence:** {{ sec_002_evidence }}

#### build-sec-security-003 — Security checks define failure criteria
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section specifies what conditions cause build failure
- **Status:** {{ sec_003_status }}
- **Evidence:** {{ sec_003_evidence }}

#### build-sec-security-004 — Security checks reference security standard
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Security Documentation mitigation strategies
- **Status:** {{ sec_004_status }}
- **Evidence:** {{ sec_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| build-security-checks-derives-security | derives_from | security:mitigation_strategies | incoming | {{ rel_sec_security }} |

---

## Section: versioning_naming

### Rules

#### build-sec-version-001 — Versioning naming section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'versioning_naming'
- **Status:** {{ version_001_status }}
- **Evidence:** {{ version_001_evidence }}

#### build-sec-version-002 — Versioning naming defines scheme
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section specifies versioning scheme (semver, calver, etc.)
- **Status:** {{ version_002_status }}
- **Evidence:** {{ version_002_evidence }}

#### build-sec-version-003 — Versioning naming defines conventions
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section specifies naming conventions for artifacts or releases
- **Status:** {{ version_003_status }}
- **Evidence:** {{ version_003_evidence }}

#### build-sec-version-004 — Versioning naming references engineering
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Engineering Documentation build standards
- **Status:** {{ version_004_status }}
- **Evidence:** {{ version_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| build-versioning-derives-engineering | derives_from | engineering:build_standards | incoming | {{ rel_version_engineering }} |

---

## Section: purpose

### Rules

#### build-sec-purpose-001 — Purpose section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'purpose'
- **Status:** {{ purpose_001_status }}
- **Evidence:** {{ purpose_001_evidence }}

#### build-sec-purpose-002 — Purpose states build intent
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains a statement of why Build Documentation exists
- **Status:** {{ purpose_002_status }}
- **Evidence:** {{ purpose_002_evidence }}

#### build-sec-purpose-003 — Purpose defines scope boundaries
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section defines what Build Documentation is and is not
- **Status:** {{ purpose_003_status }}
- **Evidence:** {{ purpose_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| build-purpose-derives-vision | derives_from | vision:purpose | incoming | {{ rel_purpose_vision }} |

---

## Section: size_checks

### Rules

#### build-sec-size_checks-001 — Size Checks section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'size_checks'
- **Status:** {{ size_001_status }}
- **Evidence:** {{ size_001_evidence }}

#### build-sec-size_checks-002 — Size Checks has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ size_002_status }}
- **Evidence:** {{ size_002_evidence }}

#### build-sec-size_checks-003 — Size Checks is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ size_003_status }}
- **Evidence:** {{ size_003_evidence }}

---

## Section: ml_artifact_management

### Rules

#### build-sec-ml_artifact_management-001 — Ml Artifact Management section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'ml_artifact_management'
- **Status:** {{ ml_001_status }}
- **Evidence:** {{ ml_001_evidence }}

#### build-sec-ml_artifact_management-002 — Ml Artifact Management has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ ml_002_status }}
- **Evidence:** {{ ml_002_evidence }}

#### build-sec-ml_artifact_management-003 — Ml Artifact Management is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ ml_003_status }}
- **Evidence:** {{ ml_003_evidence }}

---

## Section: cicd_validation

### Rules

#### build-sec-cicd_validation-001 — Cicd Validation section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'cicd_validation'
- **Status:** {{ cicd_001_status }}
- **Evidence:** {{ cicd_001_evidence }}

#### build-sec-cicd_validation-002 — Cicd Validation has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ cicd_002_status }}
- **Evidence:** {{ cicd_002_evidence }}

#### build-sec-cicd_validation-003 — Cicd Validation is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ cicd_003_status }}
- **Evidence:** {{ cicd_003_evidence }}

---

## Section: obfuscation_optimization

### Rules

#### build-sec-obfuscation_optimization-001 — Obfuscation Optimization section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'obfuscation_optimization'
- **Status:** {{ obf_001_status }}
- **Evidence:** {{ obf_001_evidence }}

#### build-sec-obfuscation_optimization-002 — Obfuscation Optimization has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ obf_002_status }}
- **Evidence:** {{ obf_002_evidence }}

#### build-sec-obfuscation_optimization-003 — Obfuscation Optimization is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ obf_003_status }}
- **Evidence:** {{ obf_003_evidence }}

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

| Rule | Severity | Section | Evidence | Regression? |
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
