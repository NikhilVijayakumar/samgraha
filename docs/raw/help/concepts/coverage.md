# Coverage Audit

## Purpose

Bidirectional contract verification between documentation and implementation. Every documented capability should be implemented; every implemented capability should be documented.

**Owns all orphan detection.** No other audit spec contains orphan checks.

## Content

**Coverage Audit** runs forward (doc→code) and reverse (code→doc) checks. Reverse checks detect orphans — code that exists but isn't documented anywhere.

### Orphan Findings

Orphans are always **Warning**, never Error. Resolution: document / remove / suppress.

Phase 1 uses a manifest + grep-based parser. Findings from the grep parser are **Suggestion** severity; promote to **Warning** when tree-sitter-based parser ships.

### Key Checks

Forward (doc→code):
- Documented Features Implemented (CV1)
- Architecture Components Exist (CV2)
- Documented APIs Available (CV3)
- Documented CLI Commands Work (CV4)
- Documented Config Keys Accepted (CV5)
- Documented Capabilities Tested (CV6, advisory)
- Documented Build Targets Exist (CV7)

Reverse (code→doc) — orphans:
- No Orphan Source Components (CV8)
- No Orphan APIs (CV9)
- No Orphan CLI Commands (CV10)
- No Orphan Config Options (CV11)
- No Orphan Dependencies (CV12)
- No Orphan Features (CV13)
- No Orphan Modules (CV14)
- No Orphan Security Mechanisms (CV15)

### Scoring

`(forward_score + reverse_score) / 2`. Zero denominator → 100%.

## Related

- [Audit Concept](audit.md)
- [Consistency Audit](consistency.md)
- [Coverage Audit Spec](../../audit/coverage-audit.md)
