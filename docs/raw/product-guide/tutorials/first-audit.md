# Tutorial: First Audit

## Purpose

Step-by-step: create a document, audit it, fix findings, and pass a quality gate.

## Content

### Step 1: Write a Document (with issues)

Create `docs/raw/feature/my-feature.md`:

```markdown
# My Feature

This feature does useful things.
```

No sections, no requirements, no acceptance criteria.

### Step 2: Compile

```bash
samgraha compile
```

### Step 3: Audit

```bash
samgraha audit
```

Expected findings:
- `feat-001` (error): Missing Purpose section
- `feat-002` (error): Missing Functional Requirements
- `feat-003` (error): Missing Acceptance Criteria

### Step 4: Fix

```markdown
# My Feature

## Purpose
This feature does something useful.

## Functional Requirements
- FR1: The system shall do X.

## Acceptance Criteria
- FR1 passes automated tests.
```

### Step 5: Re-Audit

```bash
samgraha compile
samgraha audit
```

All errors should now be resolved.

### Step 6: Quality Gate

```bash
samgraha audit --gate 80
```

### Optional: Coverage Audit

If you have implementation code, run Coverage Audit to check for orphans:

```bash
samgraha audit --pipeline coverage
```

This detects undocumented code (warnings) and unimplemented features (errors).

### What You Learned

- How to audit documents
- How to interpret findings
- How to fix common issues
- How to use quality gates
- How to run a coverage audit

## Related

- [First Repository](first-repository.md)
- [Audit Guide: Overview](../audit-guide/overview.md)
- [Audit Guide: Fixing Findings](../audit-guide/fixing-findings.md)
- [Coverage Audit Concept](../concepts/coverage.md)
