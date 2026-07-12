# {{ document_title }} — QA Semantic Section Audit Report

> **Domain:** qa
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

**Why this matters:** Semantic section audit evaluates the quality of each QA section individually — whether content is substantive, internally consistent, and project-specific rather than generic. Each section contributes to the overall test strategy coherence.

---

## Section: test_strategy

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ strategy_c1_status }}
- **Confidence:** {{ strategy_c1_confidence }}
- **Evidence:** {{ strategy_c1_evidence }}
- **Why this matters:** A Test Strategy section without project-specific content is a placeholder that provides no actual testing guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ strategy_c2_status }}
- **Confidence:** {{ strategy_c2_confidence }}
- **Evidence:** {{ strategy_c2_evidence }}
- **Why this matters:** Test Strategy that contradicts the actual test type sections creates confusion about which testing priorities are authoritative.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ strategy_c3_status }}
- **Confidence:** {{ strategy_c3_confidence }}
- **Evidence:** {{ strategy_c3_evidence }}
- **Why this matters:** "We test thoroughly" without specifying what "thoroughly" means for this project is unenforceable.

---

## Section: unit_testing

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ unit_c1_status }}
- **Confidence:** {{ unit_c1_confidence }}
- **Evidence:** {{ unit_c1_evidence }}
- **Why this matters:** Unit Testing without project-specific scope means developers don't know which components require unit tests or what conventions to follow.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ unit_c2_status }}
- **Confidence:** {{ unit_c2_confidence }}
- **Evidence:** {{ unit_c2_evidence }}
- **Why this matters:** Unit test conventions that contradict Engineering testing standards create developer confusion about which rules to follow.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ unit_c3_status }}
- **Confidence:** {{ unit_c3_confidence }}
- **Evidence:** {{ unit_c3_evidence }}
- **Why this matters:** "Write unit tests" without naming conventions, structure patterns, or coverage targets is operationally empty.

---

## Section: integration_testing

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ integration_c1_status }}
- **Confidence:** {{ integration_c1_confidence }}
- **Evidence:** {{ integration_c1_evidence }}
- **Why this matters:** Integration Testing without project-specific interface definitions means developers don't know which component boundaries are tested.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ integration_c2_status }}
- **Confidence:** {{ integration_c2_confidence }}
- **Evidence:** {{ integration_c2_evidence }}
- **Why this matters:** Integration Testing boundaries that contradict Architecture documentation create testing gaps or phantom interface tests.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ integration_c3_status }}
- **Confidence:** {{ integration_c3_confidence }}
- **Evidence:** {{ integration_c3_evidence }}
- **Why this matters:** "Test component interfaces" without naming specific interfaces, protocols, or scenarios is unimplementable.

---

## Section: security_testing

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ security_c1_status }}
- **Confidence:** {{ security_c1_confidence }}
- **Evidence:** {{ security_c1_evidence }}
- **Why this matters:** Security Testing without project-specific threat coverage means security tests are generic, not tailored to actual risks.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ security_c2_status }}
- **Confidence:** {{ security_c2_confidence }}
- **Evidence:** {{ security_c2_evidence }}
- **Why this matters:** Security Testing that contradicts the Security Documentation threat model creates confusion about which threats are actually being tested.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ security_c3_status }}
- **Confidence:** {{ security_c3_confidence }}
- **Evidence:** {{ security_c3_evidence }}
- **Why this matters:** "Run security tests" without specifying which methods (SAST, DAST, penetration) and which threats are covered is operationally empty.

---

## Section: purpose

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ purpose_c1_status }}
- **Confidence:** {{ purpose_c1_confidence }}
- **Evidence:** {{ purpose_c1_evidence }}
- **Why this matters:** QA documentation without a purpose section leaves readers guessing what this document is for and what it covers.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ purpose_c2_status }}
- **Confidence:** {{ purpose_c2_confidence }}
- **Evidence:** {{ purpose_c2_evidence }}
- **Why this matters:** Purpose that contradicts the actual scope of the document misleads readers about what is and isn't covered.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ purpose_c3_status }}
- **Confidence:** {{ purpose_c3_confidence }}
- **Evidence:** {{ purpose_c3_evidence }}
- **Why this matters:** A generic purpose statement ("this document describes QA") provides no value beyond what the filename already conveys.

---

## Section: e2e_testing

### Criteria

#### C1 — Critical user journeys named specifically
- **Weight:** mandatory
- **Score if passed:** 35
- **Status:** {{ e2e_c1_status }}
- **Confidence:** {{ e2e_c1_confidence }}
- **Evidence:** {{ e2e_c1_evidence }}
- **Why this matters:** "We test the main flows" with no journeys actually named means E2E scope is undefined — nobody knows what's being tested.

