# Documentation Structure Audit

This section details the Documentation Structure Audit.

## Purpose

Validates the documentation corpus as an integrated system, not isolated domains: structural integrity, one-to-one mapping between related domains, feature atomicity, cross-document alignment, name preservation across compilation layers, implementation traceability, and generation compliance.

See `docs/proposal.md` for the full design rationale, including which checks here delegate to an existing pipeline's finding rather than reimplementing it (SI5, MC5, AE4, CA2, CA3).

---

# Authority

Audit rules are defined by the validation checks in this document (SI1–SI7, MC1–MC8, AE1–AE6, CA1–CA8, NP1–NP6, IT1–IT5, GC1–GC5).

---

# Scope

Applies to the whole `docs/raw/` tree, `README.md`, and the declared implementation directory (`repository.implementation.dir`).

---

# Structural Integrity

This section details the Structural Integrity checks.

## SI1. Required Domains Present

`feature/`, `engineering/`, and `vision/` must exist under `docs/raw/`. These are non-negotiable — every repository documents what it does, how it is engineered, and why it exists.

---

## SI2. Excluded Domains Absent Or Legitimate

A domain listed in `domain_exclusion` should not still contain this repository's own documents, unless it's a reserved domain (`help`, `standards`) holding legitimate built-in content.

---

## SI3. Reserved Domains Guarded

`help` and `standards` are reserved for built-in knowledge. If either appears in `domain` it must also appear in `domain_exclusion`, or the repository would try to compile its own docs under a domain name that collides with built-in content.

---

## SI4. No Unexpected Directories

Every directory under `docs/raw/` must be accounted for by `domain`, `domain_exclusion`, `[repository.ignore].patterns`, or a known directory-name override (e.g. `product-guide` compiling to domain `help`). Directories that exist in this repository today but match none of those (`audit/`, `fix-plan-templates/`, `report-templates/`) are reported as a Suggestion, not a Warning — see G3 in `docs/proposal.md`.

---

## SI5. README Exists

Delegated to Readme R1 — see `docs/raw/audit/readme-audit.md`.

---

## SI6. Vision Is Singular

The Vision directory should contain exactly one document — Vision is the project's single statement of purpose, not a collection.

---

## SI7. Feature Documents Are Atomic

Each Feature document should have exactly one top-level heading. Zero means the document has no clear subject; more than one means it describes more than one capability.

---

# Mapping Consistency

This section details the Mapping Consistency checks.

## MC1. Feature → Feature Technical Mapping

When both `feature/` and `feature-technical/` are present, every `feature/X.md` should have a `feature-technical/X.md`.

---

## MC2. Feature Technical → Feature Mapping

The reverse of MC1 — every `feature-technical/X.md` should have a `feature/X.md`.

---

## MC3. Feature → Feature Design Mapping

When both `feature/` and `feature-design/` are present, every `feature/X.md` should have a `feature-design/X.md`.

---

## MC4. Feature Design → Feature Mapping

The reverse of MC3.

---

## MC5. No Duplicate Feature Titles

Delegated to Feature F1 — see `docs/raw/audit/feature-audit.md`.

---

## MC6. No Duplicate Feature Technical Titles

Each Feature Technical document must have a unique title, mirroring MC5 for the `feature-technical/` collection.

---

## MC7. Feature Technical Titles Reference Parent Feature

A Feature Technical document's title should contain a word from its matching Feature's title, so the mapping is visible from the title alone, not just the filename.

---

## MC8. Feature Design Titles Reference Parent Feature

The same requirement as MC7, applied to Feature Design documents.

---

# Atomicity Enforcement

This section details the Atomicity Enforcement checks.

## AE1. One Capability Per Feature

