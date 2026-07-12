# {{ document_title }} — QA Section Audit Report

> **Domain:** qa
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

**Why this matters:** QA sections define specific testing rules for each testing level. Section-level audits verify each concern is internally consistent and substantiated — the building blocks of a coherent test strategy.

---

## Section: test_strategy

### Rules

#### qa-sec-strategy-001 — Test strategy section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'test_strategy'
- **Status:** {{ strategy_001_status }}
- **Evidence:** {{ strategy_001_evidence }}

#### qa-sec-strategy-002 — Test strategy defines approach
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains a statement of the overall testing approach and philosophy
- **Status:** {{ strategy_002_status }}
- **Evidence:** {{ strategy_002_evidence }}

#### qa-sec-strategy-003 — Test strategy defines coverage goals
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section specifies target coverage metrics or thresholds
- **Status:** {{ strategy_003_status }}
- **Evidence:** {{ strategy_003_evidence }}

#### qa-sec-strategy-004 — Test strategy references architecture
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Architecture Documentation for test scope
- **Status:** {{ strategy_004_status }}
- **Evidence:** {{ strategy_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| qa-strategy-derives-architecture | derives_from | architecture:system_overview | incoming | {{ rel_strategy_arch }} |

---

## Section: unit_testing

### Rules

#### qa-sec-unit-001 — Unit testing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'unit_testing'
- **Status:** {{ unit_001_status }}
- **Evidence:** {{ unit_001_evidence }}

#### qa-sec-unit-002 — Unit testing defines scope
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section specifies what components or modules are covered by unit tests
- **Status:** {{ unit_002_status }}
- **Evidence:** {{ unit_002_evidence }}

#### qa-sec-unit-003 — Unit testing defines conventions
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section specifies naming conventions, structure, or patterns for unit tests
- **Status:** {{ unit_003_status }}
- **Evidence:** {{ unit_003_evidence }}

#### qa-sec-unit-004 — Unit testing references engineering standards
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Engineering Documentation testing standards
- **Status:** {{ unit_004_status }}
- **Evidence:** {{ unit_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| qa-unit-derives-engineering | derives_from | engineering:testing_standards | incoming | {{ rel_unit_engineering }} |

---

## Section: integration_testing

### Rules

#### qa-sec-integration-001 — Integration testing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'integration_testing'
- **Status:** {{ integration_001_status }}
- **Evidence:** {{ integration_001_evidence }}

#### qa-sec-integration-002 — Integration testing defines interfaces
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section specifies which component interfaces are tested
- **Status:** {{ integration_002_status }}
- **Evidence:** {{ integration_002_evidence }}

#### qa-sec-integration-003 — Integration testing defines scenarios
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one integration test scenario or use case
- **Status:** {{ integration_003_status }}
- **Evidence:** {{ integration_003_evidence }}

#### qa-sec-integration-004 — Integration testing references architecture
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Architecture Documentation communication paths or data flow
- **Status:** {{ integration_004_status }}
- **Evidence:** {{ integration_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| qa-integration-derives-architecture | derives_from | architecture:communication_paths | incoming | {{ rel_integration_arch }} |

---

## Section: security_testing

### Rules

#### qa-sec-security-001 — Security testing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'security_testing'
- **Status:** {{ security_001_status }}
- **Evidence:** {{ security_001_evidence }}

#### qa-sec-security-002 — Security testing defines threat coverage
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section specifies which security threats or attack vectors are tested
- **Status:** {{ security_002_status }}
- **Evidence:** {{ security_002_evidence }}

#### qa-sec-security-003 — Security testing defines test methods
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section specifies testing methods (SAST, DAST, penetration, etc.)
- **Status:** {{ security_003_status }}
- **Evidence:** {{ security_003_evidence }}

#### qa-sec-security-004 — Security testing references security standard
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Security Documentation threat model
- **Status:** {{ security_004_status }}
- **Evidence:** {{ security_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| qa-security-derives-security | derives_from | security:threat_model | incoming | {{ rel_security_security }} |

---

## Section: purpose

### Rules

#### qa-sec-purpose-001 — Purpose section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'purpose'
- **Status:** {{ purpose_001_status }}
- **Evidence:** {{ purpose_001_evidence }}

#### qa-sec-purpose-002 — Purpose states QA intent
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains a statement of why QA Documentation exists
- **Status:** {{ purpose_002_status }}
- **Evidence:** {{ purpose_002_evidence }}

#### qa-sec-purpose-003 — Purpose defines scope boundaries
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section defines what QA Documentation is and is not
- **Status:** {{ purpose_003_status }}
- **Evidence:** {{ purpose_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| qa-purpose-derives-vision | derives_from | vision:purpose | incoming | {{ rel_purpose_vision }} |

---

## Section: e2e_testing

### Rules

#### qa-sec-e2e_testing-001 — E2E Testing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'e2e_testing'
- **Status:** {{ e2e_001_status }}
- **Evidence:** {{ e2e_001_evidence }}

#### qa-sec-e2e_testing-002 — E2E Testing has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ e2e_002_status }}
- **Evidence:** {{ e2e_002_evidence }}

#### qa-sec-e2e_testing-003 — E2E Testing is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ e2e_003_status }}
- **Evidence:** {{ e2e_003_evidence }}

---

## Section: smoke_testing

### Rules

#### qa-sec-smoke_testing-001 — Smoke Testing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'smoke_testing'
- **Status:** {{ smoke_001_status }}
- **Evidence:** {{ smoke_001_evidence }}

#### qa-sec-smoke_testing-002 — Smoke Testing has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ smoke_002_status }}
- **Evidence:** {{ smoke_002_evidence }}

#### qa-sec-smoke_testing-003 — Smoke Testing is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ smoke_003_status }}
- **Evidence:** {{ smoke_003_evidence }}

---

## Section: load_testing

### Rules

#### qa-sec-load_testing-001 — Load Testing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'load_testing'
- **Status:** {{ load_001_status }}
- **Evidence:** {{ load_001_evidence }}

#### qa-sec-load_testing-002 — Load Testing has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ load_002_status }}
- **Evidence:** {{ load_002_evidence }}

#### qa-sec-load_testing-003 — Load Testing is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ load_003_status }}
- **Evidence:** {{ load_003_evidence }}

---

## Section: scalability_testing

### Rules

#### qa-sec-scalability_testing-001 — Scalability Testing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'scalability_testing'
- **Status:** {{ scalability_001_status }}
- **Evidence:** {{ scalability_001_evidence }}

#### qa-sec-scalability_testing-002 — Scalability Testing has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ scalability_002_status }}
- **Evidence:** {{ scalability_002_evidence }}

#### qa-sec-scalability_testing-003 — Scalability Testing is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ scalability_003_status }}
- **Evidence:** {{ scalability_003_evidence }}

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
| test_strategy | {{ strategy_weight }} | {{ strategy_score }} | {{ strategy_status }} |
| unit_testing | {{ unit_weight }} | {{ unit_score }} | {{ unit_status }} |
| integration_testing | {{ integration_weight }} | {{ integration_score }} | {{ integration_status }} |
| security_testing | {{ security_weight }} | {{ security_score }} | {{ security_status }} |
| purpose | {{ purpose_weight }} | {{ purpose_score }} | {{ purpose_status }} |
| e2e_testing | {{ e2e_weight }} | {{ e2e_score }} | {{ e2e_status }} |
| smoke_testing | {{ smoke_weight }} | {{ smoke_score }} | {{ smoke_status }} |
| load_testing | {{ load_weight }} | {{ load_score }} | {{ load_status }} |
| scalability_testing | {{ scalability_weight }} | {{ scalability_score }} | {{ scalability_status }} |
