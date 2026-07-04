# Build Separation — Feature Technical Design

This section details the Build Separation — Feature Technical Design.

## Purpose

This document describes the architectural realization of the Build Separation feature.

The Build Separation change ensures that the Saṃgraha release artifact does not ship Saṃgraha-specific design documents or pre-compiled knowledge databases. The release artifact contains only the components that belong to every user's installation: the compiled binaries, the standards library, audit definitions, and audit-standards. Saṃgraha's own knowledge is compiled by the user's runtime instance if the user chooses to register the Saṃgraha repository as a peer.

This document applies the architectural principles defined in Deployment Architecture and Component Model.

---

## Feature Specification

- **Feature:** docs/raw/feature/build-separation.md
- **Architecture:** docs/raw/architecture/deployment.md, docs/raw/architecture/component-model.md

---

## Participating Components

This section details the Participating Components.

### Build Script (build-release.ps1)

Owns artifact composition. The sole component changed by this feature. Controls which directories from `docs/raw/` are copied into the release artifact and whether `cli.exe compile` runs during the build.

### Compilation Service

Unchanged. `compile(path)` writes into the target repository's `.samgraha/` directory at the path provided. This behavior is already correct — the change is only in what the build script places into the artifact before distribution.

### Standard Registry

Unchanged. Reads `docs/raw/standards/` from disk using a path relative to the installed binary. The `standards/`, `audit/`, and `audit-standards/` directories ship unchanged and at the same path.

### Registry Store

Unchanged. Per-repository ownership is already correct. Each repository's `.samgraha/knowledge.db` is created by `compile` at registration time, not at build time.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| Build Script (build-release.ps1) | Compose release artifact; select which docs/raw/ subdirectories to include; control whether pre-compile runs |
| Compilation Service | Compile a repository's documents into `.samgraha/knowledge.db` on user's machine at their request; unchanged |
| Standard Registry | Read standards/ from artifact-relative path at runtime; unchanged |
| Registry Store | Own per-repository `.samgraha/knowledge.db`; populated at user compile time; unchanged |
| CLI (compile command) | User-invoked entry point for first-time knowledge compilation; unchanged |

---

## Component Interactions

Build time (build-release.ps1):

```text
build-release.ps1
    │
    ├── Copy-Item docs/raw/standards/        → release/samgraha/docs/raw/standards/
    ├── Copy-Item docs/raw/audit/            → release/samgraha/docs/raw/audit/
    ├── Copy-Item docs/raw/audit-standards/  → release/samgraha/docs/raw/audit-standards/
    │   [architecture/, feature/, engineering/, ... are NOT copied]
    │
    └── [pre-compile step removed — no cli.exe compile --force]
```

User install time (first-time registration of samgraha repo as peer):

```text
User runs: cli compile /path/to/samgraha-repo
    │
    └── CompilationService.execute(path, config, ...)
            └── writes /path/to/samgraha-repo/.samgraha/knowledge.db

User runs: cli sync /path/to/samgraha-repo
    └── registers in repository registry + writes .meta file
    → samgraha appears in list_repositories as a peer (not special-cased)
```

---

## Runtime Behavior

### Artifact Composition Change

```
Before build-release.ps1 change:          After:
release/samgraha/                         release/samgraha/
  docs/raw/                                 docs/raw/
    standards/        ← ships                standards/        ← ships
    audit/            ← ships                audit/            ← ships
    audit-standards/  ← ships                audit-standards/  ← ships
    architecture/     ← ships (wrong)        [not copied]
    feature/          ← ships (wrong)        [not copied]
    feature-technical/← ships (wrong)        [not copied]
    engineering/      ← ships (wrong)        [not copied]
    ...more           ← ships (wrong)        [not copied]
  .samgraha/                                .samgraha/
    knowledge.db      ← ships (wrong)        [empty — populated at compile time]
    manifest.json     ← ships (wrong)        [empty]
```

### Implementation

The entire implementation is two line changes in `scripts/build-release.ps1`:

1. **Line 69** — Change the `Copy-Item` for `docs/raw/` from a wildcard recursive copy to three selective copies:
   ```powershell
   # Before:
   Copy-Item -Recurse "$root\docs\raw\*" ...
   # After:
   Copy-Item -Recurse "$root\docs\raw\standards" ...
   Copy-Item -Recurse "$root\docs\raw\audit" ...
   Copy-Item -Recurse "$root\docs\raw\audit-standards" ...
   ```

2. **Line 74** — Remove the pre-compile invocation:
   ```powershell
   # Remove this line:
   & ".\bin\cli.exe" compile --force
   ```

No Rust code changes are required in Phase 1. All service, registry, and compilation logic is unchanged.

### Saṃgraha Self-Registration

After installation, if the user wants Saṃgraha's own documentation to appear as a knowledge source, they run two commands:

