# Purpose — Generation Template

> **Domain:** external-context
> **Section:** purpose
> **Source:** `documentation-standards/08-external-context-standards.md` §Purpose
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Purpose section for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `informs` | engineering / code_standards | Purpose must explain how this dependency informs engineering code standards |
| `informs` | feature_design / design_rationale | Purpose must explain how this dependency informs feature design rationale |
| `informs` | feature_technical / component_responsibilities | Purpose must explain how this dependency informs feature technical component responsibilities |

## Template

```markdown
## Purpose

[1 sentence: what External Context Documentation is and its role in the documentation ecosystem]

[1 sentence: what this specific document captures — the knowledge dependency this external system imposes on the repository]

[1 sentence: distinction from package dependency documentation — this documents knowledge, not package manifests]

[1 sentence: the atomic-per-dependency principle — one document per external dependency]
```

## Examples

**Correct:**
> External Context Documentation captures knowledge dependencies that live outside the repository but materially influence implementation. This document captures the knowledge required to correctly integrate with the external system. It is distinct from package dependency documentation, which tracks library versions and build artifacts. Each document describes a single external dependency, making the collection atomic and independently maintainable.

**Incorrect:**
> External Context Documentation covers npm packages, pip dependencies, and Cargo crates required by the project. It lists version numbers, installation commands, and upgrade procedures for each library.
> *Why wrong: This conflates External Context with package dependency documentation. External Context captures knowledge about external systems, not package manifests or version management.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** State what External Context is in one declarative sentence. Draw a clear boundary between knowledge dependencies and package dependencies. Reference the atomic-per-dependency principle.
- **Don't:** Describe internal architecture or implementation details. List package managers or version numbers. Use inspirational or motivational language.

**Generation Note:** When generating for a specific system, state what THIS system's external knowledge dependency is. Example: "This document captures the knowledge contributors need about the Stripe Payment API to implement payment processing correctly in the Nova platform."

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** none

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
