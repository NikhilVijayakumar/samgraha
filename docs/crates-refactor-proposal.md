# System-Class Inheritance Proposal

**Scope**: samgraha's knowledge-hub loader (`register_standard`, `sync_standards`,
the knowledge-hub loader schema, and the Rust `StandardRegistry`) — adding a
data-driven inheritance model so systems can declare a base and override-by-exception,
eliminating copy-paste drift across system trees.

**Source**: `E:\Python\Kriti\docs\limitation\mcp\01-system-class-inheritance.md` (LIM-001)

**Status**: PARTIALLY SUPERSEDED — see
`docs/generic-script-architecture-proposal.md`. Phase 0 (push-safety guard)
and Phase 1 (`system.yaml`/`extends`/`drops` for the dev class) are
**shipped and kept** — real fixes, still correct under the new direction.
Phases 2-4 (`base_academic`, DB schema columns for class/subclass/
`parent_system_id`, class-shape validation) are **dropped**: comparing
`eswa_journal`/`pcems_2026` file-by-file showed academic-class systems share
almost no content (2 files out of 9+), and baking a class taxonomy into
samgraha's own schema is exactly the domain-coupling problem the new
proposal moves away from. See that doc's §7 for the full per-phase
disposition.

---

## 1. Problem Statement

Every knowledge-hub system (`base_dev`, `electron_dev`, `fastapi_dev`, `react_dev`,
`rust_dev`, `python_hackathon`, `eswa_journal`, `pcems_2026`) is authored as a
fully independent tree. There is no mechanism to declare "this system inherits from
that system." This causes:

### 1.1 Byte-for-byte duplication

All 5 dev-class systems have identical `plan/core/loop.yaml` (77 lines each).
At least 4 files under `calculation/` are also identical across all dev systems.
Changing `base_dev` requires manually patching 4 other files with no drift detection.

### 1.2 Wrong-class copy-paste

`eswa_journal` and `pcems_2026` originally shipped with `python_hackathon`'s
`loop.yaml` — competitive leaderboard logic, `relative`/`competition` scoring,
a `cap: 20` normalization — none of which applies to academic-paper workflows.
Only caught by manual review.

### 1.3 No expressible relationship

Nothing in the tooling records that `electron_dev` extends `base_dev`, or that
`rust_dev` is a dev-class system with a reduced domain set. Cross-system
consistency is a manual audit, not a tool-checked invariant.

---

## 2. Current Architecture

### 2.1 Loader pipeline (Python)

`schema/knowledge-hub/knowledge-hub-loader.py` reads a directory tree of
YAML/Markdown files and writes a SQLite `standards.db` via 11 passes:

| Pass | Input | Output (DB tables) |
|------|-------|-------------------|
| 0 | system name | `systems`, `standards` |
| 1 | `00-domain-relationships.md`, `plan/core/loop.yaml` | `domains`, `relationship_types`, `domain_relationships` |
| 2 | `templates/generation/document/*.md`, `audit/` trees | `section_catalog` |
| 3 | `documentation-standards/*.md` | `standard_docs` |
| 4 | `script/schema/*.manifest.yaml` | `script_checks`, `script_check_dependencies` |
| 5 | `audit/deterministic/**/*.yaml`, `audit/semantic/**/*.md` | `rules`, `rule_evidence_params` |
| 6 | `templates/` | `templates` |
| 7 | `calculation/**/*.yaml` | `calculation_rules`, `calculation_inputs`, `score_bands` |
| 8 | `plan/core/loop.yaml` (settings) | `plan_settings`, `plan_scenarios` |
| 9 | `calculation/validation/*.yaml` | `validation_rules` |
| 10 | `plan/core/loop.yaml` (stages) | `workflow_stages` |

### 2.2 Rust consumption

`crates/standards/src/db_reader.rs:from_standards_db()` opens `.samgraha/standards.db`,
projects all tables into `StandardDefinition` structs via SQL queries, and populates
a `StandardRegistry`. Schema version validated via `PRAGMA user_version` (must match
`EXPECTED_SCHEMA_VERSION = 1`).

### 2.3 Entry points

- **MCP `register_standard`** (`crates/mcp/src/adapter.rs:1609`): invokes Python loader
  with `--db <path>/standards.db --system <name> --knowledge-hub <dir>`
- **CLI `knowledge publish`** (`crates/cli/src/commands.rs:1436`): same invocation
- **MCP `sync_standards`**: copies `standards.db` from global store to repo's `.samgraha/`

### 2.4 No inheritance mechanism

- Systems table has no `extends` or `parent_id` column
- Python loader has no merge/delta logic
- `db_reader.rs` has no base-system resolution
- The `knowledge-hub-loader.py` layout system (`DEFAULT_LAYOUT` + `--layout` overrides)
  allows directory renaming but not tree composition

---

## 3. Proposed Design

### 3.1 Taxonomy

```
CLASS → optional SUBCLASS → BASE SYSTEM → CONCRETE SYSTEM
```

- **Class**: `dev`, `hackathon`, `academic` (matches 3 informal classes already observed)
- **Subclass**: optional per class. `dev` → `frontend`/`backend`/`fullstack`;
  `academic` → `paper`/`journal`. Classes without meaningful subclass split skip this.
- **Base system**: a `documentation-standards`/`audit`/`calculation`/`plan`/
  `templates`/`script` tree that is the shared starting point for every concrete
  system under it. `base_dev` is the existing de facto example.
- **Concrete system**: declares which base it inherits from and supplies only its
  *delta* — files it overrides plus anything it adds. Anything not overridden is
  resolved from the base at compile/register time.

**Depth is per-branch, not fixed at 4.** The diagram above shows the
*minimum* chain (a concrete system needs at least a base to extend); it is
not a depth cap. `extends:` names one system, and any system — including a
"subclass" node — is just another `system.yaml` with its own `extends:` and
`abstract: true`. Nothing about `class`/`subclass` is a structural level;
they're informational labels a node in the chain can carry. So the chain can
be as short as `base_dev → rust_dev` (1 hop, no subclass node needed) or as
long as `base_dev → frontend_dev → js_dev → react_dev` (3 hops, two abstract
intermediates) in the same tree, simultaneously, with other branches at
different depths. Inserting a new intermediate level later (e.g. splitting
`js_dev` out from under `frontend_dev` once more JS-family systems exist) is
a metadata edit — point the new node's `extends:` at `frontend_dev`, repoint
`react_dev`'s `extends:` at the new node — not a code or schema change. This
falls directly out of §5.2's `resolve_system()` being recursive on
`meta.extends` with no depth argument, and §5.3's `_chain` set having no
size limit — the mechanism is depth-agnostic from Phase 1, see §13.7.

