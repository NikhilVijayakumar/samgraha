# SamagraIgnore — Feature Technical Design

This section details the SamagraIgnore — Feature Technical Design.

## Purpose

This document describes the architectural realization of the SamagraIgnore feature.

SamagraIgnore gives users a standard way to exclude paths from knowledge compilation — both globally via `samgraha.toml [repository.ignore]` and per-repository via a `.samagraignore` file. It replaces the hardcoded exclude list in the compilation pipeline with a unified, config-driven pattern system. It also corrects a bug in discovery's substring matching so that exclusions work reliably against relative paths.

This document applies the architectural principles defined in Component Model.

---

## Feature Specification

- **Feature:** docs/raw/feature/samagraignore.md
- **Architecture:** docs/raw/architecture/component-model.md

---

## Participating Components

This section details the Participating Components.

### IgnoreConfig (config.rs:110-126)

Existing struct with field `patterns: Vec<String>`. Currently has 3 default patterns. After this phase it has 4, adding `"**/audit-standards/**"`. Lives in `[repository.ignore]` in `samgraha.toml`. Already deserialized into `SamgrahaConfig` — no schema migration needed.

### Discovery Engine (discovery.rs:collect_markdown_files)

Responsible for walking the repository tree and collecting markdown files for compilation. Currently performs substring matching on directory names using the exclude list passed from the pipeline. The matching is upgraded to work reliably against relative paths.

### Compilation Pipeline (pipeline.rs:37-44)

Currently passes a hardcoded exclude list to DiscoveryEngine. After this phase it reads patterns from config, merges with `.samagraignore` patterns, and passes the merged result to DiscoveryEngine. The hardcoded list is removed.

### Compilation Service (compilation.rs)

Previously contained a hardcoded `audit-standards` guard applied after discovery. That guard is removed — `audit-standards` is excluded at discovery time via IgnoreConfig defaults.

### SamagraIgnore Parser (new — samagraignore.rs)

New ~80-line module. Reads `.samagraignore` from the repository root, filters comments and blank lines, and returns a `Vec<String>` of patterns. Returns an empty vec if the file is absent or unreadable (not an error). Merged with config patterns by `merge_ignore_patterns`.

---

## Component Responsibilities

| Component | Responsibility |
|---|---|
| IgnoreConfig | Hold global ignore patterns from samgraha.toml; provide Default impl with 4 standard patterns |
| Discovery Engine | Walk repository tree; exclude paths matching merged ignore patterns via `matches_ignore()` |
| Compilation Pipeline | Read config; call merge_ignore_patterns; pass merged patterns to DiscoveryEngine; no hardcoded excludes |
| Compilation Service | Invoke pipeline with config; remove secondary audit-standards guard |
| SamagraIgnore Parser | Parse .samagraignore file; return patterns; absent file is not an error |

---

## Component Interactions

```text
CompilationService::execute(root, config, ...)
    │
    ├── merge_ignore_patterns(root, config)      ← new
    │     ├── IgnoreConfig::default().patterns   ← 4 defaults
    │     ├── config.repository.ignore.patterns  ← user overrides from samgraha.toml
    │     └── parse_samagraignore(root)          ← per-repo .samagraignore file
    │
    └── DiscoveryEngine::discover(root, &[], &merged_patterns)
              └── collect_markdown_files
                    └── matches_ignore(relative_path, patterns) per directory entry
```

### Request Flow (compilation)

1. `CompilationService::execute` receives root path and `SamgrahaConfig`.
2. `merge_ignore_patterns(root, config)` is called once at compilation start.
   - Starts with `IgnoreConfig::default().patterns` (4 patterns).
   - Extends with `config.repository.ignore.patterns` (user-defined in `samgraha.toml`).
   - Extends with `parse_samagraignore(root)` (per-repo `.samagraignore` file).
   - Sorts and deduplicates.
