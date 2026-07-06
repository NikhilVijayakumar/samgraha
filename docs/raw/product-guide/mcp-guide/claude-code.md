# Connecting Claude Code

## Purpose

How to connect Claude Code to the Samgraha MCP server.

## Content

### Configuration

In Claude Code's MCP configuration (project `.mcp.json` or `claude_desktop_config.json`), point `command` directly at the `mcp.exe` binary (or its `run-mcp.cmd`/`run-mcp.sh` launcher) — there is no `samgraha mcp` subcommand:

```json
{
  "mcpServers": {
    "samgraha": {
      "command": "mcp.exe",
      "env": {
        "SAMGRAHA_DOCS_DIR": "/path/to/your/docs"
      }
    }
  }
}
```

### What Claude Can Do

Once connected, Claude can:

- Search your project documentation for relevant context
- Look up feature specifications
- Check architecture documents
- Find engineering standards
- Use help docs for Samgraha commands

### Example Queries

```
Claude: "What does the authentication feature require?"
→ Calls search(query: "authentication", domain: "feature")

Claude: "How do I compile my docs?"
→ Calls search(query: "compile", domain: "help")
```

## Related

- [MCP Overview](overview.md)
- [Installation](installation.md)
- [Tools Reference](tools.md)
- [OpenCode Integration](opencode.md)