### 3.2 Compile-time behavior (unchanged)

The base system is never compiled standalone. When a concrete system is compiled,
the loader first assembles an in-memory tree (base content + concrete overlay,
override-wins on path conflict), then runs the same 11-pass compile. Output
(`standards.db` rows) is identical in shape to a fully-standalone system. Only the
*source assembly step* changes.

### 3.3 Data-driven constraint

Inheritance is managed in YAML metadata files, not by adding class-aware logic
into the Python loader. The merge is generic and class-agnostic — the code reads
`extends:` and layers directories. Adding a new class means a new base directory
and systems pointing `extends:` at it; nothing in samgraha's source changes.

---

## 4. YAML Metadata Schema

### 4.1 Location

Each system directory gets a `system.yaml` alongside its existing files:

```
samgraha/system/{name}/system.yaml    # NEW — inheritance metadata
samgraha/system/{name}/plan/core/loop.yaml  # existing
samgraha/system/{name}/audit/         # existing
...
```

For base systems, the same file declares the class and optional `abstract` flag.

### 4.2 Schema

```yaml
# samgraha/system/{name}/system.yaml

# Required: class taxonomy
class: dev                    # dev | hackathon | academic | (future classes)

# Optional: subclass within the class
subclass: backend             # frontend | backend | fullstack | paper | journal | ...

# Required for concrete systems: which base to inherit from
extends: base_dev             # system name (directory under samgraha/system/)

# Optional: explicit list of paths this system overrides from the base.
# If omitted, the loader auto-detects by comparing the concrete tree against
# the base (any file present in concrete that also exists in base = override).
# Explicit list is preferred for documentation intent.
overrides:
  - documentation-standards/06-design-standards.md
  - documentation-standards/11-prototype-standards.md
  - audit/deterministic/document/design.yaml
  - audit/semantic/document/design.md

# Optional: paths to DROP from the base (not override, but remove entirely).
# The concrete system does not carry these files; they simply don't exist
# in the resolved tree. Use when a concrete system intentionally excludes
# a base domain or section.
drops:
  - documentation-standards/06-design-standards.md
  - audit/deterministic/document/design.yaml

# Optional: whether this system is abstract (base-only, not registrable standalone)
abstract: false               # default: false

# Optional: human-readable note about inheritance choices
note: |
  electron_dev extends base_dev but drops design/feature-design/prototype
  domains. loop.yaml and calculation/ inherited unchanged.
```

### 4.3 Format justification

YAML chosen over JSON because:
1. **Comments**: inline `#` and `note:` fields record *why* a value is what it is —
   a convention the entire `samgraha/system/` tree relies on (`loop.yaml`,
   `tiers.yaml`, `weights.yaml`, every `calculation/*.yaml`).
2. **Zero new parsing surface**: `knowledge-hub-loader.py` already calls
   `yaml.safe_load()` throughout (passes 1, 7, 8, 10) and never imports `json`.

### 4.4 Closed format inventory

| Format | Role | Existing in codebase |
|--------|------|---------------------|
| Markdown | All system content (documentation-standards, audit, templates) | Yes |
| YAML | New per-system metadata + all existing config files | Yes |
| TOML | `samgraha.toml` repo-level manifest | Yes |

No new formats introduced.

---

## 5. Preprocessor Merge Module

### 5.1 Design

A new Python module (`schema/knowledge-hub/system_merger.py`) that runs as a
preprocessing step *before* the existing `knowledge-hub-loader.py`. It:

1. Reads `system.yaml` from the target system directory
2. Resolves the inheritance chain (base → concrete, possibly multi-level)
3. Assembles a merged directory tree in a temp location (or in-memory via
   a virtual filesystem layer)
4. Hands the merged tree to the existing loader — which sees exactly the same
   directory shape it sees today

### 5.2 Merge algorithm

```python
def resolve_system(system_dir: Path) -> Path:
    """Resolve inheritance for a system, returning a merged directory tree.
    
    If the system has no system.yaml or no extends field, returns the
    original system_dir unchanged (backward compatible).
    """
    meta = load_system_metadata(system_dir)
    if not meta.extends:
        return system_dir  # no inheritance, pass through
    
    # Resolve base recursively (supports multi-level inheritance)
    base_dir = find_system_dir(meta.extends)
    base_merged = resolve_system(base_dir)  # recursive
    
    # Build merged tree: base first, then overlay concrete
    merged = tempfile.mkdtemp(prefix=f"samgraha-merge-{system_dir.name}-")
    
    # 1. Copy base tree into merged dir
    shutil.copytree(base_merged, merged, dirs_exist_ok=True)
    
    # 2. Overlay concrete system's files (override-wins)
    for item in system_dir.iterdir():
        if item.name == "system.yaml":
            continue  # metadata not part of the content tree
        dest = Path(merged) / item.name
        if item.is_dir():
            shutil.copytree(item, dest, dirs_exist_ok=True)
        else:
            shutil.copy2(item, dest)
    
    # 3. Apply drops (remove paths declared in drops[])
    for drop_path in meta.drops:
        target = Path(merged) / drop_path
        if target.exists():
            if target.is_dir():
                shutil.rmtree(target)
            else:
                target.unlink()
    
    return Path(merged)
```

### 5.3 Circular dependency detection

```python
def resolve_system(system_dir: Path, _chain: set[str] = None) -> Path:
    meta = load_system_metadata(system_dir)
    if meta.extends:
        if _chain and meta.extends in _chain:
            raise CircularInheritanceError(meta.extends, _chain)
        chain = (_chain or set()) | {system_dir.name}
        # ... resolve base ...
```

### 5.4 In-memory merge (preferred over temp dirs)

For production use, the merge should operate on a virtual path mapping rather
than copying files to a temp directory. The loader's passes already accept
`Path` objects via the `layout` dict — the merger can return a custom `Path`
implementation that transparently resolves `base_content / override_content`
without materializing on disk.

Alternative (simpler, acceptable for Phase 1): use `tempfile.mkdtemp()`,
clean up after loader finishes. The loader runs in a single transaction
(Pass 0-10), so cleanup is straightforward.

---

## 6. Loader Integration

### 6.1 Preprocessing hook in `knowledge-hub-loader.py`

The merge happens at the top of `main()`, before any pass runs:

```python
def main():
    args = parse_args()
    
    # NEW: resolve inheritance before loading
    from system_merger import resolve_system, load_system_metadata
    meta = load_system_metadata(Path(args.knowledge_hub))
    if meta and meta.extends:
        merged_dir = resolve_system(Path(args.knowledge_hub))
        args.knowledge_hub = str(merged_dir)
        # ... cleanup merged_dir after passes complete ...
    
    # ... existing pass 0-10 logic unchanged ...
```

