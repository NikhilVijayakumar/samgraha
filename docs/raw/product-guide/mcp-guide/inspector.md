# MCP Inspector Debugging

## Purpose

How to use the MCP Inspector to debug the Samgraha MCP server.

## Content

### What is MCP Inspector?

MCP Inspector is a debugging tool for MCP servers. It provides a web UI to inspect tools, call them, and see responses.

### Starting Inspector

```bash
npx @modelcontextprotocol/inspector mcp.exe
```

This starts the inspector and connects it to the Samgraha MCP server. There is no `samgraha mcp` subcommand — `mcp.exe` is a separate binary from `cli.exe`/`samgraha`, so point Inspector at it directly (or at the `run-mcp.cmd`/`run-mcp.sh` launcher, if packaged).

### What to Inspect

- **Tools list** — Verify all tools are registered correctly
- **Tool calls** — Test search and document retrieval
- **Error handling** — See how the server responds to invalid inputs
- **Response format** — Check JSON-RPC compliance

### Common Debugging Tasks

1. **Verify server starts**: Check server logs (stderr) for startup messages.
2. **Test search**: Use the inspector to call `search` with various queries.
3. **Check built-in stores**: Call `search` with `domain: "help"` to verify the built-in help store is loaded (missing `help.db`/`standards.db` are skipped silently at startup — `info` will show them as missing).
4. **Check error responses**: Call with missing required parameters (e.g. `search` without `query`) to verify error handling.

## Related

- [MCP Overview](overview.md)
- [Installation](installation.md)
- [Tools Reference](tools.md)
