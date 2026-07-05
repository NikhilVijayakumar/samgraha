# Registration Workflow

## Purpose

How to register repositories in the global registry for dependency resolution.

## Content

### Registration

There is no "register by path/URL" form — `registry register` takes no arguments. It reads `.samgraha/manifest.json` from the *current* repository (which `samgraha compile` must have already produced) and adds that repo to the local registry:

```bash
samgraha registry register
```

To register a dependency from a different machine or checkout, `cd` into that repo and run `samgraha registry register` there — or, over MCP, call the `register_repository` tool with the manifest JSON directly.

### What Registration Stores

The registry (`.samgraha/registry.db`, SQLite-backed) stores, per repository:

- Repository UUID and ID/name
- Repository root path, revision, exported domains
- Last-sync timestamp and TTL expiry
- Audit status

### Listing

```bash
samgraha registry list
samgraha registry status   # includes computed status (Registered/StaleMetadata/StaleKnowledge/etc.) per repo
```

### Syncing

```bash
samgraha registry sync      # refresh cached .meta files for this repo's [repository.dependencies]
samgraha registry refresh   # currently identical to sync
```

This reads each declared dependency's manifest and refreshes its cached metadata. Useful after:

- New documents are added to a dependency
- A dependency's manifest changes
- The cached `.meta` file's TTL (`resolver.metadata_ttl`, default 24h) has expired

Compile also runs this automatically (`resolver.auto_refresh`, default true) after a successful compile — `registry sync` is the manual/explicit path.

### Unregistering

```bash
samgraha registry unregister <uuid>
```

## Related

- [Multi-Repo Overview](overview.md)
- [Dependencies](dependencies.md)
- [Sync](sync.md)
- [Command: registry](../commands/registry.md)
