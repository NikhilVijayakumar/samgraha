# {{ document_title }} — Feature Semantic Section Audit Report

> **Domain:** feature
> **Scope:** section
> **Kind:** semantic
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

**Why this matters:** Semantic section audit evaluates the quality of each feature section individually — whether content is substantive, internally consistent, technology-independent, and project-specific rather than generic. Each section contributes to the overall feature specification coherence.

---

## Section: purpose

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ purpose_c1_status }}
- **Confidence:** {{ purpose_c1_confidence }}
- **Evidence:** {{ purpose_c1_evidence }}
- **Why this matters:** A Purpose section without project-specific content is a placeholder that provides no actual feature guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ purpose_c2_status }}
- **Confidence:** {{ purpose_c2_confidence }}
- **Evidence:** {{ purpose_c2_evidence }}
- **Why this matters:** Purpose that contradicts the actual Functional Requirements misleads stakeholders about what the feature delivers.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ purpose_c3_status }}
- **Confidence:** {{ purpose_c3_confidence }}
- **Evidence:** {{ purpose_c3_evidence }}
- **Why this matters:** A generic purpose statement ("this feature enables users") provides no value beyond what the filename already conveys.

---

## Section: functional_requirements

### Criteria

#### C1 — All requirements uniquely identified
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ freq_c1_status }}
- **Confidence:** {{ freq_c1_confidence }}
- **Evidence:** {{ freq_c1_evidence }}
- **Why this matters:** Requirements without unique identifiers cannot be traced to acceptance criteria, test cases, or downstream design artifacts.

#### C2 — Each requirement is testable
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ freq_c2_status }}
- **Confidence:** {{ freq_c2_confidence }}
- **Evidence:** {{ freq_c2_evidence }}
- **Why this matters:** Untestable requirements ("the system shall be fast") have no pass/fail boundary — they're aspirational, not contractual.

#### C3 — No implementation language
- **Weight:** recommended
- **Score if passed:** 20
- **Status:** {{ freq_c3_status }}
- **Confidence:** {{ freq_c3_confidence }}
- **Evidence:** {{ freq_c3_evidence }}
- **Why this matters:** Requirements that name specific technologies couple the specification to implementation details — when the technology changes, the requirements shouldn't need rewriting.

#### C4 — No duplicate or conflicting requirements
- **Weight:** recommended
- **Score if passed:** 20
- **Status:** {{ freq_c4_status }}
- **Confidence:** {{ freq_c4_confidence }}
- **Evidence:** {{ freq_c4_evidence }}
- **Why this matters:** Duplicate requirements create maintenance risk — one gets updated, the other doesn't, and they silently contradict.

---

## Section: acceptance_criteria

### Criteria

#### C1 — Every criterion is pass/fail testable
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ ac_c1_status }}
- **Confidence:** {{ ac_c1_confidence }}
- **Evidence:** {{ ac_c1_evidence }}
- **Why this matters:** Criteria that are too vague to write a test for have no pass/fail — they're opinions, not acceptance conditions.

#### C2 — Each criterion tests a single behavior
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ ac_c2_status }}
- **Confidence:** {{ ac_c2_confidence }}
- **Evidence:** {{ ac_c2_evidence }}
- **Why this matters:** Compound criteria joined by "and" or "or" cannot be independently verified — partial pass is ambiguous.

#### C3 — Criteria use structured Given/When/Then format
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ ac_c3_status }}
- **Confidence:** {{ ac_c3_confidence }}
- **Evidence:** {{ ac_c3_evidence }}
- **Why this matters:** Structured format eliminates interpretation ambiguity — everyone reads the same criterion the same way.

---

## Section: business_rules

### Criteria

#### C1 — Each rule is atomic and unambiguous
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ br_c1_status }}
- **Confidence:** {{ br_c1_confidence }}
- **Evidence:** {{ br_c1_evidence }}
- **Why this matters:** Non-atomic rules that combine multiple conditions or actions cannot be individually validated — partial compliance is ambiguous.

#### C2 — Rules are expressed declaratively
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ br_c2_status }}
- **Confidence:** {{ br_c2_confidence }}
- **Evidence:** {{ br_c2_evidence }}
- **Why this matters:** Rules expressed as pseudocode or procedural logic describe implementation, not business intent — they break when the implementation changes.

#### C3 — Exception paths are documented
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ br_c3_status }}
- **Confidence:** {{ br_c3_confidence }}
- **Evidence:** {{ br_c3_evidence }}
- **Why this matters:** Business rules without exception paths handle the happy case but fail silently on edge cases — the most common source of production bugs.

---

## Section: inputs

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ inputs_c1_status }}
- **Confidence:** {{ inputs_c1_confidence }}
- **Evidence:** {{ inputs_c1_evidence }}
- **Why this matters:** An Inputs section without project-specific content is a placeholder that provides no actual input specification.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ inputs_c2_status }}
- **Confidence:** {{ inputs_c2_confidence }}
- **Evidence:** {{ inputs_c2_evidence }}
- **Why this matters:** Inputs that contradict Functional Requirements create confusion about what data the feature actually accepts.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ inputs_c3_status }}
- **Confidence:** {{ inputs_c3_confidence }}
- **Evidence:** {{ inputs_c3_evidence }}
- **Why this matters:** Generic input descriptions ("user data") without specific fields, types, or validation rules are unimplementable.

