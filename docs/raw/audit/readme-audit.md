# README Audit

## Context

Validates the project README (`E:\Python\samgraha\README.md`) against the **readme.md**
standard. The README is the primary entry point to the repository.

## Authority

`docs/raw/standards/readme.md` — Audit Rules section.

## Scope

Root `E:\Python\samgraha\README.md` and any nested README files under `docs/raw/`.

## Validation Checklist

Each check maps to one item in the standard's Audit Rules.

### R1. Purpose Immediately Understandable
A reader can determine the project purpose within the first few paragraphs. The
README clearly states what the repository contains and why it exists.

**Audit Rule:** The project purpose is immediately understandable.

### R2. Responsibilities Explained
Repository responsibilities are clearly explained. What role the repository plays,
what capabilities it provides, who should use it.

**Audit Rule:** Repository responsibilities are clearly explained.

### R3. Documentation Navigation Exists
The README links to detailed documentation: Vision, Features, Architecture,
Engineering, External Context. It serves as a documentation index.

**Audit Rule:** Documentation navigation exists.

### R4. Installation Guidance Appropriate
Installation instructions exist and are appropriate for the project. Prerequisites,
build steps, and setup commands are documented.

**Audit Rule:** Installation guidance is appropriate.

### R5. Repository Structure Described
Major directories are described with their purpose. High-level structure helps
readers understand where to find specific documentation or code.

**Audit Rule:** Repository structure is described.

### R6. No Duplication of Detailed Documentation
The README summarizes content rather than reproducing detailed documentation.
Feature specs, architecture, engineering decisions belong in their respective
documents, not in the README.

**Audit Rule:** README does not duplicate detailed documentation.

### R7. Ecosystem Relationships Explained
If the repository belongs to a larger ecosystem, upstream dependencies, downstream
consumers, and related repositories are explained.

**Audit Rule:** Ecosystem relationships are explained when applicable.

### R8. Links Accurate
All links to documentation and external resources resolve correctly. No dead or
redirecting links.

**Audit Rule:** Links to documentation remain accurate.

### R9. No Bloat
The README remains concise. It does not become a project wiki. README bloat
(long irrelevant sections, duplicated content) is reported.

**Audit Rule:** README bloat should be reported as a standards violation.

## Success Criteria

All checks R1–R9 pass. Project purpose clear. Documentation navigation present.
No duplication of detailed docs. Links accurate. No bloat.

## Procedure

1. Rotate previous report per `docs/raw/audit/README.md#report-rotation`
2. Read root `README.md` and any nested READMEs under `docs/raw/`
3. Run checks R1–R9
4. Collect failures — each must specify violated check and exact location
5. Write report to `docs/raw/audit/reports/readme/latest/` following Standard Report Format in `docs/raw/audit/README.md#standard-report-format`
