# Standard Asset Sync Proposal — Scripts, Templates

**Status**: DRAFT — not yet implemented. Findings below are verified against
current code (file:line), not assumed.

**Trigger**: registering a repo (`register_repository` / `init` against
`E:\Python\Heimdall\samgraha.toml`, `standard_system = "python_hackathon"`)
does not hydrate `.samgraha/` with the standard's `script/`/`templates/`
assets the standard's own scripts need to run without relative-path or
permission errors. Heimdall's current `.samgraha/scripts` and
`.samgraha/templates` were hand-copied and hand-fixed from
`E:\Python\Kriti\samgraha\system\python_hackathon`, not produced by any
samgraha command — so they will silently drift the next time
`python_hackathon` changes in Kriti. (Lint-`config` assets were also
investigated — §1.4 — and turned out to be a separate problem with a
different target, scoped out of this proposal.)

---

## 1. Problem Statement

### 1.1 `copy_standard_scripts` drops every subdirectory

`crates/mcp/src/adapter.rs:102-113`:

```rust
fn copy_standard_scripts(source_dir: &Path, dest_dir: &Path) -> Result<usize> {
    std::fs::create_dir_all(dest_dir)?;
    let mut count = 0;
    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            std::fs::copy(entry.path(), dest_dir.join(entry.file_name()))?;
            count += 1;
        }
    }
    Ok(count)
}
```

`read_dir` is not recursive, and the `is_file()` guard silently *skips*
directories instead of erroring. `python_hackathon/script/` has 6
subdirectories (`common/`, `usecase-1-init/`, `usecase-2a-det-audit/`,
`usecase-3-calculate/`, `usecase-5-6-rendering/`, `usecase-7-pdf/`,
`verify/`, `archive/`) holding the actual audit/calculate/render logic —
`run_pipeline.py` imports from `common.db`, `usecase-2a-det-audit.audit_python`,
etc. None of that is copied. Only the 7 flat `run_*.py` entry-point files
(which then fail to import) make it through.

This function is called from exactly two places, both affected:
- `handle_register_standard` (`adapter.rs:2085,2088`) — Kriti → local
  `.samgraha/scripts` and Kriti → global `mcp_dir()/scripts`.
- Nowhere else — `init.rs`'s pull path (§1.2) doesn't even call this
  helper; it has its own, separately-broken, flat copy.

### 1.2 `sync_knowledge_system`'s pull path is *also* flat

`crates/services/src/init.rs:216-230`:

```rust
let source_scripts = mcp_dir.join("scripts");
if source_scripts.exists() {
    let local_scripts = root.join(".samgraha").join("scripts");
    std::fs::create_dir_all(&local_scripts)?;
    for entry in std::fs::read_dir(&source_scripts)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() { continue; }
        std::fs::copy(entry.path(), local_scripts.join(entry.file_name()))?;
        scripts_synced += 1;
    }
}
```

Same bug, independently implemented. This is the code path that runs when
a *new* repo (Heimdall) is registered/initialized with
`sync_knowledge_system = true` — it's the one the user is hitting.

### 1.3 Templates are never synced anywhere

Grepped every use of `templates` in `crates/`: nothing copies
`python_hackathon/templates/{audit,generation,reports/{html,markdown}}`
into either the global store or a local `.samgraha/`. Samgraha's own
`report_templates_dir()` (`runtime.rs:333`) points at
`docs/raw/report-templates` — a *different*, samgraha-internal template set
for samgraha's own doc-audit reports, unrelated to a standard's own
scoring-report templates that its Python scripts render
(`run_html.py`, `run_render.py` read `templates/reports/html/*.html` and
`templates/reports/markdown/*.md` directly).

Heimdall's `.samgraha/templates/reports/markdown/` exists only because it
was manually copied — `.samgraha/templates/reports/html/`,
`.samgraha/templates/audit/`, `.samgraha/templates/generation/` are
missing entirely (visible in the current directory listing), so `run_html.py`
will fail the moment it's invoked in Heimdall the way it does in Kriti.

### 1.4 Lint/config assets (`.bandit`, `.flake8`, `mypy.ini`, ...) target the
wrong repo entirely — this isn't a copy-destination bug, it's a different
sync problem

Heimdall's `.samgraha/config/` (`.bandit`, `.flake8`, `.isort.cfg`,
`.pylintrc`, `mypy.ini`, `pyproject.toml`, `radon.cfg`) has **no
counterpart in Kriti's `python_hackathon/`** — there is no `config/`
directory there. These were authored directly in Heimdall by hand.

Traced where these configs are actually read: `audit_python.py:120-121`
does `os.path.join(repo_path, "radon.cfg")` / `os.path.join(repo_path,
".bandit")`. `repo_path` is not Heimdall's own root — it's
`participant["repo_path"]` (`run_hackathon.py:94,103`), sourced from
`teams.json`/the DB, i.e. **each hackathon team's own cloned submission
repo, external to both Heimdall and Kriti**. `resolve_code_root()`
(`.samgraha/scripts/common/repo.py` — itself a Heimdall-only addition
with no counterpart in Kriti's `script/common/`, confirming assets have
already started drifting by hand) further dives into a team's code
subfolder before any of this runs.

