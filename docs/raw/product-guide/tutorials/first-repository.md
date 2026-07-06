# Tutorial: First Repository

## Purpose

Step-by-step: initialize a repository, write a document, compile, and search.

## Content

### Step 1: Initialize

```bash
cd my-project
samgraha init
```

Creates `.samgraha/` and default configuration.

### Step 2: Write a Document

Create `docs/raw/feature/my-feature.md`:

```markdown
# My Feature

## Purpose
This feature does something useful.

## Functional Requirements
- FR1: The system shall do X.
- FR2: The system shall do Y.

## Acceptance Criteria
- Both FRs pass testing.
- Performance meets threshold.
```

### Step 3: Compile

```bash
samgraha compile
```

### Step 4: Search

```bash
samgraha search "useful"
```

### What You Learned

- How to init a Samgraha repository
- How to write a feature document
- How to compile and search

## Related

- [First Audit](first-audit.md)
- [First MCP](first-mcp.md)
- [Concepts: Repository](../concepts/repository.md)
