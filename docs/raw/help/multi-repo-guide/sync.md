# Syncing Metadata

## Purpose

How to synchronize registry metadata when repositories or their manifests change.

## Content

### When to Sync

- After adding new repositories to the registry.
- When a dependency repository updates its manifest.
- When a dependency repository is restructured.
- On a schedule (e.g., daily in CI).

### Sync Command

```bash
samgraha registry sync
```

This reads `.samgraha/manifest.json` for each entry under this repo's `[repository.dependencies]`, and writes/refreshes a `.samgraha/dependencies/<name>.meta` cache file for each (JSON, TTL from `resolver.metadata_ttl`, default 24h). `samgraha registry refresh` does the same thing today — it's a separate subcommand but currently an alias.

### What Sync Updates

- Cached repository name, UUID, revision
- Exported domains
- Manifest revision (used to detect staleness)
- `expires` timestamp (TTL)

### Status After Sync

```bash
samgraha registry status
```

Shows each repository's computed status (`registry status`/`registry list` compute this on demand, nothing is stored as a fixed "status" field):

- `Registered` — up to date
- `SyncRequired` — dependency's manifest regenerated since last sync
- `StaleMetadata` — cached `.meta` TTL expired
- `StaleKnowledge` — cached revision is behind the dependency's actual manifest revision
- `AuditFailed` — dependency's last audit was FAIL/ERROR
- `Missing` — repository root no longer exists on disk
- `Unavailable` — repository root exists but isn't readable

### Auto-Sync

There is no `--force` flag on `sync`. Compile already runs `sync` automatically after a successful compile (`resolver.auto_refresh`, default true) — running `registry sync` by hand is only needed to refresh sooner than that.

## Related

- [Registration Workflow](registry.md)
- [Dependencies](dependencies.md)
- [Command: registry](../commands/registry.md)