### 6.2 CLI integration

```
# Existing (unchanged):
samgraha knowledge publish --path samgraha/system/rust_dev

# With inheritance (transparent):
# Same command — loader detects system.yaml, resolves inheritance automatically
samgraha knowledge publish --path samgraha/system/rust_dev
```

No new CLI flags needed. Inheritance is transparent to the caller.

### 6.3 MCP integration

`register_standard` (`crates/mcp/src/adapter.rs:1609`) passes `--knowledge-hub`
to the Python loader. The Python loader handles inheritance internally. No Rust
changes needed for the basic case.

---

## 7. Rust-Side Schema Changes

### 7.1 Database schema additions

The `systems` table gains two columns to record inheritance metadata (for
querying/auditing, not for resolution — resolution happens at Python loader time):

```sql
-- Migration: add inheritance columns to systems table
ALTER TABLE systems ADD COLUMN parent_system_id INTEGER
    REFERENCES systems(id) ON DELETE SET NULL;
ALTER TABLE systems ADD COLUMN class_name TEXT;
ALTER TABLE systems ADD COLUMN subclass_name TEXT;
ALTER TABLE systems ADD COLUMN is_abstract INTEGER NOT NULL DEFAULT 0;
```

`parent_system_id` stores the resolved base system's ID (set during Pass 0 when
the loader processes inheritance). `class_name` and `subclass_name` are informational.
`is_abstract` prevents standalone registration.

### 7.2 `db_reader.rs` changes

Minimal — the existing queries work unchanged because inheritance is resolved
before the loader runs. The `parent_system_id` column is loaded for metadata
purposes but does not affect `StandardDefinition` projection.

Optional enhancement: add `parent_system_id` to the `systems` table display
in `list_standards` output so users can see inheritance relationships.

### 7.3 `register_standard` validation

Add a check in the Python loader's Pass 0:

```python
if meta and meta.is_abstract and not parent_of_some_concrete:
    # This is an abstract-only system being registered standalone
    raise RegistrationError(
        f"System '{system_name}' is marked abstract and cannot be "
        f"registered standalone. Register a concrete system that extends it."
    )
```

For Phase 1, abstract validation is soft (warning). Phase 3 makes it hard (error).

### 7.4 `SCHEMA_VERSION` bump — rebuild, not migrate

`standards.db` is a rebuildable cache, not a source of truth (source of truth
is each system's own tree under the knowledge-hub). No `ALTER TABLE` /
incremental migration needed or wanted — confirmed by the existing code that
`init_schema()` (`knowledge-hub-loader.py:39-56`) has no migration path at
all: it either no-ops (tables exist) or runs the numbered `.sql` files fresh
after `00-reset.sql` (`--reset` flag). No caller (`knowledge_publish.rs`)
ever passes `--reset` today. This is fine under the cache model — the
process for this bump is:

1. Add the 4 columns directly to `01-systems.sql`'s `CREATE TABLE` (not an
   `ALTER TABLE` migration file).
2. Bump `SCHEMA_VERSION` (loader) and `EXPECTED_SCHEMA_VERSION`
   (`db_reader.rs:10`) from `1` to `2`.
3. Every system must be **re-registered from scratch** against a
   `--reset` (or fresh) db — there is no in-place upgrade of an existing v1
   `standards.db`. Document this as an explicit rollout step: run
   `register_standard`/`knowledge publish` for all 8 systems in one
   coordinated session, not incrementally.
4. `db_reader.rs`'s hard version-mismatch check already makes a stale v1 db
   fail loudly rather than silently — acceptable, since rebuilding is cheap
   and the check exists precisely to force the rebuild instead of a confusing
   downstream query error.

---

## 8. Migration of Existing Systems

### 8.1 dev class

Current state: 5 standalone systems, `loop.yaml` byte-identical across all.

Target state:

```
samgraha/system/
  base_dev/
    system.yaml          # class: dev, abstract: true
    plan/core/loop.yaml  # canonical loop
    calculation/          # canonical calculation rules
    audit/                # canonical audit rules
    documentation-standards/  # full 16-domain set
    templates/            # full template set
    script/               # canonical scripts
  
  frontend_dev/            # OPTIONAL abstract intermediate — not required,
    system.yaml            # shown here to prove variable depth per branch
    # class: dev, subclass: frontend, abstract: true, extends: base_dev
    documentation-standards/
      06-design-standards.md          # override: frontend-specific design doc
    # everything else inherited from base_dev unchanged

  js_dev/                  # a SECOND abstract intermediate, nested under
    system.yaml            # frontend_dev — proves depth isn't capped at 1
    # class: dev, subclass: frontend, abstract: true, extends: frontend_dev
    documentation-standards/
      09-feature-design-standards.md  # override: JS-ecosystem-specific

  electron_dev/
    system.yaml          # class: dev, subclass: frontend, extends: js_dev
    # 3 hops: base_dev -> frontend_dev -> js_dev -> electron_dev
    documentation-standards/
      11-prototype-standards.md       # omitted (dropped)
    # audit/, calculation/, templates/, script/ inherited unchanged

  react_dev/
    system.yaml          # class: dev, subclass: frontend, extends: js_dev
    # 3 hops, same intermediate chain as electron_dev, different leaf delta
    documentation-standards/
      # Only file that DIFFERS from js_dev:
      11-prototype-standards.md       # omitted

  fastapi_dev/
    system.yaml          # class: dev, subclass: backend, extends: base_dev
    # 1 hop — no backend intermediate needed yet; add one later (e.g.
    # backend_dev) by repointing extends: without touching this file's content
    documentation-standards/
      # Only files that DIFFER:
      06-design-standards.md          # omitted
      09-feature-design-standards.md  # omitted
      11-prototype-standards.md       # omitted
  
  rust_dev/
    system.yaml          # class: dev, subclass: backend, extends: base_dev
    # 1 hop, same depth choice as fastapi_dev today
    documentation-standards/
      # Only files that DIFFER:
      06-design-standards.md          # omitted
      09-feature-design-standards.md  # omitted
      11-prototype-standards.md       # omitted
```

`electron_dev`/`react_dev` resolve through 3 hops (`base_dev → frontend_dev →
js_dev → leaf`); `fastapi_dev`/`rust_dev` resolve through 1 (`base_dev →
leaf`) — same mechanism, same merge code, different chain length per branch.
If a `backend_dev` intermediate becomes useful later, it's inserted the same
way `frontend_dev`/`js_dev` were: a new abstract `system.yaml`, and
`fastapi_dev`/`rust_dev`'s `extends:` repointed at it.

