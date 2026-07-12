# Purpose — Generation Template

> **Domain:** architecture
> **Section:** purpose
> **Source:** `documentation-standards/05-architecture-standards.md` §Purpose
> **Relationships:** `audit/deterministic/document/05-architecture-relationships.yaml`

Generate the Purpose section for an Architecture document.

## Relationships

This section has the following outgoing relationships that must be satisfied:

| Relationship | Target | Constraint |
|---|---|---|
| `derives_from` | vision / purpose | Architecture Purpose must align with and derive from the Vision document's Purpose |
| `guided_by` | philosophy / guiding_principles | Architecture Purpose must be consistent with the project's guiding principles |

## Template

```markdown
This document defines the standard for [Documentation Type] within the [ecosystem name] documentation ecosystem.

[Documentation Type] describes [what it describes].

Unlike [related type], [distinctive characteristic].

[Core purpose statement.]
```

## Examples

**Correct:**
> This document defines the standard for Architecture Documentation within the engineering documentation ecosystem. Architecture Documentation describes the structural organization of a system — how responsibilities are divided among components and how those components relate. Unlike a single Vision or Feature document, Architecture is a collection of focused documents, each covering one structural concern.

**Incorrect:**
> Architecture Documentation covers the microservices layout, the React component tree, the PostgreSQL schema, and the Kubernetes deployment manifests used by the system.
> *Why wrong: names specific technologies and implementation artifacts instead of stating the document type's role and boundary — that belongs in the sections themselves, not the Purpose statement.*

## Writing Guidance

- **Tone:** structural
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** State what Architecture Documentation is and what it is not in the same breath; describe it as a collection rather than a single artifact; keep the boundary with Engineering and Feature Technical Design explicit
- **Don't:** Name specific components, technologies, or frameworks; describe how any particular system is organized; list features or capabilities

**Minimum content:** 2 paragraphs
**Length guidance:** concise
**Required diagrams:** none
**Required cross-references:** none

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
