# Dependency Governance

## Purpose

Governance framework ensuring every dependency is justified, documented, correctly owned, properly versioned, and healthy.

## Content

**Dependency Governance** is a governance framework — not yet automated. It defines how dependencies should be managed: justification, ownership, version policy, supply-chain sourcing, health checks, and scope classification.

### Key Checks (Spec)

- Every dependency justified with rationale (D1)
- Every dependency documented with owner, purpose, version policy (D2)
- Dependency ownership explicit (D3)
- Version policy respected (D4)
- Supply-chain policy applied (D5)
- Health check: deprecated, unmaintained, yanked (D6)
- Dependency scope correct: runtime/dev/build (D7)
- Orphan dependencies → Coverage Audit CV12 (D8)

### Status

**Specification only.** Automated checks not yet implemented. Manual review possible today.

## Related

- [Audit Concept](audit.md)
- [Coverage Audit](coverage.md)
- [Dependency Governance Spec](../../../proposal/archive/knowledge-system-author-guide.md)