### 8.2 academic class

```
samgraha/system/
  base_academic/
    system.yaml          # class: academic, abstract: true
    plan/core/loop.yaml  # canonical academic loop (semantic-only, no deterministic)
    calculation/          # semantic-only calculation rules
    audit/                # semantic-only audit
    documentation-standards/  # base academic domain set
    templates/            # academic templates
  
  eswa_journal/
    system.yaml          # class: academic, subclass: journal, extends: base_academic
    plan/core/loop.yaml  # OVERRIDE: eswa-specific within_tier_ordering
    documentation-standards/  # OVERRIDE: eswa-specific domain files
    audit/semantic/document/  # OVERRIDE: eswa-specific rubrics
  
  pcems_2026/
    system.yaml          # class: academic, subclass: journal, extends: base_academic
    plan/core/loop.yaml  # OVERRIDE: pcems-specific within_tier_ordering
    documentation-standards/  # OVERRIDE: pcems-specific domain files
    audit/semantic/document/  # OVERRIDE: pcems-specific rubrics
```

### 8.3 hackathon class

No migration needed — `python_hackathon` is the sole system in its class.
Optionally create `base_hackathon` as an abstract base for future hackathon systems.

### 8.4 Migration verification

After migration, re-register each system and compare the resulting `standards.db`
against the pre-migration version. The merge must produce identical rows for every
table. A diff script compares:
- `SELECT * FROM domains WHERE standard_id = ? ORDER BY key`
- `SELECT * FROM rules WHERE standard_id = ? ORDER BY rule_key`
- `SELECT * FROM section_catalog WHERE domain_id IN (SELECT id FROM domains WHERE standard_id = ?)`
- (etc. for all 15+ tables)

---

## 9. Abstract Base Support

### 9.1 The gap

Under the current architecture, every directory under `samgraha/system/` must be
independently complete enough to pass all 10 loader passes. `base_dev` works as a
normal, if generic, system today. Making bases genuinely non-standalone requires
the `abstract: true` flag.

### 9.2 Implementation

- `abstract: true` in `system.yaml` means: this system is valid only as an
  inheritance source, never as a standalone registration target.
- The Python loader's Pass 0 checks `abstract` and refuses to compile/register
  if the system is being registered directly (not as a resolved base).
- `register_standard` MCP tool returns an error if given an abstract system path.
- `knowledge publish` CLI returns an error for the same case.

### 9.3 Phasing

Phase 1: `abstract` field parsed and stored, soft warning if registered standalone.
Phase 3: hard error on standalone registration of abstract systems.

---

## 10. Phase-Wise Implementation Plan

Reordered against §13/§14: a new Phase 0 runs first — it fixes a live,
independent bug (§13.1/§14.1) rather than building inheritance, and every
later phase re-registers systems repeatedly during testing, so the earlier
this lands the fewer times the bug can bite during this project's own
development. Phase 1 now also absorbs the two LOW-complexity items that
belong in the files it's already creating (§14.5's drops-check belongs in
the new `system_merger.py`; §9.3's abstract check is clarified to not need
the Phase-3 DB column). Phase 3 is reworded from "migration" to "rebuild"
per §7.4, and absorbs §14.2/§14.4. Phase 4 absorbs §14.6.

### Phase 0: Push-Safety Prerequisites (before Week 1) — NEW

**Goal**: Fix the confirmed skip-sync clobber (§13.1) before inheritance
work begins generating more `register_standard` calls to test with. None of
this depends on inheritance — it's an existing-code fix, independent of
everything else in this plan, and cheap enough to not delay Phase 1 by more
than a couple days.

**Files to create/modify**:

| File | Action | Description |
|------|--------|-------------|
| `crates/services/src/knowledge_publish.rs` | MODIFY | Add `check_push_safe(local_db, global_db)` (§14.1): diff `systems.name` sets, refuse if global has names local doesn't |
| `crates/mcp/src/adapter.rs` | MODIFY | Call `check_push_safe` before both `std::fs::copy` sites (`handle_register_standard`, `handle_push_standards`) |
| `crates/mcp/Cargo.toml` | MODIFY | Add `serde_yaml.workspace = true` (already a workspace dep via `audit`/`schemas`, just not direct in `mcp` yet) — needed by Phase 1's abstract check too, added here so it's available from the start |

**Verification**:
- Push with global containing a system missing from local → refused, clear error naming the missing system(s)
- Push with local a superset of global (normal case) → succeeds unchanged
- Existing `register_standard`/`push_standards` tests still pass

**Deliverables**:
- Skip-sync clobber can no longer happen silently
- `serde_yaml` wired into `mcp` crate for Phase 1's Rust-side abstract check

---

### Phase 1: Metadata Schema + Preprocessor (Week 1-2)

**Goal**: Inheritance works for the dev class. No Rust *schema* changes (the
Phase-0 Cargo.toml wiring already landed). No loader-pass changes — only
the new preprocessing step.

**Files to create/modify**:

