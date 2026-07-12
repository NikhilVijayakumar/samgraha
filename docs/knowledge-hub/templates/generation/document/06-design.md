# Design Document — Generation Template

> **Domain:** design
> **Source standard:** `documentation-standards/06-design-standards.md`
> **Coherence source:** `audit/semantic/document/06-design.md`
> **Relationships:** `audit/deterministic/document/06-design-relationships.yaml`

Generate a complete Design document for a system. The document must satisfy every required section below, in the order defined by the standard.

## Required Sections

| # | Section | semantic_type | Required | Content Requirements |
|---|---------|--------------|----------|---------------------|
| 1 | Purpose | `purpose` | | Problem statement, scope definition, distinction from Feature Design |
| 2 | UX Principles | `ux_principles` | ✓ | 3+ interaction principles covering navigation, feedback, and discoverability |
| 3 | Accessibility | `accessibility` | ✓ | Compliance target (e.g. WCAG level), inclusive design principles, assistive technology guidance |
| 4 | Constraints | `constraints` | | Binding constraints with source (regulatory, platform, organizational) and enforcement scope |

## Cross-Section Coherence Constraint

> Sourced from `audit/semantic/document/06-design.md` Engineering Intent.

All sections must be mutually consistent — no contradictions between UX Principles and Accessibility, between Constraints and UX Principles, or between any two sections. Terminology must be consistent across all sections: same concept, same name. Design principles must be reusable and technology-independent throughout. No feature-specific content may appear in any section.

## Sections

---

### 1. Purpose

**Template:**

```markdown
## Purpose

[1 paragraph: what problem this Design Documentation solves — the inconsistency or design debt it prevents — and why it exists as a standalone document]

[1 paragraph: scope boundary — what this Design Documentation defines (product-level design principles, interaction philosophy, UX standards) and what it does not (feature-specific behavior, implementation details, screen specifications). Explicitly distinguish from Feature Design.]
```

**Correct example:**
> Design Documentation solves the problem of inconsistent user experience across features by establishing a shared design language, interaction philosophy, and UX standards that govern the entire product ecosystem. It exists so that every feature delivers a consistent, predictable experience without each designer redefining foundational principles.
>
> This document defines how the product should be designed at the product level — reusable principles, interaction philosophy, and accessibility standards. It does not describe how individual features behave, which screens exist, or which UI framework is used. Feature-specific design decisions belong in Feature Design, not here.

**Incorrect example:**
> This document defines the checkout flow for the payment module, including screen layouts and button placement.
> *Why wrong: This describes a feature-specific workflow, not a product-level design standard.*

**Writing guidance:**
- **Tone:** prescriptive
- **Voice:** third person
- **Structure:** paragraphs
- **Audience:** architect
- **Do:** State the problem before describing scope. Distinguish from Feature Design. Set firm scope boundaries.
- **Don't:** List specific features. Reference implementation technologies. Use motivational language.

---

### 2. UX Principles

**Template:**

```markdown
## UX Principles

[1 paragraph: how UX principles connect to the product's design philosophy and why they govern interaction decisions across all features]

### [Principle Name]

[1 paragraph: what this principle means at the product level — how it shapes user interaction across all features]

[1 cross-feature example: how this principle applies across at least two different features]

---

[Repeat for each principle — minimum 3]
```

**Correct example:**
> ### Feedback Visibility
> Users should always receive immediate, clear feedback when they perform an action. Whether submitting a form, navigating to a new view, or encountering an error, the system confirms the action occurred and indicates the result.
>
> When a user submits a support ticket, they see a confirmation toast and are redirected to the ticket detail view. When a user updates their profile settings, the same confirmation pattern applies.

**Incorrect example:**
> When the user clicks the "Submit" button in React, the onClick handler should call the API endpoint and display a Material UI Snackbar component.
> *Why wrong: Technology-specific implementation guidance, not a product-level UX principle.*

**Writing guidance:**
- **Tone:** prescriptive
- **Voice:** third person
- **Structure:** mixed
- **Audience:** product owner
- **Do:** Frame each principle as product-level philosophy. Provide cross-feature examples. Ground in user outcomes.
- **Don't:** Reference UI frameworks or libraries. Describe feature-specific workflows. Conflate with visual design specs.

---

### 3. Accessibility

**Template:**

```markdown
## Accessibility

[1 paragraph: accessibility philosophy — why inclusive design matters for this product and its users]

### Compliance Targets

[1 paragraph: applicable standard (e.g. WCAG 2.1 AA), which interfaces it applies to, how exceptions are handled]

### Inclusive Design Principles

[1 principle per subsection — minimum 3, each with rationale and product-level guidance]

[Repeat for each principle]

### Assistive Technology Support

[1 paragraph: screen readers, keyboard navigation, voice control — product-level guidance]
```

**Correct example:**
> ### Compliance Targets
> The product targets WCAG 2.1 Level AA conformance across all user-facing interfaces. Exceptions are documented per feature and must receive explicit approval from the accessibility lead.
>
> ### Inclusive Design Principles
> #### Perceivable Content
> All information and UI components must be presentable in ways users can perceive. Text alternatives for non-text content, captions for multimedia, and sufficient color contrast ratios of at least 4.5:1.

**Incorrect example:**
> All buttons must include `aria-label="Submit"` and use the `role="button"` attribute.
> *Why wrong: Component-level ARIA patterns belong to Engineering, not Design.*

**Writing guidance:**
- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** paragraphs
- **Audience:** engineer
- **Do:** Reference specific compliance standards with enforcement scope. Define inclusive design principles at product level.
- **Don't:** Specify ARIA attributes or HTML patterns. Write testing scripts. Confuse design-level with engineering-level.

---

### 4. Constraints

**Template:**

```markdown
## Constraints

[1 paragraph: how constraints shape design decisions — what makes them binding]

### [Constraint Category] (e.g. Regulatory, Platform, Organizational)

> **source:** [regulatory body, platform requirement, organizational mandate]
> **enforcement:** [binding / advisory]
> **scope:** [which features or capabilities this applies to]

[1 paragraph: what this constraint requires and why it is non-negotiable]

[Repeat for each constraint category — minimum 1]
```

**Correct example:**
> ### Regulatory
> > **source:** Federal accessibility regulation (Section 508)
> > **enforcement:** binding
> > **scope:** All user-facing interfaces
>
> The product must comply with federal accessibility requirements. This constraint is non-negotiable and applies regardless of timeline or budget.

**Incorrect example:**
> We prefer using a specific CSS framework because the team knows it well.
> *Why wrong: This is a team preference, not a binding constraint.*

**Writing guidance:**
- **Tone:** structural
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** architect
- **Do:** State each constraint with source, enforcement level, and scope. Distinguish binding from advisory.
- **Don't:** List team preferences. Omit source or enforcement level. Use vague qualifiers.

---

## Output Contract

Output a single complete markdown document containing all 4 sections above, in the order listed. Each section must:

1. Use the template skeleton as its structural basis
2. Fill every placeholder with domain-appropriate content (not lorem ipsum)
3. Satisfy the Writing Guidance for its section
4. Be consistent with every other section (cross-section coherence constraint above)
5. Omit implementation details
6. Remain technology-independent throughout
