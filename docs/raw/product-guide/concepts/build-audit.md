# Build Audit

## Purpose

Verifies Build Documentation quality and that Build Configuration and produced artifacts conform to that strategy.

## Content

**Build Audit** is a pipeline that checks build documentation (B1-B12) and verifies conformance (BC1-BC10): Build Documentation declares an **Artifact Spec** and **Runtime Spec**; the pipeline checks that the repository's declared [Pipeline Contract](pipeline-contracts.md) (`samgraha.toml [pipelines.build]`) and produced binary artifacts match — not any single build system's own files.

### Levels

| Level | Scope | When |
|---|---|---|
| Config | `[pipelines.build]` contract (command, artifacts) | Always runs |
| Artifact | Produced binary | Opt-in (`--inspect-artifact`) |

### Key Checks

- Build principles → realized in the Pipeline Contract (BC1)
- Targets → CI matrix → binary (BC2)
- Features → declared in the Pipeline Contract (BC3)
- Dependencies → rationale in docs (BC4)
- CI platform → actual CI config (BC5)
- Outputs → declared `artifacts` → binary (BC6)
- Config self-consistency (BC7)
- External deps → External Context docs (BC8)
- Artifact contents → Artifact Spec (BC9)
- Future maintainability (BC10)

## Related

- [Audit Concept](audit.md)
- [Security Audit](security-audit.md)
- [Pipeline Contracts](pipeline-contracts.md)
- [Build Audit Spec](../../../knowledge-hub/audit/semantic/document/build-audit.md)
