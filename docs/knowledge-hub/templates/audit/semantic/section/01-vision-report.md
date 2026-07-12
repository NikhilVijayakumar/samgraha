# {{ document_title }} — Vision Semantic Section Audit Report

> **Domain:** vision
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

**Why this matters:** Semantic section audit evaluates the quality of each vision section individually — whether content is substantive, internally consistent, technology-independent, and project-specific rather than generic. Each section contributes to the overall vision coherence.

---

## Section: purpose

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ purpose_c1_status }}
- **Confidence:** {{ purpose_c1_confidence }}
- **Evidence:** {{ purpose_c1_evidence }}
- **Why this matters:** A Purpose section without project-specific content is a placeholder that provides no actual vision guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ purpose_c2_status }}
- **Confidence:** {{ purpose_c2_confidence }}
- **Evidence:** {{ purpose_c2_evidence }}
- **Why this matters:** Purpose that contradicts the actual Vision Statement misleads stakeholders about what the vision aspires to.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ purpose_c3_status }}
- **Confidence:** {{ purpose_c3_confidence }}
- **Evidence:** {{ purpose_c3_evidence }}
- **Why this matters:** A generic purpose statement ("this document describes the vision") provides no value beyond what the filename already conveys.

---

## Section: vision_statement

### Criteria

#### C1 — Vision is 1-3 sentences describing a future state
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ vs_c1_status }}
- **Confidence:** {{ vs_c1_confidence }}
- **Evidence:** {{ vs_c1_evidence }}
- **Why this matters:** Vision statements longer than three sentences lose their north-star quality — they become specifications, not aspirations.

#### C2 — Vision is free of implementation-specific language
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ vs_c2_status }}
- **Confidence:** {{ vs_c2_confidence }}
- **Evidence:** {{ vs_c2_evidence }}
- **Why this matters:** Vision with implementation language ("we will build a microservices architecture") is a design doc, not a vision — it constrains solutions before the problem is fully understood.

#### C3 — Vision differentiates from current status quo or alternatives
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ vs_c3_status }}
- **Confidence:** {{ vs_c3_confidence }}
- **Evidence:** {{ vs_c3_evidence }}
- **Why this matters:** Vision indistinguishable from the status quo inspires no change — it describes what already exists, not what should exist.

---

## Section: problem

### Criteria

#### C1 — Problem stated from user/stakeholder perspective with evidence
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ problem_c1_status }}
- **Confidence:** {{ problem_c1_confidence }}
- **Evidence:** {{ problem_c1_evidence }}
- **Why this matters:** Problems described from the system's perspective ("the system lacks feature X") miss the human impact — the pain that motivates building something new.

#### C2 — Affected parties identified and problem scope bounded
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ problem_c2_status }}
- **Confidence:** {{ problem_c2_confidence }}
- **Evidence:** {{ problem_c2_evidence }}
- **Why this matters:** Unbounded problems ("users everywhere struggle with data") have no resolution criteria — the vision can never declare victory.

#### C3 — Current workarounds or alternatives acknowledged
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ problem_c3_status }}
- **Confidence:** {{ problem_c3_confidence }}
- **Evidence:** {{ problem_c3_evidence }}
- **Why this matters:** Problems that ignore existing workarounds may not be as painful as claimed — if users already have a viable alternative, the problem may be overstated.

---

## Section: solution

### Criteria

#### C1 — Solution addresses all aspects of the stated problem
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ solution_c1_status }}
- **Confidence:** {{ solution_c1_confidence }}
- **Evidence:** {{ solution_c1_evidence }}
- **Why this matters:** Solutions that address a different problem than the one Problem describes are visionary about the wrong thing.

#### C2 — Solution described at capability level without technology prescription
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ solution_c2_status }}
- **Confidence:** {{ solution_c2_confidence }}
- **Evidence:** {{ solution_c2_evidence }}
- **Why this matters:** Solutions that prescribe specific technologies constrain the solution space before the problem is fully understood.

#### C3 — Constraints and feasibility considerations acknowledged
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ solution_c3_status }}
- **Confidence:** {{ solution_c3_confidence }}
- **Evidence:** {{ solution_c3_evidence }}
- **Why this matters:** Solutions without feasibility considerations are aspirational fiction — they describe what could be, not what will be.

---

## Section: target_audience

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ ta_c1_status }}
- **Confidence:** {{ ta_c1_confidence }}
- **Evidence:** {{ ta_c1_evidence }}
- **Why this matters:** A Target Audience section without project-specific content is a placeholder that provides no actual audience guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ ta_c2_status }}
- **Confidence:** {{ ta_c2_confidence }}
- **Evidence:** {{ ta_c2_evidence }}
- **Why this matters:** Target Audience that contradicts the Problem or Solution creates confusion about who the vision actually serves.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ ta_c3_status }}
- **Confidence:** {{ ta_c3_confidence }}
- **Evidence:** {{ ta_c3_evidence }}
- **Why this matters:** Generic audience descriptions ("users", "enterprises") without specific personas or segments provide no actionable guidance.

