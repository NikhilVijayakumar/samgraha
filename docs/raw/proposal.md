# Proposal: Gap Analysis — External Repository Compilation

## Investigation Scope

Why does `samgraha compile` on an external repo (Heimdall) produce lower-quality
compilation than compiling Samgraha's own documentation? The user reports:

1. Domain assignment doesn't match (`docs/raw/` expected, flat `docs/` used)
2. No depth or atomicity in the compiled output
3. Built-in domains (`audit`, `audit-standards`, `standards`) incorrectly appear
4. The quality/validation checks that Samgraha's own docs pass are missing

## Trace: Compilation Flow

```
CLI `samgraha compile` / MCP `compile`
  → KnowledgeRuntime::compile()
    → CompilationService::execute(repository_root, config, ...)       [1]
      → CompilationPipeline::compile(repository_root, standards, ...) [2]
        → DiscoveryEngine::discover(repository_root, ...)              [3]
          → for each .md file: infer_standard(relative_path)           [4]
```

## Findings (7 Gaps)

### Gap 1 — `root_dir` is never consumed

`config.repository.documentation.root_dir` is set in `samgraha.toml` (e.g.
`"${SAMGRAHA_DOCS_DIR}"`) but is **never read** during compilation.

At step [1], `CompilationService::execute()` receives `root` (the repository
root). At step [2], `CompilationPipeline::compile()` receives this same
`root` and passes it directly to the discovery engine. The configured docs
root is ignored — the compiler always walks from the repository root.

**Evidence:** `CompilationService::execute()` at `crates/services/src/compilation.rs:84`:
```rust
let output = CompilationPipeline::compile(root, &standards, scope, &known_hashes, &ignore_patterns)?;
```
No call to `resolve_configured_dir()` for the docs root anywhere in the chain.

**Impact:** Flat docs like `docs/01-vision.md` are discovered relative to the
repo root, not under a `docs/raw/{domain}/` structure. The domain inference
(step [4]) operates on paths like `docs/01-vision.md` instead of
`raw/vision/vision.md`, leading to incorrect domain tagging.

---

### Gap 2 — `infer_standard()` fails on numbered file prefixes

`DiscoveryEngine::infer_standard()` at `crates/compiler/src/discovery.rs:66-116`
uses exact name matching against a hardcoded `domain_map`. A file stem of
`01-vision` does not match the key `"vision"`.

**Domain_map entries (excerpt):**
```rust
("vision", "vision"),
("philosophy", "philosophy"),
("system-overview", "architecture"),
```

**What happens for `docs/01-vision.md`:**
```
name  = "01-vision"       (file_stem)
parent = "docs"            (parent dir name)
→ DOMAIN_OVERRIDE check: "docs" not found
→ domain_map loop:
    "vision" key: does "01-vision" == "vision"? NO. Does "docs".contains("vision")? NO.
    "philosophy": NO. "engineering": NO. … (all fail)
→ parent.contains("architecture")? NO.
→ returns "docs"           (the parent directory — fallthrough)
```

The document is tagged with domain `"docs"`, which is not a registered
standard. The compiler processes it as generic — no `StandardDefinition`
is applied, so required sections, prohibited content, and audit rules
are never enforced.

**Impact on all flat docs:**

| File | Inferred Domain | Expected Domain |
|------|----------------|-----------------|
| `docs/01-vision.md` | `docs` | `vision` |
| `docs/02-philosophy.md` | `docs` | `philosophy` |
| `docs/03-domain-model.md` | `docs` | `architecture` |
| `docs/04-system-overview.md` | `docs` | `architecture` |
| `docs/10-roadmap.md` | `docs` | `feature` |

---

### Gap 3 — Non-standard parent directory names yield invalid domains

Files under `06-audits/`, `07-scoring/`, `08-reporting/`, `09-plugins/` have
parent directory names that don't match any known domain.