---

## Section: outputs

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ outputs_c1_status }}
- **Confidence:** {{ outputs_c1_confidence }}
- **Evidence:** {{ outputs_c1_evidence }}
- **Why this matters:** An Outputs section without project-specific content is a placeholder that provides no actual output specification.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ outputs_c2_status }}
- **Confidence:** {{ outputs_c2_confidence }}
- **Evidence:** {{ outputs_c2_evidence }}
- **Why this matters:** Outputs that contradict Acceptance Criteria create confusion about what the feature actually produces.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ outputs_c3_status }}
- **Confidence:** {{ outputs_c3_confidence }}
- **Evidence:** {{ outputs_c3_evidence }}
- **Why this matters:** Generic output descriptions ("response data") without specific fields, formats, or error states are unimplementable.

---

## Section: constraints

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ constraints_c1_status }}
- **Confidence:** {{ constraints_c1_confidence }}
- **Evidence:** {{ constraints_c1_evidence }}
- **Why this matters:** A Constraints section without project-specific content is a placeholder that provides no actual constraint guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ constraints_c2_status }}
- **Confidence:** {{ constraints_c2_confidence }}
- **Evidence:** {{ constraints_c2_evidence }}
- **Why this matters:** Constraints that contradict Functional Requirements or Business Rules create impossible compliance situations.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ constraints_c3_status }}
- **Confidence:** {{ constraints_c3_confidence }}
- **Evidence:** {{ constraints_c3_evidence }}
- **Why this matters:** Generic constraint descriptions ("performance requirements") without specific numbers or boundaries are unenforceable.

---

## Section: dependencies

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ dependencies_c1_status }}
- **Confidence:** {{ dependencies_c1_confidence }}
- **Evidence:** {{ dependencies_c1_evidence }}
- **Why this matters:** A Dependencies section without project-specific content is a placeholder that provides no actual dependency information.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ dependencies_c2_status }}
- **Confidence:** {{ dependencies_c2_confidence }}
- **Evidence:** {{ dependencies_c2_evidence }}
- **Why this matters:** Dependencies that contradict Constraints or other sections create confusion about what the feature actually requires.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ dependencies_c3_status }}
- **Confidence:** {{ dependencies_c3_confidence }}
- **Evidence:** {{ dependencies_c3_evidence }}
- **Why this matters:** Generic dependency descriptions ("external services") without specific services, versions, or SLAs are unmanageable.

---

## Section: non_goals

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ non_goals_c1_status }}
- **Confidence:** {{ non_goals_c1_confidence }}
- **Evidence:** {{ non_goals_c1_evidence }}
- **Why this matters:** A Non-goals section without project-specific content is a placeholder that provides no actual scope guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ non_goals_c2_status }}
- **Confidence:** {{ non_goals_c2_confidence }}
- **Evidence:** {{ non_goals_c2_evidence }}
- **Why this matters:** Non-goals that contradict Functional Requirements create confusion about what's in scope and what's explicitly excluded.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ non_goals_c3_status }}
- **Confidence:** {{ non_goals_c3_confidence }}
- **Evidence:** {{ non_goals_c3_evidence }}
- **Why this matters:** Generic non-goal descriptions ("out of scope") without specific exclusions provide no actual scope guidance.

---

## Section: future_extensions

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ future_c1_status }}
- **Confidence:** {{ future_c1_confidence }}
- **Evidence:** {{ future_c1_evidence }}
- **Why this matters:** A Future Extensions section without project-specific content is a placeholder that provides no actual forward-looking guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ future_c2_status }}
- **Confidence:** {{ future_c2_confidence }}
- **Evidence:** {{ future_c2_evidence }}
- **Why this matters:** Future extensions that contradict current Functional Requirements create confusion about what's planned vs. what's delivered.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ future_c3_status }}
- **Confidence:** {{ future_c3_confidence }}
- **Evidence:** {{ future_c3_evidence }}
- **Why this matters:** Generic future extension descriptions ("we might add this later") provide no actionable guidance for downstream planning.

---

## Section: traceability

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ trace_c1_status }}
- **Confidence:** {{ trace_c1_confidence }}
- **Evidence:** {{ trace_c1_evidence }}
- **Why this matters:** A Traceability section without project-specific content is a placeholder that provides no actual traceability.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ trace_c2_status }}
- **Confidence:** {{ trace_c2_confidence }}
- **Evidence:** {{ trace_c2_evidence }}
- **Why this matters:** Traceability links that contradict upstream Vision or downstream Design create confusion about the feature's actual lineage.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ trace_c3_status }}
- **Confidence:** {{ trace_c3_confidence }}
- **Evidence:** {{ trace_c3_evidence }}
- **Why this matters:** Generic traceability descriptions ("traceable to vision") without specific links provide no actual traceability value.

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

| Criterion | Severity | Section | Evidence | Regression? |
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
