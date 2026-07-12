# {{ document_title }} — Feature Technical Deterministic Section Audit Report

> **Domain:** feature-technical
> **Scope:** section
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

**Why this matters:** Section-level deterministic audit checks that each Feature Technical section exists, has substantive content, and — critically — stays technology-independent. A section that names specific frameworks, libraries, or protocols has leaked into Implementation territory.

---

## Required Sections

### purpose
#### ft-sec-purpose-001 — Purpose section exists
- **Severity:** error
- **Weight:** 1.5
- **Status:** {{ purpose_001_status }}
- **Evidence:** {{ purpose_001_evidence }}
- **Why this matters:** Feature Technical without a Purpose has no rationale — implementers cannot understand why this technical design exists.

#### ft-sec-purpose-002 — Purpose states feature technical intent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ purpose_002_status }}
- **Evidence:** {{ purpose_002_evidence }}
- **Why this matters:** A Purpose that doesn't state technical intent leaves implementers guessing about the engineering problem being solved.

#### ft-sec-purpose-003 — Purpose is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ purpose_003_status }}
- **Evidence:** {{ purpose_003_evidence }}
- **Why this matters:** Purpose that names specific technologies has leaked — it's no longer a technology-independent specification.

#### ft-sec-purpose-004 — Purpose scope boundaries defined
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ purpose_004_status }}
- **Evidence:** {{ purpose_004_evidence }}
- **Why this matters:** Purpose without scope boundaries leaves implementers unsure where this feature ends and the next begins.

---

### participating_components
#### ft-sec-components-001 — Participating Components section exists
- **Severity:** error
- **Weight:** 1.5
- **Status:** {{ components_001_status }}
- **Evidence:** {{ components_001_evidence }}
- **Why this matters:** Feature Technical without a component inventory gives implementers no understanding of what parts of the system are involved.

#### ft-sec-components-002 — Participating Components lists components
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ components_002_status }}
- **Evidence:** {{ components_002_evidence }}
- **Why this matters:** A section that exists but doesn't list components is a heading with no substance.

#### ft-sec-components-003 — Participating Components is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ components_003_status }}
- **Evidence:** {{ components_003_evidence }}
- **Why this matters:** Component names that reference implementation technologies (e.g., "KafkaConsumer") have leaked into Implementation territory.

#### ft-sec-components-004 — Participating Components defines component boundaries
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ components_004_status }}
- **Evidence:** {{ components_004_evidence }}
- **Why this matters:** Components without defined boundaries lead to responsibility overlap and unclear ownership.

---

### component_interactions
#### ft-sec-interactions-001 — Component Interactions section exists
- **Severity:** error
- **Weight:** 1.5
- **Status:** {{ interactions_001_status }}
- **Evidence:** {{ interactions_001_evidence }}
- **Why this matters:** Feature Technical without interaction documentation leaves implementers guessing how components communicate.

#### ft-sec-interactions-002 — Component Interactions describes interactions
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ interactions_002_status }}
- **Evidence:** {{ interactions_002_evidence }}
- **Why this matters:** A section that exists but doesn't describe interactions is a heading with no substance.

#### ft-sec-interactions-003 — Component Interactions is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ interactions_003_status }}
- **Evidence:** {{ interactions_003_evidence }}
- **Why this matters:** Interaction descriptions that name specific protocols (gRPC, REST, Kafka) have leaked into Implementation territory.

#### ft-sec-interactions-004 — Component Interactions defines interaction patterns
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ interactions_004_status }}
- **Evidence:** {{ interactions_004_evidence }}
- **Why this matters:** Interactions without defined patterns (sync, async, event-driven) leave implementers unsure of the communication model.

---

### data_ownership
#### ft-sec-data-001 — Data Ownership section exists
- **Severity:** error
- **Weight:** 1.5
- **Status:** {{ data_001_status }}
- **Evidence:** {{ data_001_evidence }}
- **Why this matters:** Feature Technical without data ownership leaves implementers unsure which component is the authoritative source for each data entity.

#### ft-sec-data-002 — Data Ownership defines data ownership
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ data_002_status }}
- **Evidence:** {{ data_002_evidence }}
- **Why this matters:** A section that exists but doesn't define ownership is a heading with no substance.

#### ft-sec-data-003 — Data Ownership is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ data_003_status }}
- **Evidence:** {{ data_003_evidence }}
- **Why this matters:** Data ownership that names specific databases or storage technologies has leaked into Implementation territory.

#### ft-sec-data-004 — Data Ownership defines data lifecycle
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ data_004_status }}
- **Evidence:** {{ data_004_evidence }}
- **Why this matters:** Data ownership without lifecycle documentation leaves implementers unsure how data flows through create/read/update/delete cycles.

---

## Optional Sections

### feature_specification
#### ft-sec-spec-001 — Feature Specification section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ spec_001_status }}
- **Evidence:** {{ spec_001_evidence }}
- **Why this matters:** Optional — but when present, it defines the behavioral contract of the feature at its boundary.

