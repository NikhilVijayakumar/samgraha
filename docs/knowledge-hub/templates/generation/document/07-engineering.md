# Engineering Document — Generation Template

> **Domain:** engineering
> **Source standard:** `documentation-standards/07-engineering-standards.md`
> **Coherence source:** `audit/semantic/document/07-engineering.md`
> **Relationships:** `audit/deterministic/document/07-engineering-relationships.yaml`

Generate a complete Engineering document for a system. The document must satisfy every required section below, in the order defined by the standard.

## Required Sections

| # | Section | semantic_type | Required | Content Requirements |
|---|---------|--------------|----------|---------------------|
| 1 | Purpose | `purpose` | | Document's role in ecosystem, scope boundaries, relationship to adjacent standards |
| 2 | Build Standards | `build_standards` | ✓ | Build system, pipeline stages, quality gates, rationale for each stage |
| 3 | Testing Standards | `testing_standards` | ✓ | Test types, coverage expectations, test tooling, rationale |
| 4 | Code Standards | `code_standards` | | Style guide, linting configuration, naming conventions, rationale |
| 5 | Constraints | `constraints` | | Non-functional requirements categorized by type, verifiable, connected to source |

## Cross-Section Coherence Constraint

> Sourced from `audit/semantic/document/07-engineering.md` Engineering Intent.

All sections must be mutually consistent — no contradictions between Build Standards and Testing Standards, between Code Standards and Constraints, or between any two sections. Technology rationale in Build Standards must align with Code Standards tooling choices. Constraints must derive from Architecture and External Context, not from internal preferences. Terminology must be consistent across all sections: same concept, same name.

## Sections

---

### 1. Purpose

**Template:**

```markdown
## Purpose

[1–2 paragraphs: this document's role in the documentation ecosystem — what Engineering Documentation explains (repository-wide engineering decisions, implementation standards, technology rationale, development conventions) and how it differs from adjacent standards (Architecture, Feature Technical Design)]

[1 paragraph: scope boundary — what is included and what is excluded]
```

**Correct example:**
> This document defines Engineering Documentation's role in the documentation ecosystem. It establishes repository-wide engineering decisions, implementation standards, technology selection rationale, and development conventions. Unlike Feature Technical Design, Engineering Documentation is not feature-specific — it provides reusable knowledge that governs the entire repository.
>
> Engineering Documentation explains why the repository is engineered this way. It does not describe feature implementations, embed source code, or explain algorithms.

**Incorrect example:**
> This document describes the login page implementation, including the OAuth2 flow and JWT token storage.
> *Why wrong: Feature-specific and describes implementation details, not the repository-wide role of Engineering Documentation.*

**Writing guidance:**
- **Tone:** structural
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** Define the document's role explicitly. Distinguish from adjacent standards. Set clear scope boundaries.
- **Don't:** Include implementation details. Blur boundaries with adjacent standards.

---

### 2. Build Standards

**Template:**

```markdown
## Build Standards

> [metadata block]

### Build System

[1–2 paragraphs: build tool name and configuration, rationale for choosing this build system]

### Pipeline Stages

> **diagram:** flowchart of pipeline stages

| Stage | Purpose | Inputs | Outputs | Quality Gate |
|-------|---------|--------|---------|-------------|
| [Stage name] | [what it does] | [what feeds it] | [what it produces] | [pass/fail criteria] |

### Quality Gates

[Optional: criteria that must pass before proceeding]
```

**Correct example:**
> **Build System:** The repository uses a task runner configured via `build.config.toml`. Each pipeline stage runs in an isolated container to ensure reproducibility. Rationale: deterministic builds ensure that any commit produces the same artifact regardless of the build environment.

**Incorrect example:**
> **Build System:** We use Jenkins. Our pipeline is: checkout → build → deploy to staging.
> *Why wrong: Missing rationale, missing quality gates, describes deployment which is out of scope.*

**Writing guidance:**
- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** mixed
- **Audience:** engineer
- **Do:** Document each stage with purpose, inputs, outputs, and quality gates. Explain rationale. Include a flowchart.
- **Don't:** Describe deployment. Omit quality gates. List stages without explaining why.

---

### 3. Testing Standards

**Template:**

