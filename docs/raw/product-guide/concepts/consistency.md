# Consistency Audit

## Purpose

Verifies adjacent layers in the documentation and implementation chain maintain alignment, use consistent terminology, and contain no contradictions.

## Content

**Consistency Audit** checks the full layer chain: Vision → Architecture → Feature → Feature Technical → Engineering → Implementation, plus Build→Implementation and Security→Implementation alignment.

### Key Checks

- Vision→Architecture alignment (C1)
- Architecture→Feature alignment (C2)
- Feature→Feature Technical alignment (C3)
- Feature Technical→Engineering alignment (C4)
- Engineering→Implementation alignment (C5)
- Build→Implementation alignment (C6)
- Security→Implementation alignment (C7)
- No layer skip (C8)
- Cross-document terminology consistency (C9)
- Constraint propagation (C10)
- No contradiction (C11)
- Traceability complete (C12)

### Contradiction Resolution

Contradictions against Vision or Architecture → fix code. Contradictions between Feature-Technical-and-below ↔ code → either side may be updated.

## Related

- [Audit Concept](audit.md)
- [Coverage Audit](coverage.md)
- [Consistency Audit Spec](../../../proposal/archive/knowledge-system-author-guide.md)