#### ft-sec-spec-002 — Feature Specification defines technical behavior
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ spec_002_status }}
- **Evidence:** {{ spec_002_evidence }}
- **Why this matters:** Specification without technical behavior gives implementers no contract to validate against.

#### ft-sec-spec-003 — Feature Specification is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ spec_003_status }}
- **Evidence:** {{ spec_003_evidence }}
- **Why this matters:** Specification that names specific technologies has leaked into Implementation territory.

---

### component_responsibilities
#### ft-sec-resp-001 — Component Responsibilities section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ resp_001_status }}
- **Evidence:** {{ resp_001_evidence }}
- **Why this matters:** Optional — but when present, it defines what each component owns within the feature.

#### ft-sec-resp-002 — Component Responsibilities defines responsibilities
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ resp_002_status }}
- **Evidence:** {{ resp_002_evidence }}
- **Why this matters:** Responsibilities without definitions leave implementers guessing what each component is accountable for.

#### ft-sec-resp-003 — Component Responsibilities aligns with Architecture
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ resp_003_status }}
- **Evidence:** {{ resp_003_evidence }}
- **Why this matters:** Responsibilities that contradict the Architecture component model create conflicting guidance for implementers.

---

### runtime_behavior
#### ft-sec-runtime-001 — Runtime Behavior section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ runtime_001_status }}
- **Evidence:** {{ runtime_001_evidence }}
- **Why this matters:** Optional — but when present, it describes how the feature behaves at runtime.

#### ft-sec-runtime-002 — Runtime Behavior describes runtime behavior
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ runtime_002_status }}
- **Evidence:** {{ runtime_002_evidence }}
- **Why this matters:** Runtime behavior without descriptions leaves implementers guessing about startup, steady-state, and shutdown sequences.

#### ft-sec-runtime-003 — Runtime Behavior is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ runtime_003_status }}
- **Evidence:** {{ runtime_003_evidence }}
- **Why this matters:** Runtime behavior that names specific VMs, runtimes, or frameworks has leaked into Implementation territory.

---

### communication_paths
#### ft-sec-comm-001 — Communication Paths section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ comm_001_status }}
- **Evidence:** {{ comm_001_evidence }}
- **Why this matters:** Optional — but when present, it defines the data flow topology across the feature.

#### ft-sec-comm-002 — Communication Paths defines paths
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ comm_002_status }}
- **Evidence:** {{ comm_002_evidence }}
- **Why this matters:** Communication paths without definitions leave implementers unsure how data travels from source to sink.

#### ft-sec-comm-003 — Communication Paths is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ comm_003_status }}
- **Evidence:** {{ comm_003_evidence }}
- **Why this matters:** Communication paths that name specific protocols or message brokers have leaked into Implementation territory.

---

### integration_points
#### ft-sec-integration-001 — Integration Points section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ integration_001_status }}
- **Evidence:** {{ integration_001_evidence }}
- **Why this matters:** Optional — but when present, it documents every boundary where the feature connects to external systems.

#### ft-sec-integration-002 — Integration Points defines integration points
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ integration_002_status }}
- **Evidence:** {{ integration_002_evidence }}
- **Why this matters:** Integration points without definitions leave implementers guessing about external boundaries.

#### ft-sec-integration-003 — Integration Points is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ integration_003_status }}
- **Evidence:** {{ integration_003_evidence }}
- **Why this matters:** Integration points that name specific external systems or APIs have leaked into Implementation territory.

---

### external_dependencies
#### ft-sec-extdep-001 — External Dependencies section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ extdep_001_status }}
- **Evidence:** {{ extdep_001_evidence }}
- **Why this matters:** Optional — but when present, it documents all third-party libraries, services, and infrastructure the feature relies on.

#### ft-sec-extdep-002 — External Dependencies lists dependencies
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ extdep_002_status }}
- **Evidence:** {{ extdep_002_evidence }}
- **Why this matters:** Dependencies without a list leave implementers guessing about external requirements.

#### ft-sec-extdep-003 — External Dependencies defines dependency nature
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ extdep_003_status }}
- **Evidence:** {{ extdep_003_evidence }}
- **Why this matters:** Dependencies without nature (required, optional, version constraints) leave implementers unsure of their criticality.

---

### runtime_constraints
#### ft-sec-rtcon-001 — Runtime Constraints section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ rtcon_001_status }}
- **Evidence:** {{ rtcon_001_evidence }}
- **Why this matters:** Optional — but when present, it defines constraints on runtime behavior.

#### ft-sec-rtcon-002 — Runtime Constraints defines constraints
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ rtcon_002_status }}
- **Evidence:** {{ rtcon_002_evidence }}
- **Why this matters:** Constraints without definitions leave implementers unsure of performance and availability requirements.

