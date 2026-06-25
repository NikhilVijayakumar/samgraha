# Ownership Audit

## Context

Validates ownership of external dependencies documented in `docs/raw/external-context/`
against the **external-context.md** standard. External Context documents external
systems, libraries, and platforms that materially influence the repository.

## Authority

`docs/raw/standards/external-context.md` — Audit Rules section.

## Scope

All files under `docs/raw/external-context/`. Quality evaluated individually and
across the external context collection.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### O1. Dependencies Documented Only When Necessary
External dependencies are documented only when they materially influence the
repository. Widely understood libraries without project-specific conventions do
not require External Context documents.

**Audit Rule:** External dependencies are documented only when necessary.

### O2. One Document Per Dependency
Each External Context document describes exactly one external dependency.
Documents that cover multiple unrelated dependencies should be split.

**Audit Rule:** One document describes one dependency.

### O3. Dependency Purpose Clearly Explained
Why the dependency exists is documented. The purpose is specific to this
repository — not a generic description of the external project.

**Audit Rule:** Dependency purpose is clearly explained.

### O4. Constraints Documented
Constraints that the dependency introduces (API limits, platform requirements,
version compatibility, architectural constraints) are documented.

**Audit Rule:** Constraints are documented.

### O5. External Documentation Referenced, Not Duplicated
Authoritative external documentation is referenced. The External Context document
summarizes relevant knowledge — it does not copy the external project's
documentation.

**Audit Rule:** External documentation is referenced rather than duplicated.

### O6. Repository Relevance Obvious
Why this dependency matters to this repository is obvious from the document.
The relevance to Saṃgraha's architecture, features, or engineering is explicit.

**Audit Rule:** Repository relevance is obvious.

### O7. No Internal Architecture Leak
External Context does not describe internal repository architecture, internal
feature specifications, source code, or build configuration. These belong in
their respective documentation domains.

**Audit Rule:** No internal architecture has leaked into External Context.

## Success Criteria

All checks O1–O7 pass. Dependencies necessary and atomic. Purpose clear.
Constraints documented. External docs referenced, not duplicated. Relevance
obvious. No internal architecture leak.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/external-context/`
3. For each file, run checks O1–O7
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/ownership/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