3. `DiscoveryEngine::discover(root, &[], &merged_patterns)` is called with the merged list.
4. During tree walk, `matches_ignore(relative_path, patterns)` is called per directory entry.
5. Matching entries are skipped entirely — their subtrees are not walked.

---

## Runtime Behavior

### Current State (what changes)

```rust
// discovery.rs — substring match against directory name only (to upgrade)
exclude.iter().any(|p| name.contains(p.trim_matches('*')))

// compilation.rs — secondary guard after discovery (to remove)
if !abs.exists() || rel.contains("audit-standards") {
    registry.delete_document(stored.id)?;
}
```

### Target State

```rust
// compilation.rs — read from config, no hardcoded list
let ignore_patterns = merge_ignore_patterns(root, config);
DiscoveryEngine::discover(root, &[], &ignore_patterns)

// discovery.rs — match against relative path (not just directory name)
fn matches_ignore(relative_path: &Path, patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        let normalized = p.trim_matches('*').trim_matches('/');
        relative_path.to_string_lossy().contains(normalized)
    })
}

// samagraignore.rs — new module
fn parse_samagraignore(root: &Path) -> Vec<String> {
    let path = root.join(".samagraignore");
    std::fs::read_to_string(path).ok()
        .map(|content| content.lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .map(|l| l.to_string())
            .collect())
        .unwrap_or_default()
}

fn merge_ignore_patterns(root: &Path, config: &SamgrahaConfig) -> Vec<String> {
    let mut patterns = IgnoreConfig::default().patterns;
    patterns.extend(config.repository.ignore.patterns.clone());
    patterns.extend(parse_samagraignore(root));
    patterns.sort();
    patterns.dedup();
    patterns
}
```

### IgnoreConfig Default Update

```rust
impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/audit-standards/**".to_string(),  // added
            ],
        }
    }
}
```

### Compile Flow After Change

```
CompilationService::execute(root, config, ...)
        │
        ├── merge_ignore_patterns(root, config)
        │     ├── IgnoreConfig::default().patterns  ← 4 defaults incl. audit-standards
        │     ├── config.repository.ignore.patterns ← user overrides
        │     └── parse_samagraignore(root)         ← per-repo file
        │
        └── DiscoveryEngine::discover(root, &[], &merged_patterns)
              └── collect_markdown_files
                    └── matches_ignore(relative_path, patterns) per dir entry
```

### Glob Matching — Phase 2 Limitation

Full glob crate integration is deferred. Phase 2 uses normalized substring matching: strip leading/trailing `*` and `/` characters from the pattern, then check if the relative path string contains the normalized segment. This correctly handles all four default patterns and common user patterns such as `**/vendor/**` or `docs/internal/`.

Full glob matching (supporting character classes, `?` wildcards, negation patterns) is a Phase 7 enhancement. A `// ponytail: substring match, upgrade to glob crate if negation patterns or ? wildcards are needed` comment marks this boundary in the code.

---

## Communication Paths

### Compilation Pipeline → SamagraIgnore Parser

`merge_ignore_patterns` calls `parse_samagraignore(root)` once per compilation. The parser reads one file from disk. This is a compile-time operation; no runtime communication is involved.

### Compilation Pipeline → IgnoreConfig

`merge_ignore_patterns` calls `IgnoreConfig::default()` and reads `config.repository.ignore.patterns`. Both are in-memory reads after config deserialization. No I/O.

### Discovery Engine → Pattern List

DiscoveryEngine receives the merged pattern list as a `&[String]` parameter from the pipeline. It calls `matches_ignore` per directory entry during the tree walk. No further I/O beyond the walk itself.

---

## Data Ownership

| Data | Owner | Access |
|---|---|---|
| Global ignore patterns | SamgrahaConfig / samgraha.toml | Read at compilation start |
| Per-repo ignore patterns | .samagraignore file | Read at compilation start (once per compile) |
| Merged pattern list | Compilation Pipeline (local) | Read during discovery walk |
| IgnoreConfig defaults | IgnoreConfig::default() | Read (in-memory) |

All pattern sources are read-only during compilation. No write operations on ignore data.

