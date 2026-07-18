# Knowledge Systems

## Purpose

A knowledge system is an external package that provides domain-specific validation, calculation, reporting, scaffolding, and plan generation logic to samgraha. Instead of hardcoding every audit check in Rust, samgraha delegates to system-provided scripts via capability dispatch.

## What Is a Knowledge System?

A knowledge system is a directory containing:

- **`system.yaml`** — declares the system's identity, capabilities, and script locations
- **Scripts** — executable files (Python, Node, shell, etc.) that implement one or more capabilities
- **`documentation-standards/`** — optional, defines audit rules for the system's domains
- **`templates/`** — optional, scaffold templates for documentation generation

## How Systems Work

1. A knowledge system is registered with samgraha via `standards register`
2. Samgraha discovers the system's scripts via `system.yaml`
3. When an audit or fix operation targets a domain the system owns, samgraha invokes the system's `validate` script instead of running built-in checks
4. The script receives the target document and check IDs, runs its logic, and returns structured findings

## System Priority

Multiple systems may provide scripts for the same domain. Priority order:

1. **User-registered system** (highest priority)
2. **Standard-bundled scripts** (documentation standard's own scripts)
3. **Override scripts** (user-provided in repository)
4. **Built-in Rust logic** (fallback, always available)

## System.yaml Structure

```yaml
id: kriti
name: "Kriti Knowledge System"
version: "1.0.0"
description: "Validation and audit for Kriti documentation"

capabilities:
  validate:
    script: scripts/validate.py
    async: false
  calculate:
    script: scripts/calculate.py
    async: false
  report:
    script: scripts/report.py
    async: false

domains:
  - feature
  - architecture
  - engineering

init:
  use_cases:
    - id: new-project
      phases:
        - name: "Generate Documentation"
          step: "Generate initial documentation from templates"
        - name: "Validate"
          step: "Run validation against generated docs"
```

## Related

- [Capability Scripts](scripts.md)
- [System Inheritance](inheritance.md)
- [Workflows](workflows.md)
- [Knowledge System Author Guide](../../../knowledge-system-author-guide.md)
