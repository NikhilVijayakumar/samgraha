# {{ document_title }} — README Document Audit Report

> **Domain:** readme
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

**Why this matters:** README documentation is the primary entry point for anyone encountering the project. Document-level checks ensure all required sections are present, follow standard ordering, and derive from upstream Build Documentation — not invented independently.

---

## Rule Results

### readme-doc-001 — Required sections present
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document contains all required sections per documentation-standards README requirements
- **Status:** {{ rule_001_status }}
- **Evidence:** {{ rule_001_evidence }}
- **Why this matters:** Missing sections leave onboarding incomplete — installation without usage, build without contributing, or overview without purpose means readers hit dead ends.

### readme-doc-002 — No empty required sections
- **Severity:** error
- **Weight:** 1.0
- **Condition:** every required section has non-empty content (not just a heading)
- **Status:** {{ rule_002_status }}
- **Evidence:** {{ rule_002_evidence }}
- **Why this matters:** Empty sections create the illusion of completeness. A "Getting Started" heading with no content means the project has no onboarding path.

### readme-doc-003 — Document covers one project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a single primary focus — does not mix unrelated project READMEs
- **Status:** {{ rule_003_status }}
- **Evidence:** {{ rule_003_evidence }}
- **Why this matters:** Bundling multiple project READMEs into one document makes it impossible to maintain consistent onboarding for any single project.

### readme-doc-004 — Document derives from Build
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document references upstream Build Documentation
- **Status:** {{ rule_004_status }}
- **Evidence:** {{ rule_004_evidence }}
- **Why this matters:** README without Build documentation as upstream has no source of truth for installation, build, and versioning instructions.

### readme-doc-005 — Sections appear in standard order
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** sections follow the standard README ordering
- **Status:** {{ rule_005_status }}
- **Evidence:** {{ rule_005_evidence }}
- **Why this matters:** Non-standard ordering breaks reader expectations — developers expect to find installation before usage, usage before contributing.

### readme-doc-006 — No duplicate content within document
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** no section repeats the same information as another section
- **Status:** {{ rule_006_status }}
- **Evidence:** {{ rule_006_evidence }}
- **Why this matters:** Duplicate content creates maintenance risk — one copy gets updated, the other doesn't, and the README silently diverges.

---

## Cross-Section Relationships

### readme-section-consistency (section_consistency)
- **Owner:** document
- **Description:** Sections within README are mutually consistent — usage examples match build instructions, development matches contributing
- **Status:** {{ relationship_consistency_status }}
- **Evidence:** {{ relationship_consistency_evidence }}

### readme-collection-coherence (collection_coherence)
- **Owner:** document
- **Description:** All README documents in the domain cohere as one system — no orphaned or contradictory READMEs
- **Status:** {{ relationship_coherence_status }}
- **Evidence:** {{ relationship_coherence_evidence }}

### readme-terminology-drift (terminology_drift)
- **Owner:** document
- **Description:** Terminology is consistent across all README sections — same concept, same name
- **Status:** {{ relationship_terminology_status }}
- **Evidence:** {{ relationship_terminology_evidence }}

---

## Section-Level Rule Summary

| Section | Rules Checked | Errors | Warnings | Pass Rate |
|---|---|---|---|---|
| project_name | {{ name_rules }} | {{ name_errors }} | {{ name_warnings }} | {{ name_pass }} |
| short_description | {{ short_desc_rules }} | {{ short_desc_errors }} | {{ short_desc_warnings }} | {{ short_desc_pass }} |
| overview | {{ overview_rules }} | {{ overview_errors }} | {{ overview_warnings }} | {{ overview_pass }} |
| purpose | {{ purpose_rules }} | {{ purpose_errors }} | {{ purpose_warnings }} | {{ purpose_pass }} |
| key_capabilities | {{ caps_rules }} | {{ caps_errors }} | {{ caps_warnings }} | {{ caps_pass }} |
| repository_structure | {{ repo_rules }} | {{ repo_errors }} | {{ repo_warnings }} | {{ repo_pass }} |
| documentation_structure | {{ doc_struct_rules }} | {{ doc_struct_errors }} | {{ doc_struct_warnings }} | {{ doc_struct_pass }} |
| getting_started | {{ gs_rules }} | {{ gs_errors }} | {{ gs_warnings }} | {{ gs_pass }} |
| installation | {{ install_rules }} | {{ install_errors }} | {{ install_warnings }} | {{ install_pass }} |
| build | {{ build_rules }} | {{ build_errors }} | {{ build_warnings }} | {{ build_pass }} |
| usage | {{ usage_rules }} | {{ usage_errors }} | {{ usage_warnings }} | {{ usage_pass }} |
| development | {{ dev_rules }} | {{ dev_errors }} | {{ dev_warnings }} | {{ dev_pass }} |
| contributing | {{ contrib_rules }} | {{ contrib_errors }} | {{ contrib_warnings }} | {{ contrib_pass }} |
| configuration | {{ config_rules }} | {{ config_errors }} | {{ config_warnings }} | {{ config_pass }} |
| license | {{ license_rules }} | {{ license_errors }} | {{ license_warnings }} | {{ license_pass }} |

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
| Section Ordering | 0.5 | {{ ordering_score }} | {{ ordering_status }} |
| Deduplication | 0.5 | {{ dedup_score }} | {{ dedup_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| project_name | {{ name_section_score }} | {{ name_section_weight }} | {{ name_section_status }} |
| short_description | {{ short_desc_section_score }} | {{ short_desc_section_weight }} | {{ short_desc_section_status }} |
| overview | {{ overview_section_score }} | {{ overview_section_weight }} | {{ overview_section_status }} |
| purpose | {{ purpose_section_score }} | {{ purpose_section_weight }} | {{ purpose_section_status }} |
| key_capabilities | {{ caps_section_score }} | {{ caps_section_weight }} | {{ caps_section_status }} |
| repository_structure | {{ repo_section_score }} | {{ repo_section_weight }} | {{ repo_section_status }} |
| documentation_structure | {{ doc_struct_section_score }} | {{ doc_struct_section_weight }} | {{ doc_struct_section_status }} |
| getting_started | {{ gs_section_score }} | {{ gs_section_weight }} | {{ gs_section_status }} |
| installation | {{ install_section_score }} | {{ install_section_weight }} | {{ install_section_status }} |
| build | {{ build_section_score }} | {{ build_section_weight }} | {{ build_section_status }} |
| usage | {{ usage_section_score }} | {{ usage_section_weight }} | {{ usage_section_status }} |
| development | {{ dev_section_score }} | {{ dev_section_weight }} | {{ dev_section_status }} |
| contributing | {{ contrib_section_score }} | {{ contrib_section_weight }} | {{ contrib_section_status }} |
| configuration | {{ config_section_score }} | {{ config_section_weight }} | {{ config_section_status }} |
| license | {{ license_section_score }} | {{ license_section_weight }} | {{ license_section_status }} |