| File | Action | Description |
|------|--------|-------------|
| `schema/knowledge-hub/system_merger.py` | CREATE | Merge module: `load_system_metadata()`, `resolve_system()`, circular detection (§5.2/5.3) |
| `schema/knowledge-hub/system_merger.py` | CREATE (same file) | Post-merge validation (§14.5): cross-check `00-domain-relationships.md` edges against domains still present after `drops:` — fail with a clear error instead of letting Pass 1 hit a dangling `enforce_order` edge |
| `schema/knowledge-hub/system_merger.py` | CREATE (same file) | Track every tempdir `resolve_system()` creates across recursion levels, not just the leaf (§14.8) — a list/`ExitStack` threaded through the recursive calls, not a fresh `mkdtemp()` untracked at each level |
| `schema/knowledge-hub/knowledge-hub-loader.py` | MODIFY | Add preprocessing hook in `main()` to call `resolve_system()` before Pass 0, wrapped in `try/finally` so every tracked tempdir is cleaned up whether the passes succeed or crash (§14.8); Pass 0 soft-warns on `abstract: true` registered standalone (§9.3 — no DB column needed yet, this is a load-time check only, the column lands in Phase 3) |
| `crates/mcp/src/adapter.rs` | MODIFY | `handle_register_standard`: read `system.yaml` (via the `serde_yaml` dep Phase 0 added) and **warn** (not block, matching Python's Phase-1 soft policy below) when `abstract: true` — Phase 3 upgrades this to a hard refuse in lockstep with Python's Pass 0 (§14.3/13.6) |
| `samgraha/system/base_dev/system.yaml` | CREATE | `class: dev, abstract: true` |
| `samgraha/system/electron_dev/system.yaml` | CREATE | `class: dev, extends: base_dev, drops: [...]` |
| `samgraha/system/fastapi_dev/system.yaml` | CREATE | `class: dev, extends: base_dev, drops: [...]` |
| `samgraha/system/react_dev/system.yaml` | CREATE | `class: dev, extends: base_dev, drops: [...]` |
| `samgraha/system/rust_dev/system.yaml` | CREATE | `class: dev, extends: base_dev, drops: [...]` |

**Verification**:
- `python knowledge-hub-loader.py --db test.db --system rust_dev --knowledge-hub samgraha/system/rust_dev --dry-run` succeeds
- Output identical to pre-inheritance `rust_dev` registration
- Remove `electron_dev/plan/core/loop.yaml` — inheritance resolves it from `base_dev`
- Change `base_dev/plan/core/loop.yaml` — `electron_dev` picks up the change automatically
- A `drops:` entry that leaves a dangling `domain_relationships` edge fails the merge with a clear message, not Pass 1's opaque error
- `register_standard` on `base_dev`'s path still succeeds (Phase 1 is soft) but the response includes a warning, surfaced before the Python subprocess even runs
- A 3-level chain leaves zero tempdirs behind after a successful run, and zero after a forced failure mid-pass (kill a pass partway, check the OS temp dir)

**Deliverables**:
- Working merge module with circular detection + drops referential-integrity check
- 5 YAML metadata files for dev class
- Loader preprocessing hook with guaranteed tempdir cleanup across all recursion levels, success or failure
- Rust-side + Python-side abstract soft-warning (both upgrade to hard refuse together in Phase 3)
- Merge verification test (diff old vs new `standards.db`)

---

### Phase 2: Academic + Hackathon Classes (Week 2-3)

**Goal**: Migrate `eswa_journal`, `pcems_2026`, `python_hackathon` to inheritance.

**Files to create/modify**:

| File | Action | Description |
|------|--------|-------------|
| `samgraha/system/base_academic/system.yaml` | CREATE | `class: academic, abstract: true` |
| `samgraha/system/eswa_journal/system.yaml` | CREATE | `class: academic, extends: base_academic` |
| `samgraha/system/pcems_2026/system.yaml` | CREATE | `class: academic, extends: base_academic` |
| `samgraha/system/python_hackathon/system.yaml` | CREATE | `class: hackathon` (no extends — sole system) |
| `samgraha/system/base_academic/` | CREATE | Extract shared academic content from eswa/pcems |

**Verification**:
- Academic systems resolve correctly with semantic-only audit (no deterministic)
- Each system's `loop.yaml` override produces correct `within_tier_ordering`
- `python_hackathon` unchanged (no inheritance, standalone)
- Re-register all 3, diff against pre-migration DBs

**Deliverables**:
- `base_academic` abstract base with shared academic content
- 3 YAML metadata files for academic + hackathon classes
- Academic system verification tests

---

### Phase 3: Schema Rebuild + Rust Integration (Week 3-4)

**Goal**: Inheritance metadata visible in DB and Rust-side. Per §7.4, this is
a **rebuild, not a migration** — `standards.db` is a cache, so the 5 new
columns go straight into `01-systems.sql`'s `CREATE TABLE`, `SCHEMA_VERSION`
bumps, and every system gets re-registered from scratch in one coordinated
session. No `ALTER TABLE`, no in-place upgrade path.

**Files to create/modify**:

| File | Action | Description |
|------|--------|-------------|
| `schema/knowledge-hub/01-systems.sql` | MODIFY | Add `parent_system_id`, `class_name`, `subclass_name`, `is_abstract` (inheritance, §7.1) **and** `last_registered_by` (name-collision provenance, §14.2) to the `CREATE TABLE` directly — one rebuild event covers both |
| `schema/knowledge-hub/knowledge-hub-loader.py` | MODIFY | `SCHEMA_VERSION` 1→2; Pass 0 populates the new columns from `system.yaml`; Pass 0's abstract check upgraded from soft warning to hard error |
| `crates/standards/src/db_reader.rs` | MODIFY | Bump `EXPECTED_SCHEMA_VERSION` to 2 (line 10); load new columns |
| `crates/schemas/src/standard.rs` | MODIFY | Add the new fields to `SystemDefinition` (or equivalent) |
| `crates/mcp/src/adapter.rs` | MODIFY | `handle_register_standard`'s Phase-1 warning upgraded to a hard refuse (in lockstep with Python); stamp `last_registered_by` after a successful push; `list_standards`: display inheritance info + surface a name-collision warning when `check_push_safe` (Phase 0) finds a shared name with a different `last_registered_by` |

**Rollout steps** (order matters, since bases must exist before concretes
re-register against them):
1. `--reset` (or fresh) DB, load the updated schema
2. Re-register `base_dev` and `base_academic` **first**, with `abstract: true`
   — confirms `is_abstract` lands correctly for the bases before anything
   extends them (§13.4/14.4, closes the retrofit gap — no separate migration
   step needed beyond ordering)
3. Re-register all concrete systems (dev, academic, hackathon classes)
4. Row-diff against the pre-rebuild DB (§8.4) to confirm the rebuild is lossless

**Verification**:
- `list_standards` shows `parent_system_id` and `class_name` for all migrated systems
- `register_standard` on an abstract system returns a hard error, both
  Rust-side (fast) and Python-side (authoritative)
- Two systems pushed with the same name from different `last_registered_by`
  values surfaces a warning
- Schema version check passes (DB version 2, `EXPECTED_SCHEMA_VERSION` = 2)
- All existing tests pass with new schema

**Deliverables**:
- Schema rebuilt with inheritance + provenance columns (no migration tooling built — deliberately not needed)
- Rust schema changes
- Abstract registration guard, hard in both layers
- Name-collision warning wired to Phase 0's push-safety check
- Updated `SCHEMA_VERSION` / `EXPECTED_SCHEMA_VERSION`

---

### Phase 4: Multi-Level Inheritance + Edge Cases (Week 4-5)

**Goal**: Harden the multi-level resolution that already exists since Phase 1
(`resolve_system()` is recursive on `extends:` with no depth limit — see
§3.1, §8.1's `frontend_dev`/`js_dev` example) and cover its edge cases.
Nothing here is "add support for N levels" — that mechanism ships in Phase 1.
This phase adds the safety net around it.

**Features**:
- Diamond inheritance detection (A extends B, A extends C, B extends C) —
  the one shape the recursive resolver does *not* reject on its own
- Override validation: warn if a concrete system overrides a file not present in base
- Drop validation: warn if a concrete system drops a path not present in base
- `--dry-run` flag on loader to show resolved tree without writing
- **Class-shape validation (§13.3/14.6)**: a small `class_shapes.yaml`
  (per `class`, allowed `stages:` names and `calculate.scope` values) checked
  against the *resolved* `loop.yaml` post-merge. Deliberately class-aware
  logic — §3.3's "class-agnostic" constraint is scoped to the merge
  mechanism, not to this validation pass. This is the direct fix for the
  root cause the limitation doc named for the `eswa_journal`/`pcems_2026`
  incident, as defense-in-depth on top of what inheritance already prevents
  at the source.

**Files to create/modify**:

| File | Action | Description |
|------|--------|-------------|
| `schema/knowledge-hub/system_merger.py` | MODIFY | Multi-level resolution hardening, diamond detection, validation warnings |
| `schema/knowledge-hub/class_shapes.yaml` | CREATE | Per-class allowed `stages:`/`calculate.scope` shape table |
| `schema/knowledge-hub/knowledge-hub-loader.py` | MODIFY | `--dry-run` shows resolved inheritance tree; class-shape check runs post-merge, pre-Pass-0 |
| Tests | CREATE | Multi-level, circular, diamond, override-validation, class-shape-mismatch tests |

**Verification**:
- 3-level inheritance chain resolves correctly
- Diamond inheritance detected and rejected with clear error
- Override/drop of non-existent paths produces warnings (not errors)
- `--dry-run` output shows which files came from base vs override
- A system declaring `class: academic` whose resolved `loop.yaml` has
  `calculate.scope: competition` fails with a clear class-mismatch error
  (the exact shape of the original `eswa_journal`/`pcems_2026` incident)

**Deliverables**:
- Multi-level resolution hardening
- Diamond detection
- Override/drop validation warnings
- Class-shape validation
- `--dry-run` inheritance display

---

### Phase 5: Cleanup + Documentation (Week 5-6)

**Goal**: Remove duplicated files from concrete systems, document the system.

**Actions**:
- Remove `loop.yaml` from `electron_dev`, `fastapi_dev`, `react_dev`, `rust_dev`
  (inherited from `base_dev`)
- Remove identical `calculation/` files from concrete dev systems
- Remove inherited `script/` directories where unchanged
- Update `README.md` with inheritance documentation
- Update `docs/proposal.md` or create `docs/inheritance.md`
- Add inheritance section to knowledge-hub system authoring guide, covering:
  creating new systems, choosing a base, overrides, drops, class shapes
  (§4/8's how-to) **and** the multi-repo push-safety behavior from Phase 0
  (what `check_push_safe` blocks, what a name-collision warning means,
  when to run `sync_standards` first) — this is existing behavior once
  Phase 0 ships, not new to this phase, but undocumented until now

**Verification**:
- All concrete systems register correctly with reduced file sets
- No functional regressions (full test suite passes)
- Documentation covers: creating new systems, choosing base, overrides, drops

**Deliverables**:
- Cleaned-up system directories (reduced duplication)
- Inheritance documentation
- Updated authoring guide

---

## 11. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `register_standard` push overwrites shared store when caller skipped `sync_standards` first | HIGH — confirmed live bug (§13.1), not hypothetical | `check_push_safe` guard, Phase 0, before any inheritance work starts |
| Two repos register a same-named system with different content | MEDIUM — silent overwrite, no ownership tracking today | `last_registered_by` column + collision warning, Phase 3 (§14.2) |
| Merge produces different DB than standalone | HIGH — silent data loss | Diff verification in Phase 1-2; compare every table row |
| Circular inheritance | HIGH — infinite loop | `_chain` set tracking in `resolve_system()`; clear error |
| Diamond inheritance | MEDIUM — ambiguous resolution | Detect and reject in Phase 4; require linear chains |
| Temp dir disk usage | LOW — dirs cleaned after loader | Deferred until measured (§14.7/13.5) — Phase 1 copytree cost isn't a known bottleneck yet |
| Schema rebuild loses existing systems if rollout order is wrong | MEDIUM — re-registration required regardless (§7.4), but abstract bases must go first | Explicit rollout order in Phase 3 (bases before concretes); row-diff verification |
| `abstract` base accidentally registered | LOW — produces valid but generic system | Soft warning both layers Phase 1; hard refuse both layers Phase 3 |
| Override intent unclear (which files differ?) | MEDIUM — maintenance confusion | Explicit `overrides:` list in `system.yaml`; `--dry-run` display |
| Dropped domain leaves a dangling `domain_relationships` edge | MEDIUM — opaque Pass 1 error instead of a clear one | Post-merge referential check in `system_merger.py`, Phase 1 (§14.5) |
| Hand-edited override reintroduces wrong-class shape (e.g. `scope: competition` in an academic system) | MEDIUM — same failure mode as the original eswa/pcems incident, just via override instead of copy-paste | Class-shape validation, Phase 4 (§14.6) |

---

## 12. Success Criteria

1. Changing `base_dev/plan/core/loop.yaml` once propagates to all 4 child dev systems
   without manual copy-paste
2. Creating a new dev system requires only `system.yaml` + override files
3. `register_standard` on an abstract system returns a clear error
4. All 8 existing systems produce identical `standards.db` output after the
   Phase 3 rebuild (verified by row-level diff), with bases re-registered
   before concretes so `is_abstract` is correct from the first rebuild
5. No changes to the 11-pass loader logic — inheritance is purely a preprocessing step
6. The merge module is class-agnostic — adding a future `ml_ops` class requires only
   a new base directory + metadata files, no code changes
7. `register_standard` refuses to push when the shared store has systems
   missing from the caller's local db (§14.1) — the skip-sync clobber can no
   longer happen silently
8. A hand-edited override that reintroduces the wrong class's `loop.yaml`
   shape (the original eswa/pcems failure mode) is caught by class-shape
   validation (§14.6), not just by inheritance removing the copy-paste path

---

## 13. Gaps Against LIM-001/LIM-002 (checked against current implementation)

Verified against `E:\Python\Kriti\docs\limitation\mcp\01-system-class-inheritance.md`
and the current code (`schema/knowledge-hub/knowledge-hub-loader.py`,
`crates/standards/src/db_reader.rs`). Six gaps not covered above.

### 13.1 LIM-002 confirmed as a live bug, not a hypothesis — `register_standard` clobbers the shared store

Traced the actual push path: `handle_register_standard`
(`crates/mcp/src/adapter.rs:1609-1668`) runs the Python loader against
*repo-local* `.samgraha/standards.db`, then does `std::fs::copy(&local_db,
&global_db)` — a full-file overwrite of the shared store, not a merge.
`register_standard` never requires (or triggers) a prior `sync_standards`
pull. So: repo A registers `rust_dev` and pushes → global has `rust_dev`.
Repo B, never having synced, registers `react_dev` on its own empty local db
and pushes → global now has *only* `react_dev`; `rust_dev` is gone. This is
LIM-002's "second repo" concern, confirmed in code, independent of this
proposal — but §7's schema bump increases the blast radius, since a
freshly-`--reset`, v2-only local db pushed by whichever repo migrates first
wipes every other repo's still-v1 systems in one call.

The "standards.db is a rebuildable cache" framing (§7.4) means this isn't
*data-loss* in the durable-state sense — everything can be re-registered from
source. But it's still a correctness bug for day-to-day multi-repo use
(outside of a migration event), and worth fixing before or alongside this
proposal rather than after: either (a) `register_standard`'s push step
refuses to overwrite `global_db` if `global_db` already has systems missing
from `local_db` (a name-set comparison, cheap), or (b) push writes/updates
only this system's rows into `global_db` directly (attach + `INSERT OR
REPLACE` across the two files) instead of a whole-file copy.

**Scoped per user decision**: concurrent-push races (two repos pushing at
the same moment) are out of scope for now — user-managed today, one push at
a time over stdio; enforce later if it becomes a real problem. What still
needs the fix above is the **skip-sync clobber**, which needs no concurrency
at all — a single repo pushing without syncing first overwrites the shared
store on its own, sequentially, every time. That's the one worth guarding
before/alongside this proposal; the race case can wait.

---

### 13.2 `drops[]` has no referential-integrity check against `domain_relationships`

`pass_1` (`knowledge-hub-loader.py:300-313`) raises a hard error if
`enforce_order` is set on a relationship edge with no matching
`domain_relationships` row. `00-domain-relationships.md` declares edges
between domain keys; if a concrete system's `drops:` removes a domain file
(e.g. `06-design-standards.md`) but the inherited `00-domain-relationships.md`
still references that domain key, the merged tree fails Pass 1 with an
opaque error, not a clear "you dropped a domain still referenced by an edge"
message. §5.2's merge algorithm and §8.1's dev-class drop list don't mention
this — the merger needs to either reject drops that leave dangling
relationship edges, or auto-strip matching edges, with a clear error either way.

### 13.3 No class-shape validation — only file-identity inheritance

The limitation doc's evidence section names the actual fix for the
`eswa_journal`/`pcems_2026` incident: "even just a `class:` field the loader
could validate the `stages:`/`path_selection:` shape against." This proposal's
`system.yaml` carries `class`/`subclass` (§4.2) but no pass ever checks that
a system's `loop.yaml` stage shape or scoring type actually matches its
declared class. Inheritance alone only prevents *re-copying* the wrong file;
it doesn't stop a `system.yaml` that says `class: academic` while `extends:`
points at a `hackathon`-shaped base, or a hand-written override that
reintroduces `scope: competition` into an academic system. Add a lint pass
(Phase 3 or 4) that checks resolved `stages:`/`calculate.scope` against a
known shape table per class.

### 13.4 Migration section doesn't retire the already-registered standalone `base_dev`

`base_dev` is confirmed (per the limitation doc) already registered today as
an ordinary, directly-registrable system — `is_abstract` doesn't exist yet.
§8's migration steps create `system.yaml` files but never call out
re-registering the bases with `abstract: true` so the *existing* `systems`
row flips from standalone to abstract, nor whether any existing consumer that
picked `base_dev` directly needs a deprecation notice. Add this as an
explicit migration step, and add `is_abstract` transition (0→1) to the §8.4
row-diff verification.

### 13.5 Merge cost is unoptimized for the common case (base unchanged)

§5.1-5.2's Phase 1 merge does a full `shutil.copytree` of the resolved base
tree into a fresh tempdir on *every* `register_standard`/`knowledge publish`
call, even when the base hasn't changed since the last call — likely to be
the majority of calls during iterative authoring of a single concrete
system. §5.4 already names the fix (virtual-path layer, no materialization)
as "preferred" but defers it past Phase 1. Given how often `register_standard`
gets called during normal authoring (confirmed by this repo's own MCP tool
list — `register_standard`, `sync_standards`, `compile` are all separate,
frequently-invoked calls), recommend pulling a cheap version forward: hash the
base tree once, cache the resolved merge keyed by that hash, invalidate only
when the base's content hash changes.

### 13.6 Abstract check should exist Rust-side too, not just in the Python loader

§9.2 puts the `abstract` standalone-registration guard only in the Python
loader's Pass 0. `register_standard` (`crates/mcp/src/adapter.rs:1609`) and
`knowledge publish` (`crates/cli/src/commands.rs:1436`) both shell out to the
Python loader regardless — meaning an abstract-system registration attempt
still pays the full subprocess spawn + partial-pass cost before failing.
Since `system.yaml` is a plain YAML file, the Rust side can cheaply check
`abstract: true` before invoking Python and fail fast with the same error
message, matching the two-layer pattern the codebase already uses for other
guards (e.g. schema-version check happens Rust-side before DB use).

### 13.7 N-level nesting: mechanism is depth-agnostic, doc's examples weren't

Confirmed the design doesn't cap chain depth or force uniform depth across
branches — `resolve_system()` (§5.2) recurses on `meta.extends` with no depth
parameter, and `_chain` (§5.3) is a set with no size limit, so a 1-hop branch
(`fastapi_dev extends base_dev`) and a 3-hop branch
(`react_dev extends js_dev extends frontend_dev extends base_dev`) resolve
through the identical code path. §3.1 and §8.1 were updated to say this
explicitly and show a mixed-depth example — previously the taxonomy diagram
and every migration example showed exactly one hop, which reads as a depth
cap even though the underlying mechanism never had one. Practical
consequence: adding, removing, or re-slotting an intermediate abstract level
later (e.g. splitting `js_dev` out from `frontend_dev` once more
JS-ecosystem systems exist) is purely a `system.yaml` edit — new abstract
node, repoint the affected leaves' `extends:` — no loader or schema change,
consistent with §3.3's data-driven constraint.

### 13.8 Temp-dir cleanup is missing on the happy path for multi-level chains, not just on error

User-flagged, confirmed against §5.2's actual pseudocode and sharper than
first framed. `resolve_system()` is recursive and calls
`tempfile.mkdtemp()` at *every* level of the chain that itself has an
`extends:`, but only the leaf-most call's path is returned up the stack.
For a 3-hop chain (`base_dev → frontend_dev → js_dev → react_dev`),
resolving `react_dev` creates a tempdir for `frontend_dev`'s merge and
another for `js_dev`'s merge — both get copied into the next level and then
never referenced again by anything outside that stack frame. Those two leak
**every time**, success or failure, not just on a mid-pass crash. On top of
that, §6.1's `main()` sketch shows only a comment
(`# ... cleanup merged_dir after passes complete ...`) where cleanup code
should be — as literally written, there's no cleanup at all yet, happy path
included. The error-path gap (no `try/finally` around the loader passes) is
real too, just the smaller half of this.

---

## 14. Recommended Fixes, Ranked by Implementation Complexity

Checked each open item (§13) against what's actually in this codebase today
(workspace deps, existing helper locations, existing patterns) rather than
proposing in the abstract. §10's phase plan already incorporates all seven
of these — this section is the complexity rationale behind that placement.

### 14.1 LOW — skip-sync clobber (§13.1) — do this one, Phase 0

Add one shared helper, `services::knowledge_publish::check_push_safe(local_db,
global_db) -> Result<()>`, next to the existing `resolve_knowledge_hub_loader`.
Opens both DBs (if `global_db` exists), `SELECT name FROM systems` on each,
diffs the sets. If `global_db` has a name `local_db` doesn't, refuse with
"global has systems {…} missing from your local db — run sync_standards
before pushing." Call it right before both existing `std::fs::copy` sites
(`adapter.rs:1668` in `handle_register_standard`, `adapter.rs:1707` in
`handle_push_standards` — same copy logic already duplicated between the
two, worth collapsing into one call to the new helper from both). ~30 lines,
one new function, two call-site edits, one test with a fixture global/local
pair. No new dependency, no schema change.

### 14.2 LOW — name collision across repos (found, not yet in §13), Phase 3

Piggyback on 14.1's helper: for names present in *both* sets, nothing today
records who last registered a given system, so a same-named-different-content
collision can't even be detected, only guessed at. Since §7.1 already has a
rebuild-from-scratch schema change queued for the inheritance columns, add
one more nullable column to the same `01-systems.sql` `CREATE TABLE` pass —
`last_registered_by TEXT` (hostname + repo path, populated by
`handle_register_standard` at Pass 0 time isn't possible since Pass 0 is
Python-side; simplest is the Rust push step stamping it via a plain
`UPDATE systems SET last_registered_by = ? WHERE name = ?` right after the
copy). 14.1's diff then reports "same name, different `last_registered_by`
last time" as a warning, not a hard block (avoids false positives when one
repo legitimately re-registers its own system). Bundled into the same
rebuild event as §7.1 — no second migration window.

### 14.3 LOW — Rust-side abstract fast-fail (§13.6), Phase 1 (soft) → Phase 3 (hard)

`serde_yaml` is already a workspace dependency (`crates/audit/Cargo.toml`,
`crates/schemas/Cargo.toml`), and `crates/mcp` already depends on both
`audit` and `schemas` (`crates/mcp/Cargo.toml:8,11`) — just not `serde_yaml`
directly yet. Add `serde_yaml.workspace = true` to `crates/mcp/Cargo.toml`
(one line, no new dependency to vet, already locked at the workspace level),
then in `handle_register_standard` read `path.join("system.yaml")` if it
exists and check `abstract: true` before invoking the Python loader at all.
Real YAML parse, not a regex guess — cheap given the dependency is already
in the graph. Phase 1 makes this a warning (matching Python's soft phase);
Phase 3 upgrades both to a hard refuse together.

### 14.4 LOW/none — abstract retrofit for `base_dev` (§13.4), Phase 3

Not code — collapses into a rollout checklist item once §7.4's "rebuild, not
migrate" decision is in effect. Re-register `base_dev` (and `base_academic`
once created) with `abstract: true` as the *first* step of the coordinated
re-registration pass, before any concrete system. `is_abstract` lands
correctly for free because it's read from `system.yaml` the same way every
other field is. Zero new code.

### 14.5 MEDIUM — `drops[]` referential check against `domain_relationships` (§13.2), Phase 1

Don't try to auto-edit `00-domain-relationships.md` text to strip dangling
edges — text surgery on a hand-authored Markdown+YAML file is fragile and
hides real modeling mistakes. Instead, add a validation step in
`system_merger.py`, after the merge and before handing the tree to the
loader: parse the merged `00-domain-relationships.md`'s YAML block (reuse
the same `extract_yaml_block`/regex approach `pass_1` already uses,
`knowledge-hub-loader.py:161-192`) and cross-check every edge's domain keys
against the domain files actually present post-drop. Fail with a clear
"drop of `domain X` leaves edge `X→Y` dangling" instead of letting Pass 1's
existing opaque error (`knowledge-hub-loader.py:300-313`) surface later.
New function (~40 lines) in the already-new `system_merger.py`, no changes
to the loader itself.

### 14.6 MEDIUM, deferred — class-shape validation (§13.3), Phase 4

Genuinely new logic, and deliberately class-aware (the one place that's
fine — §3.3's "class-agnostic" constraint is scoped to the *merge*
mechanism, not to every validation pass). Needs a small shape table (e.g.
`class_shapes.yaml`: per class, allowed `stages:` names and
`calculate.scope` values) plus a comparison function against the resolved
`loop.yaml`. Not a Phase 1 blocker — inheritance itself already closes most
of this hole at the source (a system that extends the right base can no
longer accidentally copy-paste the wrong `loop.yaml`); this is
defense-in-depth for hand-written overrides, not a prerequisite.

### 14.7 NONE for now — merge caching (§13.5), not scheduled

Ponytail call: skip it. Phase 1's `copytree` over a few MB of docs/YAML is
low-hundreds-of-ms, not a measured bottleneck — building a content-hash
cache now is optimizing a number nobody's collected yet. Revisit only if
real iterative-authoring use shows `register_standard` latency actually
hurts; the fix (hash base tree, cache resolved merge dir keyed by hash)
is still the right one *if* that day comes, just not today. Deliberately
absent from §10 for this reason.

### 14.8 LOW — temp-dir cleanup for all recursion levels, not just the leaf (§13.8), Phase 1

Thread a mutable list (or `contextlib.ExitStack`) through `resolve_system()`'s
recursive calls — every `tempfile.mkdtemp()` call anywhere in the chain
appends its path to the same list, passed by the caller, not created fresh
per call. `main()` wraps the top-level `resolve_system()` call and the
subsequent loader passes in `try/finally`, and the `finally` block
`shutil.rmtree()`s every path in that list, in reverse order (child before
parent, though order doesn't actually matter once nothing points into a
child from outside it). Fixes both halves of §13.8 at once — the same
`finally` block that guarantees cleanup on a mid-pass crash also guarantees
every intermediate level gets cleaned on success, not just the final one.
~15-20 lines in `system_merger.py` + `main()`, no new dependency
(`contextlib` is stdlib).