**What happens for `docs/06-audits/06-01-repository-discovery.md`:**
```
name  = "06-01-repository-discovery"
parent = "06-audits"
→ No domain_map match (parent doesn't contain "architecture" etc.)
→ returns "06-audits"
```

`"06-audits"` is not a registered standard. The document is compiled as generic.
Additionally, `"audits"` is semantically close to Samgraha's built-in `audit`
domain — yet no normalization maps it.

**Impact on subdirectory docs:**

| File | Inferred Domain | Expected |
|------|----------------|----------|
| `06-audits/*.md` | `06-audits` | `feature` |
| `07-scoring/*.md` | `07-scoring` | `feature` |
| `08-reporting/*.md` | `08-reporting` | `feature` |
| `09-plugins/*.md` | `09-plugins` | `feature` or `feature-technical` |

---

### Gap 4 — Scope filtering doesn't validate domain plausibility

`CompilationService::execute()` sets scope from `CompilationRequest`. When
the request is `CompilationScope::Repository` (the default), scope is `None`,
meaning **all discovered documents pass through** regardless of whether their
inferred domain is a registered standard.

At step [2]:
```rust
let filtered: Vec<DiscoveredDocument> = match scope {
    Some(domains) => discovered.into_iter().filter(|d| domains.contains(&d.standard)).collect(),
    None => discovered,
};
```

Documents with invalid domains like `"06-audits"`, `"docs"`, etc. are still
processed. They generate entries in the knowledge database, but those entries
are plain `Generic` body variants with no structured sections, no quality
validation, and no standard relationship resolution.

**Impact:** The knowledge DB fills with low-quality entries that have:
- No standard-driven section validation (missing required sections not flagged)
- No prohibited content checking (implementation details in vision docs)
- No relationship links between documents
- No semantic type extraction for sections
- Empty quality metrics (section counts, coverage ratio ~0)

---

### Gap 5 — No content-based domain fallback

`infer_standard()` uses only the file path. It never inspects the document
content (e.g. the H1 title like `# Vision` or front matter like
`Document ID: TAP-VISION-001`).

A file named `03-domain-model.md` under `architecture/` could be correctly
inferred, but `docs/03-domain-model.md` with parent `docs` has no way to
resolve to `architecture` because the content is never consulted.

**Impact:** Domain inference is path-dependent only. Repos using different
naming conventions (numbered prefixes, dashes, domain codes like `TAP-*`)
receive no domain signal from the content itself.

---

### Gap 6 — Hardcoded domain map is Samgraha-specific

The `domain_map` encodes Samgraha's own architectural document names:
```rust
("system-overview", "architecture"),
("component-model", "architecture"),
("communication", "architecture"),
("security-architecture", "architecture"),
...
```

These are Samgraha's specific filenames. Other repositories use different
names (e.g. Heimdall's `04-system-overview.md` or `03-domain-model.md` for
architecture). Even without the numbered prefix, `domain-model.md` would
not match any domain_map entry — it would fall through to parent-only
inference.

**Impact:** The domain_map is not a generic domain inference mechanism. It's a
hardcoded lookup table tuned to Samgraha's filenames. Any external repo with
different filenames gets no benefit from the name-based resolution and must
rely entirely on parent directory matching.

---

### Gap 7 — No `raw/` directory resolution

Samgraha's documentation lives under `docs/raw/{domain}/`. This `raw/` layer
provides correct parent-directory-based domain inference: a file at
`docs/raw/vision/vision.md` has parent `"vision"` → matches domain `"vision"`.

The compiler does **not** strip or handle the `raw/` prefix. It simply uses
whatever parent directory name is present after stripping the compile root.
For Samgraha, the compile root is the repo root, and paths look like
`docs/raw/vision/vision.md` → parent is `"vision"` → works.

For a repo configured with `root_dir = "docs"`, one would expect the compile
root to be `<repo>/docs`, not `<repo>`. But since `root_dir` is never
consumed (Gap 1), this resolution never happens. The parent directory for
`docs/vision/vision.md` would be `"vision"` — which would be correct. But for
`docs/01-vision.md` (flat), the parent is `"docs"`, which fails.

