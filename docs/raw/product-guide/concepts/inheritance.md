# System Inheritance

## Purpose

System inheritance allows knowledge systems to extend and override base configurations. A system can inherit from another system, the documentation standard, or built-in defaults, and selectively override specific capabilities.

## Inheritance Model

```
system.yaml
    │
    ├── Inherits from: base system (optional)
    │     └── Capabilities, domains, templates
    │
    ├── Overrides: specific capabilities
    │     └── validate: scripts/validate-v2.py
    │
    └── Extends: additional domains
          └── domains: [custom-domain-a, custom-domain-b]
```

## How Inheritance Works

1. A system declares `extends: <base-system-id>` in `system.yaml`
2. Samgraha loads the base system's configuration
3. The child system's configuration is merged on top
4. Child capabilities override parent capabilities of the same name
5. Child domains are appended to parent domains
6. Child templates override parent templates of the same name

## Inheritance Rules

- **Capabilities**: child replaces parent entirely for the same capability name
- **Domains**: child domains are merged with parent domains (union)
- **Templates**: child replaces parent template if names match
- **Init plans**: child init plans override parent init plans of the same use_case id
- **Metadata**: child metadata (name, version, description) replaces parent metadata

## Example

```yaml
# Base system
id: kriti-base
capabilities:
  validate:
    script: base/validate.py
  report:
    script: base/report.py

---
# Child system (extends kriti-base)
id: kriti-custom
extends: kriti-base
capabilities:
  validate:
    script: custom/validate.py  # overrides base validate
  # report inherited from base
domains:
  - feature
  - custom-domain  # added to base domains
```

## Related

- [Knowledge Systems](systems.md)
- [Capability Scripts](scripts.md)
- [Knowledge System Author Guide](../../../knowledge-system-author-guide.md)
