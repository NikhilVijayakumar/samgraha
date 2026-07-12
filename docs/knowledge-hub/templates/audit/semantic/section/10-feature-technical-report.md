# {{ document_title }} — Feature Technical Semantic Section Audit Report

> **Domain:** feature-technical
> **Scope:** section
> **Kind:** semantic
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Document-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | 100 |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | 100 |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Section-level semantic audit checks that each Feature Technical section delivers real content — not generic boilerplate, not placeholders, not implementation-specific details. Each section must be specific to this feature, technology-independent, and backed by evidence.

---

## Required Sections

### purpose
#### C1 — Technical problem and motivation clearly stated
- **Weight:** mandatory (40)
- **Status:** {{ purpose_c1_status }}
- **Evidence:** {{ purpose_c1_evidence }}
- **Why this matters:** Purpose without a clear technical problem gives implementers no understanding of what engineering value this feature delivers.

#### C2 — Engineering value proposition articulated
- **Weight:** mandatory (30)
- **Status:** {{ purpose_c2_status }}
- **Evidence:** {{ purpose_c2_evidence }}
- **Why this matters:** Purpose without an engineering value proposition leaves implementers unable to justify the technical effort.

#### C3 — Success criteria for technical impact defined
- **Weight:** recommended (30)
- **Status:** {{ purpose_c3_status }}
- **Evidence:** {{ purpose_c3_evidence }}
- **Why this matters:** Purpose without success criteria leaves implementers unable to validate the feature achieved its technical goals.

---

### participating_components
#### C1 — All participating components enumerated
- **Weight:** mandatory (30)
- **Status:** {{ components_c1_status }}
- **Evidence:** {{ components_c1_evidence }}
- **Why this matters:** Missing components mean the technical design is incomplete — implementers will discover undocumented parts during implementation.

#### C2 — Deployment unit specified per component
- **Weight:** mandatory (30)
- **Status:** {{ components_c2_status }}
- **Evidence:** {{ components_c2_evidence }}
- **Why this matters:** Components without deployment units leave implementers guessing about packaging and deployment boundaries.

#### C3 — Each component has a defined feature role
- **Weight:** recommended (20)
- **Status:** {{ components_c3_status }}
- **Evidence:** {{ components_c3_evidence }}
- **Why this matters:** Components without defined roles lead to ambiguous responsibilities during implementation.

#### C4 — State model documented per component
- **Weight:** recommended (20)
- **Status:** {{ components_c4_status }}
- **Evidence:** {{ components_c4_evidence }}
- **Why this matters:** Components without state models leave implementers unsure of lifecycle behavior (active, passive, standby).

---

### component_interactions
#### C1 — All component interactions identified
- **Weight:** mandatory (30)
- **Status:** {{ interactions_c1_status }}
- **Evidence:** {{ interactions_c1_evidence }}
- **Why this matters:** Missing interactions mean the technical design is incomplete — implementers will discover undocumented coupling during implementation.

#### C2 — Each interaction specifies direction and communication type
- **Weight:** mandatory (30)
- **Status:** {{ interactions_c2_status }}
- **Evidence:** {{ interactions_c2_evidence }}
- **Why this matters:** Interactions without direction and type leave implementers guessing about the communication model.

#### C3 — No undocumented cyclic dependencies
- **Weight:** recommended (20)
- **Status:** {{ interactions_c3_status }}
- **Evidence:** {{ interactions_c3_evidence }}
- **Why this matters:** Undocumented cyclic dependencies create initialization order problems and testing difficulties.

#### C4 — Data contracts or interfaces referenced per interaction
- **Weight:** recommended (20)
- **Status:** {{ interactions_c4_status }}
- **Evidence:** {{ interactions_c4_evidence }}
- **Why this matters:** Interactions without data contracts leave implementers guessing about the interface between components.

---

### data_ownership
#### C1 — All data entities have designated owners
- **Weight:** mandatory (30)
- **Status:** {{ data_c1_status }}
- **Evidence:** {{ data_c1_evidence }}
- **Why this matters:** Data entities without owners create ownership ambiguity — multiple components may attempt to manage the same data.

