# Architecture Audit

## Context

Validates architecture docs in `docs/raw/architecture/` against the **architecture.md**
standard. Architecture documents describe system organization, not implementation.

## Authority

`docs/raw/standards/architecture.md` — Audit Rules section.

## Scope

All files under `docs/raw/architecture/`. Quality evaluated across the complete
architecture collection, not individual files.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### A1. Modular Architecture
Architecture documentation is a collection of focused documents. Each document
describes one architectural concern. Large documents should be decomposed.

**Audit Rule:** Architecture is modular.

### A2. Responsibilities Separated
Architecture documents have clearly separated responsibilities. No two documents
describe the same concern. No overlapping scope between documents.

**Audit Rule:** Responsibilities are clearly separated.

### A3. Ownership Explicit
Every component, boundary, and subsystem has a documented owner. Ownership
defines who maintains each architectural element.

**Audit Rule:** Ownership is explicit.

### A4. Boundaries Documented
Component boundaries, runtime boundaries, and system boundaries are explicitly
documented. Implicit boundaries should not exist.

**Audit Rule:** Boundaries are documented.

### A5. Communication Paths Understandable
Data flow, IPC, event flow, and API communication paths between components are
documented and understandable without reading source code.

**Audit Rule:** Communication paths are understandable.

### A6. No Duplication
Architecture documents do not duplicate one another. If two documents describe
the same concept, one should reference the other rather than redefine it.

**Audit Rule:** Documents do not duplicate one another.

### A7. Architecture Aligns with Features
Every architectural decision supports one or more documented features. Architecture
traces to `docs/raw/feature/` via explicit references.

**Audit Rule:** Architecture aligns with Features.

### A8. No Implementation Detail
Architecture describes responsibilities, boundaries, and organization — not source
code, algorithms, function signatures, or implementation patterns. Technology
references only when architecturally significant.

**Audit Rule:** Architecture avoids implementation detail.

### A9. Cross-Repository References
When depending on another repository, architecture references external documentation
rather than duplicating it. Cross-repository architecture uses references.

**Audit Rule:** Cross-repository references are used instead of duplication.

## Success Criteria

All checks A1–A9 pass across the complete architecture collection. Architecture is
modular, responsibilities non-overlapping, communication paths clear, no
implementation detail present.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. List all files under `docs/raw/architecture/`
3. For each file, run checks A1–A9
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/architecture/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
