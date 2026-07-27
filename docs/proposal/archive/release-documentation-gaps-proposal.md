# Release Documentation Gaps Proposal

**Status**: DRAFT — Gap 2 superseded, see note below; Gaps 1/4 fixed in
docs; Gap 3 still open.

**Scope**: Four documentation inaccuracies in `docs/release/` identified by
cross-referencing documentation against actual implementation. Each gap is
a discrete, independently verifiable fix.

**Resolution note — Gap 2 was a code gap, not a doc gap.** Checked
directly against the real standards corpus
(`Kriti/samgraha/system/<category>/<name>/`): no `standard.yaml` there
declares a `category:` field at all — every one of them communicates
category purely through directory placement. Documenting "category is
DB metadata, not directory structure" (this proposal's original Gap 2
fix) would have been documenting a real mismatch between the source
corpus's own layout and what samgraha did with it, not a
misunderstanding to correct. Fixed in code instead:
`adapter.rs:handle_register_standard_globally` now infers `category`
from `path`'s parent directory name when `standard.yaml` doesn't declare
it, and nests the registry copy at
`mcp_dir()/registry/<category>/<name>/` (was flat `<name>/`) —
mirroring the real corpus, not just documenting a flat layout that
didn't match it. Re-registering a name under a different category moves
the directory rather than orphaning the old one. `docs/release/knowledge-standard.md`
§4 and `docs/release/repository-registration.md` §4 updated to match.
`subcategory` stays DB-only metadata as originally described — the real
corpus has no subcategory-level folder either.

---

## Gap 1: Build package claims to contain standard files (it doesn't)

**File**: `docs/release/build.md` (Output Structure section)

**Current doc implies**: Standard scripts, prompts, templates, and assets
are part of the MCP release build package.

**Actual behavior** (`build-release.sh:59-107`, `build-release.ps1:57-80`):
The build package contains exactly:

```
samgraha/
  bin/mcp.exe (or mcp on Linux)
  bin/cli.exe (or cli on Linux)
  .samgraha/              ← empty directory
  samgraha.toml           ← copied from repo root
  schema/registration/*.sql
  schema/knowledge/*.sql
  schema/standards/*.sql  ← reference only, not runtime
  run-mcp.sh / run-mcp.cmd
  SHA256SUMS
```

No scripts, prompts, templates, or assets. These live in the standard
author's source tree and get copied to `mcp_dir()/registry/<name>/`
only during `register_standard_globally` (the MCP tool), not during
the build.

**Fix**: Update `build.md` Output Structure section to explicitly state
that standard files are NOT in the build package and clarify where they
live (standard author's source tree → copied at registration time).

**Verification**: Read `build-release.sh` lines 59-107 and confirm the
listed files match the output structure in `build.md`.

---

## Gap 2: Category/subcategory described as directory structure (they're DB metadata)

**File**: `docs/release/knowledge-standard.md` (§4 step 1, and §2)

**Current doc states** (§2, line 33): A standard's source directory
contains files with `category`, `subcategory` fields in `standard.yaml`.

**What user described**: "files are copied to MCP build location so that
category and subcategory is preserved" — implying a directory hierarchy
like `registry/<category>/<subcategory>/<name>/`.

**Actual behavior** (`knowledge-standard.md` §4 step 1, `adapter.rs`
`handle_register_standard_globally`):

1. `copy_dir_atomic` copies the standard's source into
   `mcp_dir()/registry/<name>/` — flat, one level deep, no category
   hierarchy.
2. `category` and `subcategory` are stored as **metadata columns** in
   `standards.db`'s `standard_registry` table, not as directory structure.

The copy preserves the original file layout of the standard. Category and
subcategory are queryable DB fields used for listing/filtering, not
filesystem organization.

**Fix**: Add a note in `knowledge-standard.md` §4 clarifying that
category/subcategory are DB metadata, not directory hierarchy. The
copied files go into `registry/<standard-name>/` regardless of category.

**Verification**: Check `adapter.rs` `handle_register_standard_globally`
for the `copy_dir_atomic` call target path and the `standard_registry`
INSERT columns.

---

## Gap 3: Codex CLI integration is a stub with no configuration

**File**: `docs/release/mcp-configuration.md` (§4)

**Current doc** (line 242-243):
```markdown
## 4. Codex CLI / future IDE integrations

Test compatibility.
```

**What other sections provide**: Claude Code (§1), OpenCode (§2), and
Antigravity IDE (§3) all have concrete JSON configuration examples for
both development (source repo) and release binary paths.

**Actual state**: The Codex section has no configuration example. The
server uses standard JSON-RPC 2.0 over stdio, which Codex CLI should
support, but no config has been tested or documented.

**Fix**: Either:
- (a) Add a tested Codex CLI configuration example matching the format
  of the other sections, or
- (b) Remove the section header and merge into a "Future integrations"
  note at the bottom, or
- (c) Add a concrete config if Codex CLI supports MCP stdio servers.

**Verification**: Test the MCP binary with Codex CLI if available, or
mark the section explicitly as untested/placeholder.

---

## Gap 4: "MCP build location" conflates build output with runtime registry

**File**: `docs/release/build.md` and `docs/release/knowledge-standard.md`

**The conflation**: The build produces a static release package at
`OUTPUT_DIR/samgraha/`. The MCP registry (`mcp_dir()/registry/<name>/`)
is a **runtime** concept — created when `register_standard_globally`
runs, not by the build script.

**Actual separation**:

| Concept | When created | Where |
|---------|-------------|-------|
| Build package (`OUTPUT_DIR/samgraha/`) | Build time (`build-release.sh`) | `OUTPUT_DIR/samgraha/` |
| Runtime registry (`mcp_dir()/registry/<name>/`) | Runtime (`register_standard_globally`) | `~/.samgraha/registry/<name>/` (or equivalent) |
| Repo's local copy (`<repo>/.samgraha/<name>/`) | Runtime (`activate_standard`) | `<repo>/.samgraha/<name>/` |

The build copies `samgraha.toml` which defines `samgraha_dir` (where
`.samgraha/` lives at runtime), but the registry itself is populated
dynamically by MCP tool calls.

**Fix**: In `build.md`, add a note clarifying that the build package is
a static distribution — the runtime registry is created by
`register_standard_globally`, not by the build script. In
`knowledge-standard.md`, ensure §4 doesn't imply the build package
contains the registry.

**Verification**: Confirm `build-release.sh` does not create any
`registry/` directory structure — only `.samgraha/` (empty) and
`samgraha.toml`.

---

## Implementation Order

Each gap is independent. Recommended order by impact:

1. **Gap 1** (build contents) — most likely to mislead users trying to
   find standard files in the release package
2. **Gap 2** (category/subcategory) — architectural misunderstanding
   that affects how standards are organized
3. **Gap 4** (build vs runtime conflation) — clarifies the boundary
   between static and dynamic components
4. **Gap 3** (Codex stub) — lowest priority, needs testing or removal
