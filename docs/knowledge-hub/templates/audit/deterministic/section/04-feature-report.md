# {{ document_title }} — Feature Section Audit Report

> **Domain:** feature
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

**Why this matters:** Feature sections define specific requirements, acceptance criteria, and business rules. Section-level audits verify each concern is internally consistent, testable, and technology-independent — the building blocks of a coherent feature specification.

---

## Section: purpose

### Rules

#### feat-sec-purpose-001 — Purpose section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'purpose'
- **Status:** {{ purpose_001_status }}
- **Evidence:** {{ purpose_001_evidence }}

#### feat-sec-purpose-002 — Purpose states feature intent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains a statement of why this feature exists and the problem it solves
- **Status:** {{ purpose_002_status }}
- **Evidence:** {{ purpose_002_evidence }}

#### feat-sec-purpose-003 — Purpose is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section does not reference specific technologies, frameworks, or implementation details
- **Status:** {{ purpose_003_status }}
- **Evidence:** {{ purpose_003_evidence }}

#### feat-sec-purpose-004 — Purpose scope boundaries defined
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section defines what the feature is and is not
- **Status:** {{ purpose_004_status }}
- **Evidence:** {{ purpose_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| feat-purpose-derives-vision | derives_from | vision:vision_statement | incoming | {{ rel_purpose_vision }} |
| feat-purpose-guided-by-philosophy | guided_by | philosophy:guiding_principles | incoming | {{ rel_purpose_philosophy }} |

---

## Section: functional_requirements

### Rules

#### feat-sec-freq-001 — Functional requirements section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'functional_requirements'
- **Status:** {{ freq_001_status }}
- **Evidence:** {{ freq_001_evidence }}

#### feat-sec-freq-002 — Requirements are listed individually
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least two distinct requirements, each clearly stated
- **Status:** {{ freq_002_status }}
- **Evidence:** {{ freq_002_evidence }}

#### feat-sec-freq-003 — Each requirement is testable
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** each requirement can be verified through a concrete test or observation
- **Status:** {{ freq_003_status }}
- **Evidence:** {{ freq_003_evidence }}

#### feat-sec-freq-004 — Requirements are technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** requirements describe what the feature does, not how it is implemented
- **Status:** {{ freq_004_status }}
- **Evidence:** {{ freq_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| feat-functional-requirements-derives-feature-design | derives_from | feature_design:design_rationale | incoming | {{ rel_freq_feature_design }} |

---

## Section: acceptance_criteria

### Rules

#### feat-sec-ac-001 — Acceptance criteria section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'acceptance_criteria'
- **Status:** {{ ac_001_status }}
- **Evidence:** {{ ac_001_evidence }}

#### feat-sec-ac-002 — Acceptance criteria are testable
- **Severity:** error
- **Weight:** 1.0
- **Condition:** each criterion is phrased as a concrete, verifiable condition
- **Status:** {{ ac_002_status }}
- **Evidence:** {{ ac_002_evidence }}

#### feat-sec-ac-003 — Acceptance criteria cover happy path and edge cases
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section includes both normal flow and error/edge-case scenarios
- **Status:** {{ ac_003_status }}
- **Evidence:** {{ ac_003_evidence }}

#### feat-sec-ac-004 — Acceptance criteria are technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** criteria describe observable behavior, not implementation details
- **Status:** {{ ac_004_status }}
- **Evidence:** {{ ac_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| feat-acceptance-criteria-derives-feature-technical | derives_from | feature_technical:testing_strategy | incoming | {{ rel_ac_feature_technical }} |

---

## Section: business_rules

### Rules

#### feat-sec-br-001 — Business rules section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'business_rules'
- **Status:** {{ br_001_status }}
- **Evidence:** {{ br_001_evidence }}

#### feat-sec-br-002 — Business rules are clearly stated
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** each rule is a discrete, unambiguous statement
- **Status:** {{ br_002_status }}
- **Evidence:** {{ br_002_evidence }}

#### feat-sec-br-003 — Business rules trace to requirements
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** each rule can be linked to one or more functional requirements
- **Status:** {{ br_003_status }}
- **Evidence:** {{ br_003_evidence }}

#### feat-sec-br-004 — Business rules are technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** rules describe business logic, not technical implementation
- **Status:** {{ br_004_status }}
- **Evidence:** {{ br_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| feat-constraints-derives-philosophy | constrains | philosophy:guiding_principles | incoming | {{ rel_br_philosophy }} |

---

## Section: inputs

### Rules

#### feat-sec-inputs-001 — Inputs section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'inputs'
- **Status:** {{ inputs_001_status }}
- **Evidence:** {{ inputs_001_evidence }}

#### feat-sec-inputs-002 — Inputs has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ inputs_002_status }}
- **Evidence:** {{ inputs_002_evidence }}

---

## Section: outputs

### Rules

#### feat-sec-outputs-001 — Outputs section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'outputs'
- **Status:** {{ outputs_001_status }}
- **Evidence:** {{ outputs_001_evidence }}

#### feat-sec-outputs-002 — Outputs has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ outputs_002_status }}
- **Evidence:** {{ outputs_002_evidence }}

---

## Section: constraints

### Rules

#### feat-sec-constraints-001 — Constraints section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'constraints'
- **Status:** {{ constraints_001_status }}
- **Evidence:** {{ constraints_001_evidence }}

#### feat-sec-constraints-002 — Constraints has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ constraints_002_status }}
- **Evidence:** {{ constraints_002_evidence }}

---

## Section: dependencies

### Rules

#### feat-sec-dependencies-001 — Dependencies section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'dependencies'
- **Status:** {{ dependencies_001_status }}
- **Evidence:** {{ dependencies_001_evidence }}

#### feat-sec-dependencies-002 — Dependencies has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ dependencies_002_status }}
- **Evidence:** {{ dependencies_002_evidence }}

---

## Section: non_goals

### Rules

#### feat-sec-non_goals-001 — Non-goals section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'non_goals'
- **Status:** {{ non_goals_001_status }}
- **Evidence:** {{ non_goals_001_evidence }}

#### feat-sec-non_goals-002 — Non-goals has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ non_goals_002_status }}
- **Evidence:** {{ non_goals_002_evidence }}

---

## Section: future_extensions

### Rules

#### feat-sec-future_extensions-001 — Future extensions section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'future_extensions'
- **Status:** {{ future_001_status }}
- **Evidence:** {{ future_001_evidence }}

#### feat-sec-future_extensions-002 — Future extensions has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ future_002_status }}
- **Evidence:** {{ future_002_evidence }}

---

## Section: traceability

### Rules

#### feat-sec-trace-001 — Traceability section exists
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a section with semantic_type = 'traceability'
- **Status:** {{ trace_001_status }}
- **Evidence:** {{ trace_001_evidence }}

#### feat-sec-trace-002 — Traceability links to vision
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section traces feature back to a vision element
- **Status:** {{ trace_002_status }}
- **Evidence:** {{ trace_002_evidence }}

#### feat-sec-trace-003 — Traceability links to downstream
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section identifies downstream artifacts that derive from this feature
- **Status:** {{ trace_003_status }}
- **Evidence:** {{ trace_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| feat-traceability-derives-vision | traceable_to | vision:vision_statement | incoming | {{ rel_trace_vision }} |

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
| purpose | {{ purpose_weight }} | {{ purpose_score }} | {{ purpose_status }} |
| functional_requirements | {{ freq_weight }} | {{ freq_score }} | {{ freq_status }} |
| acceptance_criteria | {{ ac_weight }} | {{ ac_score }} | {{ ac_status }} |
| business_rules | {{ br_weight }} | {{ br_score }} | {{ br_status }} |
| inputs | {{ inputs_weight }} | {{ inputs_score }} | {{ inputs_status }} |
| outputs | {{ outputs_weight }} | {{ outputs_score }} | {{ outputs_status }} |
| constraints | {{ constraints_weight }} | {{ constraints_score }} | {{ constraints_status }} |
| dependencies | {{ dependencies_weight }} | {{ dependencies_score }} | {{ dependencies_status }} |
| non_goals | {{ non_goals_weight }} | {{ non_goals_score }} | {{ non_goals_status }} |
| future_extensions | {{ future_weight }} | {{ future_score }} | {{ future_status }} |
| traceability | {{ trace_weight }} | {{ trace_score }} | {{ trace_status }} |
