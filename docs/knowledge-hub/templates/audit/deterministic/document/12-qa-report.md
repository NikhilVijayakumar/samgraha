# {{ document_title }} — QA Document Audit Report

> **Domain:** qa
> **Scope:** document
> **Standard:** documentation-standards
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

**Why this matters:** QA documentation defines how quality is verified across the entire system. Document-level checks ensure all testing levels (unit, integration, security) are present and the collection forms a coherent test strategy — not a set of disconnected test plans.

---

## Rule Results

### qa-doc-001 — Required sections present
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document contains all required sections per documentation-standards QA requirements
- **Status:** {{ rule_001_status }}
- **Evidence:** {{ rule_001_evidence }}
- **Why this matters:** Missing sections leave testing gaps — security testing without unit testing, or integration testing without a strategy, means quality assurance is incomplete.

### qa-doc-002 — No empty required sections
- **Severity:** error
- **Weight:** 1.0
- **Condition:** every required section has non-empty content (not just a heading)
- **Status:** {{ rule_002_status }}
- **Evidence:** {{ rule_002_evidence }}
- **Why this matters:** Empty sections create the illusion of coverage. A "Security Testing" heading with no content means security isn't actually being tested.

### qa-doc-003 — Document covers one testing concern
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a single primary focus — does not mix unrelated testing strategies
- **Status:** {{ rule_003_status }}
- **Evidence:** {{ rule_003_evidence }}
- **Why this matters:** Bundling unrelated testing concerns into one document makes it impossible to audit any single testing level in isolation.

### qa-doc-004 — Test strategy covers all testing levels
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document addresses unit, integration, and security testing levels
- **Status:** {{ rule_004_status }}
- **Evidence:** {{ rule_004_evidence }}
- **Why this matters:** Incomplete testing levels leave systematic blind spots — unit tests without integration tests catch component bugs but miss interface failures.

### qa-doc-005 — Required cross-references present
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document references upstream standards (Architecture, Engineering, Security) where applicable
- **Status:** {{ rule_005_status }}
- **Evidence:** {{ rule_005_evidence }}
- **Why this matters:** QA without upstream references has no anchor — test scope is invented rather than derived from what the system actually does and what threats it faces.

### qa-doc-006 — No duplicate content within document
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** no section repeats the same information as another section
- **Status:** {{ rule_006_status }}
- **Evidence:** {{ rule_006_evidence }}
- **Why this matters:** Duplicate content creates maintenance risk — one copy gets updated, the other doesn't, and test plans silently diverge.

---

## Cross-Section Relationships

### qa-section-consistency (section_consistency)
- **Owner:** document
- **Description:** Sections within QA docs are mutually consistent — test strategy aligns with unit, integration, and security testing
- **Status:** {{ relationship_consistency_status }}
- **Evidence:** {{ relationship_consistency_evidence }}

### qa-collection-coherence (collection_coherence)
- **Owner:** document
- **Description:** All QA documents in the domain cohere as one system — no orphaned or contradictory test plans
- **Status:** {{ relationship_coherence_status }}
- **Evidence:** {{ relationship_coherence_evidence }}

### qa-terminology-drift (terminology_drift)
- **Owner:** document
- **Description:** Terminology is consistent across all QA sections — same concept, same name
- **Status:** {{ relationship_terminology_status }}
- **Evidence:** {{ relationship_terminology_evidence }}

---

## Section-Level Rule Summary

| Section | Rules Checked | Errors | Warnings | Pass Rate |
|---|---|---|---|---|
| test_strategy | {{ strategy_rules }} | {{ strategy_errors }} | {{ strategy_warnings }} | {{ strategy_pass }} |
| unit_testing | {{ unit_rules }} | {{ unit_errors }} | {{ unit_warnings }} | {{ unit_pass }} |
| integration_testing | {{ integration_rules }} | {{ integration_errors }} | {{ integration_warnings }} | {{ integration_pass }} |
| security_testing | {{ security_rules }} | {{ security_errors }} | {{ security_warnings }} | {{ security_pass }} |
| purpose | {{ purpose_rules }} | {{ purpose_errors }} | {{ purpose_warnings }} | {{ purpose_pass }} |
| e2e_testing | {{ e2e_rules }} | {{ e2e_errors }} | {{ e2e_warnings }} | {{ e2e_pass }} |
| smoke_testing | {{ smoke_rules }} | {{ smoke_errors }} | {{ smoke_warnings }} | {{ smoke_pass }} |
| load_testing | {{ load_rules }} | {{ load_errors }} | {{ load_warnings }} | {{ load_pass }} |
| scalability_testing | {{ scalability_rules }} | {{ scalability_errors }} | {{ scalability_warnings }} | {{ scalability_pass }} |

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

### Document-Level Breakdown

| Category | Weight | Score | Status |
|---|---|---|---|
| Section Completeness | 2.5 | {{ section_completeness_score }} | {{ section_completeness_status }} |
| Modularity | 0.5 | {{ modularity_score }} | {{ modularity_status }} |
| Coverage Breadth | 1.0 | {{ coverage_score }} | {{ coverage_status }} |
| Cross-References | 0.5 | {{ crossref_score }} | {{ crossref_status }} |
| Deduplication | 0.5 | {{ dedup_score }} | {{ dedup_status }} |

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
