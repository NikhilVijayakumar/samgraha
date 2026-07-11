# Build Separation

This section details Build Separation.

## Purpose

Build Separation removes samgraha-specific documentation from the release artifact and stops pre-compiling knowledge.db at build time.

The release artifact ships only universal documentation standards. Samgraha registers as a peer repository like any other repository. It has no special status in the artifact.

This eliminates the conceptual confusion of a standards engine that bundles its own source knowledge, and it removes the stale pre-compiled database from the distribution artifact.

---

## Functional Requirements

Each functional requirement below defines a verifiable capability that the feature must provide. Requirements are independently testable and traceable to the feature's Acceptance Criteria and documented standards.

## FR1. Universal Standards Only

The release artifact shall contain only the following documentation directories:

* `docs/raw/documentation-standards/`
* `docs/raw/audit/`
* `docs/raw/audit-standards/`

The following samgraha-specific directories shall be excluded from the release artifact:

* `docs/raw/architecture/`
* `docs/raw/engineering/`
* `docs/raw/feature/`
* `docs/raw/feature-technical/`
* `docs/raw/vision/`
* `docs/raw/philosophy/`
* `docs/raw/release/`
* `docs/raw/reports/`

---

## FR2. No Pre-Compiled Knowledge

The release artifact shall contain no `knowledge.db` and no `manifest.json`.

The `.samgraha/` directory in the artifact shall be absent or empty on first install.

Population of `.samgraha/` occurs only when the user explicitly runs `cli compile`.

---

## FR3. Samgraha as Peer

Samgraha's own repository shall be compiled and registered externally like any other repository.

```
cli compile /path/to/samgraha
cli sync /path/to/samgraha
```

No special-casing for samgraha shall exist in the resolve, search, or audit path.

---

## FR4. StandardRegistry Unchanged

The standards delivery path remains `docs/raw/documentation-standards/` on disk inside the artifact.

Recipients compile their own repositories without ever needing samgraha source documentation.

The StandardRegistry reads from this path and its behavior is unchanged by this feature.

---

## FR5. Post-Install Registration

After install, users may optionally register samgraha as a peer repository.

Supported registration paths:

* `samgraha init --register-samgraha`
* `cli compile /path/to/samgraha && cli sync /path/to/samgraha`

Registration is optional. Samgraha functions as a standards engine without its own source documentation being registered.

---

## Business Rules

* The release artifact is a standards engine only. No samgraha-source knowledge is bundled.
* Every repository is treated equally by the registry. Samgraha is not special.
* `compile(path)` always writes into the target repository's own `.samgraha/` directory, not the runtime's.
* Standards remain on disk in the artifact at `docs/raw/documentation-standards/`. This path is unchanged.
* The build script is the single enforcement point for artifact content. No runtime logic excludes or includes based on repository name.

---

## Artifact Layout

```text
Before:                              After:
release/samgraha/                    release/samgraha/
  bin/mcp.exe                          bin/mcp.exe
  docs/raw/                            docs/raw/
    standards/         ← keep            standards/         ← keep
    audit/             ← keep            audit/             ← keep
    audit-standards/   ← keep            audit-standards/   ← keep
    architecture/      ← REMOVE
    engineering/       ← REMOVE
    feature/           ← REMOVE
    feature-technical/ ← REMOVE
    vision/            ← REMOVE
    philosophy/        ← REMOVE
    release/           ← REMOVE
    reports/           ← REMOVE
  .samgraha/
    knowledge.db       ← REMOVE
    manifest.json      ← REMOVE
```

---

## Inputs

Build Separation consumes:

* the source repository file tree
* the release build script (`build-release.ps1` / `build-release.sh`)
* the explicit inclusion list for the artifact (`standards/`, `audit/`, `audit-standards/`)

---

## Outputs

Build Separation produces:

* a release artifact containing only universal standards directories and compiled binaries
* an empty or absent `.samgraha/` directory in the artifact

---

## Constraints

Build Separation shall:

* not change any compile, registry, or MCP tool signatures
* not affect how external repositories compile or sync
* not remove the `docs/raw/documentation-standards/` path from the artifact
* enforce artifact content through the build script, not through runtime guards

---

## Dependencies

Build Separation depends upon:

* Build script (`build-release.ps1`, `build-release.sh`) — artifact assembly
* CompilationService — unchanged; `compile(path)` behavior is unaffected
* RegistryStore — unchanged; peer registration behavior is unaffected

---

## Non-Goals

Build Separation does not:

* change compile, registry, or MCP tool signatures
* affect how external repositories compile or sync
* add runtime enforcement of artifact content
* remove samgraha source documentation from the development repository

Those responsibilities belong to other platform components or remain unchanged.

---

## Future Extensions

* `samgraha init` workflow to guide post-install peer registration interactively.
* Verification step in the build script that asserts the artifact contains no excluded directories.
* CI artifact integrity check that fails the build if `knowledge.db` is present in the output.

---

## Acceptance Criteria

The feature is successful when:

* the release artifact contains no samgraha-specific documentation directories
* the release artifact contains no `knowledge.db` or `manifest.json`
* the StandardRegistry reads standards from `docs/raw/documentation-standards/` correctly after separation
* samgraha can be compiled and registered as a peer repository and appears in `list_repositories`
* no special-casing for samgraha exists in resolve, search, or audit code paths

---

## Traceability

This feature derives from the following Vision commitments:

* **The platform is a standards engine, not a documentation host.**
* **Every repository is a peer. No repository has special runtime status.**
* **Knowledge is compiled on demand, not pre-compiled into the distribution artifact.**

**Traceability**

Vision → Feature: Build Separation
