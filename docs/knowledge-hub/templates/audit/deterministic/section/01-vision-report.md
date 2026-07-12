# {{ document_title }} — Vision Section Audit Report

> **Domain:** vision
> **Scope:** section
> **Standard:** documentation-standards
> **Date:** {{ audit_date }}
- **Auditor:** {{ auditor_name }}

---

## Section-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | {{ section_weight_sum }} |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | {{ section_weight_sum }} |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Vision sections define specific aspirational components. Section-level audits verify each concern is internally consistent, technology-independent, and substantiated — the building blocks of a coherent vision.

---

## Section: purpose

### Rules

#### vis-sec-purpose-001 — Purpose section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'purpose'
- **Status:** {{ purpose_001_status }}
- **Evidence:** {{ purpose_001_evidence }}

#### vis-sec-purpose-002 — Purpose states vision intent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains a statement of why Vision Documentation exists
- **Status:** {{ purpose_002_status }}
- **Evidence:** {{ purpose_002_evidence }}

#### vis-sec-purpose-003 — Purpose is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section does not reference specific technologies, frameworks, or implementation details
- **Status:** {{ purpose_003_status }}
- **Evidence:** {{ purpose_003_evidence }}

#### vis-sec-purpose-004 — Purpose scope boundaries defined
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section defines what Vision Documentation is and is not
- **Status:** {{ purpose_004_status }}
- **Evidence:** {{ purpose_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-purpose-inspires-philosophy | derives_from | philosophy:purpose | incoming | {{ rel_purpose_philosophy }} |

---

## Section: vision_statement

### Rules

#### vis-sec-vs-001 — Vision Statement section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'vision_statement'
- **Status:** {{ vs_001_status }}
- **Evidence:** {{ vs_001_evidence }}

#### vis-sec-vs-002 — Vision Statement describes long-term direction
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains a forward-looking statement describing the product's desired future state
- **Status:** {{ vs_002_status }}
- **Evidence:** {{ vs_002_evidence }}

#### vis-sec-vs-003 — Vision Statement is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section does not reference specific technologies, frameworks, languages, or implementation approaches
- **Status:** {{ vs_003_status }}
- **Evidence:** {{ vs_003_evidence }}

#### vis-sec-vs-004 — Vision Statement is concise
- **Severity:** warning
- **Weight:** 0.3
- **Condition:** section is no longer than 500 words
- **Status:** {{ vs_004_status }}
- **Evidence:** {{ vs_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-vision-statement-inspires-philosophy | derives_from | philosophy:vision_alignment | incoming | {{ rel_vs_philosophy }} |
| vis-vision-statement-inspires-feature | derives_from | feature:purpose | incoming | {{ rel_vs_feature }} |
| vis-vision-statement-inspires-security | derives_from | security:purpose | incoming | {{ rel_vs_security }} |

---

## Section: problem

### Rules

#### vis-sec-problem-001 — Problem section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'problem'
- **Status:** {{ problem_001_status }}
- **Evidence:** {{ problem_001_evidence }}

#### vis-sec-problem-002 — Problem states the problem clearly
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section describes what problem or pain point the product addresses
- **Status:** {{ problem_002_status }}
- **Evidence:** {{ problem_002_evidence }}

#### vis-sec-problem-003 — Problem section does not describe solutions
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section focuses on the problem, not on how it will be solved
- **Status:** {{ problem_003_status }}
- **Evidence:** {{ problem_003_evidence }}

#### vis-sec-problem-004 — Problem is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section does not reference specific technologies, frameworks, or implementation details
- **Status:** {{ problem_004_status }}
- **Evidence:** {{ problem_004_evidence }}

#### vis-sec-problem-005 — Problem section has measurable impact
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section includes or references quantifiable impact of the problem
- **Status:** {{ problem_005_status }}
- **Evidence:** {{ problem_005_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-problem-inspires-philosophy | derives_from | philosophy:problem_framing | incoming | {{ rel_problem_philosophy }} |

---

## Section: solution

### Rules

#### vis-sec-solution-001 — Solution section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'solution'
- **Status:** {{ solution_001_status }}
- **Evidence:** {{ solution_001_evidence }}

#### vis-sec-solution-002 — Solution describes the proposed approach
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section describes the high-level approach to solving the stated problem
- **Status:** {{ solution_002_status }}
- **Evidence:** {{ solution_002_evidence }}

#### vis-sec-solution-003 — Solution is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section does not reference specific technologies, frameworks, APIs, libraries, or implementation details
- **Status:** {{ solution_003_status }}
- **Evidence:** {{ solution_003_evidence }}

#### vis-sec-solution-004 — Solution does not contain implementation specifics
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section does not contain code references, API names, schema definitions, or library names
- **Status:** {{ solution_004_status }}
- **Evidence:** {{ solution_004_evidence }}

#### vis-sec-solution-005 — Solution addresses the stated problem
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references or aligns with the problem statement
- **Status:** {{ solution_005_status }}
- **Evidence:** {{ solution_005_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-solution-inspires-feature | derives_from | feature:purpose | incoming | {{ rel_solution_feature }} |
| vis-solution-inspires-architecture | derives_from | architecture:system_overview | incoming | {{ rel_solution_architecture }} |

---

## Section: target_audience

### Rules

#### vis-sec-ta-001 — Target Audience section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'target_audience'
- **Status:** {{ ta_001_status }}
- **Evidence:** {{ ta_001_evidence }}

#### vis-sec-ta-002 — Target Audience identifies who the product serves
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section clearly identifies the primary users, customers, or beneficiaries
- **Status:** {{ ta_002_status }}
- **Evidence:** {{ ta_002_evidence }}

#### vis-sec-ta-003 — Target Audience is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section does not reference specific technologies, tools, or technical skill requirements
- **Status:** {{ ta_003_status }}
- **Evidence:** {{ ta_003_evidence }}

#### vis-sec-ta-004 — Target Audience includes at least two audience segments
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section identifies two or more distinct audience groups
- **Status:** {{ ta_004_status }}
- **Evidence:** {{ ta_004_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-target-audience-informs-philosophy | informs | philosophy:user_model | incoming | {{ rel_ta_philosophy }} |

---

## Section: pillars

### Rules

#### vis-sec-pillars-001 — Pillars section exists
- **Severity:** suggestion
- **Weight:** 0.3
- **Condition:** document has a section with semantic_type = 'pillars'
- **Status:** {{ pillars_001_status }}
- **Evidence:** {{ pillars_001_evidence }}

#### vis-sec-pillars-002 — Pillars are technology-independent
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section does not reference specific technologies, frameworks, or implementation details
- **Status:** {{ pillars_002_status }}
- **Evidence:** {{ pillars_002_evidence }}

#### vis-sec-pillars-003 — Pillars list at least three pillars
- **Severity:** suggestion
- **Weight:** 0.3
- **Condition:** section lists three or more core pillars
- **Status:** {{ pillars_003_status }}
- **Evidence:** {{ pillars_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-pillars-inspires-philosophy | derives_from | philosophy:guiding_principles | incoming | {{ rel_pillars_philosophy }} |

---

## Section: philosophy

### Rules

#### vis-sec-philosophy-001 — Philosophy section exists
- **Severity:** suggestion
- **Weight:** 0.3
- **Condition:** document has a section with semantic_type = 'philosophy'
- **Status:** {{ philosophy_001_status }}
- **Evidence:** {{ philosophy_001_evidence }}

#### vis-sec-philosophy-002 — Philosophy has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ philosophy_002_status }}
- **Evidence:** {{ philosophy_002_evidence }}

---

## Section: guiding_principles

### Rules

#### vis-sec-gp-001 — Guiding Principles section exists
- **Severity:** suggestion
- **Weight:** 0.3
- **Condition:** document has a section with semantic_type = 'guiding_principles'
- **Status:** {{ gp_001_status }}
- **Evidence:** {{ gp_001_evidence }}

#### vis-sec-gp-002 — Guiding Principles has substantive content
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ gp_002_status }}
- **Evidence:** {{ gp_002_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-guiding-principles-inspires-philosophy | derives_from | philosophy:guiding_principles | incoming | {{ rel_gp_philosophy }} |

---

## Section: success_criteria

### Rules

#### vis-sec-sc-001 — Success Criteria section exists
- **Severity:** suggestion
- **Weight:** 0.3
- **Condition:** document has a section with semantic_type = 'success_criteria'
- **Status:** {{ sc_001_status }}
- **Evidence:** {{ sc_001_evidence }}

#### vis-sec-sc-002 — Success Criteria are technology-independent
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section does not reference specific technologies, frameworks, or implementation details
- **Status:** {{ sc_002_status }}
- **Evidence:** {{ sc_002_evidence }}

#### vis-sec-sc-003 — Success Criteria are measurable
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section includes quantifiable or verifiable success metrics
- **Status:** {{ sc_003_status }}
- **Evidence:** {{ sc_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-success-criteria-traceable-to-feature | traceable_to | feature:acceptance_criteria | incoming | {{ rel_sc_feature }} |

---

## Section: traceability

### Rules

#### vis-sec-trace-001 — Traceability section exists
- **Severity:** suggestion
- **Weight:** 0.3
- **Condition:** document has a section with semantic_type = 'traceability'
- **Status:** {{ trace_001_status }}
- **Evidence:** {{ trace_001_evidence }}

#### vis-sec-trace-002 — Traceability links to upstream origins
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section documents the origin or inspiration source for the vision
- **Status:** {{ trace_002_status }}
- **Evidence:** {{ trace_002_evidence }}

#### vis-sec-trace-003 — Traceability links to downstream consumers
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section lists downstream domains that consume this vision (Philosophy, Feature, Security)
- **Status:** {{ trace_003_status }}
- **Evidence:** {{ trace_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| vis-traceability-no-upstream | traceable_to | null | incoming | {{ rel_trace_upstream }} |
| vis-traceability-downstream-philosophy | traceable_to | philosophy:purpose | incoming | {{ rel_trace_philosophy }} |
| vis-traceability-downstream-feature | traceable_to | feature:purpose | incoming | {{ rel_trace_feature }} |
| vis-traceability-downstream-security | traceable_to | security:purpose | incoming | {{ rel_trace_security }} |

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
| vision_statement | {{ vs_weight }} | {{ vs_score }} | {{ vs_status }} |
| problem | {{ problem_weight }} | {{ problem_score }} | {{ problem_status }} |
| solution | {{ solution_weight }} | {{ solution_score }} | {{ solution_status }} |
| target_audience | {{ ta_weight }} | {{ ta_score }} | {{ ta_status }} |
| pillars | {{ pillars_weight }} | {{ pillars_score }} | {{ pillars_status }} |
| philosophy | {{ philosophy_weight }} | {{ philosophy_score }} | {{ philosophy_status }} |
| guiding_principles | {{ gp_weight }} | {{ gp_score }} | {{ gp_status }} |
| success_criteria | {{ sc_weight }} | {{ sc_score }} | {{ sc_status }} |
| traceability | {{ trace_weight }} | {{ trace_score }} | {{ trace_status }} |
