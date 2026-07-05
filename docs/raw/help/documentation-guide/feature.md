# Feature Domain Guide

## Purpose

How to write Feature documentation — atomic functional capability specifications.

## Content

### Purpose of Feature Docs

Feature documentation describes exactly what a feature must do. Each document describes exactly one feature.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| Purpose | Yes | Why this feature exists |
| Functional Requirements | Yes | Numbered list of what the feature must do |
| Acceptance Criteria | Yes | How to verify the feature is complete |

### Atomic Feature Principle

One document = one feature. If a feature has multiple independent capabilities, split them.

### Prohibited Content

Feature docs must NOT contain:
- Implementation details
- Architecture decisions
- Programming languages, frameworks, libraries
- API design
- Source code

### Example

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

## Related

- [Feature Design Guide](feature-design.md)
- [Feature Technical Guide](feature-technical.md)
- [Standards Reference: Feature](../../standards/feature.md)
