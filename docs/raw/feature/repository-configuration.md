# Repository Configuration

This section details the Repository Configuration.

## Purpose

Repository Configuration defines how a repository participates in the Saṃgraha platform.

It provides the configuration required to discover documentation, apply engineering standards, coordinate builds, manage knowledge, and integrate with workspace services.

Repository Configuration establishes repository identity and behavior without affecting repository documentation.

Configuration is declarative and deterministic.

---

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Repository Identity

Every participating repository shall define its identity.

Repository identity consists of:

* `uuid` — globally unique identifier, never changes
* `id` — short identifier, may change on rename
* `name` — human-readable display name, may change
* description
* version
* ownership
* metadata

The UUID is generated once at `samgraha init` and committed to version control. It is never overwritten. The UUID is the authoritative identity key in the Repository Registry.

Identity uniquely identifies engineering knowledge. Renaming a repository changes the ID and name but preserves the UUID, allowing the Repository Registry to maintain continuity.

---

## FR2. Documentation Configuration

Repository Configuration shall define documentation sources.

Configuration may specify:

* documentation locations
* documentation domains
* documentation exclusions
* supported document types
* documentation discovery rules

Documentation discovery determines compilation scope.

---

## FR3. Standards Configuration

Repository Configuration shall identify the Documentation Standards applied by the repository.

Configuration may specify:

* standard versions
* organization extensions
* custom documentation domains
* repository-specific overrides

Documentation Standards remain authoritative.

---

## FR4. Build Configuration

Repository Configuration shall define build behavior.

Configuration may include:

* build profiles
* output locations
* incremental build settings
* artifact generation
* package generation

Configuration influences build behavior without changing platform capabilities.

---

## FR5. Workspace Participation

Repository Configuration shall define workspace participation.

Configuration may identify:

* workspace membership
* shared configuration
* repository overrides
* repository visibility
* repository relationships

Workspace configuration coordinates multiple repositories.

---

## FR6. Dependency Configuration

Repository Configuration shall define repository relationships.

Configuration may specify:

* required repositories
* optional repositories
* external dependencies
* dependency policies

Dependencies define engineering relationships rather than implementation details.

---

## FR7. Platform Configuration

Repository Configuration shall control platform capabilities.

Configuration may include:

* audit policies
* enrichment policies
* runtime policies
* search configuration
* package generation
* quality gates

Platform capabilities remain independently implemented.

---

## FR8. Configuration Validation

Repository Configuration shall be validated before use.

Validation shall verify:

* structural completeness
* configuration consistency
* dependency correctness
* standards compatibility
* workspace compatibility

Invalid configuration shall be reported.

---

## FR9. Resolver Configuration

Repository Configuration shall define resolver behavior.

Configuration may specify:

* `metadata_cache` — enable local metadata caching (default: true)
* `metadata_ttl` — cache expiration duration (default: "24h")
* `auto_refresh` — automatically refresh expired cache from local Registry (default: true)
* `registry_type` — local file or remote HTTP (default: "file")
* `registry_url` — remote registry endpoint (only when registry_type is "http")

Resolver configuration controls how the Knowledge Resolver locates dependencies without contacting the Repository Registry at runtime.

---

## Business Rules

* Configuration is declarative.
* Configuration is deterministic.
* Configuration never modifies documentation.
* Repository configuration remains independent.
* Workspace configuration provides shared defaults.
* Repository configuration may override shared defaults where permitted.
* Configuration changes shall be reproducible.
* Repository UUID is generated once at `samgraha init` and never overwritten.
* Compilation generates UUID if absent, warns user, never blocks.

---

## Configuration Lifecycle

```text
Repository Configuration
          │
          ├──────────────┐
          │              │
          ▼              ▼
Workspace       Documentation Standards
Configuration
          │
          ▼
Platform Services
          │
          ├── Knowledge Compilation
          ├── Incremental Build
          ├── Audit Framework
          ├── Knowledge Enrichment
          ├── Knowledge Registry
          ├── Repository Registry
          ├── Knowledge Resolution
          ├── Knowledge Runtime
          └── Engineering CLI
```

---

## Inputs

Repository Configuration consumes:

* repository configuration
* workspace configuration
* organization configuration
* platform defaults

---

## Outputs

Repository Configuration provides:

* repository metadata
* documentation configuration
* standards configuration
* build configuration
* dependency configuration
* platform configuration

Outputs configure platform behavior.

---

## Constraints

Repository Configuration shall:

* remain deterministic
* support repository independence
* support workspace inheritance
* support validation
* support future extensions
* remain implementation independent

Configuration storage formats are implementation concerns.

---

## Dependencies

Repository Configuration depends upon:

* Workspace Management
* Documentation Standards

Repository Configuration provides configuration to:

* Knowledge Compilation
* Incremental Build
* Audit Framework
* Knowledge Enrichment
* Knowledge Registry
* Repository Registry
* Knowledge Resolution
* Knowledge Package
* Knowledge Runtime
* Engineering CLI

---

## Non-Goals

Repository Configuration does not:

* compile documentation
* execute audits
* generate knowledge
* deliver knowledge
* modify repository content

Those responsibilities belong to their respective platform capabilities.

---

## Future Extensions

The Repository Configuration framework should support future capabilities, including:

* configuration inheritance
* environment profiles
* organization policies
* remote configuration providers
* remote registry configuration (registry_url for HTTP Registry)
* encrypted configuration values
* configuration templates
* configuration migration
* policy validation
* configuration composition

Future capabilities should integrate without changing the logical configuration model.

---

## Acceptance Criteria

The feature is successful when:

* repositories configure platform behavior consistently
* workspace participation is deterministic
* documentation discovery is correctly configured
* platform capabilities receive valid configuration
* repository independence is preserved
* configuration remains reusable and extensible
* configuration can evolve without changing platform architecture

---

## Traceability

This feature derives from the following Vision commitments:

* **Documentation is the source of truth.**
* **Configuration defines behavior, not knowledge.**
* **Engineering processes are deterministic and reproducible.**
* **Repositories remain independently configurable while participating in shared workspaces.**
* **Platform capabilities are configured rather than hardcoded.**

**Traceability**

Vision → Feature: Repository Configuration
