# Declaring Interests

## Purpose

How to declare interest in another repository's documentation without a hard dependency.

## Content

### What Are Interests

Interests are soft dependencies. They make another repo's documentation available for search (within an MCP session) without ever being required.

### samgraha.toml Declaration

Interests are declared the same way as dependencies — by name, in the consumer repo's own `samgraha.toml`:

```toml
[knowledge]
interests = ["analytics-service"]
```

Resolution follows the same path (config `[[repository.dependencies]]` entry, or a cached `.samgraha/dependencies/<name>.meta`) as [Dependencies](dependencies.md), but an unresolved interest is never an error — its status is simply `Unresolved`/`Missing` and it's dropped from the plan.

### Interests vs Dependencies

| Aspect | Dependency | Interest |
|--------|------------|----------|
| Required | Always (hard) | Never (soft) |
| Loaded | Eagerly, at session start | Lazily, on first query that needs a store beyond primary+deps |
| Search inclusion | Within an MCP session only | Within an MCP session only |
| Failure behavior | `RequiredMissing` — error state | `Unresolved`/`Missing` — no error |
| Use case | Shared library | Optional visibility |

### Use Cases

- A service wants visibility into another service's API contracts.
- A frontend app wants to see backend feature specs.
- Cross-team awareness without formal dependency.

## Related

- [Dependencies](dependencies.md)
- [Multi-Repo Overview](overview.md)
- [Concepts: Planner](../concepts/planner.md)
