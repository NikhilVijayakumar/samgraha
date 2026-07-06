# Multi-Repo Overview

## Purpose

Why and when to use multiple Samgraha repositories together.

## Content

### Why Multi-Repo

Most real-world products span multiple Git repositories. Samgraha supports this through:

- **Dependencies** — Repo A depends on Repo B's documentation
- **Interests** — Repo A is interested in Repo C's documentation (but doesn't depend on it)
- **Workspaces** — Group repos together for coordinated search and status

### When to Use Each

| Scenario | Approach |
|----------|----------|
| Shared library used by multiple consumers | Library declares exports, consumers declare dependency |
| Service that depends on another service's API | Consumer declares dependency on producer |
| Related but independent projects | Workspace membership without hard dependencies |
| Team A wants visibility into Team B's docs | Interest declaration |

### Knowledge Flow

```
Repo A (consumer)
  ├── depends on → Repo B (library)
  │                 └── provides: feature designs, architecture
  ├── interested in → Repo C (sibling service)
  │                    └── provides: API contracts
  └── workspace member → Repo D, Repo E
                          └── shared search context
```

## Related

- [Dependencies](dependencies.md)
- [Interests](interests.md)
- [Workspace](workspace.md)
