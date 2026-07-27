# Repository Registration

What "registering a repository" means, why it's a separate concept from
activating a knowledge standard, and how a standard's files/data actually
travel from samgraha's global registry into a repo's own `.samgraha/`.
Companion doc: `knowledge-standard.md` covers what a standard *is* and
the activation mechanics this document's §3-4 summarize from the
repo-registration side; `build.md` covers producing the binaries.

## 1. Two registrations, easy to conflate, genuinely separate

Samgraha has two independent things called "registration," both backed
by SQLite files that happen to sit in the same `.samgraha/` directory,
and they answer different questions:

| | Repository registration (this doc) | Standard activation (`knowledge-standard.md`) |
|---|---|---|
| Question it answers | "What other repos does this repo depend on, and what do I know about them?" | "Which knowledge standard is this repo currently running, and what workflow does it declare?" |
| MCP tools | `register_repository`, `unregister_repository`, `list_repositories`, `repository_status` | `register_standard_globally`, `register_standard` (activation), `run_script_step`, etc. |
| Backing store | `registry.db`, table `repository_cache` | `registry.db`, table `active_standard` (metadata); `knowledge.db` (workflow rows) |
| Scope | Cross-repo dependency graph — this repo, and every repo it declares a dependency on | This repo only — one standard active at a time |

Both tables live in the same physical `registry.db` file
(`<samgraha_dir>/registry.db`) because both are "facts about this repo's
registration," but they don't reference each other and don't share rows.
A repo can be fully registered in the dependency-graph sense with zero
standards activated, and vice versa.

## 2. Why repository registration exists

