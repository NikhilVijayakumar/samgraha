# Connecting OpenCode

## Purpose

How to connect OpenCode to the Samgraha MCP server.

## Content

### Configuration

In OpenCode's configuration (`~/.config/opencode/opencode.json` or similar), point `command` at the `mcp.exe` binary directly — there is no `samgraha mcp` subcommand (`cli.exe`/`samgraha` and `mcp.exe` are separate binaries built from the same workspace):

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

### What OpenCode Can Do

Once connected, OpenCode can:

- Search project documentation for relevant context
- Look up requirements and specifications
- Query architecture and design decisions
- Access built-in help and standards

### Example Usage

```
$ opencode
> what does the authentication feature require?
→ OpenCode searches Samgraha knowledge and returns relevant sections.
```

## Related

- [MCP Overview](overview.md)
- [Installation](installation.md)
- [Claude Integration](claude-code.md)
- [Codex CLI Integration](codex.md)
