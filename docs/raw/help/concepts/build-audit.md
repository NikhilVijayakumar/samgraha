# Build Audit

## Purpose

Verifies Build Documentation quality and that Build Configuration and produced artifacts conform to that strategy.

## Content

**Build Audit** is a pipeline that checks build documentation (B1-B12) and verifies conformance (BC1-BC10): Build Documentation declares an **Artifact Spec** and **Runtime Spec**; the pipeline checks that configuration and binary artifacts match.

### Levels

| Level | Scope | When |
|---|---|---|
| Config | Cargo.toml, CI YAML, build.rs | Always runs |
| Artifact | Produced binary | Opt-in (`--inspect-artifact`) |

### Key Checks

- Build principles → realized in config (BC1)
- Targets → CI matrix → binary (BC2)
- Features → Cargo.toml (BC3)
- Dependencies → rationale in docs (BC4)
- CI platform → actual CI config (BC5)
- Outputs → [[bin]] → binary (BC6)
- Config self-consistency (BC7)
- External deps → External Context docs (BC8)
- Artifact contents → Artifact Spec (BC9)
- Future maintainability (BC10)

## Related

- [Audit Concept](audit.md)
- [Security Audit](security-audit.md)
- [Build Audit Spec](../../audit/build-audit.md)
