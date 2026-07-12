# External Context Document — Generation Template

> **Domain:** external-context
> **Source standard:** `documentation-standards/08-external-context-standards.md`
> **Coherence source:** `audit/semantic/document/08-external-context.md`
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate a complete External Context document for a single external dependency. The document must satisfy every required section below, in the order defined by the standard.

## Required Sections

| # | Section | semantic_type | Required | Content Requirements |
|---|---------|--------------|----------|---------------------|
| 1 | Purpose | `purpose` | ✓ | Why external context exists, what it documents, what it does not; distinguish from package dependency documentation |
| 2 | Integration Contract | `integration_contract` | ✓ | Endpoints, protocols, data formats, authentication, error behaviors, versioning; reference authoritative API docs |
| 3 | Constraints | `constraints` | | Categorized limitations (functional, performance, legal, compliance) sourced from the external system |
| 4 | Dependencies | `dependencies` | | Transitive requirements, platform prerequisites, companion systems; distinguish runtime from build-time; note criticality |

## Cross-Section Coherence Constraint

> Sourced from `audit/semantic/document/08-external-context.md` Engineering Intent.

All sections must be mutually consistent — no contradictions between Integration Contract capabilities and Constraints limitations, between Dependencies and Integration Contract availability requirements, or between any two sections. If Integration Contract describes a rate-limited endpoint, Constraints must document the exact rate limit. If Dependencies lists a critical runtime service, Integration Contract must reflect its availability constraints. Terminology must be consistent across all sections: same concept, same name.

## Sections

---

### 1. Purpose

**Template:**

```markdown
## Purpose

[1 sentence: what External Context Documentation is and its role in the documentation ecosystem]

[1 sentence: what this specific document captures — the knowledge dependency this external system imposes on the repository]

[1 sentence: distinction from package dependency documentation — this documents knowledge, not package manifests]

[1 sentence: the atomic-per-dependency principle — one document per external dependency]
```

**Correct example:**
> External Context Documentation captures knowledge dependencies that live outside the repository but materially influence implementation. This document captures the knowledge required to correctly integrate with the external system. It is distinct from package dependency documentation, which tracks library versions and build artifacts. Each document describes a single external dependency, making the collection atomic and independently maintainable.

**Incorrect example:**
> External Context Documentation covers npm packages, pip dependencies, and Cargo crates required by the project.
> *Why wrong: Conflates External Context with package dependency documentation.*

**Writing guidance:**
- **Tone:** technical
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** State what External Context is. Draw clear boundary between knowledge and package dependencies. Reference atomic-per-dependency principle.
- **Don't:** Describe internal architecture. List package managers. Use motivational language.

---

### 2. Integration Contract

**Template:**

```markdown
## Integration Contract

[Integration surface overview — what the external system exposes]

### Endpoints / Protocols

| Endpoint / Protocol | Purpose | Method | Required | Authentication | Notes |
|--------------------|---------|--------|----------|---------------|-------|
| [Path or protocol name] | [what it does] | [HTTP method / protocol type] | [yes/no] | [auth type] | [constraints] |

### Data Formats

| Direction | Format | Content Type | Encoding | Constraints |
|-----------|--------|-------------|----------|-------------|
| Request | [format] | [MIME type] | [encoding] | [limits] |
| Response | [format] | [MIME type] | [encoding] | [limits] |

### Authentication

| Aspect | Detail |
|--------|--------|
| Mechanism | [OAuth 2.0, API key, mutual TLS, etc.] |
| Flow | [client credentials, authorization code, etc.] |
| Token type | [Bearer, MAC, etc.] |
| Token lifetime | [expiration duration] |
| Refresh | [supported / not supported] |
| Credential storage | [how credentials should be stored] |

### Error Handling

| Error Code / Status | Meaning | Retry | Backoff Strategy |
|---------------------|---------|-------|-----------------|
| [code] | [what it means] | [yes/no] | [strategy] |

### Versioning

| Aspect | Detail |
|--------|--------|
| Versioning scheme | [URL path, header, query parameter] |
| Current version | [version identifier] |
| Deprecation policy | [how deprecation is communicated] |
| Migration guidance | [how to upgrade between versions] |
```

