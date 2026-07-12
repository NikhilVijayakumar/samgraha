# {{ document_title }} — Feature Technical Audit Summary

> **Domain:** feature-technical
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Overall Score

| Metric | Value |
|---|---|
| **Deterministic Document** | {{ det_doc_score }} |
| **Deterministic Section** | {{ det_sec_score }} |
| **Semantic Document** | {{ sem_doc_score }} |
| **Semantic Section** | {{ sem_sec_score }} |
| **Combined Score** | {{ combined_score }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Feature Technical Documentation is the technology-independent realization of a feature — Tier 3, between Feature Design and Prototype. The combined score aggregates structural rules (sections present, populated, technology-free), cross-section coherence (component-data-runtime consistency), tier enforcement (no lower-tier derivation), and per-section quality (substantive, technology-independent, specific).

---

## Deterministic Document Audit

### Rule Pass Rate

| Rules Checked | Passed | Failed | Errors | Warnings |
|---|---|---|---|---|
| {{ det_doc_rules_checked }} | {{ det_doc_passed }} | {{ det_doc_failed }} | {{ det_doc_errors }} | {{ det_doc_warnings }} |

### Failures

| Rule | Severity | Weight | Evidence |
|---|---|---|---|
{{ det_doc_failures }}

---

## Deterministic Section Audit

### Rule Pass Rate by Section

| Section | Rules | Passed | Failed | Errors | Warnings |
|---|---|---|---|---|---|
| purpose | {{ purpose_det_rules }} | {{ purpose_det_passed }} | {{ purpose_det_failed }} | {{ purpose_det_errors }} | {{ purpose_det_warnings }} |
| participating_components | {{ components_det_rules }} | {{ components_det_passed }} | {{ components_det_failed }} | {{ components_det_errors }} | {{ components_det_warnings }} |
| component_interactions | {{ interactions_det_rules }} | {{ interactions_det_passed }} | {{ interactions_det_failed }} | {{ interactions_det_errors }} | {{ interactions_det_warnings }} |
| data_ownership | {{ data_det_rules }} | {{ data_det_passed }} | {{ data_det_failed }} | {{ data_det_errors }} | {{ data_det_warnings }} |
| feature_specification | {{ spec_det_rules }} | {{ spec_det_passed }} | {{ spec_det_failed }} | {{ spec_det_errors }} | {{ spec_det_warnings }} |
| component_responsibilities | {{ resp_det_rules }} | {{ resp_det_passed }} | {{ resp_det_failed }} | {{ resp_det_errors }} | {{ resp_det_warnings }} |
| runtime_behavior | {{ runtime_det_rules }} | {{ runtime_det_passed }} | {{ runtime_det_failed }} | {{ runtime_det_errors }} | {{ runtime_det_warnings }} |
| communication_paths | {{ comm_det_rules }} | {{ comm_det_passed }} | {{ comm_det_failed }} | {{ comm_det_errors }} | {{ comm_det_warnings }} |
| integration_points | {{ integration_det_rules }} | {{ integration_det_passed }} | {{ integration_det_failed }} | {{ integration_det_errors }} | {{ integration_det_warnings }} |
| external_dependencies | {{ extdep_det_rules }} | {{ extdep_det_passed }} | {{ extdep_det_failed }} | {{ extdep_det_errors }} | {{ extdep_det_warnings }} |
| runtime_constraints | {{ rtcon_det_rules }} | {{ rtcon_det_passed }} | {{ rtcon_det_failed }} | {{ rtcon_det_errors }} | {{ rtcon_det_warnings }} |
| architectural_constraints | {{ archcon_det_rules }} | {{ archcon_det_passed }} | {{ archcon_det_failed }} | {{ archcon_det_errors }} | {{ archcon_det_warnings }} |
| security_considerations | {{ security_det_rules }} | {{ security_det_passed }} | {{ security_det_failed }} | {{ security_det_errors }} | {{ security_det_warnings }} |
| performance_considerations | {{ perf_det_rules }} | {{ perf_det_passed }} | {{ perf_det_failed }} | {{ perf_det_errors }} | {{ perf_det_warnings }} |
| failure_handling | {{ failure_det_rules }} | {{ failure_det_passed }} | {{ failure_det_failed }} | {{ failure_det_errors }} | {{ failure_det_warnings }} |
| extension_points | {{ extension_det_rules }} | {{ extension_det_passed }} | {{ extension_det_failed }} | {{ extension_det_errors }} | {{ extension_det_warnings }} |
| traceability | {{ trace_det_rules }} | {{ trace_det_passed }} | {{ trace_det_failed }} | {{ trace_det_errors }} | {{ trace_det_warnings }} |

---

## Semantic Document Audit

### Criteria Results

| Criterion | Weight | Score | Status | Confidence |
|---|---|---|---|---|
| C1: Cross-Section Consistency | mandatory (40) | {{ sem_doc_c1_score }} | {{ sem_doc_c1_status }} | {{ sem_doc_c1_confidence }} |
| C2: Terminology Consistency | mandatory (30) | {{ sem_doc_c2_score }} | {{ sem_doc_c2_status }} | {{ sem_doc_c2_confidence }} |
| C3: Collection Coherence | recommended (30) | {{ sem_doc_c3_score }} | {{ sem_doc_c3_status }} | {{ sem_doc_c3_confidence }} |

### Failures

| Criterion | Severity | Evidence |
|---|---|---|
{{ sem_doc_failures }}

---

## Semantic Section Audit

### Scores by Section

| Section | Score | C1 | C2 | C3 | C4 |
|---|---|---|---|---|---|
| purpose | {{ purpose_sem_score }} | {{ purpose_c1_status }} | {{ purpose_c2_status }} | {{ purpose_c3_status }} | — |
| participating_components | {{ components_sem_score }} | {{ components_c1_status }} | {{ components_c2_status }} | {{ components_c3_status }} | {{ components_c4_status }} |
| component_interactions | {{ interactions_sem_score }} | {{ interactions_c1_status }} | {{ interactions_c2_status }} | {{ interactions_c3_status }} | {{ interactions_c4_status }} |
| data_ownership | {{ data_sem_score }} | {{ data_c1_status }} | {{ data_c2_status }} | {{ data_c3_status }} | {{ data_c4_status }} |
| feature_specification | {{ spec_sem_score }} | {{ spec_c1_status }} | {{ spec_c2_status }} | {{ spec_c3_status }} | {{ spec_c4_status }} |
| component_responsibilities | {{ resp_sem_score }} | {{ resp_c1_status }} | {{ resp_c2_status }} | {{ resp_c3_status }} | {{ resp_c4_status }} |
| runtime_behavior | {{ runtime_sem_score }} | {{ runtime_c1_status }} | {{ runtime_c2_status }} | {{ runtime_c3_status }} | {{ runtime_c4_status }} |
| communication_paths | {{ comm_sem_score }} | {{ comm_c1_status }} | {{ comm_c2_status }} | {{ comm_c3_status }} | {{ comm_c4_status }} |
| integration_points | {{ integration_sem_score }} | {{ integration_c1_status }} | {{ integration_c2_status }} | {{ integration_c3_status }} | {{ integration_c4_status }} |
| external_dependencies | {{ extdep_sem_score }} | {{ extdep_c1_status }} | {{ extdep_c2_status }} | {{ extdep_c3_status }} | {{ extdep_c4_status }} |
| runtime_constraints | {{ rtcon_sem_score }} | {{ rtcon_c1_status }} | {{ rtcon_c2_status }} | {{ rtcon_c3_status }} | {{ rtcon_c4_status }} |
| architectural_constraints | {{ archcon_sem_score }} | {{ archcon_c1_status }} | {{ archcon_c2_status }} | {{ archcon_c3_status }} | {{ archcon_c4_status }} |
| security_considerations | {{ security_sem_score }} | {{ security_c1_status }} | {{ security_c2_status }} | {{ security_c3_status }} | {{ security_c4_status }} |
| performance_considerations | {{ perf_sem_score }} | {{ perf_c1_status }} | {{ perf_c2_status }} | {{ perf_c3_status }} | {{ perf_c4_status }} |
| failure_handling | {{ failure_sem_score }} | {{ failure_c1_status }} | {{ failure_c2_status }} | {{ failure_c3_status }} | {{ failure_c4_status }} |
| extension_points | {{ extension_sem_score }} | {{ extension_c1_status }} | {{ extension_c2_status }} | {{ extension_c3_status }} | {{ extension_c4_status }} |
| traceability | {{ trace_sem_score }} | {{ trace_c1_status }} | {{ trace_c2_status }} | {{ trace_c3_status }} | {{ trace_c4_status }} |

### Failures

| Section | Criterion | Severity | Evidence |
|---|---|---|---|
{{ sem_sec_failures }}

---

## Failures Summary

### Regressions from Previous Audit

| Rule | Previous Score | Current Score | Change | Evidence |
|---|---|---|---|---|
{{ regressions_table }}

### Persistent Failures

| Rule | First Failed | Consecutive Failures | Evidence |
|---|---|---|---|
{{ persistent_failures_table }}

---

## Document-Level Breakdown

| Category | Weight | Score | Percentage | Verdict |
|---|---|---|---|---|
| Structural Completeness | 4.0 | {{ structural_score }} | {{ structural_pct }} | {{ structural_verdict }} |
| Technology Independence | 1.0 | {{ tech_score }} | {{ tech_pct }} | {{ tech_verdict }} |
| Tier Enforcement | 1.0 | {{ tier_score }} | {{ tier_pct }} | {{ tier_verdict }} |
| Modularity | 0.5 | {{ modularity_score }} | {{ modularity_pct }} | {{ modularity_verdict }} |

---

## Section-Level Breakdown

| Section Category | Sections | Avg Score | Worst | Verdict |
|---|---|---|---|---|
| Required Sections | purpose, participating_components, component_interactions, data_ownership | {{ required_avg }} | {{ required_worst }} | {{ required_verdict }} |
| Technical Detail | feature_specification, component_responsibilities, runtime_behavior, communication_paths | {{ detail_avg }} | {{ detail_worst }} | {{ detail_verdict }} |
| Integration & Dependencies | integration_points, external_dependencies | {{ integration_avg }} | {{ integration_worst }} | {{ integration_verdict }} |
| Constraints & Quality | runtime_constraints, architectural_constraints, security_considerations, performance_considerations | {{ constraints_avg }} | {{ constraints_worst }} | {{ constraints_verdict }} |
| Resilience & Extension | failure_handling, extension_points | {{ resilience_avg }} | {{ resilience_worst }} | {{ resilience_verdict }} |
| Traceability | traceability | {{ trace_avg }} | {{ trace_worst }} | {{ trace_verdict }} |

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ combined_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})