#### C2 — Expected outcomes and acceptance criteria stated per journey
- **Weight:** mandatory
- **Score if passed:** 35
- **Status:** {{ e2e_c2_status }}
- **Confidence:** {{ e2e_c2_confidence }}
- **Evidence:** {{ e2e_c2_evidence }}
- **Why this matters:** Journeys without expected outcomes or acceptance criteria have no pass/fail — testing is performative, not verifiable.

#### C3 — Journeys map to Design(06)'s documented workflows
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ e2e_c3_status }}
- **Confidence:** {{ e2e_c3_confidence }}
- **Evidence:** {{ e2e_c3_evidence }}
- **Why this matters:** E2E journeys invented independently of Design workflows test assumptions, not documented user expectations.

---

## Section: smoke_testing

### Criteria

#### C1 — Core function scope defined narrowly and explicitly
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ smoke_c1_status }}
- **Confidence:** {{ smoke_c1_confidence }}
- **Evidence:** {{ smoke_c1_evidence }}
- **Why this matters:** Smoke test scope that overlaps heavily with the full test suite defeats the "fast check" purpose — it's just a slow regression test with a different name.

#### C2 — Pass/fail criteria stated explicitly
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ smoke_c2_status }}
- **Confidence:** {{ smoke_c2_confidence }}
- **Evidence:** {{ smoke_c2_evidence }}
- **Why this matters:** Ambiguous pass/fail criteria ("looks okay") make smoke tests subjective — different operators reach different conclusions from the same results.

#### C3 — Execution timing and maximum duration threshold specified
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ smoke_c3_status }}
- **Confidence:** {{ smoke_c3_confidence }}
- **Evidence:** {{ smoke_c3_evidence }}
- **Why this matters:** A smoke test without a duration threshold could silently take as long as the full regression suite, defeating its purpose.

---

## Section: load_testing

### Criteria

#### C1 — Load profiles defined with concrete numbers
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ load_c1_status }}
- **Confidence:** {{ load_c1_confidence }}
- **Evidence:** {{ load_c1_evidence }}
- **Why this matters:** Load profiles described qualitatively ("normal load", "high load") with no numbers are unverifiable — there's no way to reproduce the test conditions.

#### C2 — Performance targets stated per profile
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ load_c2_status }}
- **Confidence:** {{ load_c2_confidence }}
- **Evidence:** {{ load_c2_evidence }}
- **Why this matters:** Performance targets without a load profile are meaningless — "response time under 200ms" means nothing without knowing under what load.

#### C3 — Acceptable degradation thresholds specified
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ load_c3_status }}
- **Confidence:** {{ load_c3_confidence }}
- **Evidence:** {{ load_c3_evidence }}
- **Why this matters:** No degradation threshold means pass/fail is all-or-nothing — a system that degrades gracefully under peak load has no defined boundary between "acceptable slowdown" and "failure".

---

## Section: scalability_testing

### Criteria

#### C1 — Growth scenarios defined as concrete multiples of baseline
- **Weight:** mandatory
- **Score if passed:** 35
- **Status:** {{ scalability_c1_status }}
- **Confidence:** {{ scalability_c1_confidence }}
- **Evidence:** {{ scalability_c1_evidence }}
- **Why this matters:** Scalability discussed only qualitatively ("the system should scale") provides no basis for capacity planning or architectural decisions.

#### C2 — Breaking points identified with specific thresholds
- **Weight:** mandatory
- **Score if passed:** 35
- **Status:** {{ scalability_c2_status }}
- **Confidence:** {{ scalability_c2_confidence }}
- **Evidence:** {{ scalability_c2_evidence }}
- **Why this matters:** Testing that stops before finding the actual breaking point leaves the growth curve unknown — you discover the limit in production, not in testing.

#### C3 — Scaling behavior characterized (linear/sub-linear/cliff)
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ scalability_c3_status }}
- **Confidence:** {{ scalability_c3_confidence }}
- **Evidence:** {{ scalability_c3_evidence }}
- **Why this matters:** Scaling behavior reduced to pass/fail with no characterization of the curve gives no information about where the system bends or how it degrades.

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
| test_strategy | {{ strategy_weight }} | {{ strategy_score }} | {{ strategy_status }} |
| unit_testing | {{ unit_weight }} | {{ unit_score }} | {{ unit_status }} |
| integration_testing | {{ integration_weight }} | {{ integration_score }} | {{ integration_status }} |
| security_testing | {{ security_weight }} | {{ security_score }} | {{ security_status }} |
| purpose | {{ purpose_weight }} | {{ purpose_score }} | {{ purpose_status }} |
| e2e_testing | {{ e2e_weight }} | {{ e2e_score }} | {{ e2e_status }} |
| smoke_testing | {{ smoke_weight }} | {{ smoke_score }} | {{ smoke_status }} |
| load_testing | {{ load_weight }} | {{ load_score }} | {{ load_status }} |
| scalability_testing | {{ scalability_weight }} | {{ scalability_score }} | {{ scalability_status }} |