No script anywhere copies `config/*` into a participant repo. So
Heimdall's `.samgraha/config/` currently has **no consuming code path** —
it's stray, not misplaced. Putting these files in `.samgraha/config/`
(§3.3's original framing) would fix nothing; `audit_python.py` would never
look there.

**Resolution**: config seeding is a distinct sync problem from
scripts/templates (§3.1-3.2) — its target is N external participant repos
picked up at audit time, not the standard-owning repo's own `.samgraha/`.
It doesn't belong in the `mcp_dir()/systems/<name>/{scripts,templates}`
mechanism at all. Scope it out of this proposal and track separately as a
"seed config into participant repo at registration" step
(`usecase-1-init`-time, alongside `resolve_code_root`), owned by
`run_hackathon.py`'s registration flow, not `register_standard`/
`sync_knowledge_system`. §3.3/§3.4 below cover scripts + templates only.

### 1.5 The global store is flat and unnamespaced by system

`mcp_dir()/scripts/` (both the push side, `adapter.rs:2087`, and the pull
side, `init.rs:190,217`) is one shared flat directory for *every*
registered standard. Two systems both shipping a `common/db.py` or
`run_pipeline.py` overwrite each other in the global store with no
warning. `python_hackathon` and any future system (e.g. a second hackathon
variant) would collide today.

It also has no exclusion list — `python_hackathon/script/` already ships
`__pycache__/*.pyc` next to the source (`archive/__pycache__/`,
`common/__pycache__/`, etc., visible in the current tree), and nothing
today stops those from being pushed into the shared global store and
re-pulled into every other repo that syncs, dragging one system's stale
bytecode across process/Python-version boundaries. Motivates the
exclusion patterns in §3.2/§3.3.

### 1.6 No per-repo control over which assets ship

The user wants `samgraha.toml`/the standard's manifest to say what a given
registration should include — right now `sync_knowledge_system` is
all-or-nothing (whatever's flat-copyable, minus subdirectories) with zero
per-repo filtering. There's also no manifest on the standard side
(`system.yaml`, checked at `schemas/standard.rs` — fields are just
`name`/`extends`/`description`/`drops`) declaring which directories under a
standard's root *are* its shippable assets versus authoring-time-only
content (`plan/`, `domains/`, `html-design/`, `analysis/`, `00-domain-
relationships.md`, `teams.json`, `hackathon.db` are all present in Kriti's
`python_hackathon/` and clearly shouldn't ship to every consuming repo).

---

## 2. Current Flow (as built)

```
Kriti/samgraha/system/python_hackathon/
  script/{common,usecase-*,verify,archive}/*.py   ─┐
  templates/{audit,generation,reports/{html,md}}/*  │  register_standard
                                                     ▼
                              (adapter.rs:2029) handle_register_standard
                                     │
                    ┌────────────────┴─────────────────┐
                    ▼                                   ▼
     Heimdall/.samgraha/scripts/         mcp_dir()/scripts/      (flat, top-level
     (copy_standard_scripts,             (copy_standard_scripts,  files only —
      non-recursive)                     non-recursive, unnamespaced)  §1.1, §1.5

     templates/  → never touched anywhere
```

```
                          init_repository(sync_knowledge_system=true)
                                     │           (init.rs:187)
                                     ▼
                  mcp_dir()/scripts/*  (flat files only)  ──► local .samgraha/scripts/
                  (init.rs:216-230, independently non-recursive)

                  templates/  → still never touched
```

---

## 3. Proposal

### 3.1 Namespace the global store per system

```
mcp_dir()/
  standards.db
  systems/
    python_hackathon/
      scripts/        (full recursive tree)
      templates/       (full recursive tree)
    fastapi_dev/
      scripts/
      ...
```

Replaces the flat `mcp_dir()/scripts/`. Removes the cross-system
collision risk (§1.5) and gives `sync_knowledge_system` an unambiguous
source scoped to `samgraha.toml`'s own `standard_system`.

### 3.2 One recursive copy helper, shared both directions

Add `copy_dir_recursive(src, dest, exclude: &[glob::Pattern]) ->
Result<usize>` to `crates/common/src/fs_sync.rs` (new module; `common` is
already a dependency of both `mcp` and `services`/`cli` — `adapter.rs`
already calls `common::env::mcp_dir()`, so this adds no new edge to the
crate graph). Walks subdirectories, applies the exclude list (default
`**/__pycache__/**`, `**/*.pyc` — see §1.5), returns file count copied.
Both `handle_register_standard` (push) and `sync_knowledge_system` (pull)
call this one function instead of the two independently-buggy flat copies
in `adapter.rs:102` and `init.rs:216`. Apply it to both remaining asset
kinds: `script/` → `scripts/`, `templates/` → `templates/`. (`config/` is
out of scope here — see §1.4.)

**Atomicity**: write into a sibling temp dir
(`mcp_dir()/systems/<name>/.scripts-tmp-<uuid>/`) and `fs::rename` it onto
the final `scripts/`/`templates/` path once the copy completes without
error. A same-filesystem directory rename is atomic on both Windows and
Unix, so a failure mid-copy (disk full, permission error partway through)
leaves the previous good tree untouched instead of a half-overwritten
`scripts/` — mirrors the integrity-check-before-copy pattern
`sync_knowledge_system` already uses for `standards.db`
(`init.rs:193-208`). On error, delete the temp dir and return `Err`; never
partially rename.

### 3.3 Give a standard a declared asset manifest

Extend `system.yaml` (optional block, defaults preserve current behavior
for systems that don't set it):

```yaml
name: python_hackathon
assets:
  scripts: script          # dir name under the standard root; default "script"
  templates: templates      # default "templates"
  exclude:
    - "**/__pycache__/**"
    - "**/*.pyc"
    - archive/**             # standard owner marks dead code explicitly
```

Only `scripts`/`templates` — no `config` key (§1.4: config seeding targets
participant repos, not this pipeline, and is tracked separately).

### 3.4 Let a repo's `samgraha.toml` select a subset

New optional table under `[repository.documentation]`, mirroring the
existing `script_overrides`/`check_overrides` shape:

```toml
[repository.documentation.asset_sync]
scripts = true
templates = true
# Optional narrowing — only pull these entries instead of everything the
# standard ships. Each entry is a path relative to the standard's scripts
# root (or templates root, for template_include) and is
# directory-inclusive: naming a directory pulls its whole subtree,
# naming a file pulls just that file. Omit *_include entirely to take
# the standard's full asset set (default).
script_include = ["common", "usecase-2a-det-audit", "run_det_audit.py"]
#                  ^ whole common/ subtree
#                              ^ whole usecase-2a-det-audit/ subtree
#                                                          ^ single file
template_include = ["reports/markdown"]
```

`init_repository`/`sync_knowledge_system` reads this to decide, per asset
kind: sync everything (default), sync nothing (`= false`), or sync a
filtered subset (`*_include` list, directory-inclusive as above). Answers
the user's "specify script/template and what all can be included, per
standard" ask directly.

### 3.5 Updated flow

```
register_standard  →  mcp_dir()/systems/<name>/{scripts,templates}/
                       (recursive copy, per system.yaml's assets: block,
                        atomic temp-dir + rename per §3.2)

init/sync_standards  →  reads samgraha.toml's standard_system +
                         [repository.documentation.asset_sync]
                      →  recursive, filtered copy from
                         mcp_dir()/systems/<name>/* into
                         <repo>/.samgraha/{scripts,templates}/
```

Both directions share the same copy primitive (§3.2) — no more "push
recurses, pull doesn't" divergence.

### 3.6 Script discovery chain gets the same namespace-aware treatment

The audit engine's script resolution tiers (`check_runner.rs`,
`providers.rs`, `capability.rs`) all had a Tier 4 fallback that looked
in the flat `mcp_dir()/scripts/`. With the global store namespaced
(§3.1), that path no longer holds standard-shipped scripts. Updated
Tier 4 in all three files to first check
`mcp_dir()/systems/<name>/scripts/` (reading `standard_system` from
`samgraha.toml`), then fall back to the legacy flat path for backward
compat with any system that hasn't been re-registered yet.

This was not in the original proposal scope (which spec'd only the
sync/copy mechanism), but without it the discovery chain would silently
stop finding standard scripts after migration. Added during
implementation to avoid a latent breakage.

---

## 4. Migration

- One-time: existing flat `mcp_dir()/scripts/` is not auto-migrated (can't
  reliably attribute each flat file back to its owning system after the
  fact). Re-run `register_standard` for every currently-registered system
  once this ships — it repopulates `mcp_dir()/systems/<name>/` from each
  system's own source tree, which is authoritative anyway.
- Heimdall specifically: after the fix lands, re-run `sync_standards
  --force` (or re-`init`) against `python_hackathon` and diff the result
  against the current hand-copied `.samgraha/scripts` and
  `.samgraha/templates` — reconcile any manual fixes made directly in
  Heimdall by porting them back into Kriti's `python_hackathon/script`
  (single source of truth) rather than re-hand-patching the copy. Note
  `.samgraha/scripts/common/repo.py` specifically (§1.4) — it exists only
  in Heimdall's copy, so it needs to be ported *into* Kriti, not overwritten
  by the re-sync.
- Config seeding (§1.4) is tracked as a separate follow-up, not part of
  this migration.

## 5. Testing

- Unit: `copy_dir_recursive` on a fixture tree with nested dirs +
  `__pycache__` + `.pyc` — asserts subdirectories survive and excluded
  patterns don't.
- Integration: extend `sync_lifecycle_full` (`init.rs` tests) to assert
  `templates/` and `scripts/<subdir>/*.py` exist post-sync, not just
  `standards.db`.
- Regression: register `python_hackathon` fresh into a scratch repo, run
  its `run_pipeline.py` end-to-end — this is the actual failure mode
  reported ("relative path or permission errors") and the only way to
  confirm it's gone.