#### C2 — Write authority is assigned and non-conflicting
- **Weight:** mandatory (30)
- **Status:** {{ data_c2_status }}
- **Evidence:** {{ data_c2_evidence }}
- **Why this matters:** Conflicting write authority leads to data races and inconsistent state.

#### C3 — Data lifecycle documented per entity
- **Weight:** recommended (20)
- **Status:** {{ data_c3_status }}
- **Evidence:** {{ data_c3_evidence }}
- **Why this matters:** Data without lifecycle documentation leaves implementers guessing about create/read/update/delete patterns.

#### C4 — Data retention and purging policies defined
- **Weight:** recommended (20)
- **Status:** {{ data_c4_status }}
- **Evidence:** {{ data_c4_evidence }}
- **Why this matters:** Data without retention policies leads to unbounded growth and compliance risks.

---

## Optional Sections

### feature_specification
#### C1 — Feature scope with inclusion and exclusion criteria
- **Weight:** mandatory (30)
- **Status:** {{ spec_c1_status }}
- **Evidence:** {{ spec_c1_evidence }}
- **Why this matters:** Specification without scope boundaries leaves implementers unsure what the feature does and does not cover.

#### C2 — Preconditions and postconditions defined
- **Weight:** mandatory (30)
- **Status:** {{ spec_c2_status }}
- **Evidence:** {{ spec_c2_evidence }}
- **Why this matters:** Specification without preconditions and postconditions leaves implementers unable to validate correct behavior.

#### C3 — Input and output specifications with types
- **Weight:** recommended (20)
- **Status:** {{ spec_c3_status }}
- **Evidence:** {{ spec_c3_evidence }}
- **Why this matters:** Specification without typed inputs and outputs leaves implementers guessing about the interface contract.

#### C4 — Feature-level invariants documented
- **Weight:** recommended (20)
- **Status:** {{ spec_c4_status }}
- **Evidence:** {{ spec_c4_evidence }}
- **Why this matters:** Specification without invariants leaves implementers unable to validate the feature maintains its guarantees.

---

### component_responsibilities
#### C1 — All components have explicit responsibility statements
- **Weight:** mandatory (30)
- **Status:** {{ resp_c1_status }}
- **Evidence:** {{ resp_c1_evidence }}
- **Why this matters:** Components without responsibility statements leave implementers guessing about accountability.

#### C2 — No overlapping responsibilities across components
- **Weight:** mandatory (30)
- **Status:** {{ resp_c2_status }}
- **Evidence:** {{ resp_c2_evidence }}
- **Why this matters:** Overlapping responsibilities lead to duplicated logic and inconsistent behavior.

#### C3 — Each component has a primary single responsibility
- **Weight:** recommended (20)
- **Status:** {{ resp_c3_status }}
- **Evidence:** {{ resp_c3_evidence }}
- **Why this matters:** Components with compound responsibilities are harder to test, maintain, and replace.

#### C4 — Gap analysis performed for uncovered capabilities
- **Weight:** recommended (20)
- **Status:** {{ resp_c4_status }}
- **Evidence:** {{ resp_c4_evidence }}
- **Why this matters:** Gaps in capability coverage mean some required behavior has no owner.

---

### runtime_behavior
#### C1 — Startup and shutdown sequences documented
- **Weight:** mandatory (30)
- **Status:** {{ runtime_c1_status }}
- **Evidence:** {{ runtime_c1_evidence }}
- **Why this matters:** Runtime behavior without startup/shutdown sequences leaves implementers guessing about initialization order and cleanup.

#### C2 — State transitions with triggers enumerated
- **Weight:** mandatory (30)
- **Status:** {{ runtime_c2_status }}
- **Evidence:** {{ runtime_c2_evidence }}
- **Why this matters:** State transitions without triggers leave implementers unable to determine when the feature changes state.

#### C3 — Concurrency and threading model described
- **Weight:** recommended (20)
- **Status:** {{ runtime_c3_status }}
- **Evidence:** {{ runtime_c3_evidence }}
- **Why this matters:** Runtime behavior without concurrency documentation leaves implementers guessing about thread safety requirements.

