# Registry Missing or Unresolved

## Purpose

Registry-related issues and how to resolve them.

## Content

### Problem: "Repository not found in registry"

**Cause**: There is no `samgraha registry add <path>` command. The registry (`.samgraha/registry.db`, per-repository, not global) is populated two ways: `samgraha registry register` registers the *current* repository's own manifest (run it from inside that repo, after a successful compile), and `samgraha registry sync` reads `[[repository.dependencies]]` from `samgraha.toml` and pulls in each declared dependency's manifest.

**Solution**:
```bash
# In the dependency's own repo, after compiling it:
samgraha registry register

# Back in the consuming repo, declare the dependency in samgraha.toml:
#   [[repository.dependencies]]
#   name = "core-lib"
#   path = "../core-lib"
#   required = true
samgraha registry sync
```

### Problem: "Dependency unresolved"

**Cause**: A dependency declared in `samgraha.toml`'s `[[repository.dependencies]]` could not be resolved — usually its `path` is wrong, or it hasn't been compiled/registered yet.

**Solution**:
1. Compile and register the dependency in its own repo: `samgraha compile` then `samgraha registry register`.
2. Sync from the consuming repo: `samgraha registry sync`.

### Problem: Dependency path not found

**Cause**: Dependencies are resolved by local filesystem `path` only (relative to the consuming repo, or absolute) — there is no remote/URL manifest fetching in the current implementation.

**Solution**: Verify `path` in `[[repository.dependencies]]` points at a real, compiled repository on disk (it needs `.samgraha/manifest.json` to exist there).

### Problem: "UUID mismatch for dependency"

**Cause**: `samgraha registry sync` rejects a dependency outright (hard error, not a warning) if its manifest's UUID doesn't match the UUID already cached under that repository id — this guards against UUID spoofing. It usually means the dependency's `.samgraha/` was regenerated (e.g. after deleting and re-initializing it) without updating the consuming repo's registry.

**Solution**: Unregister the stale entry and re-sync:
```bash
samgraha registry unregister <old-uuid>
samgraha registry sync
```

## Related

- [Command: registry](../commands/registry.md)
- [Multi-Repo Guide: Registry](../multi-repo-guide/registry.md)
- [Troubleshooting Index](manifest-missing.md)
