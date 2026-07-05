# Security Audit

## Purpose

Verifies Security Documentation quality and that security configuration and runtime behavior conform to declared security properties.

## Content

**Security Audit** is a pipeline that checks security documentation (SEC1-SEC12) and verifies conformance (SC1-SC11). Security Documentation declares a **Security Properties** section; the pipeline checks configuration, source code, and (opt-in) runtime behavior.

### Levels

| Level | Scope | When |
|---|---|---|
| Static | Source code patterns | Always runs |
| Config | Security configuration (auth, secrets, TLS) | Always runs |
| Runtime | Running application syscall behavior | Opt-in (`--runtime`) |

### Key Checks

- Dependency vulnerability scanning (SC1)
- Authentication config (SC2)
- Authorization config (SC3)
- Secrets isolation (SC4)
- TLS configuration (SC5)
- Properties match runtime (SC6)
- No security regression (SC7)
- External dependency versions (SC8)
- Runtime dependency chain (SC9)
- Runtime secret handling (SC10, Linux only)
- Future maintainability (SC11)

## Related

- [Audit Concept](audit.md)
- [Build Audit](build-audit.md)
- [Security Audit Spec](../../audit/security-audit.md)
