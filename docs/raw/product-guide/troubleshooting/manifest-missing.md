# Manifest Missing or Invalid

## Purpose

Issues with `.samgraha/manifest.json` and how to resolve them.

## Content

### Problem: "manifest.json not found"

**Cause**: `.samgraha/manifest.json` is only written by a **successful** compile with zero failures — `samgraha init` does not create it (it only creates `samgraha.toml` and the `.samgraha/` directory). If compilation has errors, the manifest simply won't be written or updated.

**Solution**: Resolve any compile errors, then compile:
```bash
samgraha compile
```

### Problem: "Invalid manifest JSON"

**Cause**: The manifest file contains invalid JSON syntax (e.g. a hand edit, or an interrupted write).

**Solution**: `samgraha init --force` only rewrites `samgraha.toml`, not the manifest. To regenerate `manifest.json`, delete it and recompile — a successful compile rewrites it from scratch:
```bash
rm .samgraha/manifest.json
samgraha compile
```

### Problem: "Manifest missing required fields"

**Cause**: The manifest is missing required fields (uuid, name) — usually from hand-editing rather than letting compile generate it.

**Solution**: Delete and let compile regenerate it, same as above.

### Problem: "Manifest UUID changed"

**Cause**: The repository UUID changed (e.g., after re-initialization). This can break dependency references.

**Solution**: Update all consumers that depend on this repository's old UUID, or restore the original UUID in the manifest.

## Related

- [Command: init](../commands/init.md)
- [Command: registry](../commands/registry.md)
- [Troubleshooting Index](registry-missing.md)
