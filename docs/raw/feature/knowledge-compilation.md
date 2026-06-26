# Knowledge Compilation

## Purpose

Knowledge Compilation transforms authoritative engineering documentation into deterministic, structured knowledge artifacts.

The compiler is the foundation of the Saṃgraha platform.

It discovers repository documentation, validates supported sources, extracts engineering knowledge, builds structured representations, and produces the Knowledge Registry consumed by downstream platform components.

Compilation converts documentation into reusable engineering knowledge without changing the original documentation.

---

# Functional Requirements

## FR1. Documentation Discovery

The compiler shall discover documentation sources within a repository or workspace.

Discovery shall support:

* configured documentation locations
* repository conventions
* workspace repositories
* supported document types

Documentation discovery determines the compilation scope.

---

## FR2. Source Processing

The compiler shall process supported documentation formats.

Processing shall:

* validate document structure
* extract engineering content
* preserve document hierarchy
* preserve authored relationships
* preserve metadata

Source processing shall remain deterministic.

---

## FR3. Metadata Extraction

The compiler shall extract structured metadata.

Metadata may include:

* document identity
* documentation domain
* ownership
* status
* relationships
* references
* repository metadata

Metadata supports downstream platform services.

---

## FR4. Knowledge Extraction

The compiler shall transform documentation into structured knowledge artifacts.

Examples include:

* compiled documents
* semantic sections
* section metadata
* metadata
* references
* indexes
* dependency information
* future knowledge artifacts

Knowledge extraction preserves engineering intent.

Each compiled document shall contain semantic sections resolved from the applicable Documentation Standard. Sections shall carry their semantic type, canonical name, required flag, and content. Sections not matching any standard definition shall be preserved with semantic type `generic`.

---

## FR4a. Standard Resolution

The compiler shall resolve the applicable Documentation Standard for each document.

Standard Resolution shall:

* identify the document's documentation domain
* load the corresponding Standard definition
* match document headings against section definitions using canonical names and aliases
* assign semantic types to matched sections
* preserve unmatched sections as generic sections
* report missing required sections as compilation warnings

Standard Resolution executes after Source Processing and before Knowledge Extraction.

Standard Resolution is deterministic and requires no AI providers.

---

## FR5. Relationship Resolution

The compiler shall identify relationships within repository knowledge.

Examples include:

* document references
* feature mappings
* architecture relationships
* dependency relationships
* ownership relationships

Resolved relationships become part of compiled knowledge.

---

## FR6. Knowledge Generation

The compiler shall generate the Knowledge Registry.

Generated artifacts may include:

* compiled documentation
* indexes
* metadata
* dependency metadata
* registry metadata
* build metadata

Generation shall remain deterministic.

---

## FR7. Deterministic Compilation

Compilation shall produce identical knowledge artifacts from identical documentation.

Compilation shall not depend upon:

* runtime state
* network availability
* random values
* AI providers

Deterministic execution is mandatory.

---

## FR8. Extensible Compilation

The compiler shall support additional documentation processors.

Future source formats should integrate through compiler extensions without changing the compilation pipeline.

---

# Business Rules

* Documentation is the authoritative source of knowledge.
* Compilation never modifies documentation.
* Generated artifacts are disposable.
* Compilation is deterministic.
* Compilation operates offline.
* Compilation is reproducible.
* Unsupported documentation shall be reported.
* Compilation may fail without affecting repository documentation.

---

# Compilation Lifecycle

```text
Repository Documentation
        │
        ▼
Documentation Discovery
        │
        ▼
Source Processing
        │
        ▼
Standard Resolution          ← Documentation Standard + Section Definitions
        │
        ▼
Semantic Section Mapping     ← Headings → Semantic Types via aliases
        │
        ▼
Metadata Extraction
        │
        ▼
Knowledge Extraction         ← Produces Semantic Documents
        │
        ▼
Relationship Resolution
        │
        ▼
Knowledge Registry           ← Stores Documents + Semantic Sections
```

A Semantic Document is a document interpreted through its Documentation Standard. Rather than storing headings and text, a Semantic Document stores engineering semantics: Purpose, Functional Requirements, Business Rules, Constraints, Dependencies, and so on. This richer representation makes every downstream capability (audit, search, packaging, runtime) significantly more powerful without relying on AI.

---

# Inputs

Knowledge Compilation consumes:

* repository documentation
* repository configuration
* workspace configuration
* supported documentation sources

---

# Outputs

Knowledge Compilation produces:

* Knowledge Registry
* compiled knowledge
* semantic documents
* semantic sections with assigned types
* missing required section warnings
* metadata
* indexes
* dependency metadata
* build metadata

Outputs become the foundation for downstream platform capabilities.

---

# Constraints

Knowledge Compilation shall:

* support large repositories
* support workspaces
* operate deterministically
* remain offline
* preserve documentation
* support incremental builds
* recover through clean rebuilds
* remain extensible

Compilation technologies are implementation concerns.

---

# Dependencies

Knowledge Compilation depends upon:

* Repository Configuration
* Workspace Management

Knowledge Compilation provides compiled knowledge to:

* Knowledge Registry
* Audit Framework
* Knowledge Enrichment
* Knowledge Search
* Knowledge Resolution
* Incremental Build

---

# Non-Goals

Knowledge Compilation does not:

* execute audits
* generate enrichment
* resolve knowledge packages
* deliver knowledge to consumers
* modify documentation

Those responsibilities belong to their respective platform components.

---

# Future Extensions

The compilation framework should support future capabilities, including:

* additional documentation formats
* custom document processors
* repository plugins
* semantic preprocessing
* architecture extraction
* diagram extraction
* structured specification import
* compiler extensions

Future capabilities should integrate without changing the logical compilation model.

---

# Success Criteria

The feature is successful when:

* documentation is transformed into deterministic knowledge artifacts
* generated knowledge remains reproducible
* repository relationships are preserved
* downstream platform services consume compiled knowledge consistently
* incremental builds remain correct
* compilation remains extensible
* documentation continues to be the authoritative engineering source
* document headings are resolved to semantic types via standard section definitions
* missing required sections are reported as warnings
* unrecognized sections are preserved as generic sections rather than discarded
* semantic sections are stored in the registry and available to all downstream services

---

# Traceability

This feature derives from the following Vision commitments:

* **Documentation is the source of truth.**
* **Knowledge is compiled before delivery.**
* **Generated artifacts are disposable.**
* **Engineering knowledge is deterministic and reproducible.**
* **Compilation is the foundation of the knowledge pipeline.**

**Traceability**

Vision → Feature: Knowledge Compilation
