# {{ document_title }} — Feature Document Audit Report

> **Domain:** feature
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

**Why this matters:** Feature documentation defines what the system must do. Document-level checks ensure all required sections are present and the collection forms a coherent, technology-independent specification — not a set of disconnected requirements.

---

## Rule Results

### feat-doc-001 — Required sections present
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document contains all required sections per documentation-standards Required Sections table
- **Status:** {{ rule_001_status }}
- **Evidence:** {{ rule_001_evidence }}
- **Why this matters:** Missing sections leave feature specification incomplete — requirements without acceptance criteria are unverifiable, purpose without requirements is directionless.

### feat-doc-002 — No empty required sections
- **Severity:** error
- **Weight:** 1.0
- **Condition:** every required section has non-empty content (not just a heading)
- **Status:** {{ rule_002_status }}
- **Evidence:** {{ rule_002_evidence }}
- **Why this matters:** Empty sections create the illusion of coverage. A "Functional Requirements" heading with no requirements means the feature is undefined.

### feat-doc-003 — Document covers one feature
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a single primary focus — does not mix unrelated feature concerns
- **Status:** {{ rule_003_status }}
- **Evidence:** {{ rule_003_evidence }}
- **Why this matters:** Bundling multiple features into one document makes it impossible to trace requirements to specific capabilities.

### feat-doc-004 — No implementation technology references
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document does not name specific programming languages, frameworks, libraries, APIs, database schemas, or protocols
- **Status:** {{ rule_004_status }}
- **Evidence:** {{ rule_004_evidence }}
- **Why this matters:** Technology references in feature docs couple requirements to implementation details — when the technology changes, the requirements shouldn't need rewriting.

### feat-doc-005 — Required cross-references present
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document references upstream standards (Vision, Philosophy) and downstream standards (Feature Design, Feature Technical) where applicable
- **Status:** {{ rule_005_status }}
- **Evidence:** {{ rule_005_evidence }}
- **Why this matters:** Feature docs without upstream references have no source of truth for why the feature exists. Without downstream references, the chain to design and implementation is broken.

### feat-doc-006 — No duplicate content within document
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** no section repeats the same information as another section
- **Status:** {{ rule_006_status }}
- **Evidence:** {{ rule_006_evidence }}
- **Why this matters:** Duplicate content creates maintenance risk — one copy gets updated, the other doesn't, and requirements silently diverge.

---

## Cross-Section Relationships

### feat-section-consistency (section_consistency)
- **Owner:** document
- **Description:** Sections within feature docs are mutually consistent — no contradictions between functional_requirements, acceptance_criteria, and business_rules
- **Status:** {{ relationship_consistency_status }}
- **Evidence:** {{ relationship_consistency_evidence }}

### feat-collection-coherence (collection_coherence)
- **Owner:** document
- **Description:** All feature documents in the domain cohere as one system — no orphaned or contradictory documents
- **Status:** {{ relationship_coherence_status }}
- **Evidence:** {{ relationship_coherence_evidence }}

### feat-terminology-drift (terminology_drift)
- **Owner:** document
- **Description:** Terminology is consistent across all feature sections — same concept, same name
- **Status:** {{ relationship_terminology_status }}
- **Evidence:** {{ relationship_terminology_evidence }}

---

## Section-Level Rule Summary

| Section | Rules Checked | Errors | Warnings | Pass Rate |
|---|---|---|---|---|
| purpose | {{ purpose_rules }} | {{ purpose_errors }} | {{ purpose_warnings }} | {{ purpose_pass }} |
| functional_requirements | {{ freq_rules }} | {{ freq_errors }} | {{ freq_warnings }} | {{ freq_pass }} |
| acceptance_criteria | {{ ac_rules }} | {{ ac_errors }} | {{ ac_warnings }} | {{ ac_pass }} |
| business_rules | {{ br_rules }} | {{ br_errors }} | {{ br_warnings }} | {{ br_pass }} |
| inputs | {{ inputs_rules }} | {{ inputs_errors }} | {{ inputs_warnings }} | {{ inputs_pass }} |
| outputs | {{ outputs_rules }} | {{ outputs_errors }} | {{ outputs_warnings }} | {{ outputs_pass }} |
| constraints | {{ constraints_rules }} | {{ constraints_errors }} | {{ constraints_warnings }} | {{ constraints_pass }} |
| dependencies | {{ dependencies_rules }} | {{ dependencies_errors }} | {{ dependencies_warnings }} | {{ dependencies_pass }} |
| non_goals | {{ non_goals_rules }} | {{ non_goals_errors }} | {{ non_goals_warnings }} | {{ non_goals_pass }} |
| future_extensions | {{ future_rules }} | {{ future_errors }} | {{ future_warnings }} | {{ future_pass }} |
| traceability | {{ trace_rules }} | {{ trace_errors }} | {{ trace_warnings }} | {{ trace_pass }} |

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
| Technology Independence | 1.0 | {{ tech_independence_score }} | {{ tech_independence_status }} |
| Cross-References | 0.5 | {{ crossref_score }} | {{ crossref_status }} |
| Deduplication | 0.5 | {{ dedup_score }} | {{ dedup_status }} |

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