Samgraha repos can declare dependencies on other repos
(`[repository] dependencies` in `samgraha.toml` — name + optional
relative `path` + `required`). Resolving "what does my dependency
actually export, what's its current revision" by reading every
dependency's manifest from disk on every query would mean every repo in
a dependency chain has to be reachable, on disk, every time. The
registry cache exists so that answer is available from a local SQLite
read instead — `repository_cache` (`schema/registration/00-repository_cache.sql`)
stores `id`/`uuid`/`name`/`repository_root`/`knowledge_db` location/
`revision`/`exports`/`audit` status/`dependencies` (JSON list of
dependency names)/`last_sync`/`expires`, one row per known repo (itself
and every dependency it's synced).

Its own header comment states this plainly: *"disposable/rebuildable
from dependency manifests, not authoritative data."* If `registry.db`
is deleted, it can be rebuilt entirely by re-running registration/sync —
nothing here is a source of truth samgraha can't reconstruct.

## 3. The `RegistryClient` trait — one interface, one real implementation

`services::registry_client::RegistryClient` is the interface every MCP
handler that touches repository registration goes through:
`register`/`unregister`/`sync`/`discover`/`get_metadata`/`list`. The only
implementation, `FileRegistryClient`, is SQLite-backed via `RegistryDb`
(`registry::registry_db::RegistryDb`, opened at `<repo_root>/.samgraha/registry.db`).
`RegistryType::Http` exists in `samgraha.toml`'s `[resolver] registry_type`
enum as a reserved-for-later value — nothing implements it; every real
registry today is the local file, regardless of what `registry_type` says.

- **`register(manifest)`** — upserts this repo's own `repository_cache`
  row from a `RepositoryManifest` (id/uuid/name/root/knowledge location/
  revision/exports/dependencies), setting `last_sync`/`expires` from the
  manifest's `generated_at` + a TTL.
- **`sync(config)`** — for every `[repository] dependencies` entry with a
  resolvable `path`, reads that dependency's own manifest off disk
  (`MetadataCache::read_dependency_manifest`) and upserts its
  `repository_cache` row too — this is what actually populates entries
  for repos *other than* this one.
- **`discover(query)`** / **`list()`** / **`get_metadata(uuid)`** — read
  paths, filtering by uuid/id/export.
- **`unregister(uuid)`** — deletes the row.

A UUID mismatch on an existing `id` (the manifest's UUID differs from
what's cached) is logged as a warning during `register`, or hard-rejected
during `sync` (`registry_client.rs:124-134`, tagged `ENG-GAP-06` — UUID
spoofing prevention) — `register` treats the manifest as authoritative
(e.g. after a `.samgraha/` recovery regenerates a new UUID for the same
`id`); `sync`, reading a *dependency's* manifest rather than this repo's
own, does not extend it the same trust.

## 4. How a standard's data actually gets from global samgraha to a repo

Two copies, two databases, in this exact order — see
`knowledge-standard.md` §4/§6 for the full step-by-step; this is the
shape from the registration side:

```
Standard author's source tree (e.g. Kriti/samgraha/system/<category>/<name>/)
        │  register_standard_globally: copy_dir_atomic + smoke_test + metadata validate
        ▼
mcp_dir()/registry/<category>/<name>/  ← samgraha's own installation-wide registry
        │  standards.db: standard_registry row (source_path = the copy above)
        │
        │  register_standard (activation): copy_dir_atomic
        ▼
<repo>/.samgraha/<name>/               ← this specific repo's local copy (flat — one repo, one active standard, no category ambiguity to preserve)
        │  seeder runs against <repo>/.samgraha/knowledge.db
        │  registry.db: active_standard row (name/version/etc, from the standards.db row)
        ▼
<repo>/.samgraha/knowledge.db          ← usecase/step/script/prompt/... rows
```

The registry tier nests by `<category>` (`knowledge-standard.md` §4 —
mirroring the real standards corpus's own on-disk layout); the per-repo
tier doesn't need to, since a repo only ever has one standard active at
a time (§7 of `knowledge-standard.md`) — nothing to disambiguate by
category once you're inside a single repo's `.samgraha/`.

Neither copy is a symlink or a reference — both are full,
`copy_dir_atomic` copies (temp-dir-then-rename, exclude `__pycache__`/
`.pyc`). This is deliberate, not an oversight: `standards.db` stores no
live pointer into the standard author's original source repo (§2 of
`knowledge-standard.md`'s reasoning — that repo might move, get deleted,
or be unreachable from this machine), and a repo's own
`.samgraha/<name>/` has to survive independently of samgraha's own
installation being available later (a script needs its own copy to
execute against, not a shared one another repo might be mutating).

**Cost accepted, not fixed here**: N repos activating the same standard
means N copies on disk. No dedup/caching layer exists across repos —
deliberately out of scope for this design, same reasoning as not
building a shared standards cache: no standard on disk today is large
enough for the duplication to matter, and a shared-cache layer is exactly
the kind of speculative infrastructure this project avoids adding ahead
of a real need.

## 5. How `knowledge.db` gets its seed data

Not by samgraha parsing anything — by running the standard's own seeder
script (`knowledge-standard.md` §6 step 4) against the repo's real
`knowledge.db`, with `_samgraha_dir`/`_knowledge_db` injected into its
`--in` envelope so it never has to guess where either one is. The seeder
is arbitrary code (any interpreter `env.rs`'s `script_command` dispatch
table already supports) making its own `INSERT` statements directly —
samgraha's role is invocation, envelope construction, status validation,
and the post-seed structural audit (`knowledge-standard.md` §8), never
generating the rows itself.

## 6. `init_repository` — what it does and does not do

`services::init_repository` (`init.rs:38`) writes `samgraha.toml` (fresh,
or backfilling missing keys onto an existing one — never destructive
unless `force: true`) and, with `auto_detect_dirs: true`, probes for
`docs/`/`src`|`crates/`/`tests/`/`scripts/` and records literal paths for
whichever exist. It resolves and creates the `.samgraha/` directory
(`resolve_samgraha_dir`, same function `knowledge-standard.md` §12
references).

**It does not activate any standard.** `InitOptions.sync_knowledge_system`
suggests otherwise by name and by its own doc comment ("sync the declared
Knowledge System from global store into the local `.samgraha/` after
writing `samgraha.toml`") — verified: the field is declared, set in
exactly one test, and read nowhere in `init_repository`'s actual body.
Tracked as a real gap in
`docs/proposal/samgraha-toml-configuration-contract-proposal.md` §5.3,
not fixed by this document. Until that's resolved, activating a standard
in a freshly-initialized repo always requires an explicit
`register_standard` MCP call — `init` alone never does it.

## 7. `samgraha.toml` fields relevant to repository registration

Full contract: `docs/proposal/samgraha-toml-configuration-contract-proposal.md`.
The fields that matter here specifically:

- **`[repository] id` / `name` / `uuid`** — identity fields written by
  tooling (`register_repository`), not hand-edited. No env-vs-toml
  question applies to a value nothing ever types by hand.
- **`[repository] dependencies`** — plain policy list (repo name +
  relative `path` + `required`); a *repo-relative* path here is correct,
  not a violation of the "no absolute machine paths in toml" rule — it's
  pointing at a sibling checkout, which is itself project policy (every
  clone of this repo is expected to have that sibling at that relative
  location), not a machine-specific override.
- **`[repository] samgraha_dir`** — where `.samgraha/registry.db` (and
  `knowledge.db`, and every activated standard's local copy) actually
  lives. `${SAMGRAHA_DIR}` pattern, falls back to `<repo-root>/.samgraha`.
- **`[resolver] metadata_ttl` / `knowledge_ttl`** — how long a synced
  dependency's cached metadata (`metadata_ttl`, ~1 day default) or an
  assembled Knowledge Package (`knowledge_ttl`, ~30 days default) is
  trusted before a re-sync is needed. Plain policy — same for every
  machine running this repo.
- **`[resolver] registry_type` / `registry_url`** — `registry_type` is
  currently always effectively `"file"` in practice (§3); `registry_url`
  is reserved alongside the unimplemented `Http` variant.
