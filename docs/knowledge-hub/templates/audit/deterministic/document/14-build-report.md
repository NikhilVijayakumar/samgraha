# {{ document_title }} — Build Document Audit Report

> **Domain:** build
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

**Why this matters:** Build documentation specifies packaging, distribution, and CI/CD validation. If security checks, versioning, and documentation quality contradict each other, builds ship with unvetted assumptions. Document-level checks catch cross-section drift that section-level audits miss.

---

## Rule Results

### build-doc-001 — Required sections present
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document contains all required sections per documentation-standards Build requirements
- **Status:** {{ rule_001_status }}
- **Evidence:** {{ rule_001_evidence }}
- **Why this matters:** Missing sections leave build policy incomplete — packaging without security validation, or versioning without size checks.

### build-doc-002 — No empty required sections
- **Severity:** error
- **Weight:** 1.0
- **Condition:** every required section has non-empty content (not just a heading)
- **Status:** {{ rule_002_status }}
- **Evidence:** {{ rule_002_evidence }}
- **Why this matters:** Empty sections create the illusion of coverage. A heading with no content is worse than no heading at all.

### build-doc-003 — Document covers one build concern
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a single primary focus — does not mix unrelated build configurations
- **Status:** {{ rule_003_status }}
- **Evidence:** {{ rule_003_evidence }}
- **Why this matters:** Bundling unrelated build configs into one document makes it impossible to audit any single concern in isolation.

### build-doc-004 — Document derives from Implementation
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document references upstream Implementation Documentation
- **Status:** {{ rule_004_status }}
- **Evidence:** {{ rule_004_evidence }}
- **Why this matters:** Build documentation without an implementation anchor has no source of truth for what is being built or packaged.

### build-doc-005 — Required cross-references present
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document references downstream Readme Documentation where applicable
- **Status:** {{ rule_005_status }}
- **Evidence:** {{ rule_005_evidence }}
- **Why this matters:** Build docs feed into Readme — if the chain breaks, installers and user-facing docs diverge from what was actually packaged.

### build-doc-006 — No duplicate content within document
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** no section repeats the same information as another section
- **Status:** {{ rule_006_status }}
- **Evidence:** {{ rule_006_evidence }}
- **Why this matters:** Duplicate content creates maintenance risk — one copy gets updated, the other doesn't, and they silently contradict.

---

## Cross-Section Relationships

### build-section-consistency (section_consistency)
- **Owner:** document
- **Description:** Sections within build docs are mutually consistent — security checks align with versioning, documentation quality aligns with both
- **Status:** {{ relationship_consistency_status }}
- **Evidence:** {{ relationship_consistency_evidence }}

### build-collection-coherence (collection_coherence)
- **Owner:** document
- **Description:** All build documents in the domain cohere as one system — no orphaned or contradictory build configurations
- **Status:** {{ relationship_coherence_status }}
- **Evidence:** {{ relationship_coherence_evidence }}

### build-terminology-drift (terminology_drift)
- **Owner:** document
- **Description:** Terminology is consistent across all build sections — same concept, same name
- **Status:** {{ relationship_terminology_status }}
- **Evidence:** {{ relationship_terminology_evidence }}

---

## Section-Level Rule Summary

| Section | Rules Checked | Errors | Warnings | Pass Rate |
|---|---|---|---|---|
| documentation_quality | {{ doc_quality_rules }} | {{ doc_quality_errors }} | {{ doc_quality_warnings }} | {{ doc_quality_pass }} |
| security_checks | {{ sec_rules }} | {{ sec_errors }} | {{ sec_warnings }} | {{ sec_pass }} |
| versioning_naming | {{ version_rules }} | {{ version_errors }} | {{ version_warnings }} | {{ version_pass }} |
| purpose | {{ purpose_rules }} | {{ purpose_errors }} | {{ purpose_warnings }} | {{ purpose_pass }} |
| size_checks | {{ size_rules }} | {{ size_errors }} | {{ size_warnings }} | {{ size_pass }} |
| ml_artifact_management | {{ ml_rules }} | {{ ml_errors }} | {{ ml_warnings }} | {{ ml_pass }} |
| cicd_validation | {{ cicd_rules }} | {{ cicd_errors }} | {{ cicd_warnings }} | {{ cicd_pass }} |
| obfuscation_optimization | {{ obf_rules }} | {{ obf_errors }} | {{ obf_warnings }} | {{ obf_pass }} |

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
| Derivation | 1.0 | {{ derivation_score }} | {{ derivation_status }} |
| Cross-References | 0.5 | {{ crossref_score }} | {{ crossref_status }} |
| Deduplication | 0.5 | {{ dedup_score }} | {{ dedup_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| documentation_quality | 3.5 | {{ doc_quality_section_score }} | {{ doc_quality_section_status }} |
| security_checks | 3.5 | {{ sec_section_score }} | {{ sec_section_status }} |
| versioning_naming | 3.5 | {{ version_section_score }} | {{ version_section_status }} |
| purpose | 1.5 | {{ purpose_section_score }} | {{ purpose_section_status }} |
| size_checks | 3.0 | {{ size_section_score }} | {{ size_section_status }} |
| ml_artifact_management | 3.0 | {{ ml_section_score }} | {{ ml_section_status }} |
| cicd_validation | 3.0 | {{ cicd_section_score }} | {{ cicd_section_status }} |
| obfuscation_optimization | 3.0 | {{ obf_section_score }} | {{ obf_section_status }} |
