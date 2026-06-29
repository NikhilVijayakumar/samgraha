# Engineering Principles

## Purpose

This document defines the engineering principles that govern implementation of the Saṃgraha platform.

Engineering principles shape every engineering decision — technology selection, code organization, build strategy, testing approach, and operational practices. They provide a consistent foundation for implementation across all platform components.

---

## Technology Selection

The project is built using Rust as the primary implementation language with a multi-crate workspace structure. Key technology choices include SQLite (via rusqlite with bundled feature) for the knowledge registry, pulldown-cmark for Markdown parsing, Rayon for parallel processing, and Serde for serialization. See [Technology Selection](technology-selection.md) for the complete rationale behind each technology choice.

## Principles

### Documentation First

Implementation realizes documented engineering intent. Documentation is the authoritative specification. Code is derived. When documentation and implementation disagree, documentation prevails. Changes to behavior must be reflected in documentation before or alongside implementation.

### Architecture First

Architecture defines component boundaries and responsibilities before implementation begins. Implementation must respect architectural boundaries — components communicate only through defined interfaces, ownership is never bypassed, and architectural layering is preserved.

### Deterministic by Default

Platform behavior must be deterministic whenever possible. Identical inputs must produce identical outputs. Randomness, hidden state, and environment-dependent behavior are permitted only when explicitly justified and isolated. AI-assisted features must never compromise deterministic core behavior.

### Offline First

Core platform capabilities must operate without network connectivity. AI providers, remote services, and cloud infrastructure are optional enhancements. The platform must remain fully functional when disconnected. Network access is never required for compilation, audit, search, or runtime operation.

### Local First

Engineering knowledge is processed and stored locally. The platform operates on local files and local storage. Remote synchronization and distributed operation are future concerns — the local-first architecture must not be compromised for hypothetical remote scenarios.

### Minimal Dependencies

Every dependency carries maintenance cost, security risk, and compatibility burden. Dependencies must be justified by clear engineering value. Where reasonable, implement rather than import. Prefer standard library and well-established crates over niche alternatives. Audit every dependency for maintenance status, safety, and license compatibility.

### Explicit Configuration

Configuration is declarative and explicit. Implicit behavior, magic discovery, and convention-over-configuration are avoided. Every configurable behavior has a documented default. Configuration is validated before use. Invalid configuration produces clear error messages.

### Fail Fast

Errors are detected and reported as early as possible. Configuration errors fail at startup. Compilation errors fail with precise file and line information. Runtime errors fail with actionable messages. Silent failures, swallowed errors, and best-effort degradation are avoided unless explicitly required for optional features.

### Secure by Default

Security is a platform property, not a feature. Path traversal, repository isolation, and knowledge integrity are enforced at the architectural level rather than through application-level checks. Least privilege guides component design. Security-sensitive operations are validated at the boundary.

### Observable Systems

Platform behavior must be observable. Compilation, audit, search, and runtime operations produce structured output including progress, errors, warnings, and results. Performance metrics and diagnostic information are accessible without requiring specialized tooling.

### Progressive Enhancement

Capabilities are organized in layers. Core functionality is deterministic and offline. Optional enhancements (AI enrichment, semantic audit, advanced search) build on the core without modifying it. Each enhancement level is independently configurable and independently testable.

### Repository Isolation

Engineering knowledge is owned by repositories. The platform enforces repository boundaries. Cross-repository knowledge sharing occurs only through explicitly declared dependencies. Repository isolation applies at every architectural layer — compilation, persistence, runtime, and delivery.

---

## Decision Framework

Every engineering decision should be evaluated against these principles:

1. Does this preserve determinism?
2. Does this work offline?
3. Does this respect architectural boundaries?
4. Does this add unnecessary dependencies?
5. Does this make behavior less explicit?
6. Does this compromise security?
7. Does this reduce observability?

If the answer to any of questions 3–6 is yes, the decision requires explicit justification and review.

---

## Traceability

This document derives from:

- Architecture: System Overview
- Architecture: Component Model
- Architecture: Security Architecture
- Vision: Product Philosophy

Engineering Principles provide the foundation for:

- Technology Selection
- Repository Structure
- Build Standards
- Testing Standards
- All Engineering Documentation

## Build Standards

Engineering principles inform the build system: deterministic by default, offline first, minimal dependencies. Build profiles are explicit and configurable. See [Build Standards](build-standards.md) for detailed build system specification.

## Testing Standards

Testing follows from engineering principles: fail fast, secure by default, observable systems. Tests validate deterministic behavior across environments. See [Testing Standards](testing-standards.md) for testing approach and framework.

Traceability:

```
Architecture → Engineering Principles → Engineering Documentation → Implementation
```