#### C4 — Observable side effects listed
- **Weight:** recommended (20)
- **Status:** {{ runtime_c4_status }}
- **Evidence:** {{ runtime_c4_evidence }}
- **Why this matters:** Runtime behavior without observable side effects leaves implementers unable to verify the feature is working correctly.

---

### communication_paths
#### C1 — All communication paths enumerated source-to-sink
- **Weight:** mandatory (30)
- **Status:** {{ comm_c1_status }}
- **Evidence:** {{ comm_c1_evidence }}
- **Why this matters:** Missing paths mean the technical design is incomplete — implementers will discover undocumented data flows during implementation.

#### C2 — Delivery guarantees defined per path
- **Weight:** mandatory (30)
- **Status:** {{ comm_c2_status }}
- **Evidence:** {{ comm_c2_evidence }}
- **Why this matters:** Paths without delivery guarantees leave implementers guessing about exactly-once, at-least-once, or at-most-once semantics.

#### C3 — Backpressure or flow control documented
- **Weight:** recommended (20)
- **Status:** {{ comm_c3_status }}
- **Evidence:** {{ comm_c3_evidence }}
- **Why this matters:** Paths without backpressure documentation may lead to resource exhaustion under load.

#### C4 — Serialization format specified per path
- **Weight:** recommended (20)
- **Status:** {{ comm_c4_status }}
- **Evidence:** {{ comm_c4_evidence }}
- **Why this matters:** Paths without serialization format leave implementers guessing about data encoding.

---

### integration_points
#### C1 — All integration points enumerated with external system
- **Weight:** mandatory (30)
- **Status:** {{ integration_c1_status }}
- **Evidence:** {{ integration_c1_evidence }}
- **Why this matters:** Missing integration points mean the technical design is incomplete — implementers will discover undocumented external boundaries during implementation.

#### C2 — Interface contract specified per integration point
- **Weight:** mandatory (30)
- **Status:** {{ integration_c2_status }}
- **Evidence:** {{ integration_c2_evidence }}
- **Why this matters:** Integration points without interface contracts leave implementers guessing about the external boundary.

#### C3 — Error contract documented per boundary
- **Weight:** recommended (20)
- **Status:** {{ integration_c3_status }}
- **Evidence:** {{ integration_c3_evidence }}
- **Why this matters:** Integration points without error contracts leave implementers unable to handle external failures.

#### C4 — SLA parameters defined for each integration point
- **Weight:** recommended (20)
- **Status:** {{ integration_c4_status }}
- **Evidence:** {{ integration_c4_evidence }}
- **Why this matters:** Integration points without SLA parameters leave implementers unable to validate performance requirements.

---

### external_dependencies
#### C1 — All external dependencies listed with name and version
- **Weight:** mandatory (30)
- **Status:** {{ extdep_c1_status }}
- **Evidence:** {{ extdep_c1_evidence }}
- **Why this matters:** Dependencies without names and versions leave implementers unable to set up the build environment.

#### C2 — Purpose documented per dependency
- **Weight:** mandatory (30)
- **Status:** {{ extdep_c2_status }}
- **Evidence:** {{ extdep_c2_evidence }}
- **Why this matters:** Dependencies without purpose leave implementers unable to evaluate whether a dependency is still needed.

#### C3 — Integration method specified per dependency
- **Weight:** recommended (20)
- **Status:** {{ extdep_c3_status }}
- **Evidence:** {{ extdep_c3_evidence }}
- **Why this matters:** Dependencies without integration method leave implementers guessing about how to connect.

#### C4 — Licensing and operational constraints documented
- **Weight:** recommended (20)
- **Status:** {{ extdep_c4_status }}
- **Evidence:** {{ extdep_c4_evidence }}
- **Why this matters:** Dependencies without licensing constraints may create legal or operational risks.

---

### runtime_constraints
#### C1 — All runtime constraints enumerated
- **Weight:** mandatory (30)
- **Status:** {{ rtcon_c1_status }}
- **Evidence:** {{ rtcon_c1_evidence }}
- **Why this matters:** Missing constraints mean implementers may build a feature that violates unstated requirements.