**Impact:** The `raw/` convention works for Samgraha but is invisible to the
compiler as a configurable path component. No code strips or normalizes it.
A repo that places docs directly in `docs/{domain}/` (without `raw/`) would
actually work for path-based inference — but the `root_dir` not being used
prevents this from being configured properly.

**Correction on framing:** `docs/raw/` is not an "expected" or consumed root —
`SAMGRAHA_DOCS_DIR` is unset in **both** repos' `.env`, so `root_dir` falls
back to `${repo}/docs` identically for Samgraha and Heimdall (confirmed via
`resolve_configured_dir`'s `"docs"` fallback and the `.env.example` comment).
Samgraha's own compile only works because its markdown happens to live at
`docs/raw/{domain}/file.md`, so the *immediate parent directory* is already a
domain name — regardless of where the compile root sits. Nothing in the
compiler reads or depends on the literal string `raw`. This should not be
read as "the compiler expects `docs/raw/`" — it should be read as "path-based
inference happens to work when parent-dir == domain-name, and Samgraha's own
layout accidentally satisfies that."

---

### Gap 8 — Built-in domain leakage is real, but not where symptom #3 describes it

Investigation Scope symptom #3 claims `audit`, `audit-standards`, `standards`
"incorrectly appear" as built-in domains. Verified against current code:

**`audit` — reproducible, and it affects Samgraha's own self-compile, not just external repos.**
`docs/raw/audit/` contains 19 real `.md` files (`vision-audit.md`,
`engineering-audit.md`, etc.). Samgraha's own `[repository.ignore].patterns`
excludes `audit-standards`, `standards`, `help`, `release`, `philosophy` —
but **not** `audit`. Since `"audit"` is not a registered `StandardDefinition`
(`crates/standards/src/builtin.rs` registers readme, vision, philosophy,
architecture, feature, feature-design, feature-technical, design,
engineering, external-context, prototype, help, standards — no `audit`),
these 19 files hit the exact same fallback path as Gap 3: `infer_standard()`
returns the literal parent dir name `"audit"`, no `StandardDefinition` is
found for it, and every file compiles as `DocumentBody::Generic` with no
section validation. This is Gap 3/4's failure mode demonstrated inside
Samgraha's own repository — proof the bug isn't Heimdall-specific.

**`standards` and `help` — reproducible, but via a different mechanism than Gaps 1-7.**
`RuntimeInfo::info()` (`crates/services/src/runtime/runtime.rs:1497-1528`)
builds two separate fields:
```rust
standards: self.standard_registry.domains().into_iter()
    .filter(|d| (declared.is_empty() || declared.contains(d)) && !excluded.contains(d))
    .collect(),
...
builtin_stores: crate::builtin::BUILTIN_DOMAINS.iter()
    .map(|(domain, _)| format!("{} ({})", domain, if loaded { "loaded" } else { "missing" }))
    .collect(),
```
`standards` is filtered by the repo's `domain`/`domain_exclusion` config.
`builtin_stores` (`standards`, `help` — from `BUILTIN_DOMAINS` in
`crates/services/src/builtin.rs:7`) is **not** — it always lists both,
labeled loaded/missing, for every repo regardless of config. `samgraha info`
on Heimdall will always show a `Built-in: standards (...), help (...)` line
even though Heimdall's `domain_exclusion` is empty and never asked for them.

**`audit-standards` — not reproducible.** It's excluded by Samgraha's own
ignore pattern, it's not in `BUILTIN_DOMAINS`, and it's not a registered
standard. No code path produces it. This part of the reported symptom likely
reflects a stale observation or a different repo state — worth confirming
the exact command/output before scoping a fix for it.

**Impact:** Two distinct, real bugs got merged into one symptom line. `audit`
needs the same treatment as Gap 3 (normalize or exclude). `standards`/`help`
need `builtin_stores` to respect `domain_exclusion` the way `standards`
already does.

---

## Summary of Root Causes

| Gap | Severity | Description |
|-----|----------|-------------|
| 1 | **Critical** | `root_dir` config is decorative; compiler always walks from repo root |
| 2 | **Critical** | `infer_standard()` can't handle numbered filename prefixes |
| 3 | **High** | Non-standard parent directories produce invalid domains |
| 4 | **Medium** | No validation that inferred domain is a registered standard |
| 5 | **Medium** | Content-based domain fallback doesn't exist |
| 6 | **Low** | Domain_map is project-specific and not extensible |
| 7 | **Low** | `raw/` prefix handling is implicit, not configurable |
| 8 | **Medium** | `audit` hits Gap 3's fallback inside Samgraha's own repo; `builtin_stores` ignores `domain_exclusion`; `audit-standards` is not reproducible as reported |

The three critical gaps (1, 2, 3) form a chain: Gap 1 means docs aren't
discovered from the configured root; Gap 2 means flat docs can't be domain-
tagged correctly; Gap 3 means docs in unconventional directories get wrong
tags. The result is that all validation, section enforcement, and quality
analysis is bypassed. Gap 8 shows this isn't purely an external-repo problem
— Samgraha's own `docs/raw/audit/` silently falls into the same ungoverned
fallback today.

## Suggested Fix Approaches (not yet implemented — for validation)

### Fix A: Consume `root_dir` in compilation
In `CompilationService::execute()`, resolve the docs root using
`resolve_configured_dir(&config.repository.documentation.root_dir, root, "docs")`
and pass the resolved path to `CompilationPipeline::compile()` instead of
the repository root.

**Implementation risk:** `root` in `CompilationService::execute()` is reused
for three other things besides discovery — reading `.samagraignore` (line
25), the manifest's `repository_root` field, and the post-compile
existence-check cleanup (`root.join(&stored.path.0)`, lines 102-108). If the
discovery root is resolved to a different path than `root`, those three
uses must keep pointing at the true repository root or ignore-pattern
matching and stale-document cleanup will break. Introduce a second variable
(e.g. `docs_root`) rather than reassigning `root`.

### Fix B: Normalize numbered prefixes in `infer_standard()`
Strip leading digits and separator characters (e.g. `01-vision` → `vision`,
`05-01-technology-selection` → `technology-selection`) before checking the
domain_map. This enables flat `docs/` files to be correctly inferred.

### Fix C: Add domain alias / normalization table
Extend `DOMAIN_OVERRIDE` to map common variant names:
```
("audits", "feature"),
("scoring", "feature"),
("reporting", "feature"),
("plugins", "feature-technical"),
```

### Fix D: Validate domains against registered standards
In `CompilationPipeline::compile()`, emit a warning/error when a discovered
document's inferred standard is not found in the standard registry.

### Fix E: Multi-layered domain resolution
Try in order: path-based inference → content-based (H1 title, Document ID
prefix) → `DOMAIN_OVERRIDE` → configured domain list.

### Fix F: Make domain_map extensible via config
Allow repos to configure domain→filename mappings in `samgraha.toml` so
`infer_standard()` can be customized per-project.

### Fix G: Stop `docs/raw/audit/` falling into the unregistered fallback
Either add `"**/audit/**"` to Samgraha's own `[repository.ignore].patterns`
(consistent with the existing `audit-standards`/`standards`/`help`
exclusions), or add `("audit", "engineering")` (or a new registered `audit`
standard) to `DOMAIN_OVERRIDE` in `discovery.rs`. Without one of these, the
19 files under `docs/raw/audit/` keep compiling as ungoverned `Generic`
documents in Samgraha's own knowledge base.

### Fix H: Filter `builtin_stores` by `domain_exclusion`
In `RuntimeInfo::info()` (`runtime.rs:1520-1526`), apply the same
`!excluded.contains(domain)` filter to `builtin_stores` that the `standards`
field already gets at line 1511, so `samgraha info` on a repo that never
declared `standards`/`help` doesn't unconditionally list them as available.
