# Engineering Domain Guide

## Purpose

How to write Engineering documentation — repository-wide engineering decisions and standards.

## Content

### Purpose of Engineering Docs

Engineering documentation defines how the project is built, tested, and maintained. It captures the team's engineering principles and tooling choices.

### Required Sections

| Section | Required | Description |
|---------|----------|-------------|
| Engineering Principles | Yes | Core engineering principles |
| Technology Selection | Yes | Why technologies were chosen |
| Build Standards | Yes | How the project is built |
| Testing Standards | Yes | How the project is tested |

### Writing Tips

- Engineering docs change as the project evolves (unlike Vision or Philosophy).
- Explain *why* a technology was chosen, not just what was chosen.
- Build and testing standards should be actionable enough for CI/CD setup.
- Reference Feature Technical docs for per-feature decisions.

### Profiles

Engineering defines two profiles:

- **onboarding** — principles, build, testing, code standards (for new contributors)
- **review** — purpose, rationale, constraints, traceability (for reviewers)

## Related

- [Architecture Guide](architecture.md)
- [Feature Technical Guide](feature-technical.md)
- [Standards Reference: Engineering](../../standards/engineering.md)
