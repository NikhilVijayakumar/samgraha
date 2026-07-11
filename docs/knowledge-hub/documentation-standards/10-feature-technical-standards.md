# Feature Technical Design Standard

## Table of Contents
- [Purpose](#purpose)
- [Participating Components](#participating-components)
- [Component Interactions](#component-interactions)
- [Data Ownership](#data-ownership)
- [Feature Specification](#feature-specification)
- [Component Responsibilities](#component-responsibilities)
- [Runtime Behavior](#runtime-behavior)
- [Communication Paths](#communication-paths)
- [Integration Points](#integration-points)
- [External Dependency Integration](#external-dependency-integration)
- [Runtime Constraints](#runtime-constraints)
- [Architectural Constraints](#architectural-constraints)
- [Security Considerations](#security-considerations)
- [Performance Considerations](#performance-considerations)
- [Failure Handling](#failure-handling)
- [Extension Points](#extension-points)
- [Required Sections](#required-sections)
- [Goals](#goals)
- [Non-Goals](#non-goals)
- [Success Criteria](#success-criteria)
- [Responsibilities](#responsibilities)
- [Scope](#scope)
- [Out of Scope](#out-of-scope)
- [Inputs](#inputs)
- [Outputs](#outputs)
- [Traceability](#traceability)
- [Relationships](#relationships)
- [Required Characteristics](#required-characteristics)
- [Audit Rules](#audit-rules)
- [Validation Rules](#validation-rules)
- [Generation Rules](#generation-rules)
- [Enhancement Rules](#enhancement-rules)
- [Summary](#summary)
- [Common Mistakes](#common-mistakes)
- [Documentation Folder](#documentation-folder)
- [Usage](#usage)
- [Related](#related)
- [One-to-One Mapping](#one-to-one-mapping)
- [Architecture Principle Application](#architecture-principle-application)
- [External Context Application](#external-context-application)
- [Feature Design Considerations](#feature-design-considerations)
- [Architectural Realization Principles](#architectural-realization-principles)
- [Technology Independence](#technology-independence)
- [Quality Requirements](#quality-requirements)

---


## Purpose

> **semantic_type:** `purpose`
> **scope:** Why Feature Technical Design Documentation exists — its role in translating Feature Specifications into architectural realization
> **out_of_scope:** Implementation details, source code, shared architecture definitions, technology selection decisions
> **contributes:** Establishes the root intent for all Feature Technical Design sections and defines its boundary within the documentation ecosystem
> **relationships:** Derived from Feature(04) and Architecture(05); feeds Engineering(07) and Implementation; references External Context(08)
> **responsibilities:** Define Feature Technical Design's reason for being and its boundary between feature requirements and implementation
> **generation_rules:** State what Feature Technical Design is; explain its one-to-one relationship with Feature Specifications; distinguish from shared Architecture
> **enhancement_rules:** Strengthen clarity of scope boundaries; remove overlap with Architecture and Engineering standards; keep stable over time
> **validation_rules:** Purpose is clearly defined; one-to-one relationship with Feature is stated; boundary with Architecture is explicit
> **audit_rules:** Must exist; must define Feature Technical Design as architectural realization; must not contain feature lists or implementation details

### Template

> **minimum_content:** 2 paragraphs
> **length_guidance:** concise
> **diagram_requirements:** none

[Definition paragraph: what Feature Technical Design is and its role in the documentation ecosystem]

[Scope paragraph: what it covers and what it does not, including one-to-one relationship with Feature and distinction from Architecture]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Feature(04), Architecture(05)

This document defines the standard for Feature Technical Design Documentation within the engineering documentation ecosystem.

Feature Technical Design translates a single Feature Specification into its architectural realization by applying the shared principles, boundaries, and constraints defined by the Architecture Documentation together with any relevant External Context.

Feature Technical Design is **not** shared Architecture.

It is the application of reusable architectural principles to one specific feature.

Every Feature Technical Design document has a strict one-to-one relationship with a Feature Specification.

Feature Technical Design explains **how the system architecture realizes a feature**.
It does not describe implementation details or source code.

### Examples

**Correct:**
> Feature Technical Design is the architectural realization of a single Feature Specification. It applies the shared principles defined by Architecture Documentation together with relevant External Context to one specific feature. Every Feature Technical Design document maintains a strict one-to-one relationship with its corresponding Feature Specification, explaining how the system architecture realizes that feature without describing implementation details.

**Incorrect:**
> Feature Technical Design covers all features in the system, including User Authentication, Order Processing, Payment Handling, and Notification Delivery. It combines shared architecture with feature-specific requirements into a single comprehensive document.
> *Why wrong: This violates the one-to-one relationship principle — Feature Technical Design must correspond to exactly one Feature Specification, not combine multiple features into a single document.*

### Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** Define Feature Technical Design in relation to the documentation ecosystem; explicitly state what it is not; maintain the one-to-one mapping constraint throughout
- **Don't:** Drift into implementation specifics; conflate with Architecture or Feature Design; list features or technologies

---

## Participating Components

> **semantic_type:** `participating_components`
> **scope:** The system components involved in realizing this feature — which components participate and why each is needed
> **out_of_scope:** Component internals, implementation details, library choices, API signatures, class hierarchies
> **contributes:** Establishes the component boundary for the feature so interactions, data ownership, and responsibilities can be assigned
> **relationships:** Derived from Architecture(05) component model; feeds Component Interactions, Component Responsibilities, and Data Ownership; references Feature Specification inputs
> **responsibilities:** List every component that participates in the feature with a brief statement of why it is involved
> **generation_rules:** Start from the Feature Specification; apply Architecture's component model; include only components necessary for this feature; reference shared components by name from Architecture
> **enhancement_rules:** Remove components that are not directly involved; add components when missing interactions are discovered; keep component names consistent with Architecture
> **validation_rules:** Every listed component exists in Architecture; every component has a reason for participation; no implementation-specific components appear
> **audit_rules:** Must exist; must be marked required; must list components from Architecture; must not contain implementation details; every component must have a stated purpose

### Template

> **minimum_content:** 1 paragraph + table or list
> **length_guidance:** moderate
> **diagram_requirements:** component

[Component list or table: each participating component with a brief statement of why it is involved]

[Optional paragraph: relationship to Architecture component model]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** component diagram showing participating components
**Required cross-references:** Architecture(05) component model, Feature Specification

*(To be written. This section lists the components involved in realizing the feature.)*

### Examples

**Correct:**
> | Component | Reason for Participation |
> |---|---|
> | Authentication Component | Validates user credentials and manages session lifecycle for the login feature |
> | Data Component | Stores and retrieves user account data required for authentication |
> | Notification Component | Delivers security alerts (e.g., new device login) triggered by authentication events |
> | UI Component | Presents the login interface and communicates user input to the Authentication Component |

**Incorrect:**
> | Component | Technology |
> |---|---|
> | AuthService | Node.js microservice using Express framework |
> | UserDatabase | PostgreSQL 15 with TypeORM |
> | EmailSender | Nodemailer with SMTP transport |
> | LoginView | React 18 with TypeScript |
> *Why wrong: This lists implementation technologies (Node.js, PostgreSQL, React) rather than architectural components with their purpose for participation. Component names should come from the Architecture component model, not from implementation.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** tables
- **Audience:** architect
- **Do:** Use component names from Architecture Documentation; state the reason each component participates in this specific feature; include a component diagram
- **Don't:** List technologies, frameworks, or library names; describe component internals; include components not directly involved in the feature

---

## Component Interactions

> **semantic_type:** `component_interactions`
> **scope:** How participating components communicate and interact to realize the feature — the sequence and nature of component exchanges
> **out_of_scope:** Implementation protocols, message formats, serialization details, library-specific communication, source code
> **contributes:** Defines the behavioral contract between components so Implementation can realize the correct communication patterns
> **relationships:** Derived from Participating Components and Architecture(05) communication model; feeds Runtime Behavior and Communication Paths; references Component Responsibilities
> **responsibilities:** Describe every interaction between components, including the triggering condition, the nature of the exchange, and the expected outcome
> **generation_rules:** Derive interactions from Feature Specification behaviors; map each behavior to component exchanges; use Architecture's communication model; keep interactions at the architectural level
> **enhancement_rules:** Add interactions when missing behaviors are discovered; remove interactions that duplicate existing ones; clarify the nature of each exchange
> **validation_rules:** Every component interaction is traceable to a Feature Specification behavior; no implementation details appear; interactions are consistent with Architecture
> **audit_rules:** Must exist; must be marked required; must trace to Feature Specification behaviors; must not contain implementation details; must be consistent with Architecture communication model

### Template

> **minimum_content:** 1 paragraph + interaction list
> **length_guidance:** moderate
> **diagram_requirements:** sequence

[Interaction list: each interaction with triggering condition, nature of exchange, and expected outcome]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** sequence diagram showing component interactions
**Required cross-references:** Participating Components, Feature Specification, Architecture(05) communication model

*(To be written. This section describes how components interact to realize the feature.)*

### Examples

**Correct:**
> **Interaction: Order Submission**
> - Triggering condition: User submits an order through the UI Component
> - Nature of exchange: UI Component sends a synchronous request to the Order Component; Order Component validates and delegates payment to the Payment Component
> - Expected outcome: Order is created in Submitted state; payment is initiated; user receives confirmation or rejection
>
> **Interaction: Order Completion Notification**
> - Triggering condition: Order Component transitions order to Completed state
> - Nature of exchange: Order Component publishes an asynchronous event to the Event Bus; Notification Component consumes the event
> - Expected outcome: User receives a notification confirming order completion

**Incorrect:**
> **Interaction: Order Submission**
> - The React `OrderForm` component calls `POST /api/orders` using axios
> - Express router passes to `OrderController.submit()` which calls `OrderService.process()`
> - `OrderService` calls `PaymentClient.charge()` with a JWT token in the Authorization header
> *Why wrong: This describes implementation-level details (React components, axios, Express routes, specific class methods, JWT tokens) rather than architectural component interactions, triggering conditions, and expected outcomes.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** State the triggering condition, nature of exchange, and expected outcome for each interaction; trace every interaction to a Feature Specification behavior; include a sequence diagram
- **Don't:** Name specific classes, methods, or API endpoints; describe serialization formats or protocols; use implementation-level communication details

---

## Data Ownership

> **semantic_type:** `data_ownership`
> **scope:** Which components own which data in the feature's realization — the boundaries of data responsibility and access
> **out_of_scope:** Database schemas, table structures, column definitions, ORM mappings, data access implementations
> **contributes:** Prevents data ownership conflicts and defines clear boundaries for data responsibility across components
> **relationships:** Derived from Architecture(05) ownership rules and Persistence Architecture; feeds Component Responsibilities and Runtime Constraints; references Participating Components
> **responsibilities:** Define which component owns each piece of data, who can read it, who can write it, and what constraints govern access
> **generation_rules:** Identify all data the feature uses; assign ownership to the appropriate component per Architecture ownership rules; state read/write access for each data element; reference Architecture for ownership principles
> **enhancement_rules:** Clarify ownership when conflicts arise; add access rules when new data flows are discovered; keep ownership consistent with Architecture
> **validation_rules:** Every data element has a clear owner; access rules are defined; ownership is consistent with Architecture; no implementation details appear
> **audit_rules:** Must exist; must be marked required; must assign clear ownership; must define access patterns; must not contain database schemas or implementation details

### Template

> **minimum_content:** 1 paragraph + ownership table or list
> **length_guidance:** moderate
> **diagram_requirements:** ER

[Data ownership table or list: each data element with owner component, read access, write access, and constraints]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** ER diagram showing data ownership boundaries
**Required cross-references:** Architecture(05) ownership rules, Participating Components

*(To be written. This section defines data ownership boundaries for the feature.)*

### Examples

**Correct:**
> | Data Element | Owner | Read Access | Write Access | Constraints |
> |---|---|---|---|---|
> | User Credentials | Authentication Component | Authentication Component only | Authentication Component only | Must not be exposed outside security boundary |
> | Order Records | Order Component | Order Component, Notification Component (read-only) | Order Component only | Order state transitions follow the lifecycle defined in Runtime Behavior |
> | User Preferences | Data Component | Notification Component, Order Component | Data Component only | Preferences affect notification routing |

**Incorrect:**
> | Data Element | Table | Column | Type |
> |---|---|---|---|
> | User Credentials | users | password_hash | VARCHAR(255) |
> | Order Records | orders | status | ENUM('pending','confirmed','failed') |
> | User Preferences | user_preferences | notification_enabled | BOOLEAN |
> *Why wrong: This describes database schema details (table names, column names, data types) rather than component ownership, read/write access boundaries, and architectural constraints.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** tables
- **Audience:** architect
- **Do:** Assign one owning component per data element; define read and write access boundaries explicitly; reference Architecture ownership rules; include an ER diagram
- **Don't:** Describe database schemas, column types, or ORM mappings; allow multiple owners for the same data element; use implementation-specific storage details

---

## Feature Specification

> **semantic_type:** `feature_specification`
> **scope:** The Feature Specification this design realizes — the reference to the upstream functional specification being architected
> **out_of_scope:** Feature requirements content, acceptance criteria, business rules (these belong in the Feature document itself)
> **contributes:** Establishes the one-to-one traceability link between this technical design and its corresponding Feature Specification
> **relationships:** References Feature(04) directly; feeds Traceability; consumed by Audit Rules to verify one-to-one mapping
> **responsibilities:** Reference exactly one Feature Specification by name; confirm the scope of this design matches the Feature scope
> **generation_rules:** Reference the Feature document by name; confirm one-to-one mapping; state the Feature's purpose briefly without duplicating its content
> **enhancement_rules:** Update the reference when Feature scope changes; ensure the reference remains current; remove stale references
> **validation_rules:** Exactly one Feature Specification is referenced; the reference is valid; no Feature content is duplicated
> **audit_rules:** Must reference exactly one Feature Specification; must not duplicate Feature content; must maintain one-to-one mapping

### Template

> **minimum_content:** 1 paragraph
> **length_guidance:** concise
> **diagram_requirements:** none

[Feature reference: name of the Feature Specification and brief confirmation that scope matches]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Feature(04)

*(To be written. This section references the Feature Specification this design realizes.)*

### Examples

**Correct:**
> This Feature Technical Design realizes the **User Authentication** Feature Specification. The scope of this design covers credential validation, session management, and secure access — matching the Feature's defined scope. No additional features are addressed in this document.

**Incorrect:**
> This Feature Technical Design realizes the **User Authentication** Feature Specification, which requires: users must be able to log in with email and password, reset passwords via email link, enable two-factor authentication, and manage session devices. The login form must validate input in real time and show inline error messages.
> *Why wrong: This duplicates Feature Specification content (requirements, acceptance criteria, UI behavior) rather than referencing the Feature by name and confirming scope alignment.*

### Writing Guidance

- **Tone:** structural
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** Reference the Feature by exact name; confirm scope alignment between Feature and this design; state that no additional features are addressed
- **Don't:** Duplicate Feature requirements, acceptance criteria, or user stories; paraphrase Feature content; list multiple Feature Specifications

---

## Component Responsibilities

> **semantic_type:** `component_responsibilities`
> **scope:** What each participating component is responsible for in the feature's realization — the assignment of duties to components
> **out_of_scope:** Implementation details, function signatures, class responsibilities, code-level separation of concerns
> **contributes:** Makes component duties explicit so Implementation can realize the correct separation of concerns
> **relationships:** Derived from Participating Components and Component Interactions; feeds Implementation and Testing; references Architecture(05) component model
> **responsibilities:** Define the primary responsibility of each participating component; ensure responsibilities do not overlap inappropriately; align with Architecture ownership rules
> **generation_rules:** Assign responsibilities based on Architecture component model; derive responsibilities from Feature Specification behaviors; ensure each component has a clear primary responsibility
> **enhancement_rules:** Clarify ambiguous responsibilities; remove overlapping duties; add responsibilities when new interactions are discovered
> **validation_rules:** Every component has a defined responsibility; responsibilities are non-overlapping; responsibilities align with Architecture; no implementation details appear
> **audit_rules:** Must exist; must assign responsibility to every participating component; must not overlap; must align with Architecture; must not contain implementation details

### Template

> **minimum_content:** 1 paragraph + responsibility list
> **length_guidance:** moderate
> **diagram_requirements:** component

[Responsibility list: each participating component with its primary responsibility and alignment with Architecture]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** component diagram showing responsibility assignments
**Required cross-references:** Participating Components, Architecture(05) component model

*(To be written. This section defines what each component is responsible for.)*

### Examples

**Correct:**
> **Authentication Component:** Responsible for validating user credentials and issuing session tokens. Operates within the security boundary defined in Architecture.
>
> **Data Component:** Responsible for storing and retrieving user account information. Owns all account data per Architecture ownership rules. No other component may write account data directly.
>
> **Notification Component:** Responsible for delivering user-facing messages. Reads user preferences from the Data Component but does not own or modify account data.

**Incorrect:**
> **Authentication Component:** The `AuthService` class extends `BaseAuth` and implements the `login()` method using bcrypt.compare() for password verification and jsonwebtoken.sign() for token generation.
>
> **Data Component:** Uses TypeORM `@Repository` pattern with PostgreSQL. The `UserRepository` class provides `findById()`, `findByEmail()`, and `save()` methods.
> *Why wrong: This describes class hierarchies, specific method implementations, library usage, and ORM patterns rather than architectural responsibilities and ownership boundaries.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Assign a primary responsibility to every participating component; ensure responsibilities do not overlap; align each responsibility with Architecture ownership rules; include a component diagram
- **Don't:** Describe class hierarchies, method signatures, or library usage; assign shared ownership; leave any participating component without a defined responsibility

---

## Runtime Behavior

> **semantic_type:** `runtime_behavior`
> **scope:** The runtime execution model for the feature — how components behave at runtime, including lifecycle, state, and execution flow
> **out_of_scope:** Code execution details, threading implementations, specific runtime frameworks, performance metrics, profiling data
> **contributes:** Defines the runtime contract so Implementation can build the correct execution model
> **relationships:** Derived from Component Interactions and Architecture(05) runtime boundaries; feeds Communication Paths and Runtime Constraints; references Component Responsibilities
> **responsibilities:** Describe the runtime lifecycle of the feature, including initialization, execution flow, state transitions, and shutdown
> **generation_rules:** Derive runtime behavior from Component Interactions; apply Architecture runtime boundaries; describe execution at the architectural level; keep implementation independent
> **enhancement_rules:** Clarify lifecycle phases; add state transitions when missing; remove implementation-specific runtime details
> **validation_rules:** Runtime behavior is consistent with Architecture; lifecycle is complete; no implementation details appear; state transitions are defined
> **audit_rules:** Must exist; must describe runtime lifecycle; must be consistent with Architecture runtime boundaries; must not contain implementation details

### Template

> **minimum_content:** 1 paragraph + lifecycle description
> **length_guidance:** extensive
> **diagram_requirements:** flowchart

[Lifecycle description: initialization, execution flow, state transitions, and shutdown at the architectural level]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** flowchart showing runtime lifecycle and state transitions
**Required cross-references:** Component Interactions, Architecture(05) runtime boundaries

*(To be written. This section describes the runtime execution model for the feature.)*

### Examples

**Correct:**
> **Lifecycle: Order Processing**
> 1. **Initialization:** The Order Component starts and registers with the Event Bus. It subscribes to order submission events.
> 2. **Execution Flow:** When an order submission event arrives, the Order Component validates the request, delegates payment to the Payment Component, and updates order state.
> 3. **State Transitions:** An order moves through states: Submitted → Validated → PaymentPending → Confirmed → Completed (or Failed).
> 4. **Shutdown:** The Order Component unsubscribes from the Event Bus and completes any in-flight order processing before terminating.

**Incorrect:**
> **Lifecycle: Order Processing**
> 1. Initialize Spring Boot application context and connect to RabbitMQ using `amqp://guest:guest@localhost:5672`.
> 2. `OrderService.processOrder()` method validates input, calls `PaymentClient.charge()`, then `OrderRepository.save()`.
> 3. Use JPA entity states: `@Entity` Order with `@Enumerated` OrderStatus field.
> 4. `@PreDestroy` method closes RabbitMQ connection.
> *Why wrong: This describes Spring Boot initialization, AMQP connection strings, specific method names, JPA annotations, and `@PreDestroy` hooks rather than architectural runtime lifecycle, execution flow, and state transitions.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** mixed
- **Audience:** engineer
- **Do:** Describe the full lifecycle — initialization, execution flow, state transitions, and shutdown — at the architectural level; define each state and valid transitions; include a flowchart
- **Don't:** Name specific runtime frameworks, connection strings, or configuration files; describe threading or process models; use implementation-level lifecycle hooks

---

## Communication Paths

> **semantic_type:** `communication_paths`
> **scope:** The communication channels and protocols between components — how data and signals flow between participants
> **out_of_scope:** Message serialization formats, protocol buffer definitions, HTTP header details, socket implementations, library-specific communication
> **contributes:** Defines the communication contract so Implementation can realize the correct message flows
> **relationships:** Derived from Component Interactions and Architecture(05) communication model; feeds Integration Points and Runtime Constraints; references Participating Components
> **responsibilities:** Define every communication path between components, including the direction, nature, and architectural protocol
> **generation_rules:** Derive paths from Component Interactions; apply Architecture communication model; state the architectural protocol (e.g., event bus, REST gateway, message queue); keep paths at the architectural level
> **enhancement_rules:** Add paths when missing communication is discovered; remove paths that are not used by the feature; clarify the nature of each path
> **validation_rules:** Every communication path is traceable to a Component Interaction; protocol references are architectural; no implementation details appear
> **audit_rules:** Must exist; must define communication paths for all component interactions; must use architectural protocol references; must not contain implementation details

### Template

> **minimum_content:** 1 paragraph + path list
> **length_guidance:** moderate
> **diagram_requirements:** sequence

[Communication path list: each path with direction, nature, and architectural protocol]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** sequence diagram showing communication paths
**Required cross-references:** Component Interactions, Architecture(05) communication model

*(To be written. This section defines communication paths between components.)*

### Examples

**Correct:**
> **Path: Order Component → Notification Component**
> - Direction: Outbound from Order to Notification
> - Nature: Asynchronous event publication — order completion triggers notification delivery
> - Architectural protocol: Event Bus (as defined in Architecture communication model)
>
> **Path: UI Component → Order Component**
> - Direction: Inbound request from UI to Order
> - Nature: Synchronous request — UI submits order and awaits confirmation
> - Architectural protocol: REST Gateway (as defined in Architecture communication model)

**Incorrect:**
> **Path: Order Component → Notification Component**
> - Direction: POST request to `https://notification-service/internal/events`
> - Nature: JSON payload with order ID and status fields
> - Protocol: HTTP/1.1 with Content-Type: application/json
> *Why wrong: This describes HTTP methods, URLs, payload formats, and protocol versions rather than the architectural communication path direction, nature, and protocol reference.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Define direction, nature, and architectural protocol for every communication path; trace each path to a Component Interaction; reference the Architecture communication model
- **Don't:** Specify HTTP methods, URLs, or payload schemas; describe serialization or wire formats; use library-specific communication patterns

---

## Integration Points

> **semantic_type:** `integration_points`
> **scope:** Where this feature integrates with other systems, services, or components — the boundaries between this feature and the external world
> **out_of_scope:** API endpoint implementations, authentication token formats, request/response schemas, client library usage
> **contributes:** Makes integration boundaries explicit so Implementation can realize the correct external connections
> **relationships:** Derived from Feature Specification dependencies and Architecture(05) boundaries; feeds External Dependency Integration and Security Considerations; references Component Interactions
> **responsibilities:** Identify every integration point, the systems involved, the nature of the integration, and the architectural boundary it crosses
> **generation_rules:** Identify integrations from Feature Specification dependencies; classify each by boundary type (internal, external, third-party); state the architectural nature of each integration
> **enhancement_rules:** Add integration points when new dependencies are discovered; remove integration points that are not part of this feature; clarify the nature of each integration
> **validation_rules:** Every integration point is traceable to a Feature Specification dependency or Architecture boundary; integration types are defined; no implementation details appear
> **audit_rules:** Must exist; must identify all integration points; must classify by boundary type; must not contain API implementations or implementation details

### Template

> **minimum_content:** 1 paragraph + integration list
> **length_guidance:** moderate
> **diagram_requirements:** component

[Integration list: each integration point with systems involved, nature, and boundary type (internal, external, third-party)]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** component diagram showing integration boundaries
**Required cross-references:** Feature Specification, Architecture(05) boundaries

*(To be written. This section identifies where the feature integrates with external systems.)*

### Examples

**Correct:**
> **Integration Point: Payment Processing**
> - Systems involved: Order Component and external Payment Processor
> - Nature: Synchronous request-response for transaction authorization
> - Boundary type: External — crosses the system boundary to a third-party service
>
> **Integration Point: User Notifications**
> - Systems involved: Notification Component and internal Messaging Platform
> - Nature: Asynchronous event publication
> - Boundary type: Internal — communication between components within the system boundary

**Incorrect:**
> **Integration Point: Payment Processing**
> - Call `POST https://api.paymentprocessor.com/v2/charge` with API key in `Authorization: Bearer` header
> - Parse JSON response and extract `transaction_id` field
> - Retry on HTTP 503 with exponential backoff
> *Why wrong: This describes API endpoint URLs, HTTP headers, response parsing, and retry logic rather than the architectural integration boundary, systems involved, and nature of the integration.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Identify every integration point with systems involved, nature of integration, and boundary type; classify each as internal, external, or third-party; include a component diagram
- **Don't:** Describe API endpoints, authentication token formats, or request/response schemas; specify retry logic or error handling at the integration; name client libraries

---

## External Dependency Integration

> **semantic_type:** `external_dependencies`
> **scope:** How external systems and dependencies participate in the feature's architectural realization — the role of external context in the design
> **out_of_scope:** External system internals, API implementation details, authentication token formats, specific library versions
> **contributes:** Makes external dependencies explicit so Implementation understands what external systems are required and how they participate
> **relationships:** References External Context(08); derived from Integration Points and Feature Specification dependencies; feeds Security Considerations and Runtime Constraints
> **responsibilities:** Describe each external dependency, its role in the feature, the nature of the integration, and any constraints it imposes
> **generation_rules:** Reference External Context documents by name; describe the role of each dependency; state constraints imposed by external systems; avoid duplicating External Context content
> **enhancement_rules:** Add dependencies when new external systems are discovered; remove dependencies that are not part of this feature; update references when External Context changes
> **validation_rules:** Every external dependency is referenced (not duplicated); the role of each dependency is stated; constraints are documented; no implementation details appear
> **audit_rules:** Must exist if external dependencies are involved; must reference External Context by name; must not duplicate External Context content; must state the role of each dependency

### Template

> **minimum_content:** 1 paragraph + dependency list
> **length_guidance:** moderate
> **diagram_requirements:** component

[Dependency list: each external dependency with name, role in feature, nature of integration, and constraints imposed]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** component diagram showing external dependencies
**Required cross-references:** External Context(08), Integration Points

*(To be written. This section describes how external dependencies participate in the feature.)*

### Examples

**Correct:**
> **Dependency: Identity Provider** (reference: External Context identity services)
> - Role in feature: Authenticates user credentials during login flow; provides identity verification for protected operations
> - Nature of integration: The Authentication Component delegates credential verification to the Identity Provider
> - Constraints: The Identity Provider requires network connectivity; authentication is unavailable if the provider is unreachable (see Failure Handling)

**Incorrect:**
> **Dependency: Identity Provider**
> - Use Auth0 SDK v4.2.1 to call the `/oauth/token` endpoint
> - Store refresh tokens in memory using the `auth0-spa-js` library
> - Handle 401 responses by redirecting to `/login`
> *Why wrong: This describes SDK versions, API endpoints, library usage, and HTTP status codes rather than the architectural role of the external dependency and constraints it imposes.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Reference External Context documents by name; state the role of each dependency in the feature; describe constraints imposed without duplicating External Context content; include a component diagram
- **Don't:** Duplicate External Context content; describe SDK versions or library APIs; specify implementation-level integration patterns

---

## Runtime Constraints

> **semantic_type:** `runtime_constraints`
> **scope:** Operational constraints on the feature's runtime behavior — the limits and requirements that govern how the feature operates at runtime
> **out_of_scope:** Performance benchmarks, specific resource limits, deployment configurations, infrastructure specifications
> **contributes:** Defines the operational boundaries within which the feature must function, ensuring reliability and correctness
> **relationships:** Derived from Architecture(05) runtime boundaries and External Context(08) platform constraints; feeds Performance Considerations and Failure Handling; references Runtime Behavior
> **responsibilities:** Define every operational constraint that affects runtime behavior, including concurrency limits, resource boundaries, and operational requirements
> **generation_rules:** Derive constraints from Architecture runtime boundaries; apply External Context platform constraints; state each constraint as a clear operational limitation
> **enhancement_rules:** Add constraints when new operational requirements are discovered; remove constraints that no longer apply; clarify ambiguous constraints
> **validation_rules:** Constraints are consistent with Architecture; constraints are traceable to External Context where applicable; no implementation details appear
> **audit_rules:** Must exist; must define runtime constraints; must be consistent with Architecture; must not contain deployment configurations or implementation details

### Template

> **minimum_content:** 1 paragraph + constraint list
> **length_guidance:** moderate
> **diagram_requirements:** none

[Constraint list: each operational constraint with source (Architecture or External Context) and application to this feature]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Architecture(05) runtime boundaries, External Context(08)

*(To be written. This section defines operational constraints on runtime behavior.)*

### Examples

**Correct:**
> **Constraint: Concurrency Limit** (source: Architecture runtime boundaries)
> The Order Processing Component must handle concurrent order submissions without data corruption. The architectural model limits concurrent processing to the resource allocation defined for this component class.
>
> **Constraint: Resource Boundary** (source: External Context platform constraints)
> The feature must operate within the memory and compute boundaries defined by the hosting platform. Component lifecycle must respect the platform's resource management model.

**Incorrect:**
> **Constraint: Concurrency Limit**
> Use `java.util.concurrent.Semaphore` with permits=10 in the OrderService. Configure Tomcat max threads to 200 in server.xml. Set JVM heap to 4GB with `-Xmx4g`.
> *Why wrong: This specifies implementation-level concurrency tools (Java Semaphore), server configuration (Tomcat, server.xml), and JVM settings rather than architectural operational constraints.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** State each constraint as a clear operational limitation; cite the source from Architecture or External Context; explain how the constraint applies to this specific feature
- **Don't:** Specify implementation tools, frameworks, or configuration files; provide performance benchmarks or numerical thresholds; describe deployment or infrastructure details

---

## Architectural Constraints

> **semantic_type:** `architectural_constraints`
> **scope:** Architectural constraints this feature must respect — the shared architectural rules and boundaries that apply to this feature's realization
> **out_of_scope:** Implementation constraints, coding standards, framework limitations, technology-specific constraints
> **contributes:** Ensures the feature respects shared architectural principles and does not introduce architectural violations
> **relationships:** Derived from Architecture(05) principles and constraints; feeds Component Responsibilities and Runtime Behavior; references Participating Components
> **responsibilities:** List every architectural constraint that applies to this feature, with the source principle from Architecture
> **generation_rules:** Extract relevant constraints from Architecture Documentation; reference each constraint by its Architecture source; state how the constraint applies to this specific feature
> **enhancement_rules:** Add constraints when Architecture evolves; remove constraints that no longer apply; update references when Architecture is reorganized
> **validation_rules:** Every constraint traces to Architecture; the application to this feature is stated; no architectural principles are redefined; no implementation details appear
> **audit_rules:** Must exist; must reference Architecture constraints by source; must not redefine architectural principles; must not contain implementation details

### Template

> **minimum_content:** 1 paragraph + constraint list
> **length_guidance:** moderate
> **diagram_requirements:** none

[Constraint list: each architectural constraint with Architecture source principle and application to this feature]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Architecture(05) principles and constraints

*(To be written. This section defines architectural constraints the feature must respect.)*

### Examples

**Correct:**
> **Constraint: Component Ownership** (source: Architecture ownership rules)
> Each data element must have exactly one owning component. The Authentication Component owns credential data; no other component may write to it directly.
>
> **Constraint: Communication Model** (source: Architecture communication model)
> All inter-component communication must use the event bus defined in Architecture. Direct component-to-component calls are not permitted.

**Incorrect:**
> **Constraint: Component Ownership**
> Use a single PostgreSQL database for all component data. Each component writes to its own schema within the shared database using TypeORM repositories.
> *Why wrong: This specifies technology choices (PostgreSQL, TypeORM) and implementation patterns (shared database, schema separation) rather than architectural ownership principles and communication model constraints.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Reference each architectural constraint by its Architecture source principle; state how the constraint applies to this feature; avoid redefining architectural principles
- **Don't:** Redefine or paraphrase Architecture principles; introduce implementation-level constraints; name technologies or frameworks as constraints

---

## Security Considerations

> **semantic_type:** `security_considerations`
> **scope:** Security boundaries, authentication, and authorization for the feature — how the feature respects and implements security architecture
> **out_of_scope:** Encryption implementations, authentication token formats, specific security library usage, code-level security patterns
> **contributes:** Ensures the feature respects security architecture and identifies security-relevant decisions for Implementation
> **relationships:** Derived from Architecture(05) security boundaries and External Context(08) security constraints; feeds Failure Handling and Runtime Constraints; references Security Documentation(03) if applicable
> **responsibilities:** Define security boundaries, authentication requirements, authorization rules, and data protection needs for the feature
> **generation_rules:** Apply Architecture security boundaries; identify security-relevant interactions; state authentication and authorization requirements at the architectural level; reference Security Documentation where applicable
> **enhancement_rules:** Add security considerations when new sensitive data flows are discovered; remove considerations that are already covered by Architecture; clarify authorization rules
> **validation_rules:** Security boundaries are consistent with Architecture; authentication and authorization are defined; no implementation details appear; sensitive data is identified
> **audit_rules:** Must exist if the feature involves authentication, authorization, or sensitive data; must reference Architecture security boundaries; must not contain implementation details

### Template

> **minimum_content:** 1 paragraph + security list
> **length_guidance:** moderate
> **diagram_requirements:** none

[Security list: security boundaries, authentication requirements, authorization rules, and sensitive data identification]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Architecture(05) security boundaries, External Context(08), Security Documentation(03)

*(To be written. This section defines security considerations for the feature.)*

### Examples

**Correct:**
> **Security Boundary:** The Authentication Component operates within the security boundary defined by Architecture. Only authenticated requests may access the Data Component.
> **Authentication:** Requests from external systems must be validated against the identity provider defined in External Context(08).
> **Authorization:** The Notification Component may read user preferences but may not modify account data. Account data modification is restricted to the Account Component.
> **Sensitive Data:** User credentials and session tokens are classified as sensitive; they must not appear in communication paths that cross the external boundary.

**Incorrect:**
> **Security Boundary:** Use bcrypt for password hashing with 12 salt rounds. Implement JWT tokens using the jsonwebtoken npm library with RS256 algorithm. Store tokens in httpOnly cookies with SameSite=Strict.
> *Why wrong: This specifies implementation-level security details (specific algorithms, libraries, cookie configurations) rather than architectural security boundaries, authentication requirements, and authorization rules.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Define security boundaries, authentication requirements, and authorization rules at the architectural level; identify sensitive data; reference Architecture security boundaries and External Context security constraints
- **Don't:** Name specific encryption algorithms, libraries, or token formats; describe code-level security patterns; specify cookie or header configurations

---

## Performance Considerations

> **semantic_type:** `performance_considerations`
> **scope:** Performance requirements and constraints for the feature — the non-functional performance expectations that affect architectural decisions
> **out_of_scope:** Profiling results, specific latency numbers, throughput benchmarks, optimization techniques, caching implementations
> **contributes:** Makes performance expectations explicit so Implementation can build with appropriate performance characteristics
> **relationships:** Derived from Feature Specification requirements and Architecture(05) performance patterns; feeds Runtime Constraints and Failure Handling; references Component Interactions
> **responsibilities:** Define performance expectations, throughput requirements, latency constraints, and resource limitations at the architectural level
> **generation_rules:** Derive performance needs from Feature Specification; apply Architecture performance patterns; state expectations as architectural constraints; avoid implementation-specific optimization
> **enhancement_rules:** Add performance considerations when requirements clarify; remove considerations that are implementation details; update expectations when Architecture evolves
> **validation_rules:** Performance expectations are traceable to Feature Specification; expectations are architectural (not implementation); no profiling data or benchmarks appear
> **audit_rules:** Must exist if the feature has performance requirements; must be architectural in nature; must not contain benchmarks, profiling data, or implementation details

### Template

> **minimum_content:** 1 paragraph + performance list
> **length_guidance:** moderate
> **diagram_requirements:** none

[Performance list: performance expectations, throughput requirements, latency constraints, and resource limitations at the architectural level]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Feature Specification, Architecture(05) performance patterns

*(To be written. This section defines performance considerations for the feature.)*

### Examples

**Correct:**
> **Performance Expectation: Search Response**
> - Throughput: The search component must support concurrent queries from multiple client components without degrading response times
> - Latency constraint: Search results must be available to the UI component within the time expected for interactive use
> - Resource limitation: Search indexing must not consume more than the resource allocation defined in Architecture runtime boundaries

**Incorrect:**
> **Performance Expectation: Search Response**
> - Elasticsearch query must complete in under 200ms
> - Node.js event loop must not be blocked for more than 50ms
> - Use Redis caching with TTL of 300 seconds
> *Why wrong: This specifies technology-specific benchmarks (Elasticsearch, Node.js, Redis), exact latency numbers, and implementation details (caching TTL) rather than architectural performance expectations.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** State performance expectations as architectural constraints traceable to Feature Specification; define throughput, latency, and resource expectations at the system level; reference Architecture performance patterns
- **Don't:** Provide specific latency numbers or benchmarks; name profiling tools or caching libraries; describe optimization techniques or implementation strategies

---

## Failure Handling

> **semantic_type:** `failure_handling`
> **scope:** How failures, errors, and faults are handled in the feature — the error propagation, recovery, and resilience strategy at the architectural level
> **out_of_scope:** Exception types, error codes, try/catch patterns, retry implementations, logging frameworks, specific error handling libraries
> **contributes:** Defines the resilience contract so Implementation can realize correct error handling without redefining architectural boundaries
> **relationships:** Derived from Component Interactions and Architecture(05) error boundaries; feeds Runtime Constraints and Testing; references Component Responsibilities and Communication Paths
> **responsibilities:** Define failure modes, error propagation paths, recovery strategies, and resilience boundaries for the feature
> **generation_rules:** Identify failure modes from Component Interactions; apply Architecture error boundaries; define error propagation at the architectural level; state recovery strategies without implementation details
> **enhancement_rules:** Add failure modes when new interactions are discovered; clarify recovery strategies; remove implementation-specific error handling
> **validation_rules:** Failure modes cover all component interactions; error propagation is consistent with Architecture; recovery strategies are defined; no implementation details appear
> **audit_rules:** Must exist; must define failure modes for component interactions; must be consistent with Architecture error boundaries; must not contain exception types or implementation details

### Template

> **minimum_content:** 1 paragraph + failure list
> **length_guidance:** extensive
> **diagram_requirements:** flowchart

[Failure list: failure modes per interaction, error propagation paths, recovery strategies, and resilience boundaries]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** flowchart showing error propagation and recovery paths
**Required cross-references:** Component Interactions, Architecture(05) error boundaries, Component Responsibilities

*(To be written. This section defines how failures and errors are handled.)*

### Examples

**Correct:**
> **Failure Mode: External Service Unavailable**
> - Interaction affected: Order Service queries Payment Gateway
> - Propagation: Failure propagates from Payment Gateway to Order Service to Notification Service
> - Recovery: Order Service queues the order for retry; Notification Service informs the user that processing is delayed
> - Resilience boundary: Failure does not propagate beyond Order Service to the User Interface layer

**Incorrect:**
> **Failure Mode: External Service Unavailable**
> - Catch `PaymentTimeoutException` in `OrderService.java` line 142
> - Retry 3 times with exponential backoff using Spring Retry `@Retryable`
> - Log error with `LOGGER.error("Payment failed", e)`
> *Why wrong: This describes implementation-level error handling (specific exception types, code lines, framework annotations, logging calls) rather than architectural failure modes and recovery strategies.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** mixed
- **Audience:** engineer
- **Do:** Define failure modes for every component interaction; trace error propagation paths across architectural boundaries; state recovery strategies and resilience boundaries; include a flowchart
- **Don't:** Name specific exception types, error codes, or try/catch patterns; describe retry implementations or logging frameworks; reference code locations

---

## Extension Points

> **semantic_type:** `extension_points`
> **scope:** Where the feature can be extended without modification — the plugin points, hooks, and extensibility boundaries in the feature's architecture
> **out_of_scope:** Plugin implementations, hook callbacks, specific extension mechanisms, library APIs for extension
> **contributes:** Makes extensibility explicit so future features can extend this feature without modifying its architecture
> **relationships:** Derived from Architecture(05) plugin architecture and extensibility patterns; feeds Future Extensions and Testing; references Component Responsibilities
> **responsibilities:** Identify every point where the feature can be extended, the type of extension supported, and the constraints on extensions
> **generation_rules:** Identify extension points from Architecture plugin model; state the type of extension (plugin, hook, event, configuration); define constraints on extensions; keep extension descriptions architectural
> **enhancement_rules:** Add extension points when Architecture evolves; remove extension points that are not supported; clarify extension constraints
> **validation_rules:** Extension points are consistent with Architecture plugin model; constraints are defined; no implementation details appear; extension points are architecturally sound
> **audit_rules:** Must exist if the feature has extensibility; must reference Architecture plugin model; must not contain implementation details; must define extension constraints

### Template

> **minimum_content:** 1 paragraph + extension list
> **length_guidance:** moderate
> **diagram_requirements:** component

[Extension list: each extension point with type (plugin, hook, event, configuration) and constraints on extensions]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** component diagram showing extension points
**Required cross-references:** Architecture(05) plugin architecture, Component Responsibilities

*(To be written. This section identifies where the feature can be extended.)*

### Examples

**Correct:**
> **Extension Point: Notification Dispatch**
> - Type: Event hook
> - Constraint: Extensions must implement the notification dispatch contract defined in Architecture plugin model; extensions cannot modify core notification routing

**Incorrect:**
> **Extension Point: Notification Dispatch**
> - Type: Custom JavaScript class extending BaseNotifier
> - Constraint: Must override onSend() method using the EventEmitter library API
> *Why wrong: This specifies implementation-level details (JavaScript class, method names, library APIs) rather than architectural extension type and constraints.*

### Writing Guidance

- **Tone:** technical
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Identify extension points with type (plugin, hook, event, configuration) and constraints; reference the Architecture plugin model; ensure extension points are architecturally sound
- **Don't:** Name specific programming languages, class hierarchies, or library APIs; describe callback implementations; define extension mechanisms not present in Architecture

---

## Required Sections

Every Feature Technical Design document must contain the following sections.
Sections are identified by heading text; the compiler maps each to a semantic type.

| Section | semantic_type | Required | Aliases | Content Requirements |
|---------|--------------|----------|---------|----------------------|
| Purpose | `purpose` | ✓ | Overview, Summary | Definition of Feature Technical Design; one-to-one relationship with Feature; boundary with Architecture |
| Participating Components | `participating_components` | ✓ | Components, Involved Components | List of components with brief purpose for each |
| Component Interactions | `component_interactions` | ✓ | Interactions, Communication Flows | Each interaction with triggering condition, nature of exchange, and expected outcome |
| Data Ownership | `data_ownership` | ✓ | Ownership, Data Responsibilities | Ownership assignment, read/write access, and access constraints per data element |
| Feature Specification | `feature_specification` | | Feature Spec, Specification | Reference to exactly one Feature Specification by name |
| Component Responsibilities | `component_responsibilities` | | Responsibilities, Component Roles | Primary responsibility per participating component |
| Runtime Behavior | `runtime_behavior` | | Behavior, Execution Model | Runtime lifecycle: initialization, execution flow, state transitions, shutdown |
| Communication Paths | `communication_paths` | | Communication, Message Flows | Direction, nature, and architectural protocol for each path |
| Integration Points | `integration_points` | | Integration, External Integration | Each integration point with systems involved, nature, and boundary type |
| External Dependency Integration | `external_dependencies` | | External Dependencies, External Systems | Dependency name, role in feature, nature of integration, constraints imposed |
| Runtime Constraints | `runtime_constraints` | | Constraints, Operational Constraints | Operational constraints: concurrency limits, resource boundaries, requirements |
| Architectural Constraints | `architectural_constraints` | | Architecture Constraints | Architecture source principle and application to this feature |
| Security Considerations | `security_considerations` | | Security | Security boundaries, authentication requirements, authorization rules |
| Performance Considerations | `performance_considerations` | | Performance | Performance expectations, throughput, latency constraints at architectural level |
| Failure Handling | `failure_handling` | | Error Handling, Failures, Fault Handling | Failure modes per interaction, error propagation paths, recovery strategies |
| Extension Points | `extension_points` | | Extensions, Extension, Extensibility | Extension points with type and constraints on extensions |
| Traceability | `traceability` | | Traces To, Derived From | Derivation chain from Vision through Feature to Implementation |

Section headings are case-insensitive. Sections not listed here are stored as `generic` type — preserved but not queryable by type.

---

## Goals

Feature Technical Design aims to:

* Give every feature a single authoritative technical realization plan.
* Keep it constrained by Architecture rather than reinventing structure per feature.
* Make the plan specific enough for Prototype to validate and Implementation to follow.

---

## Non-Goals

Feature Technical Design does not define:

* Product Vision
* Feature Requirements
* User Experience Design
* Shared Architecture
* Engineering rationale
* Technology selection decisions
* Source Code
* Algorithms
* API implementations

These responsibilities belong to other documentation standards.

---

## Success Criteria

Feature Technical Design is successful when:

* Every Feature has one corresponding Feature Technical Design.
* Shared architectural principles are consistently applied.
* Relevant External Context has been incorporated where necessary.
* User experience requirements are realized without redefining them.
* Engineers understand how the feature integrates into the system.
* Implementation can proceed without redefining architectural responsibilities.
* AI systems can implement the feature while preserving architecture, external constraints, and design intent.

---

## Responsibilities

Feature Technical Design is responsible for defining:

* Architectural realization of the feature
* Participating components
* Component responsibilities
* Component interactions
* Runtime behavior
* Communication paths
* Data ownership
* Integration points
* External dependency integration
* Runtime constraints
* Architectural constraints
* Security considerations
* Performance considerations
* Failure handling
* Extension points

Feature Technical Design bridges feature requirements and implementation through architecture.

---

## Scope

Feature Technical Design may describe:

* Component interactions
* Request flows
* Event flows
* Data flow
* State ownership
* Runtime lifecycle
* Persistence responsibilities
* Service boundaries
* IPC communication
* API interactions
* External integrations
* Plugin interactions
* Security boundaries
* Error propagation
* Performance considerations
* Cross-repository interactions

Every Feature Technical Design document should remain focused on one feature.

---

## Out of Scope

Feature Technical Design must not describe:

* Product Vision
* Feature Requirements
* User Experience
* Design Principles
* Shared Architecture
* Engineering rationale
* Programming languages
* Framework implementations
* Algorithms
* Source code
* Function implementations
* Build configuration
* Library APIs

Shared Architecture belongs to Architecture Documentation.

Technology rationale belongs to Engineering Documentation.

Implementation belongs to source code.

---

## Inputs

Feature Technical Design derives from:

* Feature Specification
* Architecture Documentation
* Relevant External Context (optional)
* Engineering Constraints

Feature Design is not a required input. It is considered only where user experience decisions directly influence architectural realization — see Feature Design Considerations.

Feature Technical Design should not derive from source code.

---

## Outputs

Feature Technical Design provides direction for:

* Engineering Documentation
* Source Code Implementation
* Unit Testing
* Integration Testing
* Performance Testing
* Security Validation

Implementation should conform to the documented technical design.

---

## Traceability

> **semantic_type:** `traceability`
> **scope:** How Feature Technical Design connects to the documentation hierarchy — the derivation chain from Vision through Feature to Implementation
> **out_of_scope:** Implementation traceability, test traceability, bug tracking, version history
> **contributes:** Makes Feature Technical Design's position in the documentation ecosystem visible and verifiable
> **relationships:** Vision(01) → Feature(04) → Feature Technical Design(10) → Engineering(07) → Implementation; references Architecture(05) and External Context(08)
> **responsibilities:** Show the derivation path; assert one-to-one mapping with Feature Specification; show how Architecture and External Context are applied
> **generation_rules:** Use the tier model diagram; list which standards derive from or feed into Feature Technical Design; state the one-to-one constraint
> **enhancement_rules:** Update the diagram when new standards are added; ensure derivation paths remain accurate
> **validation_rules:** Derivation paths are complete; one-to-one mapping is stated; no orphaned references
> **audit_rules:** Must exist; must include tier diagram; must show one-to-one Feature mapping; must reference Architecture and External Context as inputs

### Template

> **minimum_content:** 1 paragraph + derivation diagram
> **length_guidance:** moderate
> **diagram_requirements:** flowchart

[Derivation chain: Vision → Feature → Feature Technical Design → Engineering → Implementation]

[One-to-one mapping statement and input references to Architecture and External Context]

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** flowchart showing derivation chain
**Required cross-references:** Vision(01), Feature(04), Architecture(05), Engineering(07), External Context(08)

Feature Technical Design remains traceable.

```text
Vision
    │
    ├─────────────────────────────────┐
    ↓                                 ↓
Feature                         Architecture
    ↓                           (technology decisions,
Design (optional)               platform constraints)
    ↓                                 │
Feature Design (optional)             │
    │                                 │
    └──────────────────────────────→ Feature Technical Design
                                      ↓
                                  Engineering
                                      ↓
                                  Implementation
```

Feature Specification and Architecture Documentation are required inputs. Feature Design is an optional input considered only where UX decisions influence architectural realization.

Every Feature Technical Design should trace directly to exactly one Feature Specification.

### Examples

**Correct:**
> Vision(01) → Feature: Authentication → Feature Technical Design: Authentication → Engineering: Authentication → Implementation: Authentication
>
> This Feature Technical Design traces to exactly one Feature Specification (Authentication). Architecture(05) security boundaries and External Context(08) identity provider constraints are applied as inputs.

**Incorrect:**
> Feature Technical Design: Authentication derives from the authentication API implementation in the source code.
> *Why wrong: Traceability must flow from Feature Specification and Architecture, not from source code. The derivation chain starts at Vision, not at implementation.*

### Writing Guidance

- **Tone:** structural
- **Voice:** third person
- **Structure:** mixed
- **Audience:** architect
- **Do:** Show the full derivation chain from Vision through Feature to Implementation; assert one-to-one mapping with Feature Specification; reference Architecture and External Context as inputs; include a flowchart
- **Don't:** Trace to source code or implementation artifacts; omit upstream or downstream standards; leave the one-to-one mapping unstated

---

## Relationships

| Document         | Relationship                                                       |
| ---------------- | ------------------------------------------------------------------ |
| Feature          | One-to-one mapping                                                 |
| Feature Design   | Realizes UX-driven architectural needs                             |
| Architecture     | Applies shared architectural principles                            |
| External Context | Applies external architectural constraints                         |
| Engineering      | Explains technology choices used to implement the technical design |

---

## Required Characteristics

Feature Technical Design should be:

* Architecturally consistent
* Traceable to exactly one Feature
* Specific enough to be technology-aware, unlike Feature Design
* Testable — Prototype can validate it, Implementation can follow it
* Boundary-respecting

---

## Audit Rules

An audit should verify:

* A one-to-one mapping exists between Feature and Feature Technical Design.
* Shared Architecture Documentation has been applied.
* Relevant External Context has been identified.
* Feature Design considerations have been respected.
* Component responsibilities are clearly defined.
* Communication paths are understandable.
* Runtime boundaries are respected.
* External architectural constraints are reflected.
* Technology references remain architectural.
* No implementation details appear.
* Architecture and External Context are referenced instead of duplicated.

Feature Technical Design quality is evaluated individually and across the feature collection.

---

## Validation Rules

Feature Technical Design is considered valid when:

* One document corresponds to one Feature.
* Shared Architecture Documentation has been applied.
* Relevant External Context has been identified.
* Component responsibilities are clearly defined.
* Runtime interactions are documented.
* External architectural constraints are respected.
* Feature Design considerations have been incorporated where appropriate.
* Technology references remain architectural rather than implementation specific.
* No source code appears.
* Technical Design remains traceable to the Feature Specification.

---

## Generation Rules

When generating Feature Technical Design:

* Start from the Feature Specification.
* Apply Architecture Documentation.
* Apply relevant External Context.
* Consider Feature Design where it influences architecture.
* Focus on responsibilities and interactions.
* Keep one document per feature.
* Reference shared Architecture.
* Reference External Context rather than duplicating it.
* Avoid implementation details.
* Preserve architectural consistency.

---

## Enhancement Rules

When enhancing Feature Technical Design:

* Improve architectural clarity.
* Improve component responsibility definitions.
* Strengthen consistency with Architecture Documentation.
* Strengthen consistency with External Context.
* Improve alignment with Feature Design.
* Remove duplicated architectural principles.
* Remove duplicated external documentation.
* Remove implementation leakage.
* Improve traceability.
* Preserve architectural intent.

Technical Design should become clearer without changing feature behavior.

---

## Summary

Feature Technical Design Documentation is the architectural realization of a single Feature Specification.

Each document maintains a strict one-to-one relationship with its corresponding Feature, applying the shared principles defined by the Architecture Documentation together with any relevant External Context and considering Feature Design where architectural decisions affect user experience.

Its purpose is to bridge feature requirements and implementation by defining responsibilities, interactions, boundaries, integrations, and architectural constraints while preserving consistency across the product ecosystem and avoiding implementation-specific details.

---

## Common Mistakes

Examples include:

* Combining multiple features.
* Rewriting Architecture Documentation.
* Rewriting External Context.
* Embedding source code.
* Describing algorithms.
* Introducing implementation patterns.
* Ignoring Feature Design.
* Ignoring external architectural constraints.
* Duplicating shared architectural principles.

These should be reported during audits.

---

## Documentation Folder

Feature Technical Design documents live under:

```text
docs/raw/feature-technical/
```

---

## Usage

Written by the engineer implementing a Feature, applying Architecture's shared principles to that one feature before code is written. Use `samgraha audit --domain feature-technical` to check that Participating Components, Component Interactions, and Data Ownership are all documented before implementation starts.

## Related

- [Feature Standard](04-feature-standards.md) — one-to-one mapping
- [Architecture Standard](05-architecture-standards.md) — shared principles this standard applies
- [Engineering Standard](07-engineering-standards.md) — technology rationale used to implement this design
- [Standards Reference Standard](standards.md) — how this standard itself is documented

## One-to-One Mapping

Every Feature Specification should have exactly one corresponding Feature Technical Design document.

Example:

```text
features/

    authentication.md

feature-technical/

    authentication.md
```

Both documents describe the same feature from different perspectives.

No Feature Technical Design should describe multiple unrelated features.

---

## Architecture Principle Application

Feature Technical Design applies the reusable principles defined by Architecture Documentation.

Examples include:

* Component Model
* Runtime Boundaries
* Communication Model
* Security Boundaries
* Persistence Architecture
* Plugin Architecture
* Deployment Constraints
* Ownership Rules

Feature Technical Design should reference these principles rather than redefining them.

---

## External Context Application

Feature Technical Design should identify the External Context required to realize the feature.

Examples include:

* Internal shared frameworks
* Shared runtime libraries
* Platform capabilities
* Operating system services
* External APIs
* Communication protocols
* Authentication providers
* AI platforms
* Storage platforms

Feature Technical Design should describe how these external dependencies participate in the architectural realization of the feature.

External Context should be referenced rather than duplicated.

Only context relevant to the feature should be included.

---

## Feature Design Considerations

Feature Technical Design should consider Feature Design whenever user experience influences architectural decisions.

Examples include:

* Navigation requiring routing architecture
* Accessibility requiring architectural support
* Localization requiring resource architecture
* Offline behavior requiring synchronization architecture
* Responsive behavior requiring layout architecture
* Long-running workflows requiring orchestration

Feature Technical Design should realize Feature Design without redefining user experience.

---

## Architectural Realization Principles

Every Feature Technical Design should:

* Respect architectural boundaries.
* Preserve component ownership.
* Minimize coupling.
* Maximize cohesion.
* Reuse existing architectural patterns.
* Avoid introducing new architecture unnecessarily.
* Clearly define responsibilities.
* Clearly define communication.
* Respect external architectural constraints.

---

## Technology Independence

Feature Technical Design should remain implementation independent.

It may reference technologies only when they are architecturally significant.

Examples:

Acceptable:

* Electron Main Process
* Plugin Runtime
* SQLite Persistence Layer
* Event Bus
* REST Gateway
* Message Queue

Not Acceptable:

* React Hooks
* Axios usage
* SQL queries
* TypeScript interfaces
* Rust traits
* Function implementations

Implementation belongs to source code.

---

## Quality Requirements

Feature Technical Design should be:

* Feature-specific
* Architecturally consistent
* Traceable
* Cohesive
* Modular
* Maintainable
* Implementation independent
* Consistent with Architecture
* Consistent with External Context

Each document should remain focused on one feature.

---