```markdown
## Testing Standards

> [metadata block]

### Test Types

| Test Type | Purpose | Scope | Tooling | Execution |
|-----------|---------|-------|---------|-----------|
| [Type name] | [what it validates] | [what it covers] | [test runner, assertions] | [when it runs] |

### Coverage Expectations

| Metric | Target | Scope | Rationale |
|--------|--------|-------|-----------|
| [Metric name] | [threshold] | [what it covers] | [why this threshold] |

### Test Tooling

[Optional: test runner, assertion libraries, mocking frameworks]
```

**Correct example:**
> | Test Type | Purpose | Scope | Tooling | Execution |
> |-----------|---------|-------|---------|-----------|
> | Unit | Validate individual modules | Single module | Vitest | Every commit |
> | Integration | Verify module interactions | Cross-module | Vitest + supertest | Every commit |
> | E2E | Exercise critical user journeys | Full system | Playwright | Nightly |

**Incorrect example:**
> We have unit tests and some integration tests. Coverage is pretty good.
> *Why wrong: Vague, no specific test type definitions, no coverage targets, no rationale.*

**Writing guidance:**
- **Tone:** technical
- **Voice:** imperative
- **Structure:** mixed
- **Audience:** engineer
- **Do:** Define each test type with purpose, scope, tooling. Set measurable coverage targets.
- **Don't:** Use vague claims. Omit specific thresholds. Describe feature-specific tests.

---

### 4. Code Standards

**Template:**

```markdown
## Code Standards

> [metadata block]

### Style Guide

[1 paragraph: language-specific style guide reference, key rules]

### Linting Configuration

[1 paragraph: linter tool, config file location, key rules]

| Rule Category | Description | Enforcement |
|--------------|-------------|-------------|
| [Category] | [what it checks] | [error/warning] |

### Naming Conventions

[Optional: naming patterns for files, modules, functions, variables]
```

**Correct example:**
> **Style Guide:** All modules follow the TypeScript Official Style Guide. Functions are named with verb-noun convention. Files are named after the module they contain.
> **Linting Configuration:** ESLint is configured via `.eslintrc.cjs` at the repository root. All CI builds must pass the linter before merge.

**Incorrect example:**
> We use tabs for indentation and camelCase for variables. Our linter catches some errors.
> *Why wrong: Missing rationale, missing configuration location, no connection to engineering principles.*

**Writing guidance:**
- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** Reference the specific style guide. Document linting config location and key rules. Explain non-obvious conventions.
- **Don't:** Assume universal knowledge. Omit config locations. Describe feature-specific patterns.

---

### 5. Constraints

**Template:**

```markdown
## Constraints

> [metadata block]

[1–2 paragraphs: overview of non-functional requirements and engineering limitations]

### Performance Constraints

| Constraint | Threshold | Source | Verifiability |
|-----------|-----------|--------|---------------|
| [Constraint name] | [specific value] | [where it comes from] | [how to verify] |

### Security Constraints

| Constraint | Requirement | Source | Verifiability |
|-----------|-------------|--------|---------------|
| [Constraint name] | [specific requirement] | [where it comes from] | [how to verify] |

### Compliance Constraints

[Optional: regulatory, organizational policy requirements]
```

**Correct example:**
> ### Performance Constraints
> | Constraint | Threshold | Source | Verifiability |
> |-----------|-----------|--------|---------------|
> | API response time | ≤200ms at p95 | Architecture Section 4.3 | Load testing |

**Incorrect example:**
> The application should be fast and secure. We follow industry best practices.
> *Why wrong: Not verifiable, no specific thresholds, no source attribution.*

**Writing guidance:**
- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** Categorize by type. Cite source. Make verifiable with specific thresholds.
- **Don't:** State vague requirements. Omit source attribution. List without categorization.

---

## Output Contract

Output a single complete markdown document containing all 5 sections above, in the order listed. Each section must:

1. Use the template skeleton as its structural basis
2. Fill every placeholder with domain-appropriate content (not lorem ipsum)
3. Satisfy the Writing Guidance for its section
4. Be consistent with every other section (cross-section coherence constraint above)
5. Include diagrams where Required diagrams are specified
6. Omit implementation details — explain rationale, not code
