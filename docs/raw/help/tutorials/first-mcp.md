# Tutorial: First MCP Connection

## Purpose

Step-by-step: start the MCP server and connect an AI assistant.

## Content

### Step 1: Compile Docs

```bash
cd my-project
samgraha compile
```

### Step 2: Start MCP Server

There is no `samgraha mcp` subcommand — the MCP server is a separate binary, `mcp.exe` (built alongside `cli.exe` from the same Cargo workspace). It speaks JSON-RPC over stdin/stdout, not a network port:

```bash
./bin/mcp.exe
```

In a packaged release, use the launcher script instead: `run-mcp.cmd` (Windows) or `run-mcp.sh` (Unix), which just execs `bin/mcp.exe` with any passthrough args.

### Step 3: Configure AI Assistant

**Claude Code** (MCP server config):

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "C:/path/to/samgraha/bin/mcp.exe"
    }
  }
}
```

`mcp.exe` must be started from (or discover upward from) a samgraha repository — it walks up from the current directory looking for `.samgraha/` or `samgraha.toml`, same as the CLI.

### Step 4: Query

```
Developer: "What features are documented in this project?"
→ AI assistant searches Samgraha and returns a list.

Developer: "Show me the architecture overview."
→ AI assistant retrieves the architecture document.
```

### What You Learned

- How to start the MCP server
- How to configure AI assistants
- How to query documentation via MCP

## Related

- [MCP Guide: Overview](../mcp-guide/overview.md)
- [MCP Guide: Claude Code](../mcp-guide/claude-code.md)
- [MCP Guide: Tools](../mcp-guide/tools.md)
