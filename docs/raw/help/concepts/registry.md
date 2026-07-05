# Registry

## Purpose

The repository registry concept — how Samgraha tracks known repositories and their metadata.

## Content

The **registry** is a catalog of repositories known to Samgraha. Each registered repository has a UUID, a manifest URL, and metadata about its exported knowledge.

### Local Registry

Each repository has a local `.samgraha/registry.db` that tracks:

- Known documents (paths, hashes, timestamps)
- Compilation state of each document
- Content change detection for incremental compilation

### Repository Registry

`samgraha registry <action>` manages the local, file-based registry of repositories known on this machine:

```bash
samgraha registry register          # Register the current repo (reads its .samgraha/manifest.json)
samgraha registry unregister <uuid> # Unregister a repository by UUID
samgraha registry sync              # Sync cached dependency metadata from their manifests
samgraha registry refresh           # Refresh all cached dependency metadata
samgraha registry status            # Show registry status
samgraha registry list              # List registered repositories
samgraha registry resolve runtime   # Resolve dependencies for runtime (writes .samgraha/resolved)
```

`register` takes no arguments — it reads the current repository's own `.samgraha/manifest.json` (so `compile` must have run at least once first). A successful `compile` also auto-syncs dependency metadata in the background (unless `resolver.auto_refresh` is disabled), so `sync` rarely needs to be run by hand.

Registration/sync enables dependency resolution: when repo A depends on repo B, the Planner can find repo B's cached metadata (or its manifest directly) and make its knowledge available.

### Manifest

Each repository's `.samgraha/manifest.json` declares:

- Repository UUID and name
- Exported document domains
- Dependencies on other repositories
- Interests in other repositories' knowledge

## Related

- [Repository](repository.md)
- [Workspace](workspace.md)
- [Planner](planner.md)
- [Command: registry](../commands/registry.md)
