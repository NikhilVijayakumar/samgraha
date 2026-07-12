# Purpose — Generation Template

> **Domain:** philosophy
> **Section:** purpose
> **Source:** `documentation-standards/02-philosophy-standards.md` §Purpose
> **Relationships:** `audit/deterministic/document/02-philosophy-relationships.yaml`

Generate the Purpose section for a Philosophy document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `derives_from` | vision / purpose | Purpose must trace to Vision's purpose — Philosophy explains *how* the team decides, given *why* the product exists |
| `guided_by` | philosophy / guiding_principles (self) | Purpose must be consistent with the guiding principles it introduces |

## Template

```markdown
## Purpose

This document defines the standard for [System Name] documentation within the engineering documentation ecosystem.

[One paragraph explaining what this documentation type does and why it exists.]

[One paragraph distinguishing it from related standards — what it covers that others do not.]
```

## Examples

**Correct:**
> This document defines the standard for Project Horizon documentation within the engineering documentation ecosystem.
>
> Project Horizon Documentation establishes the product's guiding principles, values, and the deliberate trade-offs that shape every downstream decision.
>
> Unlike Vision, which explains **why** the product exists, Philosophy explains **how the people building it choose to think and decide**.

**Incorrect:**
> This document defines the Philosophy for the React frontend and PostgreSQL backend of Project Horizon.
> *Why wrong: Technology-specific — references concrete technologies instead of describing Philosophy's role in the ecosystem.*

## Writing Guidance

- **Tone:** inspirational
- **Voice:** first person plural
- **Structure:** paragraphs
- **Audience:** product owner
- **Do:** State Philosophy's role before listing contents; distinguish it from Vision using a clear contrast; use language that endures across technology changes
- **Don't:** Mention specific technologies or frameworks; describe features or architecture; use implementation-level vocabulary

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Vision(01)

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
