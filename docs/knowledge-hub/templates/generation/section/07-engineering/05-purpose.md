# Purpose — Generation Template

> **Domain:** engineering
> **Section:** purpose
> **Source:** `documentation-standards/07-engineering-standards.md` §Purpose
> **Relationships:** `audit/deterministic/document/07-engineering-relationships.yaml`

Generate the Purpose section for an Engineering document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `derives_from` | philosophy / purpose | Engineering Purpose must derive from the product's philosophical purpose |
| `guided_by` | philosophy / guiding_principles | Engineering Purpose must align with guiding philosophy |

## Template

```markdown
## Purpose

[1–2 paragraphs: this document's role in the documentation ecosystem — what Engineering Documentation explains (repository-wide engineering decisions, implementation standards, technology rationale, development conventions) and how it differs from adjacent standards (Architecture, Feature Technical Design)]

[1 paragraph: scope boundary — what is included (repo-wide standards, build/test/code conventions, technology rationale) and what is excluded (feature implementations, source code, algorithms, business logic)]
```

## Examples

**Correct:**
> This document defines Engineering Documentation's role in the documentation ecosystem. It establishes repository-wide engineering decisions, implementation standards, technology selection rationale, and development conventions. Unlike Feature Technical Design, Engineering Documentation is not feature-specific — it provides reusable knowledge that governs the entire repository.
>
> Engineering Documentation explains why the repository is engineered this way. It does not describe feature implementations, embed source code, or explain algorithms. Those responsibilities belong to Feature Technical Design and Implementation.

**Incorrect:**
> This document describes the login page implementation, including the OAuth2 flow, JWT token storage, and session management using Redis.
> *Why wrong: This is feature-specific and describes implementation details, not the repository-wide role of Engineering Documentation in the ecosystem.*

## Writing Guidance

- **Tone:** structural
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** Define the document's role in the documentation ecosystem explicitly. Distinguish Engineering Documentation from adjacent standards. Set clear scope boundaries with what is included and excluded.
- **Don't:** Include implementation details or feature-specific content. Blur boundaries with adjacent documentation standards. Describe what the document contains rather than why it exists.

**Generation Note:** When generating for a specific system, replace generic role descriptions with the specific engineering challenges this system faces. Example: "This document defines Engineering Documentation for the Nova platform, establishing repository-wide standards for TypeScript compilation, Vitest testing, and Turborepo build pipelines."

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Architecture(05), adjacent documentation standards

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
