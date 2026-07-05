# Tutorial: Multi-Repository Setup

## Purpose

Step-by-step: set up two repositories as a workspace for cross-repo search.

## Content

### Step 1: Initialize Two Repos

```bash
cd ~/projects/core-lib
samgraha init

cd ~/projects/api-service
samgraha init
```

### Step 2: Write Docs in Both

In `core-lib`, create `docs/raw/feature/string-utils.md`:

```markdown
# String Utilities

## Purpose
Provide string manipulation functions.

## Functional Requirements
- FR1: Provide uppercase conversion.
- FR2: Provide string trimming.

## Acceptance Criteria
- All functions have unit tests.
```

In `api-service`, create `docs/raw/feature/user-api.md`:

```markdown
# User API

## Purpose
Provide REST API for user management.

## Functional Requirements
- FR1: GET /users returns user list.
- FR2: POST /users creates a user.

## Acceptance Criteria
- All endpoints have integration tests.
```

### Step 3: Create Workspace

In `api-service/`, create `samgraha-workspace.toml` (no leading dot):

```toml
repositories = [
    "../core-lib",
]
```

Or generate it with `samgraha workspace init <name> ../core-lib`.

### Step 4: Compile Both

```bash
cd ~/projects/core-lib
samgraha compile

cd ~/projects/api-service
samgraha compile
```

### Step 5: Cross-Repo Search

Plain `samgraha search` only searches the current repository's own `.samgraha/knowledge.db`. To search across workspace members, use `samgraha workspace search` from the workspace root (`api-service/`, where `samgraha-workspace.toml` lives):

```bash
cd ~/projects/api-service
samgraha workspace search "string utility"
```

Results include the `core-lib` document even though you're in `api-service`.

### What You Learned

- How to set up a workspace
- How to search across repositories
- How workspace search aggregates results

## Related

- [First Repository](first-repository.md)
- [Multi-Repo Guide: Workspace](../multi-repo-guide/workspace.md)
- [Concepts: Workspace](../concepts/workspace.md)