#### C2 — Measurable thresholds defined per constraint
- **Weight:** mandatory (30)
- **Status:** {{ rtcon_c2_status }}
- **Evidence:** {{ rtcon_c2_evidence }}
- **Why this matters:** Constraints without measurable thresholds are unverifiable — implementers cannot validate they've met the requirement.

#### C3 — Constraint source referenced
- **Weight:** recommended (20)
- **Status:** {{ rtcon_c3_status }}
- **Evidence:** {{ rtcon_c3_evidence }}
- **Why this matters:** Constraints without source references have no provenance — implementers cannot verify the constraint is real.

#### C4 — Trade-offs between constraints documented
- **Weight:** recommended (20)
- **Status:** {{ rtcon_c4_status }}
- **Evidence:** {{ rtcon_c4_evidence }}
- **Why this matters:** Constraints without trade-off documentation leave implementers unable to make informed decisions when constraints conflict.

---

### architectural_constraints
#### C1 — All architectural constraints enumerated
- **Weight:** mandatory (30)
- **Status:** {{ archcon_c1_status }}
- **Evidence:** {{ archcon_c1_evidence }}
- **Why this matters:** Missing architectural constraints mean implementers may build a feature that violates system-wide architectural rules.

#### C2 — Constraint rationale explained
- **Weight:** mandatory (30)
- **Status:** {{ archcon_c2_status }}
- **Evidence:** {{ archcon_c2_evidence }}
- **Why this matters:** Constraints without rationale leave implementers unable to evaluate whether a workaround is acceptable.

#### C3 — Architecture domain reference provided
- **Weight:** recommended (20)
- **Status:** {{ archcon_c3_status }}
- **Evidence:** {{ archcon_c3_evidence }}
- **Why this matters:** Constraints without Architecture references have no provenance — implementers cannot verify the constraint is real.

#### C4 — Impact on feature design documented
- **Weight:** recommended (20)
- **Status:** {{ archcon_c4_status }}
- **Evidence:** {{ archcon_c4_evidence }}
- **Why this matters:** Constraints without impact documentation leave implementers unable to understand how the constraint shapes the design.

---

### security_considerations
#### C1 — All security considerations enumerated
- **Weight:** mandatory (30)
- **Status:** {{ security_c1_status }}
- **Evidence:** {{ security_c1_evidence }}
- **Why this matters:** Missing security considerations mean implementers may build a feature with unstated security requirements.

#### C2 — Threat model or risk assessment provided
- **Weight:** mandatory (30)
- **Status:** {{ security_c2_status }}
- **Evidence:** {{ security_c2_evidence }}
- **Why this matters:** Security considerations without threat model leave implementers unable to prioritize security efforts.

#### C3 — Mitigation strategies documented
- **Weight:** recommended (20)
- **Status:** {{ security_c3_status }}
- **Evidence:** {{ security_c3_evidence }}
- **Why this matters:** Security considerations without mitigations leave implementers aware of risks but unable to address them.

#### C4 — Compliance requirements referenced
- **Weight:** recommended (20)
- **Status:** {{ security_c4_status }}
- **Evidence:** {{ security_c4_evidence }}
- **Why this matters:** Security considerations without compliance references may miss regulatory requirements.

---

### performance_considerations
#### C1 — All performance considerations enumerated
- **Weight:** mandatory (30)
- **Status:** {{ perf_c1_status }}
- **Evidence:** {{ perf_c1_evidence }}
- **Why this matters:** Missing performance considerations mean implementers may build a feature that violates unstated performance requirements.

#### C2 — Measurable performance targets defined
- **Weight:** mandatory (30)
- **Status:** {{ perf_c2_status }}
- **Evidence:** {{ perf_c2_evidence }}
- **Why this matters:** Performance considerations without measurable targets are unverifiable — implementers cannot validate they've met the requirement.

