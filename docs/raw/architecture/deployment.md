# Deployment Architecture

## Purpose

This document defines the deployment architecture of the Saṃgraha platform.

Deployment Architecture describes how the logical components of Saṃgraha are assembled into executable runtime environments and made available to development tools.

Rather than describing installation procedures or deployment technologies, this document defines the architectural deployment model of the platform.

Implementation-specific deployment strategies are documented separately.

---

# Deployment Philosophy

Saṃgraha is designed as a local-first Knowledge Engineering Platform.

The platform should execute as close to the engineering workflow as possible.

Deployment should:

* preserve deterministic behavior,
* minimize runtime dependencies,
* support offline operation,
* remain repository-aware,
* expose consistent Knowledge Services regardless of deployment mechanism.

Deployment should never change platform behavior.

Only the execution environment changes.

---

# Deployment Model

The platform consists of one logical runtime that may be exposed through multiple deployment interfaces.

```text
                    Saṃgraha Platform
                           │
      ┌────────────────────┼────────────────────┐
      ▼                    ▼                    ▼
 Knowledge Runtime      CLI Runtime       Future Runtime
      │                    │                    │
      ├──────────────┬─────┴──────────────┬─────┤
      ▼              ▼                    ▼
   MCP Adapter   CLI Adapter      Future Adapters
      │              │                    │
      ▼              ▼                    ▼
Development Tools  Terminal        IDE / Automation
```

Every deployment hosts the same platform.

Only the transport interface differs.

---

# Deployment Components

## Core Platform

The Core Platform contains:

* Documentation Standards
* Knowledge Services
* Knowledge Compiler
* Knowledge Registry
* Knowledge Runtime

These components exist in every deployment.

---

## Transport Adapters

Transport adapters expose platform capabilities.

Examples include:

* CLI
* MCP
* Future REST APIs
* IDE integrations
* Automation interfaces

Transport adapters remain interchangeable.

---

## Provider Integrations

Optional provider integrations may be deployed alongside the platform.

Examples include:

* Local AI providers
* Remote AI providers
* Embedding providers
* Future enrichment providers

Provider availability should never affect deterministic platform capabilities.

---

# Supported Deployment Modes

## Standalone CLI

The platform executes a single operation and terminates.

Typical use cases include:

* compilation
* auditing
* validation
* generation
* repository analysis

No persistent runtime is required.

---

## Interactive Runtime

The platform remains active while serving runtime requests.

Typical use cases include:

* MCP integration
* IDE integrations
* long-running automation
* continuous repository assistance

The runtime operates entirely on compiled engineering knowledge.

---

## Automation

Saṃgraha may execute within automated workflows.

Examples include:

* continuous integration
* documentation validation
* repository quality gates
* knowledge package generation

Automation uses the same Knowledge Services as interactive deployments.

---

# Workspace Deployment

The platform supports multiple repository scopes.

```text
Knowledge Registry
        │
        ├── Repository A
        ├── Repository B
        ├── Repository C
        │
        ▼
Workspace Context
        │
        ▼
Knowledge Runtime
```

Deployment should preserve repository isolation.

Shared knowledge should originate only from explicitly declared dependencies.

---

# Runtime Independence

Deployment does not change architectural responsibilities.

Regardless of deployment mode:

* Documentation Standards remain authoritative.
* Knowledge Services remain deterministic.
* Knowledge Registry remains persistent.
* Knowledge Runtime remains the execution boundary.

Only the transport interface changes.

---

# Deployment Principles

## Local First

The platform should execute locally whenever possible.

---

## Offline First

Core platform capabilities should operate without network connectivity.

---

## Repository Aware

Deployments should preserve repository identity and dependency relationships.

---

## Interface Independence

Multiple interfaces should expose identical platform behavior.

No deployment should introduce platform-specific engineering logic.

---

## Deterministic Execution

The same deployment should produce identical results given identical engineering knowledge.

---

## Disposable Runtime

Runtime processes are transient.

Persistent engineering knowledge remains within the Knowledge Registry.

---

# Scalability

The deployment architecture supports future expansion.

Possible future deployment targets include:

* IDE extensions
* Desktop applications
* Remote Knowledge Runtime
* Team knowledge servers
* Cloud-hosted Knowledge Services
* Repository automation platforms

The logical platform architecture remains unchanged.

Only deployment adapters evolve.

---

# Technology Independence

This document intentionally avoids deployment technologies.

Executable packaging, installers, operating system integration, service managers, containers, CI/CD pipelines, cloud infrastructure, and binary distribution belong to Engineering Documentation.

Deployment Architecture defines only the logical deployment model.

---

# Traceability

This document derives from:

* Vision
* Documentation Philosophy
* System Overview
* Runtime Boundary
* Communication
* Component Model

This document provides architectural context for:

* Engineering Deployment Strategy
* Build Strategy
* Release Strategy
* Distribution Strategy

Supporting features include:

* Knowledge Runtime
* CLI Interface
* Workspace Support
* Knowledge Registry
* Knowledge Services

Traceability:

```text
Vision
    ↓
Documentation Philosophy
    ↓
System Overview
    ↓
Deployment Architecture
    ↓
Engineering
    ↓
Implementation
```
