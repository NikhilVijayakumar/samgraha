# Search Returns Empty Results

## Purpose

Why search might return no results and how to fix it.

## Content

### Problem: "No results found"

**Possible causes and solutions:**

**1. Repository not compiled**
```bash
samgraha compile
```

**2. Wrong domain filter**
```bash
# Check available domains
samgraha info

# Search without domain filter
samgraha search "query"
```

**3. Query doesn't match any content**
- Try different keywords.
- Check that the content is in the compiled database:
```bash
samgraha sections
```

**4. Built-in stores not loaded (for help/standards)**
```bash
samgraha search "compile" --domain help
```
If this returns empty, ensure `help.db` is adjacent to the binary (package root, next to `bin/`). Run `samgraha info` — it reports built-in store status explicitly. A missing built-in store is non-fatal; it just means no help/standards results.

**5. Wrong directory**
- Ensure you're in the correct repository.
- Run `samgraha info` to verify the docs root.

## Related

- [Command: search](../commands/search.md)
- [Command: sections](../commands/sections.md)
- [Search Guide: Overview](../search-guide/overview.md)