---

## Integration Points

### samgraha.toml [repository.ignore]

IgnoreConfig is already deserialized from `samgraha.toml`. No config schema changes. Users who already have `[repository.ignore]` entries continue to work; patterns are now merged rather than replacing defaults.

### .samagraignore File

A new per-repository file. Absent by default. Format: one pattern per line. Lines starting with `#` are comments. Blank lines are ignored. No schema version or header required.

### DiscoveryEngine

`collect_markdown_files` receives the merged pattern list via the existing parameter. The call site in `compilation.rs` changes; the DiscoveryEngine function signature does not need to change.

---

## External Dependency Integration

No new dependencies. `std::fs::read_to_string` handles `.samagraignore` parsing. The glob crate is not added in Phase 2.

---

## Runtime Constraints

- `merge_ignore_patterns` must be called once per compilation, not per-file.
- Pattern matching is applied at directory entry level — entire subtrees are skipped, not individual files.
- Absent `.samagraignore` is not an error condition.
- Deduplication of patterns is required — the same pattern may appear in defaults, config, and `.samagraignore`.

---

## Architectural Constraints

- No secondary post-discovery guards for specific directory names may remain in `compilation.rs`.
- `audit-standards` exclusion must be covered by `IgnoreConfig::default()`, not by pipeline or service code.
- The DiscoveryEngine interface must not be widened — pattern list is passed as `&[String]`, consistent with current usage.

---

## Security Considerations

- Ignore patterns are read from config and per-repo files. These are user-controlled inputs. A malicious pattern could exclude security-relevant documentation from compilation. This is an accepted risk: the user controls their own compilation.
- `.samagraignore` patterns do not expand to paths outside the repository root. DiscoveryEngine already constrains the walk to the root subtree.

---

## Performance Considerations

- `parse_samagraignore` reads one small file per compilation. Cost is negligible (<1ms for typical ignore files).
- `merge_ignore_patterns` involves two Vec extends and a sort+dedup on a small list (typically <20 patterns). Cost is negligible.
- `matches_ignore` is called per directory entry during the tree walk. Substring matching on a `&str` is O(n·m) where n is path length and m is pattern count. For typical patterns (<20) and paths (<200 chars), this is <1µs per entry. At 1000 files, total matching overhead is <1ms.

---

## Failure Handling

| Failure | Behavior |
|---|---|
| .samagraignore absent | `parse_samagraignore` returns empty vec; compilation proceeds with config patterns only; not logged |
| .samagraignore unreadable (permissions) | `read_to_string` returns Err; `ok()` converts to None; returns empty vec; log warning with path |
| Invalid or unsupported pattern syntax | Pattern is included as-is; normalized substring matching may not match anything; log warning per pattern if normalization produces empty string |
| Duplicate patterns across sources | Removed by sort+dedup in merge_ignore_patterns; no error |
| Pattern matches too broadly (excludes valid docs) | User responsibility; no runtime guard; same risk as any file-system exclude tool |

---

## Extension Points

### Full Glob Matching (Phase 7)

Replace `matches_ignore` with a glob crate call. The function signature is unchanged; only the matching implementation changes. The `// ponytail:` comment in the code marks this as the upgrade point.

### Negation Patterns

`.gitignore`-style negation (`!pattern`) could be supported in a future phase. The parser's filter step would need to separate include and exclude patterns. The merge function would need to apply them in order.

### Watch-Mode Invalidation

When file-watch-triggered compilation is added, changes to `.samagraignore` should trigger a re-merge of patterns and a full recompile of the affected repository. This is a future enhancement; the parse-at-compile-start model supports it naturally.

---

## Traceability

This document derives from:

- Feature: SamagraIgnore
- Architecture: Component Model

This document provides technical context for:

- Knowledge Compilation Technical Design
- Repository Discovery Technical Design
- Repository Configuration Technical Design

Traceability:

```
Feature → Architecture → Feature Technical Design → Engineering → Implementation
```
