# Knowledge DB Missing or Corrupt

## Purpose

Issues with the `.samgraha/knowledge.db` database and how to fix them.

## Content

### Problem: "knowledge.db not found"

**Cause**: The repository has not been compiled yet, or the `.samgraha/` directory was deleted.

**Solution**:
```bash
samgraha init    # Re-create .samgraha/ if missing
samgraha compile # Build the knowledge database
```

### Problem: "Database corruption"

**Cause**: The SQLite database file was corrupted (e.g., by an interrupted write, disk error).

**Solution**:
```bash
# Rebuild from source
samgraha compile --force
```

### Problem: "Schema mismatch"

**Cause**: `knowledge.db` tracks its schema version in a `_schema_version` table and auto-applies any pending migrations forward on open — most version differences (an older db opened by a newer binary) resolve themselves silently. A real mismatch usually means the opposite: the db was written by a *newer* Samgraha version than the binary you're running now.

**Solution**: Use a matching or newer binary. If you deliberately want a clean database, delete `.samgraha/knowledge.db` and recompile:
```bash
samgraha compile --force
```

### Problem: "knowledge.db is empty"

**Cause**: Compilation succeeded but no documents were found or processed.

**Solution**:
1. Check that docs exist: `ls docs/raw/`
2. Check domain configuration: `samgraha info`
3. Recompile with verbose output: `samgraha compile`

## Related

- [Command: compile](../commands/compile.md)
- [Command: init](../commands/init.md)
- [Concepts: Knowledge Database](../concepts/knowledge-db.md)
