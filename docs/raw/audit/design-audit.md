# Design Audit

## Context

Validates design docs in `docs/raw/design/` against the **design.md** standard.
Design Documentation establishes shared design language, principles, and UX
standards for the entire product. It is reusable and not feature-specific.

## Authority

`docs/raw/standards/design.md` — Audit Rules section.

## Scope

All files under `docs/raw/design/`. Quality evaluated across the complete design
collection.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### D1. Design Principles Are Reusable
Design principles apply across multiple features. They are not written for a
single feature or workflow. Reusability is the primary quality criterion.

**Audit Rule:** Design principles are reusable.

### D2. Technology Independent
Design describes interaction philosophy, presentation principles, and
communication principles — not UI frameworks, CSS libraries, frontend frameworks,
component implementations, or rendering technologies.

**Audit Rule:** Design remains technology independent.

### D3. No Feature-Specific Behavior
No individual feature workflows, application screens, or feature-specific mockups
appear in Design Documentation. Feature-specific content belongs in Feature Design.

**Audit Rule:** Feature-specific behavior has not been introduced.

### D4. Design Philosophy Defined
Product design philosophy is clearly documented. The design philosophy explains
the guiding principles behind the product's user experience.

**Audit Rule:** Design philosophy is clearly defined.

### D5. Accessibility Guidance Exists
Accessibility guidance is documented where appropriate for the product domain.
Principles for inclusive design are established.

**Audit Rule:** Accessibility guidance exists where appropriate.

### D6. Localization Guidance Exists
Localization guidance is documented where applicable. Principles for adapting
the product across languages and regions are established.

**Audit Rule:** Localization guidance exists where appropriate.

### D7. Documents Remain Modular
Design documentation is decomposed into focused documents. Each document describes
one design concern. Large documents are split rather than expanded.

**Audit Rule:** Documents remain modular.

### D8. Responsibilities Do Not Overlap
No two design documents describe the same principle or concern. Design document
scope is distinct and non-overlapping.

**Audit Rule:** Responsibilities do not overlap.

### D9. Cross-Repository Reuse Encouraged
Shared design documentation is referenced across repositories rather than
duplicated. Design principles are written for reuse.

**Audit Rule:** Cross-repository reuse is encouraged.

## Success Criteria

All checks D1–D9 pass. Principles are reusable. Technology independent. No
feature-specific content. Design philosophy defined. Documents modular,
non-overlapping. Cross-repo reuse encouraged.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/design/`
3. For each file, run checks D1–D9
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/design/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