Each Feature document should have exactly one top-level heading (same heuristic as SI7, scored under this category as well since atomicity is this category's whole purpose).

---

## AE2. Single Responsibility

A Feature title containing "and" (e.g. "Export and Import") suggests two responsibilities bundled into one document. Split it.

---

## AE3. No Unrelated Capability Combination

A Feature document referencing more than two sibling features by name has likely absorbed their scope rather than staying independently understandable.

---

## AE4. Explicit Boundaries

Delegated to Feature F14 — see `docs/raw/audit/feature-audit.md`.

---

## AE5. Feature Technical Maps To Single Feature

A Feature Technical document's title should not mention more than one Feature's name — that signals it covers multiple features rather than mapping 1:1.

---

## AE6. Feature Design Maps To Single Feature

The same requirement as AE5, applied to Feature Design documents.

---

# Cross-Document Alignment

This section details the Cross-Document Alignment checks.

## CA1. README Summarizes Vision Purpose

The README should share a majority of the Vision document's significant title words — evidence that the README's stated purpose actually reflects Vision, not just links to it.

---

## CA2. README References Present Domains

Delegated to Readme R4 — see `docs/raw/audit/readme-audit.md`.

---

## CA3. Feature Traces To Vision

Delegated to Feature F9 — see `docs/raw/audit/feature-audit.md`.

---

## CA4. Feature Technical Applies Architecture

When both `feature-technical/` and `architecture/` are present, Feature Technical documents should reference Architecture.

---

## CA5. Feature Technical Respects Engineering

When both `feature-technical/` and `engineering/` are present, Feature Technical documents should reference Engineering standards.

---

## CA6. Feature Design Applies Design Principles

When both `feature-design/` and `design/` are present, Feature Design documents should reference Design principles.

---

## CA7. No Contradictions Between Adjacent Layers

A fixed vocabulary of architecture/technology keywords is checked for negation in one layer (Vision, Architecture, Engineering) while affirmed unnegated in another — e.g. Vision says "avoid microservices" while Architecture assumes them. Narrow by design: heading/keyword scanning, not semantic analysis.

---

## CA8. Constraint Propagation From Vision

If Vision has a Constraints section, at least one of Architecture or Engineering should visibly reference "constraint" — constraints that never propagate downward are not enforced anywhere.

---

# Name Preservation

This section details the Name Preservation checks.

## NP1. Feature ↔ Feature Technical Stems Overlap

At least some `feature/` file stems should match `feature-technical/` file stems when both collections are non-empty.

---

## NP2. Feature ↔ Feature Design Stems Overlap

The same requirement as NP1, applied to `feature-design/`.

---

## NP3. Feature Technical Titles Derived From Feature Titles

For a matching stem, the Feature Technical title should contain a word from the Feature's title.

---

## NP4. Feature Design Titles Derived From Feature Titles

The same requirement as NP3, applied to Feature Design.

---

## NP5. Cross-References Resolve

A markdown link from a Feature document into `feature-technical/` or `feature-design/` should point at a stem that actually exists in that collection.

---

## NP6. Compiled Knowledge Preserves Stems

Not checkable from within a pipeline today — `Pipeline::run` receives only filesystem and config, no registry/DB handle. Verify via `samgraha compile` output directly until a DB handle is threaded through.

---

# Implementation Traceability

This section details the Implementation Traceability checks.

## IT1. Feature Has Corresponding Code

Each Feature's stem should share a token with a workspace crate name or a source file stem under the declared implementation directory.

---

## IT2. Feature Technical Has Corresponding Code

The same requirement as IT1, applied to `feature-technical/` stems.

---

## IT3. No Orphan Source Crates

Every workspace crate should be mentioned (by name or token) in at least one Feature or Feature Technical document.

---

## IT4. Feature Responsibilities Realized In Code

Backtick-quoted identifiers named in Feature documents should resolve to a real crate name or source file stem.

---

## IT5. Architecture Components Exist As Modules

The same requirement as IT4, applied to Architecture documents.

---

# Generation Compliance

This section details the Generation Compliance checks.

## GC1. Generated Stubs Have Correct Section Structure

A document containing a `TODO` marker (a generation stub) should still have a Purpose/Overview section.

---

## GC2. Generated Stubs Reference Audit Criteria

A stub document should contain a recognizable check-id pattern (e.g. `F1`, `SI2`) so it's traceable to the finding that produced it.

---

## GC3. Generated Content Follows Atomicity Rules

Stub Feature documents are held to the same one-H1-per-document rule as AE1/SI7.

---

## GC4. Generated Cross-References Are Valid

Markdown links inside a stub document should resolve to files that actually exist.

---

## GC5. Generated Documentation Passes Domain Audit

If stub markers exist in `feature/` or `feature-technical/`, that domain's own pipeline score should be at least 70 — otherwise generation hasn't reached an audit-passing state yet.
