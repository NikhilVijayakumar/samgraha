# {{ document_title }} — Vision Document Audit Report

> **Domain:** vision
> **Scope:** document
> **Standard:** documentation-standards
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Document-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | 6.0 |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | 6.0 |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Vision documentation defines the aspirational direction for the entire project. Document-level checks ensure all required sections are present, the document remains technology-independent, and it maintains its Tier 1 position — deriving from nothing, inspiring everything downstream.

---

## Rule Results

### vis-doc-001 — Required sections present
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document contains all required sections per documentation-standards/02-vision-standards.md Required Sections table
- **Status:** {{ rule_001_status }}
- **Evidence:** {{ rule_001_evidence }}
- **Why this matters:** Missing sections leave the vision incomplete — purpose without a vision statement is directionless, a vision statement without a problem is ungrounded.

### vis-doc-002 — No empty required sections
- **Severity:** error
- **Weight:** 1.0
- **Condition:** every required section has non-empty content (not just a heading)
- **Status:** {{ rule_002_status }}
- **Evidence:** {{ rule_002_evidence }}
- **Why this matters:** Empty sections create the illusion of completeness. A "Problem" heading with no content means the vision is solving a problem nobody has articulated.

### vis-doc-003 — Document covers one product vision
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a single primary focus — does not mix unrelated product visions
- **Status:** {{ rule_003_status }}
- **Evidence:** {{ rule_003_evidence }}
- **Why this matters:** Bundling multiple product visions into one document makes it impossible to trace requirements to specific capabilities.

### vis-doc-004 — No implementation technology references
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document does not name specific programming languages, frameworks, libraries, APIs, database schemas, or protocols
- **Status:** {{ rule_004_status }}
- **Evidence:** {{ rule_004_evidence }}
- **Why this matters:** Technology references in vision docs couple aspirational direction to implementation details — when the technology changes, the vision shouldn't need rewriting.

### vis-doc-005 — Vision derives from no other domain
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document has no derives_from relationships to other domains — Vision is Tier 1
- **Status:** {{ rule_005_status }}
- **Evidence:** {{ rule_005_evidence }}
- **Why this matters:** Vision is the root of the derivation chain. If it derives from something else, the entire documentation hierarchy has no anchor.

### vis-doc-006 — Vision inspires downstream domains
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document references or is traceable to Philosophy, Feature, and Security domains
- **Status:** {{ rule_006_status }}
- **Evidence:** {{ rule_006_evidence }}
- **Why this matters:** Vision without downstream references is an orphan — it inspires nothing and has no traceable impact.

### vis-doc-007 — No duplicate content within document
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** no section repeats the same information as another section
- **Status:** {{ rule_007_status }}
- **Evidence:** {{ rule_007_evidence }}
- **Why this matters:** Duplicate content creates maintenance risk — one copy gets updated, the other doesn't, and the vision silently diverges.

---

## Cross-Section Relationships

### vis-section-consistency (section_consistency)
- **Owner:** document
- **Description:** Sections within vision docs are mutually consistent — problem, solution, and vision_statement align without contradiction
- **Status:** {{ relationship_consistency_status }}
- **Evidence:** {{ relationship_consistency_evidence }}

### vis-collection-coherence (collection_coherence)
- **Owner:** document
- **Description:** All vision documents in the domain cohere as one coherent vision — no orphaned or contradictory documents
- **Status:** {{ relationship_coherence_status }}
- **Evidence:** {{ relationship_coherence_evidence }}

### vis-terminology-drift (terminology_drift)
- **Owner:** document
- **Description:** Terminology is consistent across all vision sections — same concept, same name
- **Status:** {{ relationship_terminology_status }}
- **Evidence:** {{ relationship_terminology_evidence }}

### vis-technology-independence (technology_independence)
- **Owner:** document
- **Description:** No section in the vision document contains implementation technology references — Vision is technology-independent by design
- **Status:** {{ relationship_tech_independence_status }}
- **Evidence:** {{ relationship_tech_independence_evidence }}

---

## Section-Level Rule Summary

| Section | Rules Checked | Errors | Warnings | Pass Rate |
|---|---|---|---|---|
| purpose | {{ purpose_rules }} | {{ purpose_errors }} | {{ purpose_warnings }} | {{ purpose_pass }} |
| vision_statement | {{ vs_rules }} | {{ vs_errors }} | {{ vs_warnings }} | {{ vs_pass }} |
| problem | {{ problem_rules }} | {{ problem_errors }} | {{ problem_warnings }} | {{ problem_pass }} |
| solution | {{ solution_rules }} | {{ solution_errors }} | {{ solution_warnings }} | {{ solution_pass }} |
| target_audience | {{ ta_rules }} | {{ ta_errors }} | {{ ta_warnings }} | {{ ta_pass }} |
| pillars | {{ pillars_rules }} | {{ pillars_errors }} | {{ pillars_warnings }} | {{ pillars_pass }} |
| philosophy | {{ philosophy_rules }} | {{ philosophy_errors }} | {{ philosophy_warnings }} | {{ philosophy_pass }} |
| guiding_principles | {{ gp_rules }} | {{ gp_errors }} | {{ gp_warnings }} | {{ gp_pass }} |
| success_criteria | {{ sc_rules }} | {{ sc_errors }} | {{ sc_warnings }} | {{ sc_pass }} |
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
| Tier 1 Positioning | 1.0 | {{ tier1_score }} | {{ tier1_status }} |
| Cross-References | 0.5 | {{ crossref_score }} | {{ crossref_status }} |
| Deduplication | 0.5 | {{ dedup_score }} | {{ dedup_status }} |

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| purpose | {{ purpose_section_score }} | {{ purpose_section_weight }} | {{ purpose_section_status }} |
| vision_statement | {{ vs_section_score }} | {{ vs_section_weight }} | {{ vs_section_status }} |
| problem | {{ problem_section_score }} | {{ problem_section_weight }} | {{ problem_section_status }} |
| solution | {{ solution_section_score }} | {{ solution_section_weight }} | {{ solution_section_status }} |
| target_audience | {{ ta_section_score }} | {{ ta_section_weight }} | {{ ta_section_status }} |
| pillars | {{ pillars_section_score }} | {{ pillars_section_weight }} | {{ pillars_section_status }} |
| philosophy | {{ philosophy_section_score }} | {{ philosophy_section_weight }} | {{ philosophy_section_status }} |
| guiding_principles | {{ gp_section_score }} | {{ gp_section_weight }} | {{ gp_section_status }} |
| success_criteria | {{ sc_section_score }} | {{ sc_section_weight }} | {{ sc_section_status }} |
| traceability | {{ trace_section_score }} | {{ trace_section_weight }} | {{ trace_section_status }} |
