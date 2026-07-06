# MCP Server Won't Start

## Purpose

Common MCP server startup failures and their solutions.

## Content

### Problem: Client config invokes `samgraha mcp` and fails

**Cause**: There is no `samgraha mcp` subcommand. The MCP server is a separate binary, `mcp.exe` (built from `crates/mcp`, alongside `cli.exe` from `crates/cli`). It communicates over stdin/stdout JSON-RPC — there is no network port to configure or conflict over.

**Solution**: Point the MCP client config directly at `mcp.exe` (or the packaged `run-mcp.cmd`/`run-mcp.sh` launcher), not at a `samgraha mcp` command:
```json
{
  "mcpServers": {
    "samgraha": {
      "command": "C:/path/to/samgraha/bin/mcp.exe"
    }
  }
}
```

### Problem: "Binary not found"

**Cause**: `mcp.exe` is not where the client config expects it, or the release package wasn't extracted fully.

**Solution**: Verify the binary exists and run it manually to check for startup errors:
```bash
# Verify the binary exists
where mcp.exe

# Run directly (it will block, reading JSON-RPC from stdin)
./bin/mcp.exe
```

### Problem: "fatal: not a samgraha repository"

**Cause**: Like the CLI, `mcp.exe` walks up from the current working directory looking for `.samgraha/` or `samgraha.toml` and errors if none is found.

**Solution**: Start the MCP server from within a samgraha repository (or a subdirectory of one), or `samgraha init` the target directory first.

### Problem: Built-in help/standards search returns nothing

**Cause**: `standards.db` and/or `help.db` weren't found next to the running binary (resolved via `std::env::current_exe()?.parent()`). This is **non-fatal** — `load_builtin_stores()` silently skips missing files (logs at info/warn level) rather than erroring, so the server still starts and works for the repo's own knowledge. `samgraha info` reports built-in store status explicitly under a "Built-in:" line.

**Solution**: If you do want built-in help/standards search, ensure `standards.db`/`help.db` sit in the package root next to `bin/` (not inside it), as produced by `scripts/build-release.ps1`.

### Problem: No results for the current repository's own documentation

**Cause**: This is different from the built-in stores above — it means the target repository's own `.samgraha/knowledge.db` doesn't exist or is empty because it hasn't been compiled.

**Solution**: Compile the repository, then (re)start the MCP server:
```bash
samgraha compile
./bin/mcp.exe
```

## Related

- [MCP Guide: Installation](../mcp-guide/installation.md)
- [Command: compile](../commands/compile.md)
- [Troubleshooting Index](compile-failed.md)
