# Purpose — Generation Template

> **Domain:** design
> **Section:** purpose
> **Source:** `documentation-standards/06-design-standards.md` §Purpose
> **Relationships:** `audit/deterministic/document/06-design-relationships.yaml`

Generate the Purpose section for a Design document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `guided_by` | philosophy / guiding_principles | Design Purpose must align with the product's guiding philosophy |

## Template

```markdown
## Purpose

[1 paragraph: what problem this Design Documentation solves — the inconsistency or design debt it prevents — and why it exists as a standalone document]

[1 paragraph: scope boundary — what this Design Documentation defines (product-level design principles, interaction philosophy, UX standards) and what it does not (feature-specific behavior, implementation details, screen specifications). Explicitly distinguish from Feature Design.]
```

## Examples

**Correct:**
> Design Documentation solves the problem of inconsistent user experience across features by establishing a shared design language, interaction philosophy, and UX standards that govern the entire product ecosystem. It exists so that every feature delivers a consistent, predictable experience without each designer redefining foundational principles.
>
> This document defines how the product should be designed at the product level — reusable principles, interaction philosophy, and accessibility standards. It does not describe how individual features behave, which screens exist, or which UI framework is used. Feature-specific design decisions belong in Feature Design, not here.

**Incorrect:**
> This document defines the checkout flow for the payment module, including screen layouts and button placement for the order form.
> *Why wrong: This describes a feature-specific workflow, not a product-level design standard. Design Documentation must not contain feature-specific content.*

**Incorrect:**
> Design Documentation is important because good design leads to better products and happier users.
> *Why wrong: This is motivational filler, not a structural purpose statement. The Purpose must state what problem it solves and what scope it defines.*

## Writing Guidance

- **Tone:** prescriptive
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** State the problem Design Documentation solves before describing what it defines. Distinguish Design Documentation from Feature Design explicitly. Set firm scope boundaries. Ground the purpose in preventing design inconsistency, not in aspirational goals.
- **Don't:** List specific features or products. Reference implementation technologies or frameworks. Describe how individual features behave. Use motivational or aspirational language. Conflate purpose with a feature specification.

**Generation Note:** When generating for a specific system, replace the generic problem statement with the specific design inconsistency this system faces. Example: "Design Documentation for the Nova platform solves the problem of divergent interaction patterns across the dashboard, settings, and reporting modules by establishing unified UX principles." Do NOT write meta-level language about documentation types.

**Required subsections:** none
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** Vision, Feature Design Standard

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
