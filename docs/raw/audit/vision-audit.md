# Vision Audit

## Context

Validates vision docs in `docs/raw/vision/` against the **vision.md** standard.
The Vision document establishes the long-term purpose, direction, and identity
of the product. It defines why the product exists.

## Authority

`docs/raw/standards/vision.md` — Audit Rules section.

## Scope

All files under `docs/raw/vision/`. Quality evaluated per document.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### V1. Explains Why Product Exists
The Vision clearly explains why the product exists. The problem it solves and
the purpose it serves are immediately understandable.

**Audit Rule:** The Vision explains why the product exists.

### V2. Technology Independent
No programming languages, frameworks, libraries, databases, infrastructure,
build systems, or cloud providers appear. Vision describes purpose, not
technology.

**Audit Rule:** The document is technology independent.

### V3. No Implementation Details
No implementation details, architectural decisions, feature specifications,
UI layouts, or source code appear. Implementation belongs downstream.

**Audit Rule:** No implementation details appear.

### V4. Product Philosophy Present
Product philosophy is documented. The philosophy communicates values that
guide product decisions (e.g., Documentation First, Local First, Offline First).

**Audit Rule:** Product philosophy is present.

### V5. Guiding Principles Documented
Enduring principles that influence future decisions are documented. Principles
remain stable even as features evolve.

**Audit Rule:** Guiding principles are documented.

### V6. Downstream Documentation Consistent
Feature docs, architecture, and engineering decisions do not contradict the
Vision. The Vision is the root of the documentation hierarchy.

**Audit Rule:** Downstream documentation remains consistent with the Vision.

### V7. Stable and Future-Oriented
The Vision describes long-term direction, not short-term goals. It remains
stable throughout the product lifecycle and is future-oriented.

**Audit Rule:** The Vision remains stable and future-oriented.

## Success Criteria

All checks V1–V7 pass. Vision explains product purpose clearly. Technology
independent. No implementation details. Philosophy and principles documented.
Downstream docs consistent. Stable and future-oriented.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/vision/`
3. For each file, run checks V1–V7
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/vision/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