**Correct example:**
> The external system exposes a REST API over HTTPS. Authentication uses OAuth 2.0 client credentials flow.
>
> | Endpoint / Protocol | Purpose | Method | Required | Authentication | Notes |
> |--------------------|---------|--------|----------|---------------|-------|
> | `/v1/data` | Submit data | POST | Yes | Bearer token | Max 1MB payload |
> | `/v1/data/{id}` | Retrieve data | GET | Yes | Bearer token | Returns 404 for deleted records |
>
> Authoritative documentation: `https://docs.externalsystem.example/api`.

**Incorrect example:**
> Here is the code we use to call the API:
> ```python
> resp = requests.post("https://api.externalsystem.example/v1/data", ...)
> ```
> *Why wrong: Includes implementation code rather than describing the contract.*

**Writing guidance:**
- **Tone:** technical
- **Voice:** third person
- **Structure:** mixed
- **Audience:** engineer
- **Do:** Describe contract surface in implementation-neutral terms. Always include authoritative documentation URL. Distinguish required from optional endpoints.
- **Don't:** Paste code. Document internal transformation logic. Omit authentication details.

---

### 3. Constraints

**Template:**

```markdown
## Constraints

[1 paragraph: overview of constraints the external system imposes on the repository]

### Functional Constraints

| Constraint | Limitation | Impact | Source |
|-----------|-----------|--------|--------|
| [Name] | [specific restriction] | [what this prevents] | [external source] |

### Performance Constraints

| Constraint | Limitation | Impact | Source |
|-----------|-----------|--------|--------|
| [Name] | [specific numeric limit] | [what this requires] | [external source] |

### Legal / Compliance Constraints

| Constraint | Requirement | Impact | Source |
|-----------|-------------|--------|--------|
| [Name] | [specific requirement] | [what this requires] | [external source] |
```

**Correct example:**
> ### Functional Constraints
> | Constraint | Limitation | Impact | Source |
> |-----------|-----------|--------|--------|
> | Max payload | 1 MB per request | Large file uploads must be chunked | Platform API documentation §3.2 |
>
> ### Performance Constraints
> | Constraint | Limitation | Impact | Source |
> |-----------|-----------|--------|--------|
> | Rate limit | 100 requests/minute | Request throttling needed for batch operations | Platform service limits |

**Incorrect example:**
> We decided to use connection pooling because our application needs high throughput.
> *Why wrong: Internal design decision, not a constraint imposed by the external system.*

**Writing guidance:**
- **Tone:** technical
- **Voice:** third person
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** Categorize by type. Cite external source. State numeric limits where possible.
- **Don't:** List internal decisions. Use vague qualifiers. Omit compliance constraints.

---

### 4. Dependencies

**Template:**

```markdown
## Dependencies

[1 paragraph: overview of transitive requirements the external system itself needs]

### Runtime Dependencies

| Dependency | Purpose | Criticality | Behavior if Unavailable | Source |
|-----------|---------|-------------|------------------------|--------|
| [Name] | [what it provides] | [critical/nice-to-have] | [what fails] | [where documented] |

### Build-Time Dependencies

| Dependency | Purpose | Criticality | Behavior if Unavailable | Source |
|-----------|---------|-------------|------------------------|--------|
| [Name] | [what it provides] | [critical/nice-to-have] | [what fails] | [where documented] |
```

**Correct example:**
> ### Runtime Dependencies
> | Dependency | Purpose | Criticality | Behavior if Unavailable | Source |
> |-----------|---------|-------------|------------------------|--------|
> | Message broker | Webhook delivery | Critical | Webhooks fail silently | Platform documentation §6.2 |
>
> These are transitive requirements of the platform itself, not choices made by this repository.

**Incorrect example:**
> This project depends on Express.js for HTTP routing and Jest for testing.
> *Why wrong: Internal project dependencies, not transitive dependencies of the external system.*

**Writing guidance:**
- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** architect
- **Do:** Distinguish runtime from build-time. Note criticality. Cross-reference other External Context documents for documented transitive dependencies.
- **Don't:** List package.json entries. Include dev tooling. Omit companion systems.

---

## Output Contract

Output a single complete markdown document containing all 4 sections above, in the order listed. Each section must:

1. Use the template skeleton as its structural basis
2. Fill every placeholder with domain-appropriate content for the specific external dependency being documented (not lorem ipsum)
3. Satisfy the Writing Guidance for its section
4. Be consistent with every other section (cross-section coherence constraint above)
5. Include tables where template specifies them
6. Reference authoritative external documentation URLs throughout
7. Omit internal implementation details — describe what the external system exposes, not how the repository calls it