---

## Section: pillars

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ pillars_c1_status }}
- **Confidence:** {{ pillars_c1_confidence }}
- **Evidence:** {{ pillars_c1_evidence }}
- **Why this matters:** A Pillars section without project-specific content is a placeholder that provides no actual architectural guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ pillars_c2_status }}
- **Confidence:** {{ pillars_c2_confidence }}
- **Evidence:** {{ pillars_c2_evidence }}
- **Why this matters:** Pillars that contradict the Vision Statement or Solution create confusion about what the vision actually values.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ pillars_c3_status }}
- **Confidence:** {{ pillars_c3_confidence }}
- **Evidence:** {{ pillars_c3_evidence }}
- **Why this matters:** Generic pillar descriptions ("scalability", "security") without project-specific definitions provide no actionable guidance.

---

## Section: philosophy

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ philosophy_c1_status }}
- **Confidence:** {{ philosophy_c1_confidence }}
- **Evidence:** {{ philosophy_c1_evidence }}
- **Why this matters:** A Philosophy section without project-specific content is a placeholder that provides no actual philosophical guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ philosophy_c2_status }}
- **Confidence:** {{ philosophy_c2_confidence }}
- **Evidence:** {{ philosophy_c2_evidence }}
- **Why this matters:** Philosophy that contradicts the Vision Statement or Pillars creates confusion about what the vision actually believes.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ philosophy_c3_status }}
- **Confidence:** {{ philosophy_c3_confidence }}
- **Evidence:** {{ philosophy_c3_evidence }}
- **Why this matters:** Generic philosophy descriptions ("we value quality") without project-specific beliefs provide no actionable guidance.

---

## Section: guiding_principles

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ gp_c1_status }}
- **Confidence:** {{ gp_c1_confidence }}
- **Evidence:** {{ gp_c1_evidence }}
- **Why this matters:** A Guiding Principles section without project-specific content is a placeholder that provides no actual principle guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ gp_c2_status }}
- **Confidence:** {{ gp_c2_confidence }}
- **Evidence:** {{ gp_c2_evidence }}
- **Why this matters:** Guiding Principles that contradict the Vision Statement or Pillars create confusion about what the vision actually prioritizes.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ gp_c3_status }}
- **Confidence:** {{ gp_c3_confidence }}
- **Evidence:** {{ gp_c3_evidence }}
- **Why this matters:** Generic guiding principle descriptions ("keep it simple") without project-specific applications provide no actionable guidance.

---

## Section: success_criteria

### Criteria

#### C1 — Criteria are specific, measurable, and include clear targets
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ sc_c1_status }}
- **Confidence:** {{ sc_c1_confidence }}
- **Evidence:** {{ sc_c1_evidence }}
- **Why this matters:** Success criteria that are untestable ("users will love the system") have no pass/fail — they're opinions, not success conditions.

#### C2 — Criteria span multiple dimensions (user, business, technical)
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ sc_c2_status }}
- **Confidence:** {{ sc_c2_confidence }}
- **Evidence:** {{ sc_c2_evidence }}
- **Why this matters:** Success criteria covering only one dimension (e.g., technical only) miss the full picture — a system can be technically excellent and commercially unsuccessful.

#### C3 — Criteria include timeframes, distinguish leading vs lagging, and state dimension priority for conflict resolution
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ sc_c3_status }}
- **Confidence:** {{ sc_c3_confidence }}
- **Evidence:** {{ sc_c3_evidence }}
- **Why this matters:** Multi-dimensional criteria with no conflict resolution leave stakeholders guessing which dimension wins when they point in opposite directions.

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
- **Why this matters:** Traceability links that contradict the vision's actual lineage create confusion about where the vision comes from and where it goes.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ trace_c3_status }}
- **Confidence:** {{ trace_c3_confidence }}
- **Evidence:** {{ trace_c3_evidence }}
- **Why this matters:** Generic traceability descriptions ("traceable to philosophy") without specific links provide no actual traceability value.

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
| vision_statement | {{ vs_weight }} | {{ vs_score }} | {{ vs_status }} |
| problem | {{ problem_weight }} | {{ problem_score }} | {{ problem_status }} |
| solution | {{ solution_weight }} | {{ solution_score }} | {{ solution_status }} |
| target_audience | {{ ta_weight }} | {{ ta_score }} | {{ ta_status }} |
| pillars | {{ pillars_weight }} | {{ pillars_score }} | {{ pillars_status }} |
| philosophy | {{ philosophy_weight }} | {{ philosophy_score }} | {{ philosophy_status }} |
| guiding_principles | {{ gp_weight }} | {{ gp_score }} | {{ gp_status }} |
| success_criteria | {{ sc_weight }} | {{ sc_score }} | {{ sc_status }} |
| traceability | {{ trace_weight }} | {{ trace_score }} | {{ trace_status }} |