```
cli compile /path/to/samgraha-repo
    → writes /path/to/samgraha-repo/.samgraha/knowledge.db

cli sync /path/to/samgraha-repo
    → registers in repository registry, writes .meta file
    → samgraha appears in list_repositories as a peer, not special-cased
```

This is intentional: Saṃgraha's documentation is treated as user knowledge, compiled on the user's machine against the user's installation. There is no pre-compiled artifact to become stale.

---

## Communication Paths

### Build Script → File System (build time only)

The build script reads `docs/raw/` subdirectories and writes selected ones to the release artifact path. This is a build-time-only operation. No runtime communication paths change.

### User → CLI (install time, optional)

If the user registers the Saṃgraha repository, they invoke `cli compile` and `cli sync` through the standard CLI interface. These paths are unchanged from any other repository registration flow.

---

## Data Ownership

| Data | Owner | Notes |
|---|---|---|
| docs/raw/standards/ | Release artifact | Ships; read by StandardRegistry at runtime |
| docs/raw/audit/ | Release artifact | Ships; read by Audit Framework at runtime |
| docs/raw/audit-standards/ | Release artifact | Ships; read by Audit Framework at runtime |
| docs/raw/architecture/, feature/, ... | Source repository only | Not shipped in release artifact after this change |
| .samgraha/knowledge.db | Per-repository (user's machine) | Populated by cli compile at user's request, not at build time |
| .samgraha/manifest.json | Per-repository (user's machine) | Written by compiler at registration time |

---

## Integration Points

### Standard Registry

StandardRegistry reads from a path relative to the binary location. The `standards/` directory ships at the same relative path as before. No change to StandardRegistry.

### Audit Framework

Reads `audit/` and `audit-standards/` from the artifact. Both ship at the same relative path as before. No change to Audit Framework.

### Compilation Service

`CompilationService::execute` is called by the user via CLI after installation. The service is unchanged; the build script no longer pre-invokes it.

---

## External Dependency Integration

None. Build separation is entirely a build-script change. No external services, providers, or network access involved.

---

## Runtime Constraints

- `docs/raw/standards/`, `docs/raw/audit/`, and `docs/raw/audit-standards/` must be present in the release artifact (StandardRegistry and Audit Framework depend on them).
- `docs/raw/architecture/`, `docs/raw/feature/`, `docs/raw/feature-technical/`, and `docs/raw/engineering/` must not be present in the release artifact.
- `.samgraha/knowledge.db` must not be pre-populated in the release artifact.
- No Rust code changes in Phase 1.

---

## Architectural Constraints

- StandardRegistry path resolution must not change. The three shipped directories remain at the same artifact-relative paths.
- Build-time pre-compilation must not occur. The compile step is a user action, not a build action.
- Saṃgraha's own documentation is not special-cased at runtime. If registered, it appears as a peer repository.

---

## Security Considerations

- Shipping Saṃgraha-specific design documents in user installations was an information boundary violation: users received internal implementation documentation they did not request.
- After separation, only standards, audit definitions, and audit-standards ship — these are explicitly intended for all users.
- Pre-compiled `knowledge.db` embedded in the artifact could not be audited per-installation. Compile-on-install ensures the knowledge database reflects the user's installed version.

---

## Performance Considerations

- The release artifact is smaller after this change. Fewer files are copied during the build, and the artifact does not include pre-compiled databases.
- First-time users who want Saṃgraha's own documentation must run `cli compile`. This is a deliberate trade-off: correctness over convenience. The compile step takes the same time as compiling any other repository.
- Build script execution time is reduced (no pre-compile step, fewer Copy-Item operations).

---

## Failure Handling

| Failure | Behavior |
|---|---|
| Saṃgraha-specific docs accidentally shipped | They get compiled by CompilationService and mixed with user knowledge — this is the bug being fixed. After separation, they never enter the artifact. |
| User does not run cli compile for samgraha repo | Saṃgraha documentation does not appear in list_repositories — correct behavior. User opts in explicitly. |
| cli compile fails for samgraha repo | Standard compilation error handling; no impact on other repositories or the runtime. |
| standards/ or audit/ missing from artifact | StandardRegistry or Audit Framework initialization fails at startup — same failure mode as before this change, unaffected. |

---

## Extension Points

### Additional Excluded Directories

If new top-level directories are added to `docs/raw/` that should not ship, the build script's selective copy pattern naturally excludes them. No code change required.

### Automated Self-Registration

A future installer or setup wizard could invoke `cli compile` and `cli sync` automatically for the Saṃgraha repository. This is a user-experience enhancement; it does not change the underlying separation.

---

## Traceability

This document derives from:

- Feature: Build Separation
- Architecture: Deployment Architecture
- Architecture: Component Model

This document provides technical context for:

- Build and Release Engineering
- Deployment Architecture

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