#### ft-sec-rtcon-003 — Runtime Constraints has measurable thresholds
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ rtcon_003_status }}
- **Evidence:** {{ rtcon_003_evidence }}
- **Why this matters:** Constraints without measurable thresholds are unverifiable — implementers cannot validate they've met the requirement.

---

### architectural_constraints
#### ft-sec-archcon-001 — Architectural Constraints section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ archcon_001_status }}
- **Evidence:** {{ archcon_001_evidence }}
- **Why this matters:** Optional — but when present, it defines architectural constraints that limit implementation.

#### ft-sec-archcon-002 — Architectural Constraints defines constraints
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ archcon_002_status }}
- **Evidence:** {{ archcon_002_evidence }}
- **Why this matters:** Architectural constraints without definitions leave implementers guessing about hard boundaries.

#### ft-sec-archcon-003 — Architectural Constraints references Architecture domain
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ archcon_003_status }}
- **Evidence:** {{ archcon_003_evidence }}
- **Why this matters:** Architectural constraints without Architecture references have no provenance — implementers cannot verify the constraint is real.

---

### security_considerations
#### ft-sec-security-001 — Security Considerations section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ security_001_status }}
- **Evidence:** {{ security_001_evidence }}
- **Why this matters:** Optional — but when present, it defines security aspects of the feature.

#### ft-sec-security-002 — Security Considerations defines security aspects
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ security_002_status }}
- **Evidence:** {{ security_002_evidence }}
- **Why this matters:** Security considerations without definitions leave implementers guessing about authentication, authorization, and data protection requirements.

#### ft-sec-security-003 — Security Considerations is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ security_003_status }}
- **Evidence:** {{ security_003_evidence }}
- **Why this matters:** Security considerations that name specific security libraries or frameworks have leaked into Implementation territory.

---

### performance_considerations
#### ft-sec-perf-001 — Performance Considerations section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ perf_001_status }}
- **Evidence:** {{ perf_001_evidence }}
- **Why this matters:** Optional — but when present, it defines performance aspects of the feature.

#### ft-sec-perf-002 — Performance Considerations defines performance aspects
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ perf_002_status }}
- **Evidence:** {{ perf_002_evidence }}
- **Why this matters:** Performance considerations without definitions leave implementers guessing about latency and throughput requirements.

#### ft-sec-perf-003 — Performance Considerations has measurable targets
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ perf_003_status }}
- **Evidence:** {{ perf_003_evidence }}
- **Why this matters:** Performance targets without measurable thresholds are unverifiable — implementers cannot validate they've met the requirement.

---

### failure_handling
#### ft-sec-failure-001 — Failure Handling section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ failure_001_status }}
- **Evidence:** {{ failure_001_evidence }}
- **Why this matters:** Optional — but when present, it defines failure modes and recovery strategies.

#### ft-sec-failure-002 — Failure Handling defines failure modes
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ failure_002_status }}
- **Evidence:** {{ failure_002_evidence }}
- **Why this matters:** Failure handling without defined modes leaves implementers guessing about retry, fallback, and recovery strategies.

#### ft-sec-failure-003 — Failure Handling is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ failure_003_status }}
- **Evidence:** {{ failure_003_evidence }}
- **Why this matters:** Failure handling that names specific error handling libraries or frameworks has leaked into Implementation territory.

---

### extension_points
#### ft-sec-extension-001 — Extension Points section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ extension_001_status }}
- **Evidence:** {{ extension_001_evidence }}
- **Why this matters:** Optional — but when present, it defines where and how the feature can be extended.

#### ft-sec-extension-002 — Extension Points defines extension mechanisms
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ extension_002_status }}
- **Evidence:** {{ extension_002_evidence }}
- **Why this matters:** Extension points without defined mechanisms leave implementers guessing about customization boundaries.

#### ft-sec-extension-003 — Extension Points is technology-independent
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ extension_003_status }}
- **Evidence:** {{ extension_003_evidence }}
- **Why this matters:** Extension points that name specific plugin systems or frameworks have leaked into Implementation territory.

---

### traceability
#### ft-sec-trace-001 — Traceability section exists
- **Severity:** info
- **Weight:** 0.3
- **Status:** {{ trace_001_status }}
- **Evidence:** {{ trace_001_evidence }}
- **Why this matters:** Optional — but when present, it links the technical specification back to upstream domains.

#### ft-sec-trace-002 — Traceability links to upstream domains
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ trace_002_status }}
- **Evidence:** {{ trace_002_evidence }}
- **Why this matters:** Traceability without upstream links leaves readers unable to trace why this technical design exists.

#### ft-sec-trace-003 — Traceability links to downstream implementation
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ trace_003_status }}
- **Evidence:** {{ trace_003_evidence }}
- **Why this matters:** Traceability without downstream links disconnects the spec from the realization pipeline.

---

## Failures

| Rule | Section | Severity | Weight | Evidence |
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
