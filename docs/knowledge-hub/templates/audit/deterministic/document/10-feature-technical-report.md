# {{ document_title }} — Feature Technical Document Audit Report

> **Domain:** feature-technical
> **Scope:** document
> **Standard:** documentation-standards
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Document-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | 6.5 |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | 6.5 |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Feature Technical Documentation is the technology-independent realization of a feature — the bridge between Feature Design and Implementation. Document-level checks ensure it covers one concern, stays technology-free, derives from Tier 1/2 upstreams, and references Prototype validation and Implementation derivation.

---

## Rule Results

### ft-doc-001 — Required sections present
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document contains all required sections per documentation-standards/05-feature-technical-standards.md Required Sections table
- **Status:** {{ rule_001_status }}
- **Evidence:** {{ rule_001_evidence }}
- **Why this matters:** Feature Technical without required sections (purpose, participating_components, component_interactions, data_ownership) gives implementers no technical specification to follow.

### ft-doc-002 — No empty required sections
- **Severity:** error
- **Weight:** 1.0
- **Condition:** every required section has non-empty content (not just a heading)
- **Status:** {{ rule_002_status }}
- **Evidence:** {{ rule_002_evidence }}
- **Why this matters:** Empty required sections promise a specification that never arrives — implementers see a heading and assume the content exists elsewhere.

### ft-doc-003 — Document covers one feature technical concern
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a single primary focus — does not mix unrelated feature technical concerns
- **Status:** {{ rule_003_status }}
- **Evidence:** {{ rule_003_evidence }}
- **Why this matters:** Bundling unrelated feature technical designs makes it impossible to maintain consistent component ownership — each feature drifts independently.

### ft-doc-004 — No implementation technology references
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document does not name specific programming languages, frameworks, libraries, APIs, database schemas, or protocols
- **Status:** {{ rule_004_status }}
- **Evidence:** {{ rule_004_evidence }}
- **Why this matters:** Feature Technical that names implementation technologies is no longer technology-independent — it has leaked into Implementation territory, bypassing Prototype validation.

### ft-doc-005 — Required cross-references present
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document references upstream domains (Feature, Engineering, Architecture, External Context) where applicable
- **Status:** {{ rule_005_status }}
- **Evidence:** {{ rule_005_evidence }}
- **Why this matters:** Feature Technical without upstream cross-references has no provenance — readers cannot trace why this technical design exists or what requirements it fulfills.

### ft-doc-006 — No duplicate content within document
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** no section repeats the same information as another section
- **Status:** {{ rule_006_status }}
- **Evidence:** {{ rule_006_evidence }}
- **Why this matters:** Duplicate content creates maintenance drift — one copy gets updated while the other goes stale, producing contradictions within the same technical specification.

### ft-doc-007 — Feature Technical does not derive from lower-tier documents
- **Severity:** error
- **Weight:** 1.0
- **Condition:** document does not claim derivation from Implementation or Prototype
- **Status:** {{ rule_007_status }}
- **Evidence:** {{ rule_007_evidence }}
- **Why this matters:** Feature Technical is Tier 3. Deriving from Implementation (Tier 5) or Prototype (Tier 4) reverses the derivation graph — the technical spec would be shaped by the implementation rather than the other way around.

### ft-doc-008 — Document mentions validation by Prototype
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document or its relationships reference Prototype as validation gate
- **Status:** {{ rule_008_status }}
- **Evidence:** {{ rule_008_evidence }}
- **Why this matters:** Feature Technical that doesn't reference Prototype validation has no downstream proof that the technical design is implementable.

### ft-doc-009 — Document mentions derivation of Implementation
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document or its relationships reference Implementation as downstream consumer
- **Status:** {{ rule_009_status }}
- **Evidence:** {{ rule_009_evidence }}
- **Why this matters:** Feature Technical that doesn't reference Implementation as a downstream consumer disconnects from the realization pipeline — implementers don't know this spec exists.

---

## Cross-Section Relationships

### ft-section-consistency (section_consistency)
- **Owner:** document
- **Description:** Sections within feature-technical docs are mutually consistent — no contradictions between participating_components, component_interactions, data_ownership, and runtime_behavior

### ft-collection-coherence (collection_coherence)
- **Owner:** document
- **Description:** All feature-technical documents in the domain cohere as one system — no orphaned or contradictory documents

### ft-terminology-drift (terminology_drift)
- **Owner:** document
- **Description:** Terminology is consistent across all feature-technical sections — same concept, same name

---

## Section-Level Results

| Section | Rules | Passed | Failed | Errors | Warnings |
|---|---|---|---|---|---|
| purpose | {{ purpose_rules }} | {{ purpose_passed }} | {{ purpose_failed }} | {{ purpose_errors }} | {{ purpose_warnings }} |
| participating_components | {{ components_rules }} | {{ components_passed }} | {{ components_failed }} | {{ components_errors }} | {{ components_warnings }} |
| component_interactions | {{ interactions_rules }} | {{ interactions_passed }} | {{ interactions_failed }} | {{ interactions_errors }} | {{ interactions_warnings }} |
| data_ownership | {{ data_rules }} | {{ data_passed }} | {{ data_failed }} | {{ data_errors }} | {{ data_warnings }} |
| feature_specification | {{ spec_rules }} | {{ spec_passed }} | {{ spec_failed }} | {{ spec_errors }} | {{ spec_warnings }} |
| component_responsibilities | {{ resp_rules }} | {{ resp_passed }} | {{ resp_failed }} | {{ resp_errors }} | {{ resp_warnings }} |
| runtime_behavior | {{ runtime_rules }} | {{ runtime_passed }} | {{ runtime_failed }} | {{ runtime_errors }} | {{ runtime_warnings }} |
| communication_paths | {{ comm_rules }} | {{ comm_passed }} | {{ comm_failed }} | {{ comm_errors }} | {{ comm_warnings }} |
| integration_points | {{ integration_rules }} | {{ integration_passed }} | {{ integration_failed }} | {{ integration_errors }} | {{ integration_warnings }} |
| external_dependencies | {{ extdep_rules }} | {{ extdep_passed }} | {{ extdep_failed }} | {{ extdep_errors }} | {{ extdep_warnings }} |
| runtime_constraints | {{ rtcon_rules }} | {{ rtcon_passed }} | {{ rtcon_failed }} | {{ rtcon_errors }} | {{ rtcon_warnings }} |
| architectural_constraints | {{ archcon_rules }} | {{ archcon_passed }} | {{ archcon_failed }} | {{ archcon_errors }} | {{ archcon_warnings }} |
| security_considerations | {{ security_rules }} | {{ security_passed }} | {{ security_failed }} | {{ security_errors }} | {{ security_warnings }} |
| performance_considerations | {{ perf_rules }} | {{ perf_passed }} | {{ perf_failed }} | {{ perf_errors }} | {{ perf_warnings }} |
| failure_handling | {{ failure_rules }} | {{ failure_passed }} | {{ failure_failed }} | {{ failure_errors }} | {{ failure_warnings }} |
| extension_points | {{ extension_rules }} | {{ extension_passed }} | {{ extension_failed }} | {{ extension_errors }} | {{ extension_warnings }} |
| traceability | {{ trace_rules }} | {{ trace_passed }} | {{ trace_failed }} | {{ trace_errors }} | {{ trace_warnings }} |

---

## Failures

| Rule | Severity | Weight | Evidence |
|---|---|---|---|
{{ failures_table }}

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ weighted_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})