#### C3 — Optimization strategies documented
- **Weight:** recommended (20)
- **Status:** {{ perf_c3_status }}
- **Evidence:** {{ perf_c3_evidence }}
- **Why this matters:** Performance considerations without optimization strategies leave implementers guessing about the expected approach.

#### C4 — Performance testing approach referenced
- **Weight:** recommended (20)
- **Status:** {{ perf_c4_status }}
- **Evidence:** {{ perf_c4_evidence }}
- **Why this matters:** Performance considerations without testing approach leave implementers unable to validate performance claims.

---

### failure_handling
#### C1 — All failure modes enumerated
- **Weight:** mandatory (30)
- **Status:** {{ failure_c1_status }}
- **Evidence:** {{ failure_c1_evidence }}
- **Why this matters:** Missing failure modes mean implementers may build a feature that doesn't handle known failure scenarios.

#### C2 — Recovery strategies defined per failure mode
- **Weight:** mandatory (30)
- **Status:** {{ failure_c2_status }}
- **Evidence:** {{ failure_c2_evidence }}
- **Why this matters:** Failure modes without recovery strategies leave implementers aware of risks but unable to address them.

#### C3 — Retry and backoff policies documented
- **Weight:** recommended (20)
- **Status:** {{ failure_c3_status }}
- **Evidence:** {{ failure_c3_evidence }}
- **Why this matters:** Failure handling without retry policies leaves implementers guessing about transient failure behavior.

#### C4 — Circuit breaker or fallback patterns described
- **Weight:** recommended (20)
- **Status:** {{ failure_c4_status }}
- **Evidence:** {{ failure_c4_evidence }}
- **Why this matters:** Failure handling without fallback patterns leaves implementers unable to design graceful degradation.

---

### extension_points
#### C1 — All extension points enumerated
- **Weight:** mandatory (30)
- **Status:** {{ extension_c1_status }}
- **Evidence:** {{ extension_c1_evidence }}
- **Why this matters:** Missing extension points mean implementers may build a feature that cannot be extended without modification.

#### C2 — Extension mechanism defined per point
- **Weight:** mandatory (30)
- **Status:** {{ extension_c2_status }}
- **Evidence:** {{ extension_c2_evidence }}
- **Why this matters:** Extension points without mechanisms leave implementers guessing about how to extend the feature.

#### C3 — Extension boundaries documented
- **Weight:** recommended (20)
- **Status:** {{ extension_c3_status }}
- **Evidence:** {{ extension_c3_evidence }}
- **Why this matters:** Extension points without boundaries may lead to extensions that break the feature's guarantees.

#### C4 — Extension testing approach referenced
- **Weight:** recommended (20)
- **Status:** {{ extension_c4_status }}
- **Evidence:** {{ extension_c4_evidence }}
- **Why this matters:** Extension points without testing approach leave implementers unable to validate extensions work correctly.

---

### traceability
#### C1 — Upstream domain links provided
- **Weight:** mandatory (30)
- **Status:** {{ trace_c1_status }}
- **Evidence:** {{ trace_c1_evidence }}
- **Why this matters:** Traceability without upstream links leaves readers unable to trace why this technical design exists.

#### C2 — Downstream implementation references provided
- **Weight:** mandatory (30)
- **Status:** {{ trace_c2_status }}
- **Evidence:** {{ trace_c2_evidence }}
- **Why this matters:** Traceability without downstream links disconnects the spec from the realization pipeline.

#### C3 — Bidirectional traceability verified
- **Weight:** recommended (20)
- **Status:** {{ trace_c3_status }}
- **Evidence:** {{ trace_c3_evidence }}
- **Why this matters:** One-directional traceability means upstream requirements may be lost or downstream implementations may be orphaned.

#### C4 — Traceability gaps documented
- **Weight:** recommended (20)
- **Status:** {{ trace_c4_status }}
- **Evidence:** {{ trace_c4_evidence }}
- **Why this matters:** Undocumented traceability gaps leave readers unaware of missing connections.

---

## Failures

| Section | Criterion | Severity | Weight | Evidence |
|---|---|---|---|---|
{{ failures_table }}

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ weighted_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})
