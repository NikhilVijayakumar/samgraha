# First Project Walkthrough

## Purpose

End-to-end guide: initialize a repository, write a document, compile it, and run an audit.

## Content

### Step 1: Initialize

```bash
cd my-project
samgraha init
```

Creates `.samgraha/` directory structure.

### Step 2: Create a Document

Write a feature specification at `docs/feature/authentication.md` (documents are discovered recursively under `[repository.documentation].root_dir`, which defaults to `${SAMGRAHA_DOCS_DIR}` falling back to `<repo>/docs`; the domain is inferred from the immediate parent directory name, here `feature`):

```markdown
# Authentication

## Purpose

Handle user authentication for the application.

## Functional Requirements

- FR1: Users can register with email and password.
- FR2: Users can log in with registered credentials.
- FR3: Users can reset their password.

## Acceptance Criteria

- All FRs pass automated tests.
- Login completes in under 2 seconds.
```

### Step 3: Compile

```bash
samgraha compile
```

This discovers markdown files in `docs/raw/`, parses them by standard, extracts sections, and writes them to `.samgraha/knowledge.db`.

### Step 4: Audit

```bash
samgraha audit
```

Checks documents against the Feature Standard rules:

- `feat-001` — Has purpose (pass/fail)
- `feat-002` — Has functional requirements (pass/fail)
- `feat-003` — Has acceptance criteria (pass/fail)
- `feat-004` — Technology independent (pass/fail)

### Step 5: Search

```bash
samgraha search "authentication" --domain feature
```

Returns matching sections from the compiled knowledge database.

### Step 6: View Sections

```bash
samgraha sections functional_requirements --domain feature
```

`sections` queries by semantic type (a positional argument, e.g. `functional_requirements`, `business_rules`), not by document name — it lists every matching section across documents in the domain, up to `--max` (default 50).

## Related

- [Installation](installation.md)
- [Command: compile](../commands/compile.md)
- [Command: audit](../commands/audit.md)
- [Command: search](../commands/search.md)
