# Accessibility — Generation Template

> **Domain:** design
> **Section:** accessibility
> **Source:** `documentation-standards/06-design-standards.md` §Accessibility
> **Relationships:** `audit/deterministic/document/06-design-relationships.yaml`

Generate the Accessibility section for a Design document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `guided_by` | philosophy / guiding_principles | Accessibility principles must reflect the product's guiding philosophy |

## Template

```markdown
## Accessibility

[1 paragraph: accessibility philosophy — why inclusive design matters for this product and its users]

### Compliance Targets

[1 paragraph: applicable standard (e.g. WCAG 2.1 Level AA), which interfaces it applies to, and how exceptions are handled]

### Inclusive Design Principles

[1 principle per subsection — minimum 3, each with rationale and product-level guidance]

#### [Principle Name]

[1 paragraph: what this principle requires across all features]

[1 example: how this principle applies in practice]

---

### Assistive Technology Support

[1 paragraph: screen readers, keyboard navigation, voice control — product-level guidance for how the product supports assistive technologies]
```

## Examples

**Correct:**
> ### Compliance Targets
> The product targets WCAG 2.1 Level AA conformance across all user-facing interfaces. This applies to every feature with a user interface, including web, mobile, and desktop applications. Exceptions are documented per feature and must receive explicit approval from the accessibility lead.
>
> ### Inclusive Design Principles
> #### Perceivable Content
> All information and UI components must be presentable in ways users can perceive. This means text alternatives for non-text content, captions for multimedia, and sufficient color contrast ratios of at least 4.5:1 across all interactive elements.

**Incorrect:**
> All buttons must include `aria-label="Submit"` and use the `role="button"` attribute to comply with WCAG 2.1 Level AA.
> *Why wrong: This contains component-level ARIA patterns, which belong to Engineering, not Design. Accessibility at the design level should define compliance targets and inclusive design principles, not HTML attribute requirements.*

## Writing Guidance

- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** paragraphs
- **Audience:** engineer
- **Do:** Reference specific compliance standards (e.g. WCAG level) with enforcement scope. Define inclusive design principles at the product level before any technical requirements. State which interfaces the compliance targets apply to.
- **Don't:** Specify component-level ARIA attributes or HTML patterns. Write testing scripts or code snippets. Confuse design-level accessibility with engineering-level implementation details.

**Required subsections:** Compliance Targets, Inclusive Design Principles
**Optional subsections:** Assistive Technology Support, Testing Strategy
**Required diagrams:** none
**Required cross-references:** UX Principles, Feature Design Standard, Engineering Standard

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
