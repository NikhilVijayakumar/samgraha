# Feature Design Domain Guide

## Purpose

How to write Feature Design documentation — user-centered design for a single feature.

## Content

### Purpose of Feature Design Docs

Feature Design describes how users interact with a feature. It focuses on UX, workflows, and UI states.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| User Experience | Yes | How users interact with the feature |
| Workflow | Yes | Step-by-step user workflow |
| States | Yes | Empty, loading, success, error states |

### Writing Tips

- Focus on the user journey, not the implementation.
- Describe all UI states: what happens when data is empty, loading, successful, or errored.
- Use diagrams or flowcharts for complex workflows.
- Keep implementation details separate (they belong in Feature Technical).

### State Descriptions

For each major component or screen:

| State | Description |
|-------|-------------|
| Empty | What the user sees when no data exists |
| Loading | What happens while data is being fetched |
| Success | The normal populated state |
| Error | What happens when something goes wrong |

## Related

- [Feature Guide](feature.md)
- [Design Guide](design.md)
- [Standards Reference: Feature Design](../../standards/feature-design.md)
